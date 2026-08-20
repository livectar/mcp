use async_trait::async_trait;
use mcp_protocol::schemas::{
    lifecycle::{ImplementationInfo, ServerCapabilities},
    tools::{CallToolParams, CallToolResult},
};
use mcp_sdk::{
    errors::{ServerError, ToolRegistryError},
    schemas::{
        context::RequestContext,
        credentials::{CredentialField, CredentialRequirements},
        tool_definition::ToolDefinition,
    },
    traits::{registry::McpServerRegistration, server::McpServer, tool_registry::ToolRegistry},
};
use std::sync::Arc;

use crate::{
    handlers::{
        get_chat::GetChatHandler, get_me::GetMeHandler, get_updates::GetUpdatesHandler,
        send_message::SendMessageHandler,
    },
    providers::telegram_bot::{TelegramBotProvider, TeloxideTelegramBotProvider},
};

pub const SERVER_NAME: &str = "mcp-telegram-bot";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CREDENTIAL_REQUIREMENTS: CredentialRequirements = CredentialRequirements::new(
    "telegram-bot",
    "bot_token",
    &[CredentialField::secret(
        "bot_token",
        "Bot token",
        "A secret token issued by the provider's bot management service.",
    )],
);
pub const REGISTRATION: McpServerRegistration =
    McpServerRegistration::new("telegram-bot", SERVER_NAME, SERVER_VERSION)
        .with_credentials(CREDENTIAL_REQUIREMENTS);

pub struct TelegramBotServer {
    tools: ToolRegistry,
}

impl TelegramBotServer {
    pub fn new(provider: Arc<dyn TelegramBotProvider>) -> Result<Self, ToolRegistryError> {
        Ok(Self {
            tools: ToolRegistry::try_new(vec![
                Arc::new(GetMeHandler::new(Arc::clone(&provider))),
                Arc::new(GetChatHandler::new(Arc::clone(&provider))),
                Arc::new(GetUpdatesHandler::new(Arc::clone(&provider))),
                Arc::new(SendMessageHandler::new(provider)),
            ])?,
        })
    }

    pub fn with_teloxide() -> Result<Self, ToolRegistryError> {
        Self::new(Arc::new(TeloxideTelegramBotProvider::new()))
    }
}

#[async_trait]
impl McpServer for TelegramBotServer {
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
        self.tools.definitions()
    }

    async fn call_tool(
        &self,
        context: &RequestContext,
        request: CallToolParams,
    ) -> Result<CallToolResult, ServerError> {
        self.tools.call(context, request).await
    }
}
