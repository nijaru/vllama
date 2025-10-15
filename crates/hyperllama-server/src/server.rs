use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::api;
use crate::state::ServerState;

pub struct Server {
    state: ServerState,
    host: String,
    port: u16,
}

impl Server {
    pub fn new(host: impl Into<String>, port: u16) -> crate::Result<Self> {
        let state = ServerState::new()?;

        Ok(Self {
            state,
            host: host.into(),
            port,
        })
    }

    pub async fn run(self) -> crate::Result<()> {
        let app = Router::new()
            .route("/api/generate", post(api::generate))
            .route("/api/tags", get(api::tags))
            .route("/health", get(api::health))
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http())
            .with_state(self.state);

        let addr = format!("{}:{}", self.host, self.port);
        info!("Starting HyperLlama server on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
