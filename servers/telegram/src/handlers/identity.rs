use async_trait::async_trait;
use mcp_protocol::schemas::{
    json_payload::JsonPayload,
    tools::{CallToolResult, ContentBlock},
};
use mcp_sdk::{
    errors::ServerError,
    schemas::{
        authorization::{ApprovalDecision, AuthorizationRequest, OperationName},
        context::RequestContext,
        credentials::ProviderName,
        tool_definition::{ToolAnnotations, ToolDefinition},
        tool_schema::ToolInputSchema,
    },
    traits::server::ToolHandler,
};
use std::sync::Arc;

use crate::providers::telegram::TelegramProvider;

pub struct TelegramIdentityHandler {
    provider: Arc<dyn TelegramProvider>,
}

impl TelegramIdentityHandler {
    pub fn new(provider: Arc<dyn TelegramProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl ToolHandler for TelegramIdentityHandler {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "telegram_get_identity".to_string(),
            description: "Return the identity for the configured Telegram connection.".to_string(),
            input_schema: ToolInputSchema::object(&[], &[]),
            annotations: ToolAnnotations {
                read_only_hint: Some(true),
            },
        }
    }

    async fn call(
        &self,
        context: &RequestContext,
        _arguments: Option<JsonPayload>,
    ) -> Result<CallToolResult, ServerError> {
        let operation = AuthorizationRequest {
            operation: OperationName::new("telegram_get_identity")?,
            tool_name: "telegram_get_identity".to_string(),
        };
        context.authorize(&operation).await?;
        if context.approval(&operation).await? == ApprovalDecision::Denied {
            return Err(mcp_sdk::errors::HostError::ApprovalDenied.into());
        }
        let credential = context
            .credential(ProviderName::new("telegram")?, "provider request")
            .await?;
        let identity = self
            .provider
            .get_identity(&credential)
            .await
            .map_err(ServerError::Provider)?;
        let structured_content = JsonPayload::from_serializable(&identity)
            .map_err(|error| ServerError::Protocol(error.to_string()))?;
        Ok(CallToolResult {
            content: vec![ContentBlock::Text {
                text: identity.username,
            }],
            structured_content: Some(structured_content),
            is_error: false,
        })
    }
}
