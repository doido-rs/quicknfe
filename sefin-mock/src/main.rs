use std::net::SocketAddr;

use sefin_mock::{build_app, default_state};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("sefin_mock=info".parse().unwrap()))
        .init();

    let port: u16 = std::env::var("SEFIN_MOCK_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8090);

    let app = build_app(default_state())
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("sefin-mock listening on http://127.0.0.1:{port}");
    tracing::info!("POST http://127.0.0.1:{port}/SefinNacional/nfse");
    tracing::info!("GET  http://127.0.0.1:{port}/SefinNacional/dps/{{id}}");
    tracing::info!("(alias) /API/SefinNacional/... para compatibilidade com quicknfe");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
