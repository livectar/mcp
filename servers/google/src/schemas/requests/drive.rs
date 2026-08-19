use mcp_sdk::schemas::pagination::OpaqueCursor;
use serde::{Deserialize, Serialize};

use super::super::identifiers::{
    filters::{DriveQuery, SpreadsheetNameFilter},
    limits::PageSize,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListSpreadsheetsRequest {
    pub name_contains: Option<SpreadsheetNameFilter>,
    pub query: Option<DriveQuery>,
    pub page_size: Option<PageSize>,
    pub page_cursor: Option<OpaqueCursor>,
}
