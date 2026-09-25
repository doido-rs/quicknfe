use std::sync::Arc;

use doido::controller::axum::body::Body;
use doido::controller::axum::http::{Request, Response, StatusCode, header};
use doido::controller::axum::extract::State;
use doido::controller::axum::middleware::Next;
use doido::model::sea_orm::DatabaseConnection;

use crate::mcp::config::McpConfig;
use crate::mcp::context;
use crate::oauth::handlers::bearer_from_headers;
use crate::oauth::jwt::verify_access_token;
use crate::oauth::store::access_token_valid;

#[derive(Clone)]
pub struct McpAuthState {
    pub db: DatabaseConnection,
}

pub async fn mcp_bearer_auth(
    State(auth): State<Arc<McpAuthState>>,
    mut req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let Some(token) = bearer_from_headers(req.headers()) else {
        return unauthorized_response();
    };

    let claims = match verify_access_token(&token) {
        Ok(c) => c,
        Err(_) => return unauthorized_response(),
    };

    if access_token_valid(&auth.db, &claims.jti)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        return unauthorized_response();
    }

    req.extensions_mut().insert(claims.jti.clone());
    context::with_token_jti(claims.jti, next.run(req)).await
}

fn unauthorized_response() -> Response<Body> {
    let cfg = McpConfig::get();
    let prm = cfg.prm_url();
    let www = format!(
        r#"Bearer realm="mcp", resource_metadata="{}""#,
        prm
    );
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::WWW_AUTHENTICATE, www)
        .body(Body::empty())
        .unwrap()
}
