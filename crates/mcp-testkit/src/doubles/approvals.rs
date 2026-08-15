use async_trait::async_trait;
use mcp_sdk::{
    errors::HostError,
    schemas::{
        authorization::{ApprovalDecision, AuthorizationRequest},
        caller::CallerContext,
    },
    traits::host::ToolApprovalContext,
};

pub struct AllowAllApprovals;

#[async_trait]
impl ToolApprovalContext for AllowAllApprovals {
    async fn decision(
        &self,
        _caller: &CallerContext,
        _request: &AuthorizationRequest,
    ) -> Result<ApprovalDecision, HostError> {
        Ok(ApprovalDecision::Allowed)
    }
}
