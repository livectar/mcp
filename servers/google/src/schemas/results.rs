use serde::{Deserialize, Serialize};

use mcp_sdk::schemas::pagination::{OpaqueCursor, Paginated};

use super::identifiers::{
    ids::{SheetId, SpreadsheetId},
    ranges::A1Range,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpreadsheetListItem {
    pub spreadsheet_id: SpreadsheetId,
    pub name: String,
    pub web_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListSpreadsheetsResult {
    pub spreadsheets: Vec<SpreadsheetListItem>,
    pub next_cursor: Option<OpaqueCursor>,
}

impl Paginated for ListSpreadsheetsResult {
    type Cursor = OpaqueCursor;

    fn next_cursor(&self) -> Option<&Self::Cursor> {
        self.next_cursor.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SheetIdentity {
    pub sheet_id: SheetId,
    pub title: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SheetType {
    Grid,
    Object,
    DataSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridDimensions {
    pub rows: Option<u32>,
    pub columns: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrozenPaneMetadata {
    pub rows: Option<u32>,
    pub columns: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SheetTabMetadata {
    pub identity: SheetIdentity,
    pub index: u32,
    pub sheet_type: SheetType,
    pub dimensions: GridDimensions,
    pub frozen_panes: FrozenPaneMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpreadsheetMetadataResult {
    pub spreadsheet_id: SpreadsheetId,
    pub title: String,
    pub tabs: Vec<SheetTabMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SheetMetadataResult {
    pub spreadsheet_id: SpreadsheetId,
    pub tabs: Vec<SheetTabMetadata>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ProviderRangeIdentity {
    pub spreadsheet_id: SpreadsheetId,
    pub requested_range: A1Range,
}
