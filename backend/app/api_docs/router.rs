use doido::controller::axum::http::{header, StatusCode};
use doido::controller::axum::response::{Html, IntoResponse};
use doido::controller::axum::routing::get;
use doido::controller::axum::Router;

const OPENAPI_YAML: &str = include_str!("../../../docs/api/openapi.yaml");
const SWAGGER_HTML: &str = include_str!("swagger.html");

async fn openapi_yaml() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/yaml; charset=utf-8")],
        OPENAPI_YAML,
    )
}

async fn swagger_ui() -> Html<&'static str> {
    Html(SWAGGER_HTML)
}

pub fn router() -> Router {
    Router::new()
        .route("/api-docs/openapi.yaml", get(openapi_yaml))
        .route("/api-docs", get(swagger_ui))
}
