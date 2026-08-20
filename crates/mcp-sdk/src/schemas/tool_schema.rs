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
    Number {
        name: &'static str,
        minimum: Option<f64>,
        maximum: Option<f64>,
    },
    Boolean {
        name: &'static str,
    },
    Array {
        name: &'static str,
        min_items: Option<usize>,
        max_items: Option<usize>,
        items: &'static ToolInputType,
    },
    Object {
        name: &'static str,
        schema: &'static ToolInputSchema,
    },
    OneOf {
        name: &'static str,
        variants: &'static [ToolInputType],
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ToolInputType {
    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
        enum_values: &'static [&'static str],
    },
    Integer {
        minimum: Option<u64>,
        maximum: Option<u64>,
    },
    Number {
        minimum: Option<f64>,
        maximum: Option<f64>,
    },
    Boolean,
    Array {
        min_items: Option<usize>,
        max_items: Option<usize>,
        items: &'static ToolInputType,
    },
    Object(&'static ToolInputSchema),
    OneOf(&'static [ToolInputType]),
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

    pub const fn number(name: &'static str, minimum: Option<f64>, maximum: Option<f64>) -> Self {
        Self::Number {
            name,
            minimum,
            maximum,
        }
    }

    pub const fn boolean(name: &'static str) -> Self {
        Self::Boolean { name }
    }

    pub const fn array(
        name: &'static str,
        min_items: Option<usize>,
        max_items: Option<usize>,
        items: &'static ToolInputType,
    ) -> Self {
        Self::Array {
            name,
            min_items,
            max_items,
            items,
        }
    }

    pub const fn object(name: &'static str, schema: &'static ToolInputSchema) -> Self {
        Self::Object { name, schema }
    }

    pub const fn one_of(name: &'static str, variants: &'static [ToolInputType]) -> Self {
        Self::OneOf { name, variants }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::String { name, .. }
            | Self::Integer { name, .. }
            | Self::Number { name, .. }
            | Self::Boolean { name }
            | Self::Array { name, .. }
            | Self::Object { name, .. }
            | Self::OneOf { name, .. } => name,
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
            Self::Number {
                minimum, maximum, ..
            } => {
                map.serialize_entry("type", "number")?;
                if let Some(minimum) = minimum {
                    map.serialize_entry("minimum", minimum)?;
                }
                if let Some(maximum) = maximum {
                    map.serialize_entry("maximum", maximum)?;
                }
            }
            Self::Boolean { .. } => {
                map.serialize_entry("type", "boolean")?;
            }
            Self::Array {
                min_items,
                max_items,
                items,
                ..
            } => {
                map.serialize_entry("type", "array")?;
                if let Some(min_items) = min_items {
                    map.serialize_entry("minItems", min_items)?;
                }
                if let Some(max_items) = max_items {
                    map.serialize_entry("maxItems", max_items)?;
                }
                map.serialize_entry("items", items)?;
            }
            Self::Object { schema, .. } => {
                map.serialize_entry("type", "object")?;
                map.serialize_entry("properties", &ToolInputProperties(schema.properties))?;
                if !schema.required.is_empty() {
                    map.serialize_entry("required", schema.required)?;
                }
                map.serialize_entry("additionalProperties", &false)?;
            }
            Self::OneOf { variants, .. } => {
                map.serialize_entry("oneOf", variants)?;
            }
        }
        map.end()
    }
}

impl ToolInputType {
    pub const fn string(
        min_length: Option<usize>,
        max_length: Option<usize>,
        enum_values: &'static [&'static str],
    ) -> Self {
        Self::String {
            min_length,
            max_length,
            enum_values,
        }
    }

    pub const fn integer(minimum: Option<u64>, maximum: Option<u64>) -> Self {
        Self::Integer { minimum, maximum }
    }

    pub const fn number(minimum: Option<f64>, maximum: Option<f64>) -> Self {
        Self::Number { minimum, maximum }
    }

    pub const fn boolean() -> Self {
        Self::Boolean
    }

    pub const fn array(
        min_items: Option<usize>,
        max_items: Option<usize>,
        items: &'static ToolInputType,
    ) -> Self {
        Self::Array {
            min_items,
            max_items,
            items,
        }
    }

    pub const fn object(schema: &'static ToolInputSchema) -> Self {
        Self::Object(schema)
    }

    pub const fn one_of(variants: &'static [ToolInputType]) -> Self {
        Self::OneOf(variants)
    }
}

impl Serialize for ToolInputType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::String {
                min_length,
                max_length,
                enum_values,
            } => {
                let mut map = serializer.serialize_map(None)?;
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
                map.end()
            }
            Self::Integer { minimum, maximum } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "integer")?;
                if let Some(minimum) = minimum {
                    map.serialize_entry("minimum", minimum)?;
                }
                if let Some(maximum) = maximum {
                    map.serialize_entry("maximum", maximum)?;
                }
                map.end()
            }
            Self::Number { minimum, maximum } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "number")?;
                if let Some(minimum) = minimum {
                    map.serialize_entry("minimum", minimum)?;
                }
                if let Some(maximum) = maximum {
                    map.serialize_entry("maximum", maximum)?;
                }
                map.end()
            }
            Self::Boolean => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "boolean")?;
                map.end()
            }
            Self::Array {
                min_items,
                max_items,
                items,
            } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "array")?;
                if let Some(min_items) = min_items {
                    map.serialize_entry("minItems", min_items)?;
                }
                if let Some(max_items) = max_items {
                    map.serialize_entry("maxItems", max_items)?;
                }
                map.serialize_entry("items", items)?;
                map.end()
            }
            Self::Object(schema) => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "object")?;
                if !schema.required.is_empty() {
                    map.serialize_entry("required", schema.required)?;
                }
                map.serialize_entry("properties", &ToolInputProperties(schema.properties))?;
                map.serialize_entry("additionalProperties", &false)?;
                map.end()
            }
            Self::OneOf(variants) => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("oneOf", variants)?;
                map.end()
            }
        }
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
    use super::{ToolInputProperty, ToolInputSchema, ToolInputType};

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

    const CHAT_SCHEMA: ToolInputSchema = ToolInputSchema::object(
        &["chat_id"],
        &[ToolInputProperty::one_of(
            "chat_id",
            &[
                ToolInputType::integer(None, None),
                ToolInputType::string(Some(1), Some(64), &[]),
            ],
        )],
    );

    #[test]
    fn serializes_a_typed_object_schema() {
        let payload = SCHEMA.to_json_payload().unwrap();

        assert_eq!(
            payload.as_str(),
            r#"{"type":"object","required":["spreadsheet_id"],"properties":{"spreadsheet_id":{"type":"string","minLength":1,"maxLength":256},"value_rendering":{"type":"string","enum":["formatted","unformatted","formula"]},"max_cells":{"type":"integer","minimum":1,"maximum":500}},"additionalProperties":false}"#
        );
    }

    #[test]
    fn serializes_a_typed_union_property() {
        let payload = CHAT_SCHEMA.to_json_payload().unwrap();

        assert_eq!(
            payload.as_str(),
            r#"{"type":"object","required":["chat_id"],"properties":{"chat_id":{"oneOf":[{"type":"integer"},{"type":"string","minLength":1,"maxLength":64}]}},"additionalProperties":false}"#
        );
    }
}
