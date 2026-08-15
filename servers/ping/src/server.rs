use async_trait::async_trait;
use mcp_protocol::schemas::{
    lifecycle::{ImplementationInfo, ServerCapabilities},
    tools::{CallToolParams, CallToolResult, ToolDefinition},
};
use mcp_sdk::{
    errors::ServerError,
    schemas::context::RequestContext,
    traits::server::{McpServer, ToolHandler},
};

use crate::handlers::ping::PingHandler;

pub const SERVER_NAME: &str = "mcp-ping";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PING_SERVER_KEY: &str = "ping";

pub struct PingServer {
    handler: PingHandler,
}

impl PingServer {
    pub fn new() -> Self {
        Self {
            handler: PingHandler,
        }
    }
}

impl Default for PingServer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl McpServer for PingServer {
    fn info(&self) -> ImplementationInfo {
        ImplementationInfo {
            name: SERVER_NAME.to_string(),
            version: SERVER_VERSION.to_string(),
        }
    }

    fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            tools: Some(Default::default()),
        }
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![self.handler.definition()]
    }

    async fn call_tool(
        &self,
        context: &RequestContext,
        request: CallToolParams,
    ) -> Result<CallToolResult, ServerError> {
        if request.name != "ping" {
            return Err(ServerError::ToolNotFound(request.name));
        }
        self.handler.call(context, request.arguments).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_protocol::schemas::{json_payload::JsonPayload, json_rpc::RequestId};
    use mcp_testkit::{contracts::assert_server_contract, fixtures::host::TestHost};

    #[test]
    fn ping_satisfies_the_public_server_contract() {
        assert_server_contract(&PingServer::new());
    }

    #[tokio::test]
    async fn ping_returns_typed_structured_content() {
        let server = PingServer::new();
        let host = TestHost::new();
        let context = host.context("ping-test");
        let arguments = JsonPayload::parse(r#"{"message":"hello"}"#).unwrap();

        let result = server
            .call_tool(
                &context,
                CallToolParams {
                    name: "ping".to_string(),
                    arguments: Some(arguments),
                },
            )
            .await
            .unwrap();

        let payload = result.structured_content.unwrap();
        let decoded: crate::schemas::ping::PingResult = payload.decode().unwrap();
        assert_eq!(decoded.message, "hello");
        assert_eq!(
            context.request_id,
            RequestId::String("ping-test".to_string())
        );
    }

    #[tokio::test]
    async fn ping_rejects_invalid_typed_arguments() {
        let server = PingServer::new();
        let host = TestHost::new();
        let context = host.context("invalid-ping-test");
        let arguments = JsonPayload::parse(r#"{"message":42}"#).unwrap();

        let error = server
            .call_tool(
                &context,
                CallToolParams {
                    name: "ping".to_string(),
                    arguments: Some(arguments),
                },
            )
            .await
            .expect_err("invalid message type must fail explicitly");

        assert!(matches!(
            error,
            mcp_sdk::errors::ServerError::InvalidArguments(_)
        ));
    }

    #[test]
    fn ping_exposes_only_public_tool_metadata() {
        let tool = PingServer::new().tools().pop().unwrap();
        assert_eq!(tool.name, "ping");
        assert!(tool.input_schema.as_str().contains("message"));
    }
}
