use super::tool_schema::ToolInputSchema;
use mcp_protocol::{
    errors::ProtocolError,
    schemas::tools::{
        ToolAnnotations as ProtocolToolAnnotations, ToolDefinition as ProtocolToolDefinition,
    },
};

#[derive(Debug, Clone, Default)]
pub struct ToolAnnotations {
    pub read_only_hint: Option<bool>,
}

impl From<ToolAnnotations> for ProtocolToolAnnotations {
    fn from(value: ToolAnnotations) -> Self {
        Self {
            read_only_hint: value.read_only_hint,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: ToolInputSchema,
    pub annotations: ToolAnnotations,
}

impl ToolDefinition {
    pub fn into_protocol(self) -> Result<ProtocolToolDefinition, ProtocolError> {
        Ok(ProtocolToolDefinition {
            name: self.name,
            description: self.description,
            input_schema: self.input_schema.to_json_payload()?,
            annotations: self.annotations.into(),
        })
    }
}
