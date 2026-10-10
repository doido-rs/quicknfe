use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::{dps, nfse};
use crate::AppState;

/// Rotas espelhando `/SefinNacional/*` (e alias `/API/SefinNacional/*`).
pub fn sefin_nacional_router() -> Router<AppState> {
    Router::new()
        .route("/nfse", post(nfse::post_nfse))
        .route("/dps/{id}", get(dps::get_dps))
}
