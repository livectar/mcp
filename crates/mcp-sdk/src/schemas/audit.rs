use super::{caller::CallerContext, credentials::ProviderName};
use mcp_protocol::schemas::json_rpc::RequestId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    pub request_id: RequestId,
    pub tenant_id: String,
    pub subject_id: String,
    pub provider: Option<ProviderName>,
    pub tool_name: String,
    pub outcome: AuditOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditOutcome {
    Started,
    Succeeded,
    Failed,
    Denied,
}

impl AuditEvent {
    pub fn for_caller(
        request_id: RequestId,
        caller: &CallerContext,
        provider: Option<ProviderName>,
        tool_name: String,
        outcome: AuditOutcome,
    ) -> Self {
        Self {
            request_id,
            tenant_id: caller.tenant_id.clone(),
            subject_id: caller.subject_id.clone(),
            provider,
            tool_name,
            outcome,
        }
    }
}
