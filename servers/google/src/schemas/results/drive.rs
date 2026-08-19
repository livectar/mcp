use mcp_sdk::schemas::pagination::{OpaqueCursor, Paginated};
use serde::{Deserialize, Serialize};

use super::super::identifiers::ids::SpreadsheetId;

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
