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
            append_rows::AppendRowsHandler, clear_range::ClearRangeHandler,
            create_spreadsheet::CreateSpreadsheetHandler, get_spreadsheet::GetSpreadsheetHandler,
            read_cell_text::ReadCellTextHandler, read_range::ReadRangeHandler,
            read_sheet_metadata::ReadSheetMetadataHandler, write_range::WriteRangeHandler,
        },
    },
    providers::{
        common::GoogleApiClient, drive::GoogleDriveProvider, sheets::provider::GoogleSheetsProvider,
    },
};

pub const SERVER_NAME: &str = "mcp-google";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REGISTRATION: McpServerRegistration =
    McpServerRegistration::new("google", SERVER_NAME, SERVER_VERSION);

pub struct GoogleServer {
    tools: ToolRegistry,
}

impl GoogleServer {
    pub fn new(provider: Arc<GoogleApiClient>) -> Result<Self, ToolRegistryError> {
        let drive_provider = Arc::new(GoogleDriveProvider::new(Arc::clone(&provider)));
        let sheets_provider = Arc::new(GoogleSheetsProvider::new(provider));
        let tools = ToolRegistry::try_new(vec![
            Arc::new(ListSpreadsheetsHandler::new(drive_provider)),
            Arc::new(GetSpreadsheetHandler::new(Arc::clone(&sheets_provider))),
            Arc::new(ReadCellTextHandler::new(Arc::clone(&sheets_provider))),
            Arc::new(ReadRangeHandler::new(Arc::clone(&sheets_provider))),
            Arc::new(ReadSheetMetadataHandler::new(Arc::clone(&sheets_provider))),
            Arc::new(CreateSpreadsheetHandler::new(Arc::clone(&sheets_provider))),
            Arc::new(WriteRangeHandler::new(Arc::clone(&sheets_provider))),
            Arc::new(AppendRowsHandler::new(Arc::clone(&sheets_provider))),
            Arc::new(ClearRangeHandler::new(sheets_provider)),
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
