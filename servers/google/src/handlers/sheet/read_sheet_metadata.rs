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
    handlers::common::{authorize_and_credential, decode_required_arguments, success},
    providers::sheets::provider::GoogleSheetsProvider,
    schemas::requests::sheets_read::ReadSheetMetadataRequest,
};

pub const TOOL_NAME: &str = "sheets_read_sheet_metadata";

const INPUT_SCHEMA: ToolInputSchema = ToolInputSchema::object(
    &["spreadsheet_id"],
    &[ToolInputProperty::string(
        "spreadsheet_id",
        Some(1),
        Some(256),
    )],
);

pub struct ReadSheetMetadataHandler {
    provider: Arc<GoogleSheetsProvider>,
}

impl ReadSheetMetadataHandler {
    pub fn new(provider: Arc<GoogleSheetsProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl ToolHandler for ReadSheetMetadataHandler {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: TOOL_NAME.to_string(),
            description:
                "Return Google Sheets tab IDs, titles, dimensions, and frozen-pane metadata."
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
        let request = decode_required_arguments::<ReadSheetMetadataRequest>(arguments)?;
        let credential = authorize_and_credential(context, TOOL_NAME).await?;
        let result = self
            .provider
            .read_sheet_metadata(&credential, request)
            .await?;
        success(
            format!("Read metadata for {} sheet tabs.", result.tabs.len()),
            &result,
        )
    }
}
