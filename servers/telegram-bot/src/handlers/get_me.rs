use async_trait::async_trait;
use mcp_protocol::schemas::{json_payload::JsonPayload, tools::CallToolResult};
use mcp_sdk::{
    errors::ServerError,
    schemas::{
        context::RequestContext,
        tool_definition::{ToolAnnotations, ToolDefinition},
        tool_schema::ToolInputSchema,
    },
    traits::server::ToolHandler,
};
use std::sync::Arc;

use crate::{
    handlers::common::{authorize_and_credential, provider_error, success},
    providers::telegram_bot::TelegramBotProvider,
};

pub const TOOL_NAME: &str = "telegram_get_me";

pub struct GetMeHandler {
    provider: Arc<dyn TelegramBotProvider>,
}

impl GetMeHandler {
    pub fn new(provider: Arc<dyn TelegramBotProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl ToolHandler for GetMeHandler {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: TOOL_NAME.to_string(),
            description: "Return the identity and capabilities of the configured Telegram bot."
                .to_string(),
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
        let credential =
            authorize_and_credential(context, TOOL_NAME, "Telegram Bot API identity lookup")
                .await?;
        let result = self
            .provider
            .get_me(&credential)
            .await
            .map_err(provider_error)?;
        success("Retrieved Telegram bot identity.".to_string(), &result)
    }
}
