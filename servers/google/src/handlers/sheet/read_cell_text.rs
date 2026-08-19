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
    schemas::{identifiers::limits::TextChunkSize, requests::sheets_read::ReadCellTextRequest},
};

pub const TOOL_NAME: &str = "sheets_read_cell_text";
const VALUE_RENDERING_VALUES: &[&str] = &["formatted", "unformatted", "formula"];

const INPUT_SCHEMA: ToolInputSchema = ToolInputSchema::object(
    &["spreadsheet_id", "cell"],
    &[
        ToolInputProperty::string("spreadsheet_id", Some(1), Some(256)),
        ToolInputProperty::string("cell", Some(1), Some(256)),
        ToolInputProperty::string_enum("value_rendering", VALUE_RENDERING_VALUES),
        ToolInputProperty::integer(
            "chunk_bytes",
            Some(TextChunkSize::MIN as u64),
            Some(TextChunkSize::MAX as u64),
        ),
        ToolInputProperty::continuation_cursor(),
    ],
);

pub struct ReadCellTextHandler {
    provider: Arc<GoogleSheetsProvider>,
}

impl ReadCellTextHandler {
    pub fn new(provider: Arc<GoogleSheetsProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl ToolHandler for ReadCellTextHandler {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: TOOL_NAME.to_string(),
            description: "Read one Google Sheets text or formula cell in lossless chunks."
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
        let request = decode_required_arguments::<ReadCellTextRequest>(arguments)?;
        let credential = authorize_and_credential(context, TOOL_NAME).await?;
        let result = self.provider.read_cell_text(&credential, request).await?;
        success(
            format!("Read {} bytes from the requested cell.", result.text.len()),
            &result,
        )
    }
}
