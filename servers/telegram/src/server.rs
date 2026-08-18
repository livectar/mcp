use async_trait::async_trait;
use mcp_protocol::schemas::{
    lifecycle::{ImplementationInfo, ServerCapabilities},
    tools::{CallToolParams, CallToolResult},
};
use mcp_sdk::{
    errors::{ServerError, ToolRegistryError},
    schemas::{context::RequestContext, tool_definition::ToolDefinition},
    traits::{server::McpServer, tool_registry::ToolRegistry},
};
use std::sync::Arc;

use crate::{handlers::identity::TelegramIdentityHandler, providers::telegram::TelegramProvider};

pub struct TelegramServer {
    tools: ToolRegistry,
}

impl TelegramServer {
    pub fn new(provider: Arc<dyn TelegramProvider>) -> Result<Self, ToolRegistryError> {
        Ok(Self {
            tools: ToolRegistry::try_new(vec![Arc::new(TelegramIdentityHandler::new(provider))])?,
        })
    }
}

#[async_trait]
impl McpServer for TelegramServer {
    fn info(&self) -> ImplementationInfo {
        ImplementationInfo {
            name: "mcp-telegram".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            tools: Some(Default::default()),
        }
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        self.tools.definitions()
    }

    async fn call_tool(
        &self,
        context: &RequestContext,
        request: CallToolParams,
    ) -> Result<CallToolResult, ServerError> {
        self.tools.call(context, request).await
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
            credentials::{CredentialRequest, ProviderCredential},
        },
        traits::host::CredentialResolver,
    };
    use mcp_testkit::doubles::{
        approvals::AllowAllApprovals, audit::RecordingAuditSink,
        authorization::AllowAllAuthorization,
    };

    struct MockTelegram;

    #[async_trait]
    impl TelegramProvider for MockTelegram {
        async fn get_identity(
            &self,
            credential: &ProviderCredential,
        ) -> Result<crate::schemas::identity::TelegramIdentity, String> {
            assert_eq!(credential.expose_secret(), "mock-telegram-token");
            Ok(crate::schemas::identity::TelegramIdentity {
                username: "mock_telegram".to_string(),
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
            assert_eq!(request.provider.as_str(), "telegram");
            ProviderCredential::new("mock-telegram-token")
        }
    }

    #[tokio::test]
    async fn provider_receives_host_injected_credential() {
        let server = TelegramServer::new(Arc::new(MockTelegram)).unwrap();
        let services = mcp_sdk::schemas::context::HostServices {
            credentials: Arc::new(MockCredentialResolver),
            authorization: Arc::new(AllowAllAuthorization),
            approvals: Arc::new(AllowAllApprovals),
            audit: Arc::new(RecordingAuditSink::default()),
        };
        let context = RequestContext::new(
            mcp_protocol::schemas::json_rpc::RequestId::String("telegram-test".to_string()),
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
                    name: "telegram_get_identity".to_string(),
                    arguments: None,
                },
            )
            .await
            .unwrap();
        assert!(!result.is_error);
    }
}
