use serde::Deserialize;

use super::super::super::cells::values::{CellValue, ValueRenderMode};

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
    pub(crate) fn into_cell_value(self, rendering: ValueRenderMode) -> CellValue {
        match self {
            Self::Empty(()) => CellValue::Empty,
            Self::Boolean(value) => CellValue::Boolean(value),
            Self::Number(value) => CellValue::Number(value),
            Self::Text(value)
                if rendering == ValueRenderMode::Formula && value.starts_with('=') =>
            {
                CellValue::Formula(value)
            }
            Self::Text(value) => CellValue::Text(value),
        }
    }
}
