use serde::{Deserialize, Serialize};

pub const MAX_MESSAGE_TEXT_LENGTH: usize = 4096;
pub const MAX_CHAT_USERNAME_LENGTH: usize = 64;
pub const MAX_UPDATES_LIMIT: u8 = 100;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TelegramChatId {
    Numeric(i64),
    Username(String),
}

impl TelegramChatId {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Numeric(value) => {
                if *value == 0 {
                    return Err("chat_id must not be zero".to_string());
                }
            }
            Self::Username(value) => {
                let value = value.strip_prefix('@').unwrap_or(value);
                if let Ok(numeric) = value.parse::<i64>() {
                    if numeric == 0 {
                        return Err("chat_id must not be zero".to_string());
                    }
                    return Ok(());
                }
                if value.is_empty()
                    || value.len() > MAX_CHAT_USERNAME_LENGTH
                    || !value
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                {
                    return Err(
                        "chat username must contain only letters, numbers, or underscores"
                            .to_string(),
                    );
                }
            }
        }
        Ok(())
    }

    pub fn numeric(&self) -> Option<i64> {
        match self {
            Self::Numeric(value) => Some(*value),
            Self::Username(value) => value.parse().ok(),
        }
    }

    pub fn username(&self) -> Option<&str> {
        match self {
            Self::Numeric(_) => None,
            Self::Username(value) => Some(value.strip_prefix('@').unwrap_or(value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetChatRequest {
    pub chat_id: TelegramChatId,
}

impl GetChatRequest {
    pub fn validate(&self) -> Result<(), String> {
        self.chat_id.validate()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetUpdatesRequest {
    #[serde(default)]
    pub offset: Option<i32>,
    #[serde(default)]
    pub limit: Option<u8>,
}

impl GetUpdatesRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self
            .limit
            .is_some_and(|limit| limit == 0 || limit > MAX_UPDATES_LIMIT)
        {
            return Err(format!("limit must be between 1 and {MAX_UPDATES_LIMIT}"));
        }
        Ok(())
    }
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
    pub chat_id: TelegramChatId,
    pub text: String,
    #[serde(default)]
    pub parse_mode: Option<TelegramParseMode>,
}

impl SendMessageRequest {
    pub fn validate(&self) -> Result<(), String> {
        self.chat_id.validate()?;
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

#[cfg(test)]
mod tests {
    use super::{GetChatRequest, GetUpdatesRequest, SendMessageRequest, TelegramChatId};

    #[test]
    fn accepts_numeric_ids_and_usernames() {
        let numeric = serde_json::from_str::<GetChatRequest>(r#"{"chat_id":-100123}"#)
            .expect("numeric chat IDs should decode");
        assert_eq!(numeric.chat_id.numeric(), Some(-100123));

        let username = serde_json::from_str::<GetChatRequest>(r#"{"chat_id":"@example_channel"}"#)
            .expect("usernames should decode");
        assert_eq!(username.chat_id.username(), Some("example_channel"));
        username.validate().expect("valid usernames should pass");
    }

    #[test]
    fn accepts_numeric_ids_sent_as_strings() {
        let request =
            serde_json::from_str::<SendMessageRequest>(r#"{"chat_id":"-100123","text":"hello"}"#)
                .expect("numeric string chat IDs should decode");
        request.validate().expect("numeric string IDs should pass");
        assert_eq!(request.chat_id.numeric(), Some(-100123));
    }

    #[test]
    fn rejects_zero_chat_ids() {
        let request = GetChatRequest {
            chat_id: TelegramChatId::Numeric(0),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn validates_update_polling_limits() {
        GetUpdatesRequest {
            offset: Some(42),
            limit: Some(100),
        }
        .validate()
        .expect("Telegram accepts limits up to 100");

        assert!(GetUpdatesRequest {
            offset: None,
            limit: Some(0),
        }
        .validate()
        .is_err());
        assert!(GetUpdatesRequest {
            offset: None,
            limit: Some(101),
        }
        .validate()
        .is_err());
    }
}
