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
    handlers::{
        common::{authorize_and_credential, decode_required_arguments, success},
        sheet::mutation_schema::cell_rows_property,
    },
    providers::sheets::provider::GoogleSheetsProvider,
    schemas::cells::matrix::MUTATION_CELL_FORMAT_DESCRIPTION,
    schemas::requests::sheets_mutations::AppendRowsRequest,
};

pub const TOOL_NAME: &str = "sheets_append_rows";

const INPUT_SCHEMA: ToolInputSchema = ToolInputSchema::object(
    &["spreadsheet_id", "range", "rows"],
    &[
        ToolInputProperty::string("spreadsheet_id", Some(1), Some(256)),
        ToolInputProperty::string("range", Some(1), Some(256)),
        cell_rows_property("rows"),
    ],
);

pub struct AppendRowsHandler {
    provider: Arc<GoogleSheetsProvider>,
}

impl AppendRowsHandler {
    pub fn new(provider: Arc<GoogleSheetsProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl ToolHandler for AppendRowsHandler {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: TOOL_NAME.to_string(),
            description: format!(
                "Append rows to a Google Sheets range; ambiguous outcomes require an explicit retry. {}",
                MUTATION_CELL_FORMAT_DESCRIPTION
            ),
            input_schema: INPUT_SCHEMA,
            annotations: ToolAnnotations {
                read_only_hint: Some(false),
            },
        }
    }

    async fn call(
        &self,
        context: &RequestContext,
        arguments: Option<JsonPayload>,
    ) -> Result<mcp_protocol::schemas::tools::CallToolResult, ServerError> {
        let request = decode_required_arguments::<AppendRowsRequest>(arguments)?;
        let credential = authorize_and_credential(context, TOOL_NAME).await?;
        let result = self.provider.append_rows(&credential, request).await?;
        success(
            format!("Appended {} cells.", result.affected_cell_count),
            &result,
        )
    }
}
