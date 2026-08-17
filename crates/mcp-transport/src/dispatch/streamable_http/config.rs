use super::resolver::McpServerResolver;
use crate::schemas::limits::TransportLimits;
use mcp_sdk::{
    schemas::{caller::CallerContext, context::HostServices},
    traits::host::CallerContextResolver,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct TransportConfig {
    pub resolver: Arc<dyn McpServerResolver>,
    pub caller: CallerContext,
    pub services: HostServices,
    pub caller_resolver: Option<Arc<dyn CallerContextResolver>>,
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
            caller_resolver: None,
            limits: TransportLimits::default(),
            allowed_origins: Vec::new(),
        }
    }

    pub fn with_caller_resolver(mut self, resolver: Arc<dyn CallerContextResolver>) -> Self {
        self.caller_resolver = Some(resolver);
        self
    }
}
