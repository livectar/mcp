use serde::{Deserialize, Serialize};

use super::super::identifiers::ids::{SheetId, SpreadsheetId};

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
