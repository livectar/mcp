use mcp_protocol::schemas::tools::{CallToolResult, ContentBlock};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HostError {
    #[error("caller authentication failed")]
    CallerAuthenticationFailed,
    #[error("host request is invalid: {0}")]
    InvalidRequest(String),
    #[error("authorization denied")]
    AuthorizationDenied,
    #[error("approval denied")]
    ApprovalDenied,
    #[error("credential unavailable")]
    CredentialUnavailable,
    #[error("host service failed: {0}")]
    Service(String),
}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("tool not found: {0}")]
    ToolNotFound(String),
    #[error("invalid tool arguments: {0}")]
    InvalidArguments(String),
    #[error("host service failed: {0}")]
    Host(#[from] HostError),
    #[error("provider call failed: {0}")]
    Provider(String),
    #[error("protocol serialization failed: {0}")]
    Protocol(String),
}

impl ServerError {
    pub fn error_result(self) -> CallToolResult {
        CallToolResult {
            content: vec![ContentBlock::Text {
                text: self.to_string(),
            }],
            structured_content: None,
            is_error: true,
        }
    }
}
