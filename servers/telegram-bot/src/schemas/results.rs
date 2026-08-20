use serde::{Deserialize, Serialize};
use teloxide::types::{Chat, ChatFullInfo, Update, UpdateKind};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TelegramBotIdentity {
    pub id: u64,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub can_join_groups: bool,
    pub can_read_all_group_messages: bool,
    pub supports_inline_queries: bool,
    pub can_connect_to_business: bool,
    pub has_main_web_app: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TelegramChatKind {
    Private,
    Group,
    Supergroup,
    Channel,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TelegramChat {
    pub id: i64,
    pub kind: TelegramChatKind,
    pub title: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TelegramMessage {
    pub message_id: i32,
    pub chat_id: i64,
    pub date_unix_seconds: i64,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TelegramUpdateKind {
    Message,
    EditedMessage,
    ChannelPost,
    EditedChannelPost,
    BusinessMessage,
    EditedBusinessMessage,
    CallbackQuery,
    MyChatMember,
    ChatMember,
    ChatJoinRequest,
    MessageReaction,
    MessageReactionCount,
    ChatBoost,
    RemovedChatBoost,
    DeletedBusinessMessages,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TelegramChatUpdate {
    pub update_id: u32,
    pub kind: TelegramUpdateKind,
    pub chat_id: i64,
    pub chat: TelegramChat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TelegramUpdates {
    pub updates: Vec<TelegramChatUpdate>,
    pub next_offset: Option<i32>,
    pub ignored_update_count: u32,
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum TelegramUpdateConversionError {
    #[error("Telegram update does not contain a chat")]
    MissingChat,
    #[error("Telegram update kind is not supported for chat discovery")]
    UnsupportedKind,
}

impl TryFrom<&UpdateKind> for TelegramUpdateKind {
    type Error = TelegramUpdateConversionError;

    fn try_from(value: &UpdateKind) -> Result<Self, Self::Error> {
        match value {
            UpdateKind::Message(_) => Ok(Self::Message),
            UpdateKind::EditedMessage(_) => Ok(Self::EditedMessage),
            UpdateKind::ChannelPost(_) => Ok(Self::ChannelPost),
            UpdateKind::EditedChannelPost(_) => Ok(Self::EditedChannelPost),
            UpdateKind::BusinessMessage(_) => Ok(Self::BusinessMessage),
            UpdateKind::EditedBusinessMessage(_) => Ok(Self::EditedBusinessMessage),
            UpdateKind::CallbackQuery(_) => Ok(Self::CallbackQuery),
            UpdateKind::MyChatMember(_) => Ok(Self::MyChatMember),
            UpdateKind::ChatMember(_) => Ok(Self::ChatMember),
            UpdateKind::ChatJoinRequest(_) => Ok(Self::ChatJoinRequest),
            UpdateKind::MessageReaction(_) => Ok(Self::MessageReaction),
            UpdateKind::MessageReactionCount(_) => Ok(Self::MessageReactionCount),
            UpdateKind::ChatBoost(_) => Ok(Self::ChatBoost),
            UpdateKind::RemovedChatBoost(_) => Ok(Self::RemovedChatBoost),
            UpdateKind::DeletedBusinessMessages(_) => Ok(Self::DeletedBusinessMessages),
            UpdateKind::BusinessConnection(_)
            | UpdateKind::ChosenInlineResult(_)
            | UpdateKind::InlineQuery(_)
            | UpdateKind::Poll(_)
            | UpdateKind::PollAnswer(_)
            | UpdateKind::PreCheckoutQuery(_)
            | UpdateKind::PurchasedPaidMedia(_)
            | UpdateKind::ShippingQuery(_)
            | UpdateKind::Error(_) => Err(TelegramUpdateConversionError::UnsupportedKind),
        }
    }
}

impl TryFrom<&Update> for TelegramChatUpdate {
    type Error = TelegramUpdateConversionError;

    fn try_from(value: &Update) -> Result<Self, Self::Error> {
        let chat = value
            .chat()
            .ok_or(TelegramUpdateConversionError::MissingChat)?;
        let kind = TelegramUpdateKind::try_from(&value.kind)?;

        Ok(Self {
            update_id: value.id.0,
            kind,
            chat_id: chat.id.0,
            chat: TelegramChat::from(chat),
        })
    }
}

impl From<&Chat> for TelegramChat {
    fn from(value: &Chat) -> Self {
        let kind = if value.is_private() {
            TelegramChatKind::Private
        } else if value.is_group() {
            TelegramChatKind::Group
        } else if value.is_supergroup() {
            TelegramChatKind::Supergroup
        } else if value.is_channel() {
            TelegramChatKind::Channel
        } else {
            TelegramChatKind::Unknown
        };

        Self {
            id: value.id.0,
            kind,
            title: value.title().map(str::to_owned),
            username: value.username().map(str::to_owned),
            first_name: value.first_name().map(str::to_owned),
            last_name: value.last_name().map(str::to_owned),
            description: None,
        }
    }
}

impl From<&ChatFullInfo> for TelegramChat {
    fn from(value: &ChatFullInfo) -> Self {
        let kind = if value.is_private() {
            TelegramChatKind::Private
        } else if value.is_group() {
            TelegramChatKind::Group
        } else if value.is_supergroup() {
            TelegramChatKind::Supergroup
        } else if value.is_channel() {
            TelegramChatKind::Channel
        } else {
            TelegramChatKind::Unknown
        };

        Self {
            id: value.id.0,
            kind,
            title: value.title().map(str::to_owned),
            username: value.username().map(str::to_owned),
            first_name: value.first_name().map(str::to_owned),
            last_name: value.last_name().map(str::to_owned),
            description: value.description().map(str::to_owned),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TelegramChatKind, TelegramChatUpdate, TelegramUpdateKind};
    use teloxide::types::Update;

    #[test]
    fn converts_message_updates_to_exact_chat_ids() {
        let update: Update = serde_json::from_str(
            r#"{
                "update_id": 7,
                "message": {
                    "message_id": 3,
                    "date": 1700000000,
                    "chat": {
                        "id": -1005457542726,
                        "type": "supergroup",
                        "title": "Example group"
                    },
                    "text": "/chat_id"
                }
            }"#,
        )
        .expect("Telegram message update should decode");
        let result =
            TelegramChatUpdate::try_from(&update).expect("Telegram message update should convert");

        assert_eq!(result.chat_id, -1005457542726);
        assert_eq!(result.kind, TelegramUpdateKind::Message);
        assert_eq!(result.chat.kind, TelegramChatKind::Supergroup);
        assert_eq!(result.chat.title.as_deref(), Some("Example group"));
    }
}
