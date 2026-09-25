use std::sync::Arc;

use doido::controller::axum::routing::{get, post};
use doido::controller::axum::Router;
use doido::model::pool;

use super::handlers::{
    authorization_server_metadata, authorize_get, authorize_post, protected_resource_metadata,
    register_client_handler, token, OAuthState,
};

pub fn router(api_router: Router) -> Router {
    let db = pool::pool().clone();
    let state = Arc::new(OAuthState { db, api_router });

    Router::new()
        .route(
            "/.well-known/oauth-protected-resource",
            get(protected_resource_metadata),
        )
        .route(
            "/.well-known/oauth-authorization-server",
            get(authorization_server_metadata),
        )
        .route("/oauth/authorize", get(authorize_get).post(authorize_post))
        .route("/oauth/token", post(token))
        .route("/oauth/register", post(register_client_handler))
        .with_state(state)
}
