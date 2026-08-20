use thiserror::Error;

#[derive(Debug, Error)]
pub enum TelegramBotError {
    #[error("invalid Telegram chat ID: {0}")]
    InvalidChatId(String),

    #[error("Telegram Bot API request failed: {0}")]
    Api(String),
}

impl From<teloxide::RequestError> for TelegramBotError {
    fn from(error: teloxide::RequestError) -> Self {
        Self::Api(error.to_string())
    }
}
