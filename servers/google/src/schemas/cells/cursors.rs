use serde::{Deserialize, Serialize};

use super::super::identifiers::{
    ids::SpreadsheetId,
    limits::{CellLimit, TextChunkSize},
    ranges::A1Range,
};
use super::values::ValueRenderMode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RangeCursorPayload {
    pub spreadsheet_id: SpreadsheetId,
    pub range: A1Range,
    pub value_rendering: ValueRenderMode,
    pub max_cells: CellLimit,
    pub row_offset: u32,
    pub column_offset: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TextChunkCursorPayload {
    pub spreadsheet_id: SpreadsheetId,
    pub cell: A1Range,
    pub value_rendering: ValueRenderMode,
    pub chunk_bytes: TextChunkSize,
    pub offset: u32,
}
