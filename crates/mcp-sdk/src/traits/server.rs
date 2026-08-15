use crate::{errors::ServerError, schemas::context::RequestContext};
use async_trait::async_trait;
use mcp_protocol::schemas::{
    lifecycle::{ImplementationInfo, ServerCapabilities},
    tools::{CallToolParams, CallToolResult, ToolDefinition},
};

#[async_trait]
pub trait ToolHandler: Send + Sync {
    fn definition(&self) -> ToolDefinition;

    async fn call(
        &self,
        context: &RequestContext,
        arguments: Option<mcp_protocol::schemas::json_payload::JsonPayload>,
    ) -> Result<CallToolResult, ServerError>;
}

#[async_trait]
pub trait McpServer: Send + Sync {
    fn info(&self) -> ImplementationInfo;

    fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            tools: Some(Default::default()),
        }
    }

    fn tools(&self) -> Vec<ToolDefinition>;

    async fn call_tool(
        &self,
        context: &RequestContext,
        request: CallToolParams,
    ) -> Result<CallToolResult, ServerError>;
}
