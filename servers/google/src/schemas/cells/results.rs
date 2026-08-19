use mcp_sdk::schemas::pagination::{OpaqueCursor, Paginated};
use serde::{Deserialize, Serialize};

use super::super::{
    identifiers::{ids::SpreadsheetId, ranges::A1Range},
    results::sheets::SheetIdentity,
};
use super::text::CellTextKind;
use super::values::CellValue;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadRangeIdentity {
    pub spreadsheet_id: SpreadsheetId,
    pub tab: SheetIdentity,
    pub requested_range: A1Range,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadRangeResult {
    pub identity: ReadRangeIdentity,
    pub page_range: A1Range,
    pub values: Vec<Vec<CellValue>>,
    pub returned_cell_count: u32,
    pub next_cursor: Option<OpaqueCursor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadCellTextIdentity {
    pub spreadsheet_id: SpreadsheetId,
    pub tab: SheetIdentity,
    pub cell: A1Range,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadCellTextResult {
    pub identity: ReadCellTextIdentity,
    pub kind: CellTextKind,
    pub text: String,
    pub next_cursor: Option<OpaqueCursor>,
}

impl Paginated for ReadRangeResult {
    type Cursor = OpaqueCursor;

    fn next_cursor(&self) -> Option<&Self::Cursor> {
        self.next_cursor.as_ref()
    }
}

impl Paginated for ReadCellTextResult {
    type Cursor = OpaqueCursor;

    fn next_cursor(&self) -> Option<&Self::Cursor> {
        self.next_cursor.as_ref()
    }
}
