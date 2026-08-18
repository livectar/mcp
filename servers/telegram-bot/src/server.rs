use async_trait::async_trait;
use mcp_protocol::schemas::{
    lifecycle::{ImplementationInfo, ServerCapabilities},
    tools::{CallToolParams, CallToolResult},
};
use mcp_sdk::{
    errors::{ServerError, ToolRegistryError},
    schemas::{context::RequestContext, tool_definition::ToolDefinition},
    traits::{server::McpServer, tool_registry::ToolRegistry},
};
use std::sync::Arc;

use crate::{
    handlers::{get_chat::GetChatHandler, get_me::GetMeHandler, send_message::SendMessageHandler},
    providers::telegram_bot::{TelegramBotProvider, TeloxideTelegramBotProvider},
};

pub struct TelegramBotServer {
    tools: ToolRegistry,
}

impl TelegramBotServer {
    pub fn new(provider: Arc<dyn TelegramBotProvider>) -> Result<Self, ToolRegistryError> {
        Ok(Self {
            tools: ToolRegistry::try_new(vec![
                Arc::new(GetMeHandler::new(Arc::clone(&provider))),
                Arc::new(GetChatHandler::new(Arc::clone(&provider))),
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
            name: "mcp-telegram-bot".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
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
