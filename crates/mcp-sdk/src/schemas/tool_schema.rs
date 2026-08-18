use mcp_protocol::{errors::ProtocolError, schemas::json_payload::JsonPayload};
use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;

use super::pagination::MAX_CURSOR_BYTES;

/// A typed JSON Schema property supported by MCP tool input definitions.
#[derive(Debug, Clone, Copy)]
pub enum ToolInputProperty {
    String {
        name: &'static str,
        min_length: Option<usize>,
        max_length: Option<usize>,
        enum_values: &'static [&'static str],
    },
    Integer {
        name: &'static str,
        minimum: Option<u64>,
        maximum: Option<u64>,
    },
}

impl ToolInputProperty {
    pub const fn string(
        name: &'static str,
        min_length: Option<usize>,
        max_length: Option<usize>,
    ) -> Self {
        Self::String {
            name,
            min_length,
            max_length,
            enum_values: &[],
        }
    }

    pub const fn string_enum(name: &'static str, enum_values: &'static [&'static str]) -> Self {
        Self::String {
            name,
            min_length: None,
            max_length: None,
            enum_values,
        }
    }

    pub const fn page_cursor() -> Self {
        Self::string("page_cursor", Some(1), Some(MAX_CURSOR_BYTES))
    }

    pub const fn continuation_cursor() -> Self {
        Self::string("continuation_cursor", Some(1), Some(MAX_CURSOR_BYTES))
    }

    pub const fn integer(name: &'static str, minimum: Option<u64>, maximum: Option<u64>) -> Self {
        Self::Integer {
            name,
            minimum,
            maximum,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::String { name, .. } | Self::Integer { name, .. } => name,
        }
    }
}

impl Serialize for ToolInputProperty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        match self {
            Self::String {
                min_length,
                max_length,
                enum_values,
                ..
            } => {
                map.serialize_entry("type", "string")?;
                if let Some(min_length) = min_length {
                    map.serialize_entry("minLength", min_length)?;
                }
                if let Some(max_length) = max_length {
                    map.serialize_entry("maxLength", max_length)?;
                }
                if !enum_values.is_empty() {
                    map.serialize_entry("enum", enum_values)?;
                }
            }
            Self::Integer {
                minimum, maximum, ..
            } => {
                map.serialize_entry("type", "integer")?;
                if let Some(minimum) = minimum {
                    map.serialize_entry("minimum", minimum)?;
                }
                if let Some(maximum) = maximum {
                    map.serialize_entry("maximum", maximum)?;
                }
            }
        }
        map.end()
    }
}

/// The single object-schema definition used for MCP tool inputs.
#[derive(Debug, Clone, Copy)]
pub struct ToolInputSchema {
    required: &'static [&'static str],
    properties: &'static [ToolInputProperty],
}

impl ToolInputSchema {
    pub const fn object(
        required: &'static [&'static str],
        properties: &'static [ToolInputProperty],
    ) -> Self {
        Self {
            required,
            properties,
        }
    }

    pub fn to_json_payload(self) -> Result<JsonPayload, ProtocolError> {
        JsonPayload::from_serializable(&self)
    }
}

impl Serialize for ToolInputSchema {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "object")?;
        if !self.required.is_empty() {
            map.serialize_entry("required", self.required)?;
        }
        if !self.properties.is_empty() {
            map.serialize_entry("properties", &ToolInputProperties(self.properties))?;
        }
        map.serialize_entry("additionalProperties", &false)?;
        map.end()
    }
}

struct ToolInputProperties(&'static [ToolInputProperty]);

impl Serialize for ToolInputProperties {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for property in self.0 {
            map.serialize_entry(property.name(), property)?;
        }
        map.end()
    }
}

#[cfg(test)]
mod tests {
    use super::{ToolInputProperty, ToolInputSchema};

    const SCHEMA: ToolInputSchema = ToolInputSchema::object(
        &["spreadsheet_id"],
        &[
            ToolInputProperty::string("spreadsheet_id", Some(1), Some(256)),
            ToolInputProperty::string_enum(
                "value_rendering",
                &["formatted", "unformatted", "formula"],
            ),
            ToolInputProperty::integer("max_cells", Some(1), Some(500)),
        ],
    );

    #[test]
    fn serializes_a_typed_object_schema() {
        let payload = SCHEMA.to_json_payload().unwrap();

        assert_eq!(
            payload.as_str(),
            r#"{"type":"object","required":["spreadsheet_id"],"properties":{"spreadsheet_id":{"type":"string","minLength":1,"maxLength":256},"value_rendering":{"type":"string","enum":["formatted","unformatted","formula"]},"max_cells":{"type":"integer","minimum":1,"maximum":500}},"additionalProperties":false}"#
        );
    }
}
