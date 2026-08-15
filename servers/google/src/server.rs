use async_trait::async_trait;
use mcp_protocol::schemas::{
    lifecycle::{ImplementationInfo, ServerCapabilities},
    tools::{CallToolParams, CallToolResult, ToolDefinition},
};
use mcp_sdk::{
    errors::ServerError,
    schemas::context::RequestContext,
    traits::server::{McpServer, ToolHandler},
};
use std::sync::Arc;

use crate::{handlers::identity::GoogleIdentityHandler, providers::google::GoogleProvider};

pub struct GoogleServer {
    handler: GoogleIdentityHandler,
}

impl GoogleServer {
    pub fn new(provider: Arc<dyn GoogleProvider>) -> Self {
        Self {
            handler: GoogleIdentityHandler::new(provider),
        }
    }
}

#[async_trait]
impl McpServer for GoogleServer {
    fn info(&self) -> ImplementationInfo {
        ImplementationInfo {
            name: "mcp-google".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            tools: Some(Default::default()),
        }
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![self.handler.definition()]
    }

    async fn call_tool(
        &self,
        context: &RequestContext,
        request: CallToolParams,
    ) -> Result<CallToolResult, ServerError> {
        if request.name != "google_get_identity" {
            return Err(ServerError::ToolNotFound(request.name));
        }
        self.handler.call(context, request.arguments).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mcp_sdk::{
        errors::HostError,
        schemas::{
            caller::CallerContext,
            credentials::{CredentialRequest, ProviderCredential, ProviderName},
        },
        traits::host::CredentialResolver,
    };
    use mcp_testkit::doubles::{
        approvals::AllowAllApprovals, audit::RecordingAuditSink,
        authorization::AllowAllAuthorization,
    };

    struct MockGoogle;

    #[async_trait]
    impl GoogleProvider for MockGoogle {
        async fn get_identity(
            &self,
            credential: &ProviderCredential,
        ) -> Result<crate::schemas::identity::GoogleIdentity, String> {
            assert_eq!(credential.expose_secret(), "mock-google-token");
            Ok(crate::schemas::identity::GoogleIdentity {
                display_name: "Mock Google".to_string(),
            })
        }
    }

    struct MockCredentialResolver;

    #[async_trait]
    impl CredentialResolver for MockCredentialResolver {
        async fn resolve(
            &self,
            _caller: &CallerContext,
            request: &CredentialRequest,
        ) -> Result<ProviderCredential, HostError> {
            assert_eq!(request.provider, ProviderName::new("google").unwrap());
            ProviderCredential::new("mock-google-token")
        }
    }

    #[tokio::test]
    async fn provider_receives_host_injected_credential() {
        let server = GoogleServer::new(Arc::new(MockGoogle));
        let services = mcp_sdk::schemas::context::HostServices {
            credentials: Arc::new(MockCredentialResolver),
            authorization: Arc::new(AllowAllAuthorization),
            approvals: Arc::new(AllowAllApprovals),
            audit: Arc::new(RecordingAuditSink::default()),
        };
        let context = RequestContext::new(
            mcp_protocol::schemas::json_rpc::RequestId::String("google-test".to_string()),
            CallerContext {
                tenant_id: "tenant".to_string(),
                subject_id: "subject".to_string(),
                installation_id: None,
                connection_id: None,
            },
            services,
        );
        let result = server
            .call_tool(
                &context,
                CallToolParams {
                    name: "google_get_identity".to_string(),
                    arguments: None,
                },
            )
            .await
            .unwrap();
        assert!(!result.is_error);
    }
}
