use serde::{Deserialize, Serialize};

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
