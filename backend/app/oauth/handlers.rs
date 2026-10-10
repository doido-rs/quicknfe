use std::sync::Arc;

use doido::controller::axum::extract::{Form, Query, State};
use doido::controller::axum::http::{HeaderMap, StatusCode, header};
use doido::controller::axum::response::{Html, IntoResponse, Redirect, Response};
use doido::controller::axum::Json;
use doido::model::password::verify_password;
use doido::model::sea_orm::DatabaseConnection;
use doido_auth::AuthUser;
use serde::{Deserialize, Serialize};

use crate::mcp::config::McpConfig;
use crate::models::user::Model as User;

use super::jwt::{issue_access_token, issue_refresh_token, verify_refresh_token};
use super::pkce::{random_token, verify_s256};
use super::session_bridge::{default_company_for_user, sign_in_and_get_cookie, switch_company};
use super::store::{
    consume_auth_code, find_client, insert_access_token, insert_auth_code, redirect_uri_allowed,
    register_client, upsert_mcp_session,
};

#[derive(Clone)]
pub struct OAuthState {
    pub db: DatabaseConnection,
    pub api_router: doido::controller::axum::Router,
}

pub async fn protected_resource_metadata() -> Json<serde_json::Value> {
    let cfg = McpConfig::get();
    Json(serde_json::json!({
        "resource": cfg.resource_url(),
        "authorization_servers": [cfg.issuer()],
        "bearer_methods_supported": ["header"],
        "scopes_supported": ["mcp"],
    }))
}

