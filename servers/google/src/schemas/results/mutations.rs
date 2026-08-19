use serde::{Deserialize, Serialize};

use super::super::identifiers::{ids::SpreadsheetId, ranges::A1Range};
use super::sheets::SheetIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationOperation {
    CreateSpreadsheet,
    WriteRange,
    AppendRows,
    ClearRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationOutcome {
    Applied,
    Uncertain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MutationSummary {
    Created {
        title: String,
        tab_count: u32,
    },
    Written {
        updated_rows: u32,
        updated_columns: u32,
    },
    Appended {
        updated_rows: u32,
        updated_columns: u32,
    },
    Cleared {
        cleared_range: A1Range,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationResult {
    pub operation: MutationOperation,
    pub outcome: MutationOutcome,
    pub spreadsheet_id: SpreadsheetId,
    pub tab: Option<SheetIdentity>,
    pub range: Option<A1Range>,
    pub affected_cell_count: u32,
    pub failed_cell_count: u32,
    pub summary: MutationSummary,
}
