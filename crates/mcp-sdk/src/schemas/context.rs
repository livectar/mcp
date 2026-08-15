use super::{caller::CallerContext, credentials::ProviderName};
use crate::{
    errors::HostError,
    schemas::{
        audit::AuditEvent, authorization::AuthorizationRequest, credentials::CredentialRequest,
    },
    traits::host::{AuditSink, Authorization, CredentialResolver, ToolApprovalContext},
};
use mcp_protocol::{constants::PROTOCOL_REVISION, schemas::json_rpc::RequestId};
use std::sync::Arc;

#[derive(Clone)]
pub struct HostServices {
    pub credentials: Arc<dyn CredentialResolver>,
    pub authorization: Arc<dyn Authorization>,
    pub approvals: Arc<dyn ToolApprovalContext>,
    pub audit: Arc<dyn AuditSink>,
}

#[derive(Clone)]
pub struct RequestContext {
    pub request_id: RequestId,
    pub protocol_version: String,
    pub caller: CallerContext,
    pub services: HostServices,
}

impl RequestContext {
    pub fn new(request_id: RequestId, caller: CallerContext, services: HostServices) -> Self {
        Self {
            request_id,
            protocol_version: PROTOCOL_REVISION.to_string(),
            caller,
            services,
        }
    }

    pub async fn credential(
        &self,
        provider: ProviderName,
        purpose: impl Into<String>,
    ) -> Result<super::credentials::ProviderCredential, HostError> {
        self.services
            .credentials
            .resolve(
                &self.caller,
                &CredentialRequest {
                    provider,
                    purpose: purpose.into(),
                },
            )
            .await
    }

    pub async fn authorize(&self, request: &AuthorizationRequest) -> Result<(), HostError> {
        self.services
            .authorization
            .authorize(&self.caller, request)
            .await
    }

    pub async fn approval(
        &self,
        request: &AuthorizationRequest,
    ) -> Result<super::authorization::ApprovalDecision, HostError> {
        self.services
            .approvals
            .decision(&self.caller, request)
            .await
    }

    pub async fn audit(&self, event: AuditEvent) -> Result<(), HostError> {
        self.services.audit.record(event).await
    }
}
