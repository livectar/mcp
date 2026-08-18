use mcp_sdk::schemas::{credentials::ProviderCredential, pagination::OpaqueCursor};

use crate::{
    errors::{GoogleErrorAction, GoogleErrorCategory, GoogleProviderError},
    schemas::{
        cells::{
            split_text_chunk, CellTextKind, CellValue, RangeCursorPayload, ReadCellTextIdentity,
            ReadCellTextResult, ReadRangeIdentity, ReadRangeResult, TextChunkCursorPayload,
        },
        identifiers::{
            ids::{SheetId, SpreadsheetId},
            limits::{CellLimit, TextChunkSize},
            ranges::ResolvedA1RangeBounds,
        },
        provider::{SheetsSheet, SheetsSpreadsheetResponse, SheetsValueRange},
        requests::{
            GetSpreadsheetRequest, ReadCellTextRequest, ReadRangeRequest, ReadSheetMetadataRequest,
        },
        results::{
            FrozenPaneMetadata, GridDimensions, SheetIdentity, SheetMetadataResult,
            SheetTabMetadata, SpreadsheetMetadataResult,
        },
    },
};

use super::common::{invalid_provider_response, ApiService, GoogleApiClient, MAX_CELL_TEXT_BYTES};

impl GoogleApiClient {
    async fn sheets_metadata(
        &self,
        credential: &ProviderCredential,
        spreadsheet_id: &SpreadsheetId,
    ) -> Result<SheetsSpreadsheetResponse, GoogleProviderError> {
        let path = format!("spreadsheets/{spreadsheet_id}");
        let fields = "spreadsheetId,properties(title),sheets(properties(sheetId,title,index,sheetType,gridProperties(rowCount,columnCount,frozenRowCount,frozenColumnCount)))";
        self.get_json(
            ApiService::Sheets,
            &path,
            vec![("fields", fields.to_string())],
            credential,
            "spreadsheets.get",
        )
        .await
    }

    fn convert_metadata(
        &self,
        response: SheetsSpreadsheetResponse,
    ) -> Result<SpreadsheetMetadataResult, GoogleProviderError> {
        let spreadsheet_id =
            SpreadsheetId::new(response.spreadsheet_id).map_err(invalid_provider_response)?;
        let tabs = response
            .sheets
            .into_iter()
            .map(convert_sheet)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(SpreadsheetMetadataResult {
            spreadsheet_id,
            title: response.properties.title,
            tabs,
        })
    }

    pub(crate) async fn get_spreadsheet_impl(
        &self,
        credential: &ProviderCredential,
        request: GetSpreadsheetRequest,
    ) -> Result<SpreadsheetMetadataResult, GoogleProviderError> {
        self.convert_metadata(
            self.sheets_metadata(credential, &request.spreadsheet_id)
                .await?,
        )
    }

