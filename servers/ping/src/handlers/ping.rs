use async_trait::async_trait;
use mcp_protocol::schemas::{
    json_payload::JsonPayload,
    tools::{CallToolResult, ContentBlock, ToolAnnotations, ToolDefinition},
};
use mcp_sdk::{
    errors::ServerError,
    schemas::{
        authorization::{ApprovalDecision, AuthorizationRequest, OperationName},
        context::RequestContext,
    },
    traits::server::ToolHandler,
};

use crate::schemas::ping::{PingArguments, PingResult};

const PING_SCHEMA: &str =
    r#"{"type":"object","properties":{"message":{"type":"string"}},"additionalProperties":false}"#;

pub struct PingHandler;

#[async_trait]
impl ToolHandler for PingHandler {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "ping".to_string(),
            description: "Return a deterministic response from the MCP server.".to_string(),
            input_schema: JsonPayload::parse(PING_SCHEMA).expect("ping schema must be valid JSON"),
            annotations: ToolAnnotations {
                read_only_hint: Some(true),
            },
        }
    }

    async fn call(
        &self,
        context: &RequestContext,
        arguments: Option<JsonPayload>,
    ) -> Result<CallToolResult, ServerError> {
        let operation = AuthorizationRequest {
            operation: OperationName::new("ping")?,
            tool_name: "ping".to_string(),
        };
        context.authorize(&operation).await?;
        if context.approval(&operation).await? == ApprovalDecision::Denied {
            return Err(mcp_sdk::errors::HostError::ApprovalDenied.into());
        }

        let arguments = match arguments {
            Some(arguments) => arguments
                .decode::<PingArguments>()
                .map_err(|error| ServerError::InvalidArguments(error.to_string()))?,
            None => PingArguments::default(),
        };
        let result = PingResult {
            message: arguments.message.unwrap_or_else(|| "pong".to_string()),
            protocol_version: context.protocol_version.clone(),
        };
        let structured_content = JsonPayload::from_serializable(&result)
            .map_err(|error| ServerError::Protocol(error.to_string()))?;
        Ok(CallToolResult {
            content: vec![ContentBlock::Text {
                text: result.message.clone(),
            }],
            structured_content: Some(structured_content),
            is_error: false,
        })
    }
}
