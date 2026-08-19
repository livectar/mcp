use serde::{Deserialize, Serialize};

use super::super::super::{
    cells::values::CellValue, requests::sheets_mutations::InitialTabConfiguration,
};

#[derive(Debug, Serialize)]
pub(crate) struct CreateSpreadsheetBody {
    pub properties: CreateSpreadsheetProperties,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sheets: Option<Vec<CreateSheet>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CreateSpreadsheetProperties {
    pub title: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct CreateSheet {
    pub properties: CreateSheetProperties,
}

#[derive(Debug, Serialize)]
pub(crate) struct CreateSheetProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "gridProperties")]
    pub grid_properties: Option<CreateGridProperties>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CreateGridProperties {
    #[serde(skip_serializing_if = "Option::is_none", rename = "rowCount")]
    pub row_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "columnCount")]
    pub column_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "frozenRowCount")]
    pub frozen_row_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "frozenColumnCount")]
    pub frozen_column_count: Option<u32>,
}

impl CreateSpreadsheetBody {
    pub(crate) fn from_request(title: &str, initial_tab: Option<&InitialTabConfiguration>) -> Self {
        let sheets = initial_tab.map(|tab| {
            vec![CreateSheet {
                properties: CreateSheetProperties {
                    title: tab.title.clone(),
                    grid_properties: Some(CreateGridProperties {
                        row_count: tab.row_count,
                        column_count: tab.column_count,
                        frozen_row_count: tab.frozen_row_count,
                        frozen_column_count: tab.frozen_column_count,
                    }),
                },
            }]
        });
        Self {
            properties: CreateSpreadsheetProperties {
                title: title.to_string(),
            },
            sheets,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct UpdateValuesBody<'a> {
    pub range: &'a str,
    #[serde(rename = "majorDimension")]
    pub major_dimension: GoogleMajorDimension,
    pub values: Vec<Vec<GoogleWriteCell>>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum GoogleMajorDimension {
    Rows,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum GoogleValueInputOption {
    UserEntered,
}

impl GoogleValueInputOption {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::UserEntered => "USER_ENTERED",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum GoogleInsertDataOption {
    InsertRows,
}

impl GoogleInsertDataOption {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::InsertRows => "INSERT_ROWS",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum GoogleWriteCell {
    Text(String),
    Number(f64),
    Boolean(bool),
}

impl From<CellValue> for GoogleWriteCell {
    fn from(value: CellValue) -> Self {
        match value {
            CellValue::Empty => Self::Text(String::new()),
            CellValue::Text(value) | CellValue::Formula(value) => Self::Text(value),
            CellValue::Number(value) => Self::Number(value),
            CellValue::Boolean(value) => Self::Boolean(value),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateValuesResponse {
    #[serde(rename = "updatedRange")]
    pub updated_range: Option<String>,
    #[serde(rename = "updatedRows")]
    pub updated_rows: Option<u32>,
    #[serde(rename = "updatedColumns")]
    pub updated_columns: Option<u32>,
    #[serde(rename = "updatedCells")]
    pub updated_cells: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AppendValuesResponse {
    pub updates: UpdateValuesResponse,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ClearValuesResponse {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    #[serde(rename = "clearedRange")]
    pub cleared_range: String,
}
