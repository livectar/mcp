use mcp_sdk::schemas::{credentials::ProviderCredential, pagination::OpaqueCursor};

use crate::{
    errors::GoogleProviderError,
    schemas::{
        identifiers::ids::SpreadsheetId,
        provider::DriveFilesResponse,
        requests::ListSpreadsheetsRequest,
        results::{ListSpreadsheetsResult, SpreadsheetListItem},
    },
};

use super::common::{escape_drive_literal, invalid_provider_response, ApiService, GoogleApiClient};

impl GoogleApiClient {
    pub(crate) async fn list_spreadsheets_impl(
        &self,
        credential: &ProviderCredential,
        request: ListSpreadsheetsRequest,
    ) -> Result<ListSpreadsheetsResult, GoogleProviderError> {
        let page_size = request
            .page_size
            .unwrap_or(self.config.default_page_size)
            .get()
            .min(self.config.max_page_size.get());
        let mut clauses = vec![
            "mimeType = 'application/vnd.google-apps.spreadsheet'".to_string(),
            "trashed = false".to_string(),
        ];
        if let Some(name) = request.name_contains {
            clauses.push(format!(
                "name contains '{}'",
                escape_drive_literal(name.as_str())
            ));
        }
        if let Some(query) = request.query {
            clauses.push(format!("({})", query.as_str()));
        }
        let mut query = vec![
            ("q", clauses.join(" and ")),
            ("pageSize", page_size.to_string()),
            (
                "fields",
                "nextPageToken,files(id,name,webViewLink)".to_string(),
            ),
        ];
        if let Some(cursor) = request.page_cursor {
            query.push(("pageToken", cursor.as_str().to_string()));
        }
        let response: DriveFilesResponse = self
            .get_json(
                ApiService::Drive,
                "files",
                query,
                credential,
                "drive.files.list",
            )
            .await?;
        let spreadsheets = response
            .files
            .into_iter()
            .map(|file| {
                Ok(SpreadsheetListItem {
                    spreadsheet_id: SpreadsheetId::new(file.id)
                        .map_err(invalid_provider_response)?,
                    name: file.name,
                    web_url: file.web_view_link,
                })
            })
            .collect::<Result<Vec<_>, GoogleProviderError>>()?;
        let next_cursor = response
            .next_page_token
            .map(OpaqueCursor::new)
            .transpose()
            .map_err(|error| invalid_provider_response(error.to_string()))?;
        Ok(ListSpreadsheetsResult {
            spreadsheets,
            next_cursor,
        })
    }
}
