use serde::Deserialize;

use super::super::super::results::sheets::SheetType;

#[derive(Debug, Deserialize)]
pub(crate) struct SheetsSpreadsheetResponse {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    pub properties: SpreadsheetProperties,
    #[serde(default)]
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
    pub sheet_type: SheetType,
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
