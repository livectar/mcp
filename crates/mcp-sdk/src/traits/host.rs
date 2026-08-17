use crate::{
    errors::HostError,
    schemas::{
        audit::AuditEvent,
        authorization::{ApprovalDecision, AuthorizationRequest},
        caller::CallerContext,
        credentials::{CredentialRequest, ProviderCredential},
    },
};
use async_trait::async_trait;

#[async_trait]
pub trait WorkspaceContext: Send + Sync {
    async fn caller(&self) -> Result<CallerContext, HostError>;
}

/// Resolves the authenticated caller for one incoming MCP request.
///
/// Implementations own authentication and installation binding. The
/// transport passes only the route key and authorization header; tool
/// arguments are never used as an identity source.
#[async_trait]
pub trait CallerContextResolver: Send + Sync {
    async fn resolve(
        &self,
        server_key: &str,
        authorization: Option<&str>,
    ) -> Result<CallerContext, HostError>;
}

#[async_trait]
pub trait CredentialResolver: Send + Sync {
    async fn resolve(
        &self,
        caller: &CallerContext,
        request: &CredentialRequest,
    ) -> Result<ProviderCredential, HostError>;
}

#[async_trait]
pub trait Authorization: Send + Sync {
    async fn authorize(
        &self,
        caller: &CallerContext,
        request: &AuthorizationRequest,
    ) -> Result<(), HostError>;
}

#[async_trait]
pub trait ToolApprovalContext: Send + Sync {
    async fn decision(
        &self,
        caller: &CallerContext,
        request: &AuthorizationRequest,
    ) -> Result<ApprovalDecision, HostError>;
}

#[async_trait]
pub trait AuditSink: Send + Sync {
    async fn record(&self, event: AuditEvent) -> Result<(), HostError>;
}
