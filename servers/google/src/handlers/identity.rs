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
        credentials::ProviderName,
    },
    traits::server::ToolHandler,
};
use std::sync::Arc;

use crate::providers::google::GoogleProvider;

pub struct GoogleIdentityHandler {
    provider: Arc<dyn GoogleProvider>,
}

impl GoogleIdentityHandler {
    pub fn new(provider: Arc<dyn GoogleProvider>) -> Self {
        Self { provider }
    }

    pub fn provider(&self) -> &Arc<dyn GoogleProvider> {
        &self.provider
    }
}

#[async_trait]
impl ToolHandler for GoogleIdentityHandler {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "google_get_identity".to_string(),
            description: "Return the identity for the configured Google connection.".to_string(),
            input_schema: JsonPayload::parse(r#"{"type":"object","additionalProperties":false}"#)
                .expect("Google schema must be valid JSON"),
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
            operation: OperationName::new("google_get_identity")?,
            tool_name: "google_get_identity".to_string(),
        };
        context.authorize(&operation).await?;
        if context.approval(&operation).await? == ApprovalDecision::Denied {
            return Err(mcp_sdk::errors::HostError::ApprovalDenied.into());
        }
        let credential = context
            .credential(ProviderName::new("google")?, "provider request")
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
                text: identity.display_name,
            }],
            structured_content: Some(structured_content),
            is_error: false,
        })
    }
}
