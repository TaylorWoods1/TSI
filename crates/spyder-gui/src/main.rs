use spyder_gui::{api, AppState};
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("spyder_gui=info".parse().unwrap()))
        .init();

    let state = AppState::new_rect();
    let app = api::router(state).layer(CorsLayer::permissive());

    let addr = "127.0.0.1:7700";
    tracing::info!("spyder-gui {} listening on http://{addr}", spyder_gui::version());

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
