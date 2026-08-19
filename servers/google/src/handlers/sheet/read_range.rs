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
    schemas::{identifiers::limits::CellLimit, requests::sheets_read::ReadRangeRequest},
};

pub const TOOL_NAME: &str = "sheets_read_range";
const VALUE_RENDERING_VALUES: &[&str] = &["formatted", "unformatted", "formula"];

const INPUT_SCHEMA: ToolInputSchema = ToolInputSchema::object(
    &["spreadsheet_id", "range"],
    &[
        ToolInputProperty::string("spreadsheet_id", Some(1), Some(256)),
        ToolInputProperty::string("range", Some(1), Some(256)),
        ToolInputProperty::string_enum("value_rendering", VALUE_RENDERING_VALUES),
        ToolInputProperty::integer("max_cells", Some(1), Some(CellLimit::MAX as u64)),
        ToolInputProperty::continuation_cursor(),
    ],
);

pub struct ReadRangeHandler {
    provider: Arc<GoogleSheetsProvider>,
}

impl ReadRangeHandler {
    pub fn new(provider: Arc<GoogleSheetsProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl ToolHandler for ReadRangeHandler {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: TOOL_NAME.to_string(),
            description: "Read a Google Sheets A1 range with lossless cursor pagination. Continue until next_cursor is absent.".to_string(),
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
        let request = decode_required_arguments::<ReadRangeRequest>(arguments)?;
        let requested_range = request.range.clone();
        let credential = authorize_and_credential(context, TOOL_NAME).await?;
        let result = self.provider.read_range(&credential, request).await?;
        success(
            format!(
                "Read {} cells from {}.",
                result.returned_cell_count,
                requested_range.as_str()
            ),
            &result,
        )
    }
}
