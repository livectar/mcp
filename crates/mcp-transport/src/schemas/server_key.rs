use std::fmt;

const MAX_SERVER_KEY_BYTES: usize = 120;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ServerKey(String);

impl ServerKey {
    pub fn parse(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.is_empty() || value.len() > MAX_SERVER_KEY_BYTES {
            return None;
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        {
            return None;
        }
        Some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ServerKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("ServerKey").field(&self.0).finish()
    }
}
