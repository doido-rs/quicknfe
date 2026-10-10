use axum::Router;
use sefin_mock::{build_app, default_state};

pub fn test_app() -> Router {
    build_app(default_state())
}
