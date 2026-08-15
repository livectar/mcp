use super::resolver::McpServerResolver;
use crate::schemas::limits::TransportLimits;
use mcp_sdk::schemas::{caller::CallerContext, context::HostServices};
use std::sync::Arc;

#[derive(Clone)]
pub struct TransportConfig {
    pub resolver: Arc<dyn McpServerResolver>,
    pub caller: CallerContext,
    pub services: HostServices,
    pub limits: TransportLimits,
    pub allowed_origins: Vec<String>,
}

impl TransportConfig {
    pub fn new(
        resolver: Arc<dyn McpServerResolver>,
        caller: CallerContext,
        services: HostServices,
    ) -> Self {
        Self {
            resolver,
            caller,
            services,
            limits: TransportLimits::default(),
            allowed_origins: Vec::new(),
        }
    }
}
