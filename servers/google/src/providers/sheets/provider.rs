use mcp_sdk::schemas::{credentials::ProviderCredential, pagination::OpaqueCursor};
use std::sync::Arc;

use crate::{
    errors::{GoogleMutationOperation, GoogleProviderError},
    schemas::{
        cells::{
            cursors::{RangeCursorPayload, TextChunkCursorPayload},
            results::{
                ReadCellTextIdentity, ReadCellTextResult, ReadRangeIdentity, ReadRangeResult,
            },
            text::{split_text_chunk, CellTextKind},
            values::CellValue,
        },
        identifiers::{ids::SpreadsheetId, limits::CellLimit},
        provider::sheets::{
            metadata::SheetsSpreadsheetResponse,
            mutations::{
                AppendValuesResponse, ClearValuesResponse, CreateSpreadsheetBody,
                GoogleInsertDataOption, GoogleMajorDimension, GoogleValueInputOption,
                UpdateValuesBody, UpdateValuesResponse,
            },
            values::SheetsValueRange,
        },
        requests::{
            sheets_mutations::{
                AppendRowsRequest, ClearRangeRequest, CreateSpreadsheetRequest, WriteRangeRequest,
            },
            sheets_read::{
                GetSpreadsheetRequest, ReadCellTextRequest, ReadRangeRequest,
                ReadSheetMetadataRequest,
            },
        },
        results::{
            mutations::{MutationOperation, MutationOutcome, MutationResult, MutationSummary},
            sheets::{SheetMetadataResult, SpreadsheetMetadataResult},
        },
    },
};

use super::super::common::{
    invalid_provider_response, ApiService, GoogleApiClient, MutationRetryPolicy,
};
use super::{
    conversion::{self, convert_values},
    cursor::{decode_range_cursor, decode_text_cursor},
    paging::RangePage,
    types::ClearValuesBody,
    validation::{invalid_request, parse_response_range, range_cell_count, resolve_tab},
};

#[derive(Clone)]
pub struct GoogleSheetsProvider {
    client: Arc<GoogleApiClient>,
}

impl GoogleSheetsProvider {
    pub fn new(client: Arc<GoogleApiClient>) -> Self {
        Self { client }
    }

    async fn sheets_metadata(
        &self,
        credential: &ProviderCredential,
        spreadsheet_id: &SpreadsheetId,
    ) -> Result<SheetsSpreadsheetResponse, GoogleProviderError> {
        let path = format!("spreadsheets/{spreadsheet_id}");
        let fields = "spreadsheetId,properties(title),sheets(properties(sheetId,title,index,sheetType,gridProperties(rowCount,columnCount,frozenRowCount,frozenColumnCount)))";
        self.client
            .get_json(
                ApiService::Sheets,
                &path,
                vec![("fields", fields.to_string())],
                credential,
                "spreadsheets.get",
            )
            .await
    }

    pub async fn get_spreadsheet(
        &self,
        credential: &ProviderCredential,
        request: GetSpreadsheetRequest,
    ) -> Result<SpreadsheetMetadataResult, GoogleProviderError> {
        conversion::convert_metadata(
            self.sheets_metadata(credential, &request.spreadsheet_id)
                .await?,
        )
    }

