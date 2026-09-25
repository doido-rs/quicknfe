use std::sync::Arc;

use doido::controller::axum::middleware::from_fn_with_state;
use doido::controller::axum::Router;
use doido::model::pool;
use rmcp::transport::{
    StreamableHttpServerConfig, StreamableHttpService,
    streamable_http_server::session::local::LocalSessionManager,
};
use tokio_util::sync::CancellationToken;

use super::middleware::{mcp_bearer_auth, McpAuthState};
use super::server::{McpSharedState, QuickNfeMcpServer};

pub fn router(api_router: Router) -> Router {
    let db = pool::pool().clone();
    let shared = Arc::new(McpSharedState {
        db: db.clone(),
        api_router,
    });
    let shared_factory = shared.clone();
    let ct = CancellationToken::new();
    let config = StreamableHttpServerConfig::default()
        .with_json_response(true)
        .with_cancellation_token(ct);

    let service = StreamableHttpService::new(
        move || Ok(QuickNfeMcpServer::new(shared_factory.clone())),
        LocalSessionManager::default().into(),
        config,
    );

    let auth_state = Arc::new(McpAuthState { db });

    Router::new()
        .nest_service("/mcp", service)
        .layer(from_fn_with_state(auth_state.clone(), mcp_bearer_auth))
        .with_state(auth_state)
}