pub async fn authorization_server_metadata() -> Json<serde_json::Value> {
    let cfg = McpConfig::get();
    Json(serde_json::json!({
        "issuer": cfg.issuer(),
        "authorization_endpoint": format!("{}/oauth/authorize", cfg.public_url),
        "token_endpoint": format!("{}/oauth/token", cfg.public_url),
        "registration_endpoint": format!("{}/oauth/register", cfg.public_url),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "code_challenge_methods_supported": ["S256"],
        "token_endpoint_auth_methods_supported": ["none"],
        "scopes_supported": ["mcp"],
    }))
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub state: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeForm {
    pub email: String,
    pub password: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub code_challenge: String,
    pub state: Option<String>,
}

pub async fn authorize_get(
    State(state): State<Arc<OAuthState>>,
    Query(q): Query<AuthorizeQuery>,
) -> Html<String> {
    super::store::ensure_dev_client_once(&state.db).await;
    let state_field = q
        .state
        .as_ref()
        .map(|s| format!(r#"<input type="hidden" name="state" value="{s}"/>"#))
        .unwrap_or_default();
    Html(format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"/><title>QuickNFe MCP — Authorize</title></head>
<body>
<h1>Authorize MCP access</h1>
<p>Sign in to grant an MCP client access to QuickNFe on your behalf.</p>
<form method="post" action="/oauth/authorize">
<input type="hidden" name="client_id" value="{}"/>
<input type="hidden" name="redirect_uri" value="{}"/>
<input type="hidden" name="code_challenge" value="{}"/>
{state_field}
<label>Email <input name="email" type="email" required/></label><br/>
<label>Password <input name="password" type="password" required/></label><br/>
<button type="submit">Sign in and authorize</button>
</form>
</body></html>"#,
        html_escape(&q.client_id),
        html_escape(&q.redirect_uri),
        html_escape(&q.code_challenge),
    ))
}

pub async fn authorize_post(
    State(state): State<Arc<OAuthState>>,
    Form(form): Form<AuthorizeForm>,
) -> Result<Response, StatusCode> {
    if form.client_id.is_empty() || form.redirect_uri.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let client = find_client(&state.db, &form.client_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::BAD_REQUEST)?;
    if !redirect_uri_allowed(&client, &form.redirect_uri) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let user = User::find_by_email(&state.db, &form.email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    if !verify_password(&form.password, user.password_digest().unwrap_or_default()) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut session_cookie = sign_in_and_get_cookie(&state.api_router, &form.email, &form.password)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(company_id) = default_company_for_user(user.id()).await {
        session_cookie = switch_company(&state.api_router, &session_cookie, company_id)
            .await
            .unwrap_or(session_cookie);
    }

    let code = insert_auth_code(
        &state.db,
        &form.client_id,
        user.id(),
        &form.code_challenge,
        &form.redirect_uri,
        &session_cookie,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut url = format!(
        "{}?code={}",
        form.redirect_uri,
        urlencoding::encode(&code)
    );
    if let Some(st) = form.state {
        url.push_str(&format!("&state={}", urlencoding::encode(&st)));
    }
    Ok(Redirect::temporary(&url).into_response())
}

#[derive(Debug, Deserialize)]
pub struct TokenForm {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: Option<String>,
    pub code_verifier: Option<String>,
    pub refresh_token: Option<String>,
}

pub async fn token(
    State(state): State<Arc<OAuthState>>,
    Form(form): Form<TokenForm>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let cfg = McpConfig::get();
    match form.grant_type.as_str() {
        "authorization_code" => {
            let code = form.code.as_ref().ok_or(StatusCode::BAD_REQUEST)?;
            let redirect_uri = form.redirect_uri.as_ref().ok_or(StatusCode::BAD_REQUEST)?;
            let client_id = form.client_id.as_ref().ok_or(StatusCode::BAD_REQUEST)?;
            let code_verifier = form.code_verifier.as_ref().ok_or(StatusCode::BAD_REQUEST)?;

            let auth_row = consume_auth_code(&state.db, code, client_id, redirect_uri)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .ok_or(StatusCode::BAD_REQUEST)?;

            if !verify_s256(code_verifier, &auth_row.code_challenge) {
                return Err(StatusCode::BAD_REQUEST);
            }

            let access_jti = random_token(16);
            let refresh_jti = random_token(16);
            insert_access_token(
                &state.db,
                &access_jti,
                auth_row.user_id,
                cfg.access_token_ttl_secs,
                Some(refresh_jti.clone()),
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            upsert_mcp_session(&state.db, &access_jti, &auth_row.session_cookie)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let access_token = issue_access_token(
                auth_row.user_id,
                &access_jti,
                cfg.access_token_ttl_secs,
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let refresh_token = issue_refresh_token(
                auth_row.user_id,
                &refresh_jti,
                cfg.refresh_token_ttl_secs,
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            insert_access_token(
                &state.db,
                &refresh_jti,
                auth_row.user_id,
                cfg.refresh_token_ttl_secs,
                None,
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(Json(serde_json::json!({
                "access_token": access_token,
                "token_type": "Bearer",
                "expires_in": cfg.access_token_ttl_secs,
                "refresh_token": refresh_token,
                "scope": "mcp",
            })))
        }
        "refresh_token" => {
            let refresh = form.refresh_token.as_ref().ok_or(StatusCode::BAD_REQUEST)?;
            let claims = verify_refresh_token(refresh).map_err(|_| StatusCode::UNAUTHORIZED)?;
            let user_id: i64 = claims.sub.parse().map_err(|_| StatusCode::UNAUTHORIZED)?;

            let old_session = super::store::mcp_session_cookie(&state.db, &claims.jti)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .ok_or(StatusCode::UNAUTHORIZED)?;

            let access_jti = random_token(16);
            insert_access_token(
                &state.db,
                &access_jti,
                user_id,
                cfg.access_token_ttl_secs,
                Some(claims.jti.clone()),
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            upsert_mcp_session(&state.db, &access_jti, &old_session)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let access_token = issue_access_token(user_id, &access_jti, cfg.access_token_ttl_secs)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(Json(serde_json::json!({
                "access_token": access_token,
                "token_type": "Bearer",
                "expires_in": cfg.access_token_ttl_secs,
                "refresh_token": refresh,
                "scope": "mcp",
            })))
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub redirect_uris: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uris: Vec<String>,
}

pub async fn register_client_handler(
    State(state): State<Arc<OAuthState>>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    if body.redirect_uris.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let client = register_client(&state.db, body.redirect_uris.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(RegisterResponse {
        client_id: client.client_id,
        client_secret: client.client_secret,
        redirect_uris: body.redirect_uris,
    }))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn bearer_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
}
