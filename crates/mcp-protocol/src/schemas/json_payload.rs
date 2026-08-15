use crate::{constants::MAX_JSON_PAYLOAD_BYTES, errors::ProtocolError};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::value::RawValue;
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct JsonPayload(String);

impl JsonPayload {
    pub fn parse(value: impl Into<String>) -> Result<Self, ProtocolError> {
        let value = value.into();
        if value.len() > MAX_JSON_PAYLOAD_BYTES {
            return Err(ProtocolError::PayloadTooLarge {
                max_bytes: MAX_JSON_PAYLOAD_BYTES,
            });
        }
        serde_json::from_str::<Box<RawValue>>(&value)
            .map_err(|error| ProtocolError::InvalidJson(error.to_string()))?;
        Ok(Self(value))
    }

    pub fn from_serializable<T: Serialize>(value: &T) -> Result<Self, ProtocolError> {
        Self::parse(
            serde_json::to_string(value)
                .map_err(|error| ProtocolError::Serialization(error.to_string()))?,
        )
    }

    pub fn decode<T: DeserializeOwned>(&self) -> Result<T, ProtocolError> {
        serde_json::from_str(&self.0).map_err(|error| ProtocolError::InvalidJson(error.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for JsonPayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("JsonPayload").field(&self.0).finish()
    }
}

impl Serialize for JsonPayload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let raw =
            serde_json::from_str::<Box<RawValue>>(&self.0).map_err(serde::ser::Error::custom)?;
        raw.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for JsonPayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Box::<RawValue>::deserialize(deserializer)?;
        JsonPayload::parse(raw.get().to_owned()).map_err(serde::de::Error::custom)
    }
}
