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
    schemas::requests::GetUpdatesRequest,
};

pub const TOOL_NAME: &str = "get_updates";

const INPUT_SCHEMA: ToolInputSchema = ToolInputSchema::object(
    &[],
    &[
        ToolInputProperty::integer("offset", None, None),
        ToolInputProperty::integer("limit", Some(1), Some(100)),
    ],
);

pub struct GetUpdatesHandler {
    provider: Arc<dyn TelegramBotProvider>,
}

impl GetUpdatesHandler {
    pub fn new(provider: Arc<dyn TelegramBotProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl ToolHandler for GetUpdatesHandler {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: TOOL_NAME.to_string(),
            description: "Read pending bot updates and return exact chat IDs from messages and channel posts. Use next_offset in a later call to acknowledge the returned updates.".to_string(),
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
        let request = match arguments {
            Some(arguments) => decode_arguments(Some(arguments))?,
            None => GetUpdatesRequest::default(),
        };
        request.validate().map_err(ServerError::InvalidArguments)?;
        let credential =
            authorize_and_credential(context, TOOL_NAME, "Telegram Bot API update lookup").await?;
        let result = self
            .provider
            .get_updates(&credential, request)
            .await
            .map_err(provider_error)?;
        success(
            "Retrieved Telegram bot updates and chat IDs.".to_string(),
            &result,
        )
    }
}
