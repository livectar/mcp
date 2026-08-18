use serde::Deserialize;

use super::cells::ValueRenderMode;

#[derive(Debug, Deserialize)]
pub(crate) struct DriveFilesResponse {
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    pub files: Vec<DriveFile>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DriveFile {
    pub id: String,
    pub name: String,
    #[serde(rename = "webViewLink")]
    pub web_view_link: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SheetsSpreadsheetResponse {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    pub properties: SpreadsheetProperties,
    pub sheets: Vec<SheetsSheet>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SpreadsheetProperties {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SheetsSheet {
    pub properties: SheetProperties,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SheetProperties {
    #[serde(rename = "sheetId")]
    pub sheet_id: i64,
    pub title: String,
    pub index: u32,
    #[serde(rename = "sheetType")]
    pub sheet_type: super::results::SheetType,
    #[serde(rename = "gridProperties")]
    pub grid_properties: Option<GridProperties>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GridProperties {
    #[serde(rename = "rowCount")]
    pub row_count: Option<u32>,
    #[serde(rename = "columnCount")]
    pub column_count: Option<u32>,
    #[serde(rename = "frozenRowCount")]
    pub frozen_row_count: Option<u32>,
    #[serde(rename = "frozenColumnCount")]
    pub frozen_column_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SheetsValueRange {
    #[allow(dead_code)]
    pub range: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "majorDimension")]
    pub major_dimension: Option<String>,
    #[serde(default)]
    pub values: Vec<Vec<GoogleRawCell>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum GoogleRawCell {
    Empty(()),
    Boolean(bool),
    Number(f64),
    Text(String),
}

impl GoogleRawCell {
    pub fn into_cell_value(self, rendering: ValueRenderMode) -> super::cells::CellValue {
        match self {
            Self::Empty(_) => super::cells::CellValue::Empty,
            Self::Boolean(value) => super::cells::CellValue::Boolean(value),
            Self::Number(value) => super::cells::CellValue::Number(value),
            Self::Text(value)
                if rendering == ValueRenderMode::Formula && value.starts_with('=') =>
            {
                super::cells::CellValue::Formula(value)
            }
            Self::Text(value) => super::cells::CellValue::Text(value),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct GoogleErrorEnvelope {
    pub error: GoogleErrorBody,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GoogleErrorBody {
    pub code: Option<u16>,
    pub message: Option<String>,
    pub status: Option<String>,
    #[serde(default)]
    pub errors: Vec<GoogleErrorDetail>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GoogleErrorDetail {
    pub reason: Option<String>,
    pub message: Option<String>,
}
