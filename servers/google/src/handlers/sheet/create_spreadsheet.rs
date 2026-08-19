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
        sheet::mutation_schema::INITIAL_TAB_PROPERTY,
    },
    providers::sheets::provider::GoogleSheetsProvider,
    schemas::requests::sheets_mutations::CreateSpreadsheetRequest,
};

pub const TOOL_NAME: &str = "sheets_create_spreadsheet";

const INPUT_SCHEMA: ToolInputSchema = ToolInputSchema::object(
    &["title"],
    &[
        ToolInputProperty::string("title", Some(1), Some(256)),
        INITIAL_TAB_PROPERTY,
    ],
);

pub struct CreateSpreadsheetHandler {
    provider: Arc<GoogleSheetsProvider>,
}

impl CreateSpreadsheetHandler {
    pub fn new(provider: Arc<GoogleSheetsProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl ToolHandler for CreateSpreadsheetHandler {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: TOOL_NAME.to_string(),
            description: "Create a Google Sheets spreadsheet with an optional initial tab."
                .to_string(),
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
        let request = decode_required_arguments::<CreateSpreadsheetRequest>(arguments)?;
        request.validate().map_err(ServerError::InvalidArguments)?;
        let credential = authorize_and_credential(context, TOOL_NAME).await?;
        let result = self
            .provider
            .create_spreadsheet(&credential, request)
            .await?;
        success(
            format!("Created spreadsheet {}.", result.spreadsheet_id),
            &result,
        )
    }
}
