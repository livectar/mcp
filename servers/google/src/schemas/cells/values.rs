use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueRenderMode {
    Formatted,
    Unformatted,
    Formula,
}

impl Default for ValueRenderMode {
    fn default() -> Self {
        Self::Formatted
    }
}

impl ValueRenderMode {
    pub const fn provider_value(self) -> &'static str {
        match self {
            Self::Formatted => "FORMATTED_VALUE",
            Self::Unformatted => "UNFORMATTED_VALUE",
            Self::Formula => "FORMULA",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum CellValue {
    Empty,
    Text(String),
    Number(f64),
    Boolean(bool),
    Formula(String),
}
