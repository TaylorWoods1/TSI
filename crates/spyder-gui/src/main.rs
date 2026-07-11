use std::path::PathBuf;

use axum::Router;
use spyder_gui::{api, AppState};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::EnvFilter;

fn web_dist() -> PathBuf {
    if let Ok(path) = std::env::var("SPYDER_WEB_DIST") {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/dist")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("spyder_gui=info".parse().unwrap()),
        )
        .init();

    let state = AppState::new_rect();
    let dist = web_dist();
    let index = dist.join("index.html");

    let api = api::router(state);
    let spa = ServeDir::new(&dist).not_found_service(ServeFile::new(index));

    let app = Router::new().merge(api).fallback_service(spa).layer(CorsLayer::permissive());

    let addr = "127.0.0.1:7700";
    tracing::info!("spyder-gui {} listening on http://{addr}", spyder_gui::version());
    if dist.exists() {
        tracing::info!("serving SPA from {}", dist.display());
    } else {
        tracing::warn!("web/dist not found — run `cd web && npm ci && npm run build` for bundled UI");
    }

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
