use async_trait::async_trait;
use mcp_protocol::schemas::{json_payload::JsonPayload, tools::CallToolResult};
use mcp_sdk::{
    errors::ServerError,
    schemas::{
        context::RequestContext,
        tool_definition::{ToolAnnotations, ToolDefinition},
        tool_schema::{ToolInputProperty, ToolInputSchema, ToolInputType},
    },
    traits::server::ToolHandler,
};
use std::sync::Arc;

use crate::{
    handlers::common::{authorize_and_credential, decode_arguments, provider_error, success},
    providers::telegram_bot::TelegramBotProvider,
    schemas::requests::GetChatRequest,
};

pub const TOOL_NAME: &str = "get_chat";

const INPUT_SCHEMA: ToolInputSchema = ToolInputSchema::object(
    &["chat_id"],
    &[ToolInputProperty::one_of(
        "chat_id",
        &[
            ToolInputType::integer(None, None),
            ToolInputType::string(Some(1), Some(64), &[]),
        ],
    )],
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
            description: "Return metadata for a chat accessible to the bot. Use a numeric chat ID (including -100... IDs) or a public channel username such as @example_channel.".to_string(),
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
        request.validate().map_err(ServerError::InvalidArguments)?;
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
