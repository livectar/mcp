use async_trait::async_trait;
use mcp_sdk::{errors::HostError, schemas::caller::CallerContext, traits::host::WorkspaceContext};

pub struct StaticWorkspaceContext {
    pub caller: CallerContext,
}

#[async_trait]
impl WorkspaceContext for StaticWorkspaceContext {
    async fn caller(&self) -> Result<CallerContext, HostError> {
        Ok(self.caller.clone())
    }
}
