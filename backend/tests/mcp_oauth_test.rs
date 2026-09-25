//! MCP + OAuth integration — requires test DB with oauth migration applied.

#![allow(dead_code, unused_imports)]

#[path = "../app/auth/mod.rs"]
mod auth;
#[path = "../app/controllers/mod.rs"]
mod controllers;
#[path = "../app/models/mod.rs"]
mod models;
#[path = "../app/services/mod.rs"]
mod services;
#[path = "../app/helpers/mod.rs"]
mod helpers;
#[path = "../app/oauth/mod.rs"]
mod oauth;
#[path = "../app/mcp/mod.rs"]
mod mcp;
#[path = "../app/api_docs/mod.rs"]
mod api_docs;
#[path = "../config/routes.rs"]
mod routes;

use doido::controller::axum::body::Body;
use doido::controller::axum::http::{Request, StatusCode, header};
use doido::controller::stack::MiddlewareStack;
use doido::model::password::hash_password;
use doido::model::sea_orm::{ActiveModelTrait, EntityTrait, Set};
use models::user::ActiveModel as UserActive;
use tower::ServiceExt;

fn test_router() -> doido::controller::axum::Router {
    std::env::set_var("MCP_ENABLED", "true");
    std::env::set_var(
        "MCP_OAUTH_SIGNING_SECRET",
        "test-mcp-signing-secret-fixed-value",
    );
    std::env::set_var("MCP_PUBLIC_URL", "http://127.0.0.1:3001");
    mcp::McpConfig::init();

    let config = doido::controller::config::load();
    MiddlewareStack::default()
        .with_cors_config(config.middleware().cors.clone())
        .apply(routes::router())
}

async fn setup_runtime() -> std::sync::MutexGuard<'static, ()> {
    std::env::set_var("DOIDO_ENV", "test");
    let guard = doido::model::pool::test_lock();
    if doido::model::pool::try_pool().is_none() {
        doido::model::pool::init()
            .await
            .expect("connect test database");
    }
    let config = doido_auth::load_config();
    doido_auth::init(doido::model::pool::pool().clone(), &config)
        .await
        .expect("auth init");
    oauth::store::ensure_dev_client(doido::model::pool::pool())
        .await
        .expect("oauth dev client");
    guard
}

async fn raw_request(
    app: &doido::controller::axum::Router,
    method: &str,
    uri: &str,
    body: &str,
    auth: Option<&str>,
) -> (StatusCode, String, Option<String>) {
    let mut builder = Request::builder().method(method).uri(uri);
    if !body.is_empty() {
        builder = builder.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
    }
    if let Some(bearer) = auth {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {bearer}"));
    }
    let req = builder.body(Body::from(body.to_string())).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let www = resp
        .headers()
        .get(header::WWW_AUTHENTICATE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let bytes = doido::controller::axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, String::from_utf8_lossy(&bytes).to_string(), www)
}

#[tokio::test]
async fn mcp_unauthorized_returns_resource_metadata() {
    let _guard = setup_runtime().await;
    let app = test_router();
    let (status, _, www) = raw_request(&app, "POST", "/mcp", "{}", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let header = www.expect("WWW-Authenticate");
    assert!(header.contains("resource_metadata"));
    assert!(header.contains("oauth-protected-resource"));
}

#[tokio::test]
async fn oauth_token_and_mcp_session_cookie_flow() {
    let _guard = setup_runtime().await;
    let app = test_router();
    let db = doido::model::pool::pool();

    let email = format!("mcp_oauth_{}@test.local", uuid::Uuid::new_v4());
    let password = "TestPass123!";
    let user = UserActive {
        email: Set(email.clone()),
        password_digest: Set(hash_password(password).unwrap()),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("user");

    let session_cookie =
        oauth::session_bridge::sign_in_and_get_cookie(&app, &email, password)
            .await
            .expect("sign in");

    let verifier = "v".repeat(43);
    let challenge = {
        use sha2::{Digest, Sha256};
        base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            Sha256::digest(verifier.as_bytes()),
        )
    };
    let code = oauth::store::insert_auth_code(
        db,
        "quicknfe_mcp_dev",
        user.id,
        &challenge,
        "http://127.0.0.1",
        &session_cookie,
    )
    .await
    .expect("auth code");

    let body = format!(
        "grant_type=authorization_code&code={code}&redirect_uri=http://127.0.0.1&client_id=quicknfe_mcp_dev&code_verifier={verifier}"
    );
    let (status, json, _) = raw_request(&app, "POST", "/oauth/token", &body, None).await;
    assert_eq!(status, StatusCode::OK, "token response: {json}");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("json");
    let access = parsed["access_token"]
        .as_str()
        .expect("access_token")
        .to_string();

    let (status, _, _) = raw_request(&app, "POST", "/mcp", "{}", Some(&access)).await;
    assert_ne!(status, StatusCode::UNAUTHORIZED, "bearer should be accepted");
}
