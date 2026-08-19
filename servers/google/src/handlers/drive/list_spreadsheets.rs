use async_trait::async_trait;
use mcp_protocol::schemas::json_payload::JsonPayload;
use mcp_sdk::schemas::tool_schema::{ToolInputProperty, ToolInputSchema};
use mcp_sdk::{
    errors::ServerError,
    schemas::{
        context::RequestContext,
        tool_definition::{ToolAnnotations, ToolDefinition},
    },
    traits::server::ToolHandler,
};
use std::sync::Arc;

use crate::{
    handlers::common::{authorize_and_credential, decode_arguments, success},
    providers::drive::GoogleDriveProvider,
    schemas::{identifiers::limits::PageSize, requests::drive::ListSpreadsheetsRequest},
};

pub const TOOL_NAME: &str = "sheets_list_spreadsheets";

const INPUT_SCHEMA: ToolInputSchema = ToolInputSchema::object(
    &[],
    &[
        ToolInputProperty::string("name_contains", Some(1), Some(256)),
        ToolInputProperty::string("query", Some(1), Some(512)),
        ToolInputProperty::integer("page_size", Some(1), Some(PageSize::MAX as u64)),
        ToolInputProperty::page_cursor(),
    ],
);

pub struct ListSpreadsheetsHandler {
    provider: Arc<GoogleDriveProvider>,
}

impl ListSpreadsheetsHandler {
    pub fn new(provider: Arc<GoogleDriveProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl ToolHandler for ListSpreadsheetsHandler {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: TOOL_NAME.to_string(),
            description: "List accessible Google Sheets spreadsheets with bounded pagination."
                .to_string(),
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
    ) -> Result<mcp_protocol::schemas::tools::CallToolResult, ServerError> {
        let request = decode_arguments::<ListSpreadsheetsRequest>(arguments)?;
        let credential = authorize_and_credential(context, TOOL_NAME).await?;
        let result = self
            .provider
            .list_spreadsheets(&credential, request)
            .await?;
        success(
            format!(
                "Listed {} accessible Google Sheets spreadsheets.",
                result.spreadsheets.len()
            ),
            &result,
        )
    }
}
