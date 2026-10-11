pub mod compression;
pub mod dps_id;
pub mod error;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod store;

use std::sync::Arc;

use axum::Router;
use store::DpsStore;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<DpsStore>,
}

pub fn build_app(state: AppState) -> Router {
    let sefin = routes::sefin_nacional_router();
    Router::new()
        .nest("/SefinNacional", sefin.clone())
        .nest("/API/SefinNacional", sefin)
        .route(
            "/cnc/consulta/cad",
            axum::routing::get(handlers::cnc::get_cad),
        )
        .route("/health", axum::routing::get(|| async { "ok" }))
        .with_state(state)
}

pub fn default_state() -> AppState {
    AppState {
        store: Arc::new(DpsStore::default()),
    }
}