    pub async fn read_range(
        &self,
        credential: &ProviderCredential,
        request: ReadRangeRequest,
    ) -> Result<ReadRangeResult, GoogleProviderError> {
        let metadata = conversion::convert_metadata(
            self.sheets_metadata(credential, &request.spreadsheet_id)
                .await?,
        )?;
        let tab = resolve_tab(&metadata.tabs, request.range.sheet_name().as_deref())?;
        let bounds = request
            .range
            .bounds()
            .map_err(invalid_request)?
            .resolve(tab.dimensions.rows, tab.dimensions.columns)
            .map_err(invalid_request)?;
        let max_cells = self.effective_cell_limit(request.max_cells)?;
        let (row_offset, column_offset) = decode_range_cursor(&request, max_cells)?;
        let page = RangePage::new(bounds, row_offset, column_offset, max_cells)?;
        let page_range = request
            .range
            .subrange(
                page.start_row,
                page.start_column,
                page.row_count,
                page.column_count,
            )
            .map_err(invalid_request)?;
        let path = format!(
            "spreadsheets/{}/values/{}",
            request.spreadsheet_id,
            page_range.as_str()
        );
        let response: SheetsValueRange = self
            .client
            .get_json(
                ApiService::Sheets,
                &path,
                vec![(
                    "valueRenderOption",
                    request.value_rendering.provider_value().to_string(),
                )],
                credential,
                "spreadsheets.values.get",
            )
            .await?;
        let values = convert_values(
            response,
            request.value_rendering,
            page.start_row,
            page.start_column,
            max_cells,
        )?;
        let returned_cell_count = values
            .iter()
            .map(|row| u32::try_from(row.len()).unwrap_or(u32::MAX))
            .sum();
        let next_cursor = page.next_offsets().map(|(next_row, next_column)| {
            OpaqueCursor::encode(&RangeCursorPayload {
                spreadsheet_id: request.spreadsheet_id.clone(),
                range: request.range.clone(),
                value_rendering: request.value_rendering,
                max_cells,
                row_offset: next_row,
                column_offset: next_column,
            })
        });
        let next_cursor = next_cursor
            .transpose()
            .map_err(|error| invalid_request(error.to_string()))?;
        Ok(ReadRangeResult {
            identity: ReadRangeIdentity {
                spreadsheet_id: request.spreadsheet_id,
                tab: tab.identity,
                requested_range: request.range,
            },
            page_range,
            values,
            returned_cell_count,
            next_cursor,
        })
    }

    pub async fn read_cell_text(
        &self,
        credential: &ProviderCredential,
        request: ReadCellTextRequest,
    ) -> Result<ReadCellTextResult, GoogleProviderError> {
        if !request.cell.is_single_cell() {
            return Err(invalid_request("cell must identify exactly one A1 cell"));
        }
        let metadata = conversion::convert_metadata(
            self.sheets_metadata(credential, &request.spreadsheet_id)
                .await?,
        )?;
        let tab = resolve_tab(&metadata.tabs, request.cell.sheet_name().as_deref())?;
        let chunk_bytes = request.chunk_bytes.unwrap_or_default();
        let offset = decode_text_cursor(&request, chunk_bytes)?;
        let path = format!(
            "spreadsheets/{}/values/{}",
            request.spreadsheet_id,
            request.cell.as_str()
        );
        let response: SheetsValueRange = self
            .client
            .get_json(
                ApiService::Sheets,
                &path,
                vec![(
                    "valueRenderOption",
                    request.value_rendering.provider_value().to_string(),
                )],
                credential,
                "spreadsheets.values.get",
            )
            .await?;
        let value = response
            .values
            .into_iter()
            .next()
            .and_then(|row| row.into_iter().next())
            .map(|cell| cell.into_cell_value(request.value_rendering))
            .unwrap_or(CellValue::Empty);
        let (kind, value) = match value {
            CellValue::Text(value) => (CellTextKind::Text, value),
            CellValue::Formula(value) => (CellTextKind::Formula, value),
            CellValue::Empty | CellValue::Number(_) | CellValue::Boolean(_) => {
                return Err(GoogleProviderError::CellNotText {
                    cell: request.cell.as_str().to_string(),
                });
            }
        };
        let (text, next_offset) =
            split_text_chunk(&value, offset, chunk_bytes).map_err(invalid_request)?;
        let next_cursor = next_offset
            .map(|offset| {
                OpaqueCursor::encode(&TextChunkCursorPayload {
                    spreadsheet_id: request.spreadsheet_id.clone(),
                    cell: request.cell.clone(),
                    value_rendering: request.value_rendering,
                    chunk_bytes,
                    offset,
                })
            })
            .transpose()
            .map_err(|error| invalid_request(error.to_string()))?;
        Ok(ReadCellTextResult {
            identity: ReadCellTextIdentity {
                spreadsheet_id: request.spreadsheet_id,
                tab: tab.identity,
                cell: request.cell,
            },
            kind,
            text,
            next_cursor,
        })
    }

    pub async fn read_sheet_metadata(
        &self,
        credential: &ProviderCredential,
        request: ReadSheetMetadataRequest,
    ) -> Result<SheetMetadataResult, GoogleProviderError> {
        let metadata = conversion::convert_metadata(
            self.sheets_metadata(credential, &request.spreadsheet_id)
                .await?,
        )?;
        Ok(SheetMetadataResult {
            spreadsheet_id: metadata.spreadsheet_id,
            tabs: metadata.tabs,
        })
    }

