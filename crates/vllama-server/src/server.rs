use axum::{
    routing::{get, post},
    Router,
    http::{Request, Response},
    body::Body,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, Span};
use std::time::Instant;
use uuid::Uuid;

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
        // Custom trace layer with request IDs and latency tracking
        let trace_layer = TraceLayer::new_for_http()
            .make_span_with(|request: &Request<Body>| {
                let request_id = Uuid::new_v4().to_string();
                let method = request.method().as_str();
                let uri = request.uri().path();

                tracing::info_span!(
                    "request",
                    request_id = %request_id,
                    method = %method,
                    uri = %uri,
                    latency_ms = tracing::field::Empty,
                    status = tracing::field::Empty,
                )
            })
            .on_request(|_request: &Request<Body>, _span: &Span| {
                // Record request start time
                _span.record("start_time", Instant::now().elapsed().as_millis() as u64);
            })
            .on_response(|response: &Response<Body>, latency: std::time::Duration, span: &Span| {
                let latency_ms = latency.as_millis() as u64;
                let status = response.status().as_u16();

                span.record("latency_ms", latency_ms);
                span.record("status", status);

                tracing::info!(
                    latency_ms = latency_ms,
                    status = status,
                    "request completed"
                );
            });

        let app = Router::new()
            .route("/api/generate", post(api::generate))
            .route("/api/chat", post(api::chat))
            .route("/api/pull", post(api::pull))
            .route("/api/show", post(api::show))
            .route("/api/tags", get(api::tags))
            .route("/api/ps", get(api::ps))
            .route("/api/version", get(api::version))
            .route("/v1/chat/completions", post(api::openai_chat_completions))
            .route("/health", get(api::health))
            .layer(CorsLayer::permissive())
            .layer(trace_layer)
            .with_state(self.state);

        let addr = format!("{}:{}", self.host, self.port);
        info!("Starting vLLama server on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
