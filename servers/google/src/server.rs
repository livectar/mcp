use async_trait::async_trait;
use mcp_protocol::schemas::{
    lifecycle::{ImplementationInfo, ServerCapabilities},
    tools::{CallToolParams, CallToolResult},
};
use mcp_sdk::{
    errors::{ServerError, ToolRegistryError},
    schemas::{context::RequestContext, tool_definition::ToolDefinition},
    traits::{registry::McpServerRegistration, server::McpServer, tool_registry::ToolRegistry},
};
use std::sync::Arc;

use crate::{
    handlers::{
        drive::list_spreadsheets::ListSpreadsheetsHandler,
        sheet::{
            get_spreadsheet::GetSpreadsheetHandler, read_cell_text::ReadCellTextHandler,
            read_range::ReadRangeHandler, read_sheet_metadata::ReadSheetMetadataHandler,
        },
    },
    providers::common::GoogleProvider,
};

pub const SERVER_NAME: &str = "mcp-google";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REGISTRATION: McpServerRegistration =
    McpServerRegistration::new("google", SERVER_NAME, SERVER_VERSION);

pub struct GoogleServer {
    tools: ToolRegistry,
}

impl GoogleServer {
    pub fn new(provider: Arc<dyn GoogleProvider>) -> Result<Self, ToolRegistryError> {
        let tools = ToolRegistry::try_new(vec![
            Arc::new(ListSpreadsheetsHandler::new(Arc::clone(&provider))),
            Arc::new(GetSpreadsheetHandler::new(Arc::clone(&provider))),
            Arc::new(ReadCellTextHandler::new(Arc::clone(&provider))),
            Arc::new(ReadRangeHandler::new(Arc::clone(&provider))),
            Arc::new(ReadSheetMetadataHandler::new(provider)),
        ])?;
        Ok(Self { tools })
    }
}

#[async_trait]
impl McpServer for GoogleServer {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::common::GoogleApiClient;

    #[test]
    fn registers_the_phase_one_read_tools() {
        let server = GoogleServer::new(Arc::new(GoogleApiClient::default())).unwrap();
        let mut names = server
            .tools()
            .into_iter()
            .map(|tool| tool.name)
            .collect::<Vec<_>>();
        names.sort();
        assert_eq!(
            names,
            vec![
                "sheets_get_spreadsheet",
                "sheets_list_spreadsheets",
                "sheets_read_cell_text",
                "sheets_read_range",
                "sheets_read_sheet_metadata",
            ]
        );
    }
}