    fn effective_cell_limit(
        &self,
        requested: Option<CellLimit>,
    ) -> Result<CellLimit, GoogleProviderError> {
        let max_cells = requested.unwrap_or(self.client.config.max_cells);
        if max_cells.get() > self.client.config.max_cells.get() {
            return Err(invalid_request(
                "max_cells exceeds the configured safety limit",
            ));
        }
        Ok(max_cells)
    }
}

impl GoogleSheetsProvider {
    pub async fn create_spreadsheet(
        &self,
        credential: &ProviderCredential,
        request: CreateSpreadsheetRequest,
    ) -> Result<MutationResult, GoogleProviderError> {
        let body = CreateSpreadsheetBody::from_request(
            request.title.as_str(),
            request.initial_tab.as_ref(),
        );
        let response: SheetsSpreadsheetResponse = self
            .client
            .post_json(
                ApiService::Sheets,
                "spreadsheets",
                vec![
                    (
                        "fields",
                        "spreadsheetId,properties(title),sheets(properties(sheetId,title,index,sheetType,gridProperties(rowCount,columnCount,frozenRowCount,frozenColumnCount)))".to_string(),
                    ),
                ],
                &body,
                credential,
                "spreadsheets.create",
                MutationRetryPolicy::NonIdempotent {
                    operation: GoogleMutationOperation::CreateSpreadsheet,
                },
            )
            .await?;
        let metadata = conversion::convert_created_metadata(response)?;
        let tab = metadata.tabs.first().map(|tab| tab.identity.clone());
        Ok(MutationResult {
            operation: MutationOperation::CreateSpreadsheet,
            outcome: MutationOutcome::Applied,
            spreadsheet_id: metadata.spreadsheet_id,
            tab,
            range: None,
            affected_cell_count: 0,
            failed_cell_count: 0,
            summary: MutationSummary::Created {
                title: request.title,
                tab_count: u32::try_from(metadata.tabs.len()).unwrap_or(u32::MAX),
            },
        })
    }

    pub async fn write_range(
        &self,
        credential: &ProviderCredential,
        request: WriteRangeRequest,
    ) -> Result<MutationResult, GoogleProviderError> {
        request.validate().map_err(invalid_request)?;
        let metadata = self
            .mutation_metadata(credential, &request.spreadsheet_id)
            .await?;
        let tab = resolve_tab(&metadata.tabs, request.range.sheet_name().as_deref())?;
        let request_range = request.range.clone();
        let values = request.values;
        let body = UpdateValuesBody {
            range: request_range.as_str(),
            major_dimension: GoogleMajorDimension::Rows,
            values: values
                .rows()
                .iter()
                .cloned()
                .map(|row| row.into_iter().map(Into::into).collect())
                .collect(),
        };
        let response: UpdateValuesResponse = self
            .client
            .put_json(
                ApiService::Sheets,
                &format!(
                    "spreadsheets/{}/values/{}",
                    request.spreadsheet_id,
                    request_range.as_str()
                ),
                vec![
                    (
                        "valueInputOption",
                        GoogleValueInputOption::UserEntered.as_str().to_string(),
                    ),
                    ("includeValuesInResponse", "false".to_string()),
                ],
                &body,
                credential,
                "spreadsheets.values.update",
            )
            .await?;
        let range = response
            .updated_range
            .map(parse_response_range)
            .transpose()?;
        let affected_cell_count = response
            .updated_cells
            .unwrap_or_else(|| u32::try_from(values.cell_count()).unwrap_or(u32::MAX));
        Ok(MutationResult {
            operation: MutationOperation::WriteRange,
            outcome: MutationOutcome::Applied,
            spreadsheet_id: request.spreadsheet_id,
            tab: Some(tab.identity),
            range: Some(range.unwrap_or(request_range)),
            affected_cell_count,
            failed_cell_count: 0,
            summary: MutationSummary::Written {
                updated_rows: response
                    .updated_rows
                    .unwrap_or_else(|| u32::try_from(values.row_count()).unwrap_or(u32::MAX)),
                updated_columns: response
                    .updated_columns
                    .unwrap_or_else(|| u32::try_from(values.column_count()).unwrap_or(u32::MAX)),
            },
        })
    }

