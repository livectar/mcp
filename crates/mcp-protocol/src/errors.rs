use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("invalid JSON payload: {0}")]
    InvalidJson(String),
    #[error("JSON payload exceeds {max_bytes} bytes")]
    PayloadTooLarge { max_bytes: usize },
    #[error("could not serialize JSON payload: {0}")]
    Serialization(String),
}
