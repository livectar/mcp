use crate::schemas::server_key::ServerKey;
use mcp_sdk::traits::server::McpServer;
use std::sync::Arc;

pub trait McpServerResolver: Send + Sync {
    fn resolve(&self, server_key: &ServerKey) -> Option<Arc<dyn McpServer>>;
}
