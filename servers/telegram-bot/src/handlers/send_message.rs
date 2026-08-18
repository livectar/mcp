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
    schemas::requests::{SendMessageRequest, MAX_MESSAGE_TEXT_LENGTH},
};

pub const TOOL_NAME: &str = "telegram_send_message";

const INPUT_SCHEMA: ToolInputSchema = ToolInputSchema::object(
    &["chat_id", "text"],
    &[
        ToolInputProperty::integer("chat_id", None, None),
        ToolInputProperty::string("text", Some(1), Some(MAX_MESSAGE_TEXT_LENGTH)),
        ToolInputProperty::string_enum("parse_mode", &["markdown_v2", "html"]),
    ],
);

pub struct SendMessageHandler {
    provider: Arc<dyn TelegramBotProvider>,
}

impl SendMessageHandler {
    pub fn new(provider: Arc<dyn TelegramBotProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl ToolHandler for SendMessageHandler {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: TOOL_NAME.to_string(),
            description: "Send a text message through the configured Telegram bot.".to_string(),
            input_schema: INPUT_SCHEMA,
            annotations: ToolAnnotations {
                read_only_hint: Some(false),
            },
        }
    }

    async fn call(
        &self,
        context: &RequestContext,
        arguments: Option<JsonPayload>,
    ) -> Result<CallToolResult, ServerError> {
        let request = decode_arguments::<SendMessageRequest>(arguments)?;
        request.validate().map_err(ServerError::InvalidArguments)?;
        let credential =
            authorize_and_credential(context, TOOL_NAME, "Telegram Bot API message delivery")
                .await?;
        let result = self
            .provider
            .send_message(&credential, request)
            .await
            .map_err(provider_error)?;
        success("Sent Telegram message.".to_string(), &result)
    }
}
