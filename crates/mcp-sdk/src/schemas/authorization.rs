use crate::errors::HostError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationRequest {
    pub operation: OperationName,
    pub tool_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationName(String);

impl OperationName {
    pub fn new(value: impl Into<String>) -> Result<Self, HostError> {
        let value = value.into();
        if value.trim().is_empty() || value.len() > 160 {
            return Err(HostError::InvalidRequest(
                "operation name must be between 1 and 160 characters".to_string(),
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalDecision {
    Allowed,
    RequiresApproval,
    Denied,
}
