use async_trait::async_trait;
use mcp_sdk::schemas::credentials::ProviderCredential;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{ChatFullInfo, ChatId, ParseMode},
    Bot,
};

use crate::{
    errors::TelegramBotError,
    schemas::{
        requests::{GetChatRequest, SendMessageRequest, TelegramParseMode},
        results::{TelegramBotIdentity, TelegramChat, TelegramChatKind, TelegramMessage},
    },
};

#[async_trait]
pub trait TelegramBotProvider: Send + Sync {
    async fn get_me(
        &self,
        credential: &ProviderCredential,
    ) -> Result<TelegramBotIdentity, TelegramBotError>;

    async fn get_chat(
        &self,
        credential: &ProviderCredential,
        request: GetChatRequest,
    ) -> Result<TelegramChat, TelegramBotError>;

    async fn send_message(
        &self,
        credential: &ProviderCredential,
        request: SendMessageRequest,
    ) -> Result<TelegramMessage, TelegramBotError>;
}

pub struct TeloxideTelegramBotProvider {
    api_url: Option<reqwest::Url>,
}

impl TeloxideTelegramBotProvider {
    pub fn new() -> Self {
        Self { api_url: None }
    }

    pub fn with_api_url(api_url: reqwest::Url) -> Self {
        Self {
            api_url: Some(api_url),
        }
    }

    fn bot(&self, credential: &ProviderCredential) -> Bot {
        let bot = Bot::new(credential.expose_secret());
        match &self.api_url {
            Some(api_url) => bot.set_api_url(api_url.clone()),
            None => bot,
        }
    }
}

impl Default for TeloxideTelegramBotProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TelegramBotProvider for TeloxideTelegramBotProvider {
    async fn get_me(
        &self,
        credential: &ProviderCredential,
    ) -> Result<TelegramBotIdentity, TelegramBotError> {
        let me = self.bot(credential).get_me().await?;
        Ok(TelegramBotIdentity {
            id: me.user.id.0,
            is_bot: me.user.is_bot,
            first_name: me.user.first_name,
            last_name: me.user.last_name,
            username: me.user.username,
            can_join_groups: me.can_join_groups,
            can_read_all_group_messages: me.can_read_all_group_messages,
            supports_inline_queries: me.supports_inline_queries,
            can_connect_to_business: me.can_connect_to_business,
            has_main_web_app: me.has_main_web_app,
        })
    }

    async fn get_chat(
        &self,
        credential: &ProviderCredential,
        request: GetChatRequest,
    ) -> Result<TelegramChat, TelegramBotError> {
        let chat = self
            .bot(credential)
            .get_chat(ChatId(request.chat_id))
            .await?;
        Ok(chat_info_to_schema(&chat))
    }

    async fn send_message(
        &self,
        credential: &ProviderCredential,
        request: SendMessageRequest,
    ) -> Result<TelegramMessage, TelegramBotError> {
        let mut send_message = self
            .bot(credential)
            .send_message(ChatId(request.chat_id), request.text);
        if let Some(parse_mode) = request.parse_mode {
            send_message = send_message.parse_mode(to_teloxide_parse_mode(parse_mode));
        }
        let message = send_message.await?;
        Ok(TelegramMessage {
            message_id: message.id.0,
            chat_id: message.chat.id.0,
            date_unix_seconds: message.date.timestamp(),
            text: message.text().map(str::to_owned),
        })
    }
}

fn to_teloxide_parse_mode(parse_mode: TelegramParseMode) -> ParseMode {
    match parse_mode {
        TelegramParseMode::MarkdownV2 => ParseMode::MarkdownV2,
        TelegramParseMode::Html => ParseMode::Html,
    }
}

fn chat_info_to_schema(chat: &ChatFullInfo) -> TelegramChat {
    let kind = if chat.is_private() {
        TelegramChatKind::Private
    } else if chat.is_group() {
        TelegramChatKind::Group
    } else if chat.is_supergroup() {
        TelegramChatKind::Supergroup
    } else if chat.is_channel() {
        TelegramChatKind::Channel
    } else {
        TelegramChatKind::Unknown
    };

    TelegramChat {
        id: chat.id.0,
        kind,
        title: chat.title().map(str::to_owned),
        username: chat.username().map(str::to_owned),
        first_name: chat.first_name().map(str::to_owned),
        last_name: chat.last_name().map(str::to_owned),
        description: chat.description().map(str::to_owned),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Json, Router};
    use mcp_sdk::schemas::credentials::ProviderCredential;
    use serde::Serialize;
    use tokio::net::TcpListener;

    #[derive(Serialize)]
    struct ApiResponse<T> {
        ok: bool,
        result: T,
    }

    #[derive(Serialize)]
    struct UserFixture {
        id: u64,
        is_bot: bool,
        first_name: String,
        last_name: Option<String>,
        username: Option<String>,
    }

    #[derive(Serialize)]
    struct MeFixture {
        #[serde(flatten)]
        user: UserFixture,
        can_join_groups: bool,
        can_read_all_group_messages: bool,
        supports_inline_queries: bool,
        can_connect_to_business: bool,
        has_main_web_app: bool,
    }

    async fn get_me() -> Json<ApiResponse<MeFixture>> {
        Json(ApiResponse {
            ok: true,
            result: MeFixture {
                user: UserFixture {
                    id: 42,
                    is_bot: true,
                    first_name: "Mock Bot".to_string(),
                    last_name: None,
                    username: Some("mock_bot".to_string()),
                },
                can_join_groups: true,
                can_read_all_group_messages: false,
                supports_inline_queries: false,
                can_connect_to_business: false,
                has_main_web_app: false,
            },
        })
    }

    #[tokio::test]
    async fn teloxide_adapter_maps_get_me_from_bot_api() {
        let app = Router::new().fallback(get_me);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let provider = TeloxideTelegramBotProvider::with_api_url(
            reqwest::Url::parse(&format!("http://{address}/")).unwrap(),
        );
        let credential = ProviderCredential::new("mock-token").unwrap();
        let result = provider.get_me(&credential).await.unwrap();

        assert_eq!(result.id, 42);
        assert_eq!(result.username.as_deref(), Some("mock_bot"));
        assert!(result.can_join_groups);
        task.abort();
    }

    #[test]
    fn maps_both_supported_parse_modes() {
        assert_eq!(
            to_teloxide_parse_mode(TelegramParseMode::MarkdownV2),
            ParseMode::MarkdownV2
        );
        assert_eq!(
            to_teloxide_parse_mode(TelegramParseMode::Html),
            ParseMode::Html
        );
    }
}
