use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{
    de::DeserializeOwned, de::Error as _, Deserialize, Deserializer, Serialize, Serializer,
};
use thiserror::Error;

/// Maximum encoded size of an opaque MCP continuation cursor.
pub const MAX_CURSOR_BYTES: usize = 2048;

/// A typed payload that can be carried by an opaque continuation cursor.
pub trait CursorPayload: Serialize + DeserializeOwned {}

impl<T> CursorPayload for T where T: Serialize + DeserializeOwned {}

/// Errors returned while creating or decoding an opaque continuation cursor.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CursorError {
    #[error("cursor must not be empty")]
    Empty,
    #[error("cursor exceeds the safe size limit of {max_bytes} bytes")]
    TooLarge { max_bytes: usize },
    #[error("cursor must not contain control characters")]
    ControlCharacter,
    #[error("cursor payload could not be encoded")]
    Encode,
    #[error("cursor encoding is invalid")]
    InvalidEncoding,
    #[error("cursor payload is invalid")]
    InvalidPayload,
}

/// An opaque, bounded cursor shared by all MCP servers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpaqueCursor(String);

impl OpaqueCursor {
    pub fn new(value: impl Into<String>) -> Result<Self, CursorError> {
        let value = value.into();
        if value.is_empty() {
            return Err(CursorError::Empty);
        }
        if value.len() > MAX_CURSOR_BYTES {
            return Err(CursorError::TooLarge {
                max_bytes: MAX_CURSOR_BYTES,
            });
        }
        if value.chars().any(char::is_control) {
            return Err(CursorError::ControlCharacter);
        }
        Ok(Self(value))
    }

    pub fn encode<T: CursorPayload>(payload: &T) -> Result<Self, CursorError> {
        let bytes = serde_json::to_vec(payload).map_err(|_| CursorError::Encode)?;
        Self::new(URL_SAFE_NO_PAD.encode(bytes))
    }

    pub fn decode<T: CursorPayload>(&self) -> Result<T, CursorError> {
        let bytes = URL_SAFE_NO_PAD
            .decode(self.as_str())
            .map_err(|_| CursorError::InvalidEncoding)?;
        serde_json::from_slice(&bytes).map_err(|_| CursorError::InvalidPayload)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for OpaqueCursor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for OpaqueCursor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

/// The common wire contract for cursors exposed by paginated MCP results.
pub trait Cursor: Serialize + DeserializeOwned {
    fn as_str(&self) -> &str;
}

impl Cursor for OpaqueCursor {
    fn as_str(&self) -> &str {
        OpaqueCursor::as_str(self)
    }
}

/// A result that can tell callers whether another page must be requested.
pub trait Paginated {
    type Cursor: Cursor;

    fn next_cursor(&self) -> Option<&Self::Cursor>;

    fn is_complete(&self) -> bool {
        self.next_cursor().is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::{Cursor, CursorPayload, OpaqueCursor, Paginated, MAX_CURSOR_BYTES};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct TestCursorPayload {
        offset: u32,
    }

    fn assert_cursor_payload<T: CursorPayload>() {}

    fn assert_cursor<T: Cursor>() {}

    #[test]
    fn opaque_cursor_round_trips_typed_payloads() {
        assert_cursor_payload::<TestCursorPayload>();
        assert_cursor::<OpaqueCursor>();
        let cursor = OpaqueCursor::encode(&TestCursorPayload { offset: 3 }).unwrap();

        assert_eq!(
            cursor.decode::<TestCursorPayload>().unwrap(),
            TestCursorPayload { offset: 3 }
        );
    }

    #[test]
    fn opaque_cursor_rejects_values_over_the_shared_limit() {
        let value = "x".repeat(MAX_CURSOR_BYTES + 1);

        assert!(OpaqueCursor::new(value).is_err());
    }

    struct TestPage {
        next: Option<OpaqueCursor>,
    }

    impl Paginated for TestPage {
        type Cursor = OpaqueCursor;

        fn next_cursor(&self) -> Option<&Self::Cursor> {
            self.next.as_ref()
        }
    }

    #[test]
    fn pagination_is_complete_only_without_a_cursor() {
        let complete = TestPage { next: None };
        let incomplete = TestPage {
            next: Some(OpaqueCursor::new("next").unwrap()),
        };

        assert!(complete.is_complete());
        assert!(!incomplete.is_complete());
    }
}
