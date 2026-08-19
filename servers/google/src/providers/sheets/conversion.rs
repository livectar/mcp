use crate::{
    errors::GoogleProviderError,
    schemas::{
        cells::values::CellValue,
        identifiers::{
            ids::{SheetId, SpreadsheetId},
            limits::CellLimit,
        },
        provider::sheets::{
            metadata::{SheetsSheet, SheetsSpreadsheetResponse},
            values::SheetsValueRange,
        },
        results::sheets::{
            FrozenPaneMetadata, GridDimensions, SheetIdentity, SheetTabMetadata,
            SpreadsheetMetadataResult,
        },
    },
};

use super::super::common::{invalid_provider_response, MAX_CELL_TEXT_BYTES};

pub(super) fn convert_metadata(
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

pub(super) fn convert_created_metadata(
    response: SheetsSpreadsheetResponse,
) -> Result<SpreadsheetMetadataResult, GoogleProviderError> {
    let spreadsheet_id =
        SpreadsheetId::new(response.spreadsheet_id).map_err(invalid_provider_response)?;
    let tabs = response
        .sheets
        .into_iter()
        .map(|sheet| {
            let properties = sheet.properties;
            let sheet_id = SheetId::new(properties.sheet_id).map_err(invalid_provider_response)?;
            Ok(SheetTabMetadata {
                identity: SheetIdentity {
                    sheet_id,
                    title: properties.title,
                },
                index: properties.index,
                sheet_type: properties.sheet_type,
                dimensions: GridDimensions {
                    rows: properties
                        .grid_properties
                        .as_ref()
                        .and_then(|grid| grid.row_count),
                    columns: properties
                        .grid_properties
                        .as_ref()
                        .and_then(|grid| grid.column_count),
                },
                frozen_panes: FrozenPaneMetadata {
                    rows: properties
                        .grid_properties
                        .as_ref()
                        .and_then(|grid| grid.frozen_row_count),
                    columns: properties
                        .grid_properties
                        .as_ref()
                        .and_then(|grid| grid.frozen_column_count),
                },
            })
        })
        .collect::<Result<Vec<_>, GoogleProviderError>>()?;
    Ok(SpreadsheetMetadataResult {
        spreadsheet_id,
        title: response.properties.title,
        tabs,
    })
}

pub(super) fn convert_values(
    response: SheetsValueRange,
    rendering: crate::schemas::cells::values::ValueRenderMode,
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

#[cfg(test)]
mod tests {
    use super::convert_values;
    use crate::providers::common::MAX_CELL_TEXT_BYTES;
    use crate::{
        errors::GoogleProviderError,
        schemas::{
            cells::values::ValueRenderMode,
            identifiers::limits::CellLimit,
            provider::sheets::values::{GoogleRawCell, SheetsValueRange},
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
