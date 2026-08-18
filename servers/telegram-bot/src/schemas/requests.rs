use serde::{Deserialize, Serialize};

pub const MAX_MESSAGE_TEXT_LENGTH: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetChatRequest {
    pub chat_id: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TelegramParseMode {
    #[serde(rename = "markdown_v2")]
    MarkdownV2,
    #[serde(rename = "html")]
    Html,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SendMessageRequest {
    pub chat_id: i64,
    pub text: String,
    #[serde(default)]
    pub parse_mode: Option<TelegramParseMode>,
}

impl SendMessageRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.text.trim().is_empty() {
            return Err("text must not be empty".to_string());
        }
        if self.text.chars().count() > MAX_MESSAGE_TEXT_LENGTH {
            return Err(format!(
                "text must be at most {MAX_MESSAGE_TEXT_LENGTH} characters"
            ));
        }
        Ok(())
    }
}
