use crate::{
    errors::{ServerError, ToolRegistryError},
    schemas::{context::RequestContext, tool_definition::ToolDefinition},
    traits::server::ToolHandler,
};
use mcp_protocol::schemas::tools::{CallToolParams, CallToolResult};
use std::sync::Arc;

struct RegisteredTool {
    definition: ToolDefinition,
    handler: Arc<dyn ToolHandler>,
}

pub struct ToolRegistry {
    tools: Vec<RegisteredTool>,
}

impl ToolRegistry {
    pub fn try_new(handlers: Vec<Arc<dyn ToolHandler>>) -> Result<Self, ToolRegistryError> {
        let mut tools = Vec::with_capacity(handlers.len());
        for handler in handlers {
            let definition = handler.definition();
            if tools
                .iter()
                .any(|tool: &RegisteredTool| tool.definition.name == definition.name)
            {
                return Err(ToolRegistryError::DuplicateToolName(definition.name));
            }
            tools.push(RegisteredTool {
                definition,
                handler,
            });
        }
        Ok(Self { tools })
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .iter()
            .map(|tool| tool.definition.clone())
            .collect()
    }

    pub async fn call(
        &self,
        context: &RequestContext,
        request: CallToolParams,
    ) -> Result<CallToolResult, ServerError> {
        let Some(tool) = self
            .tools
            .iter()
            .find(|tool| tool.definition.name == request.name)
        else {
            return Err(ServerError::ToolNotFound(request.name));
        };
        tool.handler.call(context, request.arguments).await
    }
}

#[cfg(test)]
mod tests {
    use super::ToolRegistry;
    use crate::{
        errors::{HostError, ServerError},
        schemas::{
            audit::AuditEvent,
            authorization::{ApprovalDecision, AuthorizationRequest},
            context::RequestContext,
            credentials::{CredentialRequest, ProviderCredential},
            tool_definition::{ToolAnnotations, ToolDefinition},
            tool_schema::ToolInputSchema,
        },
        traits::{
            host::{AuditSink, Authorization, CredentialResolver, ToolApprovalContext},
            server::ToolHandler,
        },
    };
    use async_trait::async_trait;
    use mcp_protocol::schemas::{
        json_payload::JsonPayload,
        tools::{CallToolParams, CallToolResult, ContentBlock},
    };

    struct TestHandler {
        name: &'static str,
    }

    #[async_trait]
    impl ToolHandler for TestHandler {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                name: self.name.to_string(),
                description: "test tool".to_string(),
                input_schema: ToolInputSchema::object(&[], &[]),
                annotations: ToolAnnotations::default(),
            }
        }

        async fn call(
            &self,
            _context: &RequestContext,
            _arguments: Option<JsonPayload>,
        ) -> Result<CallToolResult, ServerError> {
            Ok(CallToolResult {
                content: vec![ContentBlock::Text {
                    text: self.name.to_string(),
                }],
                structured_content: None,
                is_error: false,
            })
        }
    }

    fn handler(name: &'static str) -> std::sync::Arc<dyn ToolHandler> {
        std::sync::Arc::new(TestHandler { name })
    }

    struct TestCredentialResolver;

    #[async_trait]
    impl CredentialResolver for TestCredentialResolver {
        async fn resolve(
            &self,
            _caller: &crate::schemas::caller::CallerContext,
            _request: &CredentialRequest,
        ) -> Result<ProviderCredential, HostError> {
            Err(HostError::CredentialUnavailable)
        }
    }

    struct TestAuthorization;

    #[async_trait]
    impl Authorization for TestAuthorization {
        async fn authorize(
            &self,
            _caller: &crate::schemas::caller::CallerContext,
            _request: &AuthorizationRequest,
        ) -> Result<(), HostError> {
            Ok(())
        }
    }

    struct TestApprovalContext;

    #[async_trait]
    impl ToolApprovalContext for TestApprovalContext {
        async fn decision(
            &self,
            _caller: &crate::schemas::caller::CallerContext,
            _request: &AuthorizationRequest,
        ) -> Result<ApprovalDecision, HostError> {
            Ok(ApprovalDecision::Allowed)
        }
    }

    struct TestAuditSink;

    #[async_trait]
    impl AuditSink for TestAuditSink {
        async fn record(&self, _event: AuditEvent) -> Result<(), HostError> {
            Ok(())
        }
    }

    #[test]
    fn caches_definitions_and_rejects_duplicate_names() {
        let registry = ToolRegistry::try_new(vec![handler("one"), handler("two")]).unwrap();
        assert_eq!(
            registry
                .definitions()
                .into_iter()
                .map(|tool| tool.name)
                .collect::<Vec<_>>(),
            vec!["one", "two"]
        );

        let error = ToolRegistry::try_new(vec![handler("one"), handler("one")])
            .err()
            .unwrap();
        assert!(error.to_string().contains("duplicate MCP tool name"));
    }

    #[tokio::test]
    async fn dispatches_by_cached_tool_name() {
        let registry = ToolRegistry::try_new(vec![handler("one")]).unwrap();
        let context = RequestContext::new(
            mcp_protocol::schemas::json_rpc::RequestId::Number(1),
            crate::schemas::caller::CallerContext {
                tenant_id: "tenant".to_string(),
                subject_id: "subject".to_string(),
                installation_id: None,
                connection_id: None,
            },
            crate::schemas::context::HostServices {
                credentials: std::sync::Arc::new(TestCredentialResolver),
                authorization: std::sync::Arc::new(TestAuthorization),
                approvals: std::sync::Arc::new(TestApprovalContext),
                audit: std::sync::Arc::new(TestAuditSink),
            },
        );
        let result = registry
            .call(
                &context,
                CallToolParams {
                    name: "one".to_string(),
                    arguments: None,
                },
            )
            .await
            .unwrap();
        assert!(!result.is_error);
    }
}
