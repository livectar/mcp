use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

const MAX_FILTER_BYTES: usize = 256;
const MAX_QUERY_BYTES: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct SpreadsheetNameFilter(String);

impl SpreadsheetNameFilter {
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        validate_filter_value(&value, MAX_FILTER_BYTES, "name_contains")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for SpreadsheetNameFilter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct DriveQuery(String);

impl DriveQuery {
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        validate_filter_value(&value, MAX_QUERY_BYTES, "query")?;
        if value.contains(';') {
            return Err("query must not contain statement separators".to_string());
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for DriveQuery {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

fn validate_filter_value(value: &str, max_bytes: usize, field: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > max_bytes || value.chars().any(char::is_control) {
        return Err(format!(
            "{field} must be non-empty and at most {max_bytes} bytes"
        ));
    }
    Ok(())
}
