use mcp_sdk::errors::ServerError;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoogleErrorCategory {
    InvalidRequest,
    Authentication,
    MissingScope,
    PermissionDenied,
    NotFound,
    Conflict,
    RateLimited,
    Upstream,
    Transport,
    InvalidResponse,
}

impl GoogleErrorCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::Authentication => "authentication",
            Self::MissingScope => "missing_scope",
            Self::PermissionDenied => "permission_denied",
            Self::NotFound => "not_found",
            Self::Conflict => "conflict",
            Self::RateLimited => "rate_limited",
            Self::Upstream => "upstream",
            Self::Transport => "transport",
            Self::InvalidResponse => "invalid_response",
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum GoogleProviderError {
    #[error("google provider error category={category} status={status:?} action={action}: {message}", category = category.as_str(), status = status, action = action.as_str())]
    Api {
        category: GoogleErrorCategory,
        status: Option<u16>,
        message: String,
        action: GoogleErrorAction,
        retry_after_seconds: Option<u64>,
    },
    #[error("google provider request failed category=transport action=retry: {message}")]
    Transport { message: String },
    #[error("google provider response exceeded the {max_bytes}-byte limit")]
    ResponseTooLarge { max_bytes: usize },
    #[error("google sheet cell at row {row}, column {column} exceeds the {max_bytes}-byte range cell limit; use sheets_read_cell_text for lossless chunks")]
    CellTooLarge {
        row: u32,
        column: u32,
        max_bytes: usize,
    },
    #[error("google sheet cell {cell} does not contain text or a formula")]
    CellNotText { cell: String },
    #[error("google provider returned an invalid response: {message}")]
    InvalidResponse { message: String },
    #[error("google provider retries exhausted category={category} attempts={attempts}: {message}", category = category.as_str())]
    RetryExhausted {
        category: GoogleErrorCategory,
        attempts: u32,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoogleErrorAction {
    Reauthorize,
    CheckPermissions,
    CheckSpreadsheetId,
    Retry,
    CheckRequest,
}

impl GoogleErrorAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reauthorize => "reauthorize_google_connection",
            Self::CheckPermissions => "check_google_permissions",
            Self::CheckSpreadsheetId => "check_spreadsheet_id_and_access",
            Self::Retry => "retry_later",
            Self::CheckRequest => "check_request",
        }
    }
}

impl GoogleProviderError {
    pub(crate) fn redact_secret(self, secret: &str) -> Self {
        if secret.is_empty() {
            return self;
        }
        let redact = |message: String| message.replace(secret, "[redacted]");
        match self {
            Self::Api {
                category,
                status,
                message,
                action,
                retry_after_seconds,
            } => Self::Api {
                category,
                status,
                message: redact(message),
                action,
                retry_after_seconds,
            },
            Self::Transport { message } => Self::Transport {
                message: redact(message),
            },
            Self::InvalidResponse { message } => Self::InvalidResponse {
                message: redact(message),
            },
            Self::RetryExhausted {
                category,
                attempts,
                message,
            } => Self::RetryExhausted {
                category,
                attempts,
                message: redact(message),
            },
            error => error,
        }
    }

    pub(crate) fn retryable(&self) -> bool {
        matches!(
            self,
            Self::Transport { .. }
                | Self::Api {
                    category: GoogleErrorCategory::RateLimited | GoogleErrorCategory::Upstream,
                    ..
                }
        )
    }

    pub(crate) fn retry_after_seconds(&self) -> Option<u64> {
        match self {
            Self::Api {
                retry_after_seconds,
                ..
            } => *retry_after_seconds,
            _ => None,
        }
    }

    pub(crate) fn into_retry_exhausted(self, attempts: u32) -> Self {
        match self {
            Self::Api {
                category, message, ..
            } if matches!(
                category,
                GoogleErrorCategory::RateLimited | GoogleErrorCategory::Upstream
            ) =>
            {
                Self::RetryExhausted {
                    category,
                    attempts,
                    message,
                }
            }
            Self::Transport { message } => Self::RetryExhausted {
                category: GoogleErrorCategory::Transport,
                attempts,
                message,
            },
            error => error,
        }
    }
}

impl From<GoogleProviderError> for ServerError {
    fn from(error: GoogleProviderError) -> Self {
        Self::Provider(error.to_string())
    }
}
