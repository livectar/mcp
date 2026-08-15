use super::config::TransportConfig;
use crate::schemas::limits::TransportLimits;
use mcp_sdk::schemas::{caller::CallerContext, context::HostServices};
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub(crate) struct TransportState {
    pub(crate) resolver: Arc<dyn super::resolver::McpServerResolver>,
    pub(crate) caller: CallerContext,
    pub(crate) services: HostServices,
    pub(crate) limits: Arc<TransportLimits>,
    pub(crate) concurrency: Arc<Semaphore>,
    pub(crate) allowed_origins: Arc<Vec<String>>,
}

impl TransportState {
    pub(crate) fn from_config(config: TransportConfig) -> Self {
        let limits = config.limits.normalized();
        Self {
            resolver: config.resolver,
            caller: config.caller,
            services: config.services,
            concurrency: Arc::new(Semaphore::new(limits.max_concurrency)),
            limits: Arc::new(limits),
            allowed_origins: Arc::new(config.allowed_origins),
        }
    }
}
