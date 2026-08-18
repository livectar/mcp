use async_trait::async_trait;
use mcp_protocol::schemas::{
    json_payload::JsonPayload, json_rpc::RequestId, tools::CallToolParams,
};
use mcp_sdk::{
    errors::{HostError, ServerError},
    schemas::{
        caller::CallerContext,
        context::{HostServices, RequestContext},
        credentials::{CredentialRequest, ProviderCredential},
    },
    traits::{host::CredentialResolver, server::McpServer},
};
use mcp_testkit::doubles::{
    approvals::AllowAllApprovals, audit::RecordingAuditSink, authorization::AllowAllAuthorization,
};
use std::sync::Arc;

use mcp_telegram_bot::{
    errors::TelegramBotError,
    providers::telegram_bot::TelegramBotProvider,
    schemas::{
        requests::{GetChatRequest, SendMessageRequest, TelegramParseMode},
        results::{TelegramBotIdentity, TelegramChat, TelegramChatKind, TelegramMessage},
    },
    server::TelegramBotServer,
};

struct MockTelegramBot;

#[async_trait]
impl TelegramBotProvider for MockTelegramBot {
    async fn get_me(
        &self,
        credential: &ProviderCredential,
    ) -> Result<TelegramBotIdentity, TelegramBotError> {
        assert_eq!(credential.expose_secret(), "mock-telegram-bot-token");
        Ok(TelegramBotIdentity {
            id: 42,
            is_bot: true,
            first_name: "Mock Bot".to_string(),
            last_name: None,
            username: Some("mock_bot".to_string()),
            can_join_groups: true,
            can_read_all_group_messages: false,
            supports_inline_queries: false,
            can_connect_to_business: false,
            has_main_web_app: false,
        })
    }

    async fn get_chat(
        &self,
        credential: &ProviderCredential,
        request: GetChatRequest,
    ) -> Result<TelegramChat, TelegramBotError> {
        assert_eq!(credential.expose_secret(), "mock-telegram-bot-token");
        Ok(TelegramChat {
            id: request.chat_id,
            kind: TelegramChatKind::Private,
            title: None,
            username: Some("mock_chat".to_string()),
            first_name: Some("Mock".to_string()),
            last_name: None,
            description: None,
        })
    }

    async fn send_message(
        &self,
        credential: &ProviderCredential,
        request: SendMessageRequest,
    ) -> Result<TelegramMessage, TelegramBotError> {
        assert_eq!(credential.expose_secret(), "mock-telegram-bot-token");
        assert_eq!(request.parse_mode, Some(TelegramParseMode::MarkdownV2));
        Ok(TelegramMessage {
            message_id: 7,
            chat_id: request.chat_id,
            date_unix_seconds: 1_700_000_000,
            text: Some(request.text),
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
        assert_eq!(request.provider.as_str(), "telegram-bot");
        ProviderCredential::new("mock-telegram-bot-token")
    }
}

fn context() -> RequestContext {
    let services = HostServices {
        credentials: Arc::new(MockCredentialResolver),
        authorization: Arc::new(AllowAllAuthorization),
        approvals: Arc::new(AllowAllApprovals),
        audit: Arc::new(RecordingAuditSink::default()),
    };
    RequestContext::new(
        RequestId::String("telegram-bot-test".to_string()),
        CallerContext {
            tenant_id: "tenant".to_string(),
            subject_id: "subject".to_string(),
            installation_id: None,
            connection_id: None,
        },
        services,
    )
}

#[test]
fn exposes_expected_tools() {
    let server = TelegramBotServer::new(Arc::new(MockTelegramBot)).unwrap();
    let mut names = server
        .tools()
        .into_iter()
        .map(|tool| tool.name)
        .collect::<Vec<_>>();
    names.sort();
    assert_eq!(
        names,
        vec![
            "telegram_get_chat",
            "telegram_get_me",
            "telegram_send_message"
        ]
    );
}

#[tokio::test]
async fn provider_receives_host_injected_credential() {
    let server = TelegramBotServer::new(Arc::new(MockTelegramBot)).unwrap();
    let result = server
        .call_tool(
            &context(),
            CallToolParams {
                name: "telegram_get_me".to_string(),
                arguments: None,
            },
        )
        .await
        .unwrap();
    assert!(!result.is_error);
}

#[tokio::test]
async fn validates_and_dispatches_send_message() {
    let server = TelegramBotServer::new(Arc::new(MockTelegramBot)).unwrap();
    let arguments = JsonPayload::from_serializable(&SendMessageRequest {
        chat_id: 123,
        text: "hello".to_string(),
        parse_mode: Some(TelegramParseMode::MarkdownV2),
    })
    .unwrap();
    let result = server
        .call_tool(
            &context(),
            CallToolParams {
                name: "telegram_send_message".to_string(),
                arguments: Some(arguments),
            },
        )
        .await
        .unwrap();
    assert!(!result.is_error);
}

#[tokio::test]
async fn rejects_empty_message_text() {
    let server = TelegramBotServer::new(Arc::new(MockTelegramBot)).unwrap();
    let arguments = JsonPayload::from_serializable(&SendMessageRequest {
        chat_id: 123,
        text: " ".to_string(),
        parse_mode: None,
    })
    .unwrap();
    let error = server
        .call_tool(
            &context(),
            CallToolParams {
                name: "telegram_send_message".to_string(),
                arguments: Some(arguments),
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(error, ServerError::InvalidArguments(_)));
}
