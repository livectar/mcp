use super::{config::TransportConfig, routes, state::TransportState};
use axum::{routing::post, Router};

#[derive(Clone)]
pub struct McpTransport {
    state: TransportState,
}

impl McpTransport {
    pub fn new(config: TransportConfig) -> Self {
        Self {
            state: TransportState::from_config(config),
        }
    }

    pub fn router(self) -> Router {
        let request_limit = self.state.limits.max_request_bytes;
        Router::new()
            .route(
                "/mcp/:server_key",
                post(routes::post).get(routes::get).delete(routes::delete),
            )
            .layer(axum::extract::DefaultBodyLimit::max(request_limit))
            .with_state(self.state)
    }
}