    pub async fn append_rows(
        &self,
        credential: &ProviderCredential,
        request: AppendRowsRequest,
    ) -> Result<MutationResult, GoogleProviderError> {
        let metadata = self
            .mutation_metadata(credential, &request.spreadsheet_id)
            .await?;
        let tab = resolve_tab(&metadata.tabs, request.range.sheet_name().as_deref())?;
        let request_range = request.range.clone();
        let rows = request.rows;
        let body = UpdateValuesBody {
            range: request_range.as_str(),
            major_dimension: GoogleMajorDimension::Rows,
            values: rows
                .rows()
                .iter()
                .cloned()
                .map(|row| row.into_iter().map(Into::into).collect())
                .collect(),
        };
        let response: AppendValuesResponse = self
            .client
            .post_json(
                ApiService::Sheets,
                &format!(
                    "spreadsheets/{}/values/{}:append",
                    request.spreadsheet_id,
                    request_range.as_str()
                ),
                vec![
                    (
                        "valueInputOption",
                        GoogleValueInputOption::UserEntered.as_str().to_string(),
                    ),
                    (
                        "insertDataOption",
                        GoogleInsertDataOption::InsertRows.as_str().to_string(),
                    ),
                    ("includeValuesInResponse", "false".to_string()),
                ],
                &body,
                credential,
                "spreadsheets.values.append",
                MutationRetryPolicy::NonIdempotent {
                    operation: GoogleMutationOperation::AppendRows,
                },
            )
            .await?;
        let range = response
            .updates
            .updated_range
            .map(parse_response_range)
            .transpose()?
            .unwrap_or(request_range);
        let affected_cell_count = response
            .updates
            .updated_cells
            .unwrap_or_else(|| u32::try_from(rows.cell_count()).unwrap_or(u32::MAX));
        Ok(MutationResult {
            operation: MutationOperation::AppendRows,
            outcome: MutationOutcome::Applied,
            spreadsheet_id: request.spreadsheet_id,
            tab: Some(tab.identity),
            range: Some(range),
            affected_cell_count,
            failed_cell_count: 0,
            summary: MutationSummary::Appended {
                updated_rows: response
                    .updates
                    .updated_rows
                    .unwrap_or_else(|| u32::try_from(rows.rows().len()).unwrap_or(u32::MAX)),
                updated_columns: response.updates.updated_columns.unwrap_or_else(|| {
                    u32::try_from(rows.rows().iter().map(Vec::len).max().unwrap_or(0))
                        .unwrap_or(u32::MAX)
                }),
            },
        })
    }

    pub async fn clear_range(
        &self,
        credential: &ProviderCredential,
        request: ClearRangeRequest,
    ) -> Result<MutationResult, GoogleProviderError> {
        request.validate().map_err(invalid_request)?;
        let metadata = self
            .mutation_metadata(credential, &request.spreadsheet_id)
            .await?;
        let tab = resolve_tab(&metadata.tabs, request.range.sheet_name().as_deref())?;
        let request_range = request.range.clone();
        let response: ClearValuesResponse = self
            .client
            .post_json(
                ApiService::Sheets,
                &format!(
                    "spreadsheets/{}/values/{}:clear",
                    request.spreadsheet_id,
                    request_range.as_str()
                ),
                Vec::new(),
                &ClearValuesBody {},
                credential,
                "spreadsheets.values.clear",
                MutationRetryPolicy::Safe,
            )
            .await?;
        let response_spreadsheet_id =
            SpreadsheetId::new(response.spreadsheet_id).map_err(invalid_provider_response)?;
        if response_spreadsheet_id != request.spreadsheet_id {
            return Err(invalid_provider_response(
                "clear response spreadsheet ID does not match the request".to_string(),
            ));
        }
        let cleared_range = parse_response_range(response.cleared_range)?;
        let affected_cell_count = range_cell_count(&cleared_range)?;
        Ok(MutationResult {
            operation: MutationOperation::ClearRange,
            outcome: MutationOutcome::Applied,
            spreadsheet_id: request.spreadsheet_id,
            tab: Some(tab.identity),
            range: Some(cleared_range.clone()),
            affected_cell_count,
            failed_cell_count: 0,
            summary: MutationSummary::Cleared { cleared_range },
        })
    }

    async fn mutation_metadata(
        &self,
        credential: &ProviderCredential,
        spreadsheet_id: &SpreadsheetId,
    ) -> Result<SpreadsheetMetadataResult, GoogleProviderError> {
        self.get_spreadsheet(
            credential,
            GetSpreadsheetRequest {
                spreadsheet_id: spreadsheet_id.clone(),
            },
        )
        .await
    }
}
