use serde::{Deserialize, Serialize};

use super::super::{
    cells::matrix::{CellMatrix, CellRows},
    identifiers::{ids::SpreadsheetId, ranges::A1Range},
};

pub const MAX_CREATE_SPREADSHEET_TITLE_BYTES: usize = 256;
pub const MAX_CREATE_SPREADSHEET_GRID_DIMENSION: u32 = 10_000_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitialTabConfiguration {
    pub title: Option<String>,
    pub row_count: Option<u32>,
    pub column_count: Option<u32>,
    pub frozen_row_count: Option<u32>,
    pub frozen_column_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateSpreadsheetRequest {
    pub title: String,
    pub initial_tab: Option<InitialTabConfiguration>,
}

impl CreateSpreadsheetRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_title(&self.title, "title")?;
        if let Some(initial_tab) = &self.initial_tab {
            if let Some(title) = &initial_tab.title {
                validate_title(title, "sheet title")?;
            }
            for (field, value) in [
                ("row_count", initial_tab.row_count),
                ("column_count", initial_tab.column_count),
                ("frozen_row_count", initial_tab.frozen_row_count),
                ("frozen_column_count", initial_tab.frozen_column_count),
            ] {
                if let Some(value) = value {
                    validate_grid_dimension(value, field)?;
                }
            }
        }
        Ok(())
    }
}

fn validate_title(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > MAX_CREATE_SPREADSHEET_TITLE_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(format!(
            "{field} must be non-empty and at most {MAX_CREATE_SPREADSHEET_TITLE_BYTES} bytes"
        ));
    }
    Ok(())
}

fn validate_grid_dimension(value: u32, field: &str) -> Result<(), String> {
    if !(1..=MAX_CREATE_SPREADSHEET_GRID_DIMENSION).contains(&value) {
        return Err(format!(
            "{field} must be between 1 and {MAX_CREATE_SPREADSHEET_GRID_DIMENSION}"
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WriteRangeRequest {
    pub spreadsheet_id: SpreadsheetId,
    pub range: A1Range,
    pub values: CellMatrix,
}

impl WriteRangeRequest {
    pub fn validate(&self) -> Result<(), String> {
        if !self.range.is_fully_bounded() {
            return Err("write_range range must specify both row and column bounds".to_string());
        }
        let bounds = self
            .range
            .bounds()?
            .resolve(None, None)
            .map_err(|error| format!("write_range range is invalid: {error}"))?;
        let expected_rows = bounds.row_count();
        let expected_columns = bounds.column_count();
        if usize::try_from(expected_rows).unwrap_or(usize::MAX) != self.values.row_count()
            || usize::try_from(expected_columns).unwrap_or(usize::MAX) != self.values.column_count()
        {
            return Err(format!(
                "write_range values dimensions do not match the requested range ({expected_rows}x{expected_columns})"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppendRowsRequest {
    pub spreadsheet_id: SpreadsheetId,
    pub range: A1Range,
    pub rows: CellRows,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClearRangeRequest {
    pub spreadsheet_id: SpreadsheetId,
    pub range: A1Range,
}

impl ClearRangeRequest {
    pub fn validate(&self) -> Result<(), String> {
        if !self.range.is_fully_bounded() {
            return Err("clear_range range must specify both row and column bounds".to_string());
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::{
        CellMatrix, CreateSpreadsheetRequest, InitialTabConfiguration, SpreadsheetId,
        WriteRangeRequest,
    };
    use crate::schemas::{cells::values::CellValue, identifiers::ranges::A1Range};

    #[test]
    fn write_range_requires_exact_matrix_dimensions_and_closed_bounds() {
        let spreadsheet_id = SpreadsheetId::new("spreadsheet").unwrap();
        let values = CellMatrix::new(vec![vec![CellValue::Empty]]).unwrap();
        let mismatched = WriteRangeRequest {
            spreadsheet_id: spreadsheet_id.clone(),
            range: A1Range::new("A1:B1").unwrap(),
            values: values.clone(),
        };
        assert!(mismatched.validate().is_err());

        let open_ended = WriteRangeRequest {
            spreadsheet_id,
            range: A1Range::new("A:A").unwrap(),
            values,
        };
        assert!(open_ended.validate().is_err());
    }

    #[test]
    fn create_spreadsheet_validates_titles_and_grid_dimensions() {
        let valid = CreateSpreadsheetRequest {
            title: "Created".to_string(),
            initial_tab: Some(InitialTabConfiguration {
                title: Some("Initial".to_string()),
                row_count: Some(100),
                column_count: Some(10),
                frozen_row_count: None,
                frozen_column_count: None,
            }),
        };
        assert!(valid.validate().is_ok());

        let invalid = CreateSpreadsheetRequest {
            title: String::new(),
            initial_tab: Some(InitialTabConfiguration {
                title: None,
                row_count: Some(0),
                column_count: None,
                frozen_row_count: None,
                frozen_column_count: None,
            }),
        };
        assert!(invalid.validate().is_err());
    }
}
