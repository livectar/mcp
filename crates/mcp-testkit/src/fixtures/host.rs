use mcp_protocol::schemas::json_rpc::RequestId;
use mcp_sdk::schemas::{
    caller::CallerContext,
    context::{HostServices, RequestContext},
};
use std::sync::Arc;

use crate::doubles::{
    approvals::AllowAllApprovals, audit::RecordingAuditSink, authorization::AllowAllAuthorization,
    credentials::NoCredentials,
};

#[derive(Clone)]
pub struct TestHost {
    pub caller: CallerContext,
    pub services: HostServices,
}

impl TestHost {
    pub fn new() -> Self {
        Self {
            caller: CallerContext {
                tenant_id: "test-tenant".to_string(),
                subject_id: "test-subject".to_string(),
                installation_id: None,
                connection_id: None,
            },
            services: HostServices {
                credentials: Arc::new(NoCredentials),
                authorization: Arc::new(AllowAllAuthorization),
                approvals: Arc::new(AllowAllApprovals),
                audit: Arc::new(RecordingAuditSink::default()),
            },
        }
    }

    pub fn context(&self, request_id: impl Into<String>) -> RequestContext {
        RequestContext::new(
            RequestId::String(request_id.into()),
            self.caller.clone(),
            self.services.clone(),
        )
    }
}

impl Default for TestHost {
    fn default() -> Self {
        Self::new()
    }
}
