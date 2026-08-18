use mcp_sdk::schemas::pagination::OpaqueCursor;
use serde::{Deserialize, Serialize};

use super::{
    cells::ValueRenderMode,
    identifiers::{
        filters::{DriveQuery, SpreadsheetNameFilter},
        ids::SpreadsheetId,
        limits::{CellLimit, PageSize, TextChunkSize},
        ranges::A1Range,
    },
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListSpreadsheetsRequest {
    pub name_contains: Option<SpreadsheetNameFilter>,
    pub query: Option<DriveQuery>,
    pub page_size: Option<PageSize>,
    pub page_cursor: Option<OpaqueCursor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetSpreadsheetRequest {
    pub spreadsheet_id: SpreadsheetId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadRangeRequest {
    pub spreadsheet_id: SpreadsheetId,
    pub range: A1Range,
    #[serde(default)]
    pub value_rendering: ValueRenderMode,
    pub max_cells: Option<CellLimit>,
    pub continuation_cursor: Option<OpaqueCursor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadCellTextRequest {
    pub spreadsheet_id: SpreadsheetId,
    pub cell: A1Range,
    #[serde(default)]
    pub value_rendering: ValueRenderMode,
    pub chunk_bytes: Option<TextChunkSize>,
    pub continuation_cursor: Option<OpaqueCursor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadSheetMetadataRequest {
    pub spreadsheet_id: SpreadsheetId,
}
