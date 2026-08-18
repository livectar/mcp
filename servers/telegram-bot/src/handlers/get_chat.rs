use async_trait::async_trait;
use mcp_protocol::schemas::{json_payload::JsonPayload, tools::CallToolResult};
use mcp_sdk::{
    errors::ServerError,
    schemas::{
        context::RequestContext,
        tool_definition::{ToolAnnotations, ToolDefinition},
        tool_schema::{ToolInputProperty, ToolInputSchema},
    },
    traits::server::ToolHandler,
};
use std::sync::Arc;

use crate::{
    handlers::common::{authorize_and_credential, decode_arguments, provider_error, success},
    providers::telegram_bot::TelegramBotProvider,
    schemas::requests::GetChatRequest,
};

pub const TOOL_NAME: &str = "telegram_get_chat";

const INPUT_SCHEMA: ToolInputSchema = ToolInputSchema::object(
    &["chat_id"],
    &[ToolInputProperty::integer("chat_id", None, None)],
);

pub struct GetChatHandler {
    provider: Arc<dyn TelegramBotProvider>,
}

impl GetChatHandler {
    pub fn new(provider: Arc<dyn TelegramBotProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl ToolHandler for GetChatHandler {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: TOOL_NAME.to_string(),
            description: "Return metadata for a Telegram chat accessible to the bot.".to_string(),
            input_schema: INPUT_SCHEMA,
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
        let request = decode_arguments::<GetChatRequest>(arguments)?;
        let credential =
            authorize_and_credential(context, TOOL_NAME, "Telegram Bot API chat lookup").await?;
        let result = self
            .provider
            .get_chat(&credential, request)
            .await
            .map_err(provider_error)?;
        success("Retrieved Telegram chat metadata.".to_string(), &result)
    }
}