    pub(crate) async fn read_range_impl(
        &self,
        credential: &ProviderCredential,
        request: ReadRangeRequest,
    ) -> Result<ReadRangeResult, GoogleProviderError> {
        let metadata = self.convert_metadata(
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

    pub(crate) async fn read_cell_text_impl(
        &self,
        credential: &ProviderCredential,
        request: ReadCellTextRequest,
    ) -> Result<ReadCellTextResult, GoogleProviderError> {
        if !request.cell.is_single_cell() {
            return Err(invalid_request("cell must identify exactly one A1 cell"));
        }
        let metadata = self.convert_metadata(
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

    pub(crate) async fn read_sheet_metadata_impl(
        &self,
        credential: &ProviderCredential,
        request: ReadSheetMetadataRequest,
    ) -> Result<SheetMetadataResult, GoogleProviderError> {
        let metadata = self.convert_metadata(
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
        let max_cells = requested.unwrap_or(self.config.max_cells);
        if max_cells.get() > self.config.max_cells.get() {
            return Err(invalid_request(
                "max_cells exceeds the configured safety limit",
            ));
        }
        Ok(max_cells)
    }
}

#[derive(Debug, Clone, Copy)]
struct RangePage {
    bounds: ResolvedA1RangeBounds,
    row_offset: u32,
    column_offset: u32,
    start_row: u32,
    start_column: u32,
    row_count: u32,
    column_count: u32,
}

impl RangePage {
    fn new(
        bounds: ResolvedA1RangeBounds,
        row_offset: u32,
        column_offset: u32,
        max_cells: CellLimit,
    ) -> Result<Self, GoogleProviderError> {
        if row_offset >= bounds.row_count() || column_offset >= bounds.column_count() {
            return Err(invalid_request("range continuation cursor is exhausted"));
        }
        let remaining_rows = bounds.row_count() - row_offset;
        let remaining_columns = bounds.column_count() - column_offset;
        let column_count = remaining_columns.min(max_cells.get());
        let row_count = remaining_rows.min((max_cells.get() / column_count).max(1));
        Ok(Self {
            bounds,
            row_offset,
            column_offset,
            start_row: bounds.start_row + row_offset,
            start_column: bounds.start_column + column_offset,
            row_count,
            column_count,
        })
    }

    fn next_offsets(self) -> Option<(u32, u32)> {
        if self.column_offset + self.column_count < self.bounds.column_count() {
            Some((self.row_offset, self.column_offset + self.column_count))
        } else if self.row_offset + self.row_count < self.bounds.row_count() {
            Some((self.row_offset + self.row_count, 0))
        } else {
            None
        }
    }
}

fn convert_values(
    response: SheetsValueRange,
    rendering: crate::schemas::cells::ValueRenderMode,
    start_row: u32,
    start_column: u32,
    max_cells: CellLimit,
) -> Result<Vec<Vec<CellValue>>, GoogleProviderError> {
    let mut count = 0_u32;
    let mut values = Vec::new();
    for (row_index, row) in response.values.into_iter().enumerate() {
        let row_number = start_row + u32::try_from(row_index).unwrap_or(u32::MAX);
        let mut converted_row = Vec::new();
        for (column_index, cell) in row.into_iter().enumerate() {
            count = count.saturating_add(1);
            if count > max_cells.get() {
                return Err(GoogleProviderError::InvalidResponse {
                    message: "Sheets returned more cells than the requested page limit".to_string(),
                });
            }
            let value = cell.into_cell_value(rendering);
            let text_bytes = match &value {
                CellValue::Text(value) | CellValue::Formula(value) => value.len(),
                CellValue::Empty | CellValue::Number(_) | CellValue::Boolean(_) => 0,
            };
            if text_bytes > MAX_CELL_TEXT_BYTES {
                return Err(GoogleProviderError::CellTooLarge {
                    row: row_number,
                    column: start_column + u32::try_from(column_index).unwrap_or(u32::MAX),
                    max_bytes: MAX_CELL_TEXT_BYTES,
                });
            }
            converted_row.push(value);
        }
        if !converted_row.is_empty() {
            values.push(converted_row);
        }
    }
    Ok(values)
}

fn decode_range_cursor(
    request: &ReadRangeRequest,
    max_cells: CellLimit,
) -> Result<(u32, u32), GoogleProviderError> {
    let Some(cursor) = request.continuation_cursor.as_ref() else {
        return Ok((0, 0));
    };
    let payload = cursor
        .decode::<RangeCursorPayload>()
        .map_err(|error| invalid_request(error.to_string()))?;
    if payload.spreadsheet_id != request.spreadsheet_id
        || payload.range != request.range
        || payload.value_rendering != request.value_rendering
        || payload.max_cells != max_cells
    {
        return Err(invalid_request(
            "range continuation cursor does not match the requested read",
        ));
    }
    Ok((payload.row_offset, payload.column_offset))
}

fn decode_text_cursor(
    request: &ReadCellTextRequest,
    chunk_bytes: TextChunkSize,
) -> Result<u32, GoogleProviderError> {
    let Some(cursor) = request.continuation_cursor.as_ref() else {
        return Ok(0);
    };
    let payload = cursor
        .decode::<TextChunkCursorPayload>()
        .map_err(|error| invalid_request(error.to_string()))?;
    if payload.spreadsheet_id != request.spreadsheet_id
        || payload.cell != request.cell
        || payload.value_rendering != request.value_rendering
        || payload.chunk_bytes != chunk_bytes
    {
        return Err(invalid_request(
            "text continuation cursor does not match the requested read",
        ));
    }
    Ok(payload.offset)
}

fn convert_sheet(sheet: SheetsSheet) -> Result<SheetTabMetadata, GoogleProviderError> {
    let identity = SheetIdentity {
        sheet_id: SheetId::new(sheet.properties.sheet_id).map_err(invalid_provider_response)?,
        title: sheet.properties.title,
    };
    let grid = sheet.properties.grid_properties;
    Ok(SheetTabMetadata {
        identity,
        index: sheet.properties.index,
        sheet_type: sheet.properties.sheet_type,
        dimensions: GridDimensions {
            rows: grid.as_ref().and_then(|value| value.row_count),
            columns: grid.as_ref().and_then(|value| value.column_count),
        },
        frozen_panes: FrozenPaneMetadata {
            rows: grid.as_ref().and_then(|value| value.frozen_row_count),
            columns: grid.and_then(|value| value.frozen_column_count),
        },
    })
}

fn resolve_tab(
    tabs: &[SheetTabMetadata],
    requested_title: Option<&str>,
) -> Result<SheetTabMetadata, GoogleProviderError> {
    let tab = requested_title
        .and_then(|title| tabs.iter().find(|tab| tab.identity.title == title))
        .or_else(|| tabs.first())
        .ok_or_else(|| GoogleProviderError::Api {
            category: GoogleErrorCategory::NotFound,
            status: None,
            message: "spreadsheet has no readable sheet tabs".to_string(),
            action: GoogleErrorAction::CheckSpreadsheetId,
            retry_after_seconds: None,
        })?;
    if requested_title.is_some_and(|title| tab.identity.title != title) {
        return Err(GoogleProviderError::Api {
            category: GoogleErrorCategory::NotFound,
            status: None,
            message: "requested sheet tab was not found".to_string(),
            action: GoogleErrorAction::CheckSpreadsheetId,
            retry_after_seconds: None,
        });
    }
    Ok(tab.clone())
}

fn invalid_request(message: impl Into<String>) -> GoogleProviderError {
    GoogleProviderError::Api {
        category: GoogleErrorCategory::InvalidRequest,
        status: None,
        message: message.into(),
        action: GoogleErrorAction::CheckRequest,
        retry_after_seconds: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{convert_values, MAX_CELL_TEXT_BYTES};
    use crate::{
        errors::GoogleProviderError,
        schemas::{
            cells::ValueRenderMode,
            identifiers::limits::CellLimit,
            provider::{GoogleRawCell, SheetsValueRange},
        },
    };

    #[test]
    fn rejects_oversized_cell_instead_of_truncating_it() {
        let response = SheetsValueRange {
            range: None,
            major_dimension: None,
            values: vec![vec![GoogleRawCell::Text(
                "x".repeat(MAX_CELL_TEXT_BYTES + 1),
            )]],
        };
        let error = convert_values(
            response,
            ValueRenderMode::Formatted,
            1,
            1,
            CellLimit::new(1).unwrap(),
        )
        .unwrap_err();

        assert!(matches!(error, GoogleProviderError::CellTooLarge { .. }));
    }
}
