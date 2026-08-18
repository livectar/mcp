use async_trait::async_trait;
use mcp_sdk::schemas::credentials::ProviderCredential;
use reqwest::{header::HeaderValue, Client, StatusCode, Url};
use serde::de::DeserializeOwned;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

use crate::{
    errors::{GoogleErrorAction, GoogleErrorCategory, GoogleProviderError},
    schemas::{
        cells::{ReadCellTextResult, ReadRangeResult},
        identifiers::limits::{CellLimit, PageSize},
        provider::GoogleErrorEnvelope,
        requests::{
            GetSpreadsheetRequest, ListSpreadsheetsRequest, ReadCellTextRequest, ReadRangeRequest,
            ReadSheetMetadataRequest,
        },
        results::{ListSpreadsheetsResult, SheetMetadataResult, SpreadsheetMetadataResult},
        scopes::{GoogleScope, GoogleSheetsScopeProfile},
    },
};

pub const DEFAULT_SHEETS_API_BASE_URL: &str = "https://sheets.googleapis.com/v4/";
pub const DEFAULT_DRIVE_API_BASE_URL: &str = "https://www.googleapis.com/drive/v3/";
pub(crate) const MAX_SAFE_RESULT_CELLS: u32 = 500;
pub(crate) const MAX_CELL_TEXT_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone)]
pub struct GoogleClientConfig {
    pub sheets_api_base_url: String,
    pub drive_api_base_url: String,
    pub request_timeout: Duration,
    pub max_response_bytes: usize,
    pub default_page_size: PageSize,
    pub max_page_size: PageSize,
    pub max_cells: CellLimit,
    pub max_retries: u8,
    pub retry_delay: Duration,
}

impl Default for GoogleClientConfig {
    fn default() -> Self {
        Self {
            sheets_api_base_url: DEFAULT_SHEETS_API_BASE_URL.to_string(),
            drive_api_base_url: DEFAULT_DRIVE_API_BASE_URL.to_string(),
            request_timeout: Duration::from_secs(15),
            max_response_bytes: 2 * 1024 * 1024,
            default_page_size: PageSize::new(25).expect("default page size is valid"),
            max_page_size: PageSize::new(100).expect("maximum page size is valid"),
            max_cells: CellLimit::new(500).expect("default cell limit is valid"),
            max_retries: 2,
            retry_delay: Duration::from_millis(200),
        }
    }
}

impl GoogleClientConfig {
    pub fn validate(&self) -> Result<(), GoogleProviderError> {
        if self.request_timeout.is_zero() {
            return Err(GoogleProviderError::InvalidResponse {
                message: "request timeout must be positive".to_string(),
            });
        }
        if self.max_response_bytes == 0 || self.max_response_bytes > 16 * 1024 * 1024 {
            return Err(GoogleProviderError::InvalidResponse {
                message: "maximum provider response bytes are outside the safe range".to_string(),
            });
        }
        if self.max_cells.get() > MAX_SAFE_RESULT_CELLS {
            return Err(GoogleProviderError::InvalidResponse {
                message: "maximum result cells exceed the MCP response safety limit".to_string(),
            });
        }
        if self.default_page_size.get() > self.max_page_size.get() {
            return Err(GoogleProviderError::InvalidResponse {
                message: "default page size cannot exceed maximum page size".to_string(),
            });
        }
        Url::parse(&self.sheets_api_base_url)
            .map_err(|_| GoogleProviderError::InvalidResponse {
                message: "Sheets API base URL is invalid".to_string(),
            })
            .and_then(|url| validate_api_url(&url, "Sheets API base URL"))?;
        Url::parse(&self.drive_api_base_url)
            .map_err(|_| GoogleProviderError::InvalidResponse {
                message: "Drive API base URL is invalid".to_string(),
            })
            .and_then(|url| validate_api_url(&url, "Drive API base URL"))?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct GoogleApiClient {
    pub(crate) http: Client,
    pub(crate) sheets_api_base_url: Url,
    pub(crate) drive_api_base_url: Url,
    pub(crate) config: Arc<GoogleClientConfig>,
}

impl Default for GoogleApiClient {
    fn default() -> Self {
        Self::new(GoogleClientConfig::default())
            .expect("default Google client configuration is valid")
    }
}

impl GoogleApiClient {
    pub fn new(config: GoogleClientConfig) -> Result<Self, GoogleProviderError> {
        config.validate()?;
        let http = Client::builder()
            .timeout(config.request_timeout)
            .user_agent("ai-social-mcp-google/0.1")
            .build()
            .map_err(|error| GoogleProviderError::Transport {
                message: bounded_message(error.to_string()),
            })?;
        Self::with_http_client(http, config)
    }

    pub fn with_http_client(
        http: Client,
        config: GoogleClientConfig,
    ) -> Result<Self, GoogleProviderError> {
        config.validate()?;
        let sheets_api_base_url = Url::parse(&config.sheets_api_base_url).map_err(|_| {
            GoogleProviderError::InvalidResponse {
                message: "Sheets API base URL is invalid".to_string(),
            }
        })?;
        let drive_api_base_url = Url::parse(&config.drive_api_base_url).map_err(|_| {
            GoogleProviderError::InvalidResponse {
                message: "Drive API base URL is invalid".to_string(),
            }
        })?;
        Ok(Self {
            http,
            sheets_api_base_url,
            drive_api_base_url,
            config: Arc::new(config),
        })
    }

    pub fn scope_profile() -> [GoogleScope; 2] {
        GoogleSheetsScopeProfile::read_only()
    }

    pub fn mutation_scope_profile() -> [GoogleScope; 1] {
        GoogleSheetsScopeProfile::mutations()
    }

    pub(crate) async fn get_json<T: DeserializeOwned>(
        &self,
        service: ApiService,
        path: &str,
        query: Vec<(&str, String)>,
        credential: &ProviderCredential,
        operation: &'static str,
    ) -> Result<T, GoogleProviderError> {
        let base_url = match service {
            ApiService::Sheets => &self.sheets_api_base_url,
            ApiService::Drive => &self.drive_api_base_url,
        };
        let url = base_url
            .join(path)
            .map_err(|_| GoogleProviderError::InvalidResponse {
                message: "Google API request path is invalid".to_string(),
            })?;
        let attempts = u32::from(self.config.max_retries) + 1;
        let mut attempt = 0_u32;
        loop {
            attempt += 1;
            match self
                .get_json_once(&url, &query, credential, operation)
                .await
            {
                Ok(value) => return Ok(value),
                Err(error) if error.retryable() && attempt < attempts => {
                    let delay = error
                        .retry_after_seconds()
                        .map(Duration::from_secs)
                        .unwrap_or(self.config.retry_delay)
                        .min(Duration::from_secs(5));
                    sleep(delay).await;
                }
                Err(error) if error.retryable() => {
                    return Err(error.into_retry_exhausted(attempts));
                }
                Err(error) => return Err(error),
            }
        }
    }

    async fn get_json_once<T: DeserializeOwned>(
        &self,
        url: &Url,
        query: &[(&str, String)],
        credential: &ProviderCredential,
        operation: &'static str,
    ) -> Result<T, GoogleProviderError> {
        let bearer = format!("Bearer {}", credential.expose_secret());
        let authorization =
            HeaderValue::try_from(bearer).map_err(|_| GoogleProviderError::Transport {
                message: "Google credential is not a valid authorization value".to_string(),
            })?;
        let response = self
            .http
            .get(url.clone())
            .query(query)
            .header(reqwest::header::AUTHORIZATION, authorization)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(|error| {
                if error.is_timeout() {
                    GoogleProviderError::Transport {
                        message: format!("{operation} timed out"),
                    }
                } else {
                    GoogleProviderError::Transport {
                        message: bounded_message(error.to_string()),
                    }
                }
            })?;
        let status = response.status();
        if response
            .content_length()
            .is_some_and(|length| length > self.config.max_response_bytes as u64)
        {
            return Err(GoogleProviderError::ResponseTooLarge {
                max_bytes: self.config.max_response_bytes,
            });
        }
        let retry_after_seconds = response
            .headers()
            .get("retry-after")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok());
        let body = response
            .bytes()
            .await
            .map_err(|error| GoogleProviderError::Transport {
                message: bounded_message(error.to_string()),
            })?;
        if body.len() > self.config.max_response_bytes {
            return Err(GoogleProviderError::ResponseTooLarge {
                max_bytes: self.config.max_response_bytes,
            });
        }
        if !status.is_success() {
            return Err(map_http_error(status, retry_after_seconds, &body)
                .redact_secret(credential.expose_secret()));
        }
        serde_json::from_slice(&body).map_err(|error| GoogleProviderError::InvalidResponse {
            message: format!(
                "{operation} payload is invalid: {}",
                bounded_message(error.to_string())
            ),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ApiService {
    Sheets,
    Drive,
}

#[async_trait]
pub trait GoogleProvider: Send + Sync {
    async fn list_spreadsheets(
        &self,
        credential: &ProviderCredential,
        request: ListSpreadsheetsRequest,
    ) -> Result<ListSpreadsheetsResult, GoogleProviderError>;

    async fn get_spreadsheet(
        &self,
        credential: &ProviderCredential,
        request: GetSpreadsheetRequest,
    ) -> Result<SpreadsheetMetadataResult, GoogleProviderError>;

    async fn read_range(
        &self,
        credential: &ProviderCredential,
        request: ReadRangeRequest,
    ) -> Result<ReadRangeResult, GoogleProviderError>;

    async fn read_cell_text(
        &self,
        credential: &ProviderCredential,
        request: ReadCellTextRequest,
    ) -> Result<ReadCellTextResult, GoogleProviderError>;

    async fn read_sheet_metadata(
        &self,
        credential: &ProviderCredential,
        request: ReadSheetMetadataRequest,
    ) -> Result<SheetMetadataResult, GoogleProviderError>;
}

#[async_trait]
impl GoogleProvider for GoogleApiClient {
    async fn list_spreadsheets(
        &self,
        credential: &ProviderCredential,
        request: ListSpreadsheetsRequest,
    ) -> Result<ListSpreadsheetsResult, GoogleProviderError> {
        self.list_spreadsheets_impl(credential, request).await
    }

    async fn get_spreadsheet(
        &self,
        credential: &ProviderCredential,
        request: GetSpreadsheetRequest,
    ) -> Result<SpreadsheetMetadataResult, GoogleProviderError> {
        self.get_spreadsheet_impl(credential, request).await
    }

    async fn read_range(
        &self,
        credential: &ProviderCredential,
        request: ReadRangeRequest,
    ) -> Result<ReadRangeResult, GoogleProviderError> {
        self.read_range_impl(credential, request).await
    }

    async fn read_cell_text(
        &self,
        credential: &ProviderCredential,
        request: ReadCellTextRequest,
    ) -> Result<ReadCellTextResult, GoogleProviderError> {
        self.read_cell_text_impl(credential, request).await
    }

    async fn read_sheet_metadata(
        &self,
        credential: &ProviderCredential,
        request: ReadSheetMetadataRequest,
    ) -> Result<SheetMetadataResult, GoogleProviderError> {
        self.read_sheet_metadata_impl(credential, request).await
    }
}

pub(crate) fn map_http_error(
    status: StatusCode,
    retry_after_seconds: Option<u64>,
    body: &[u8],
) -> GoogleProviderError {
    let envelope = serde_json::from_slice::<GoogleErrorEnvelope>(body).ok();
    let error = envelope.as_ref().map(|value| &value.error);
    let message = error
        .and_then(|value| value.message.as_deref())
        .or_else(|| {
            error.and_then(|value| {
                value
                    .errors
                    .first()
                    .and_then(|detail| detail.message.as_deref())
            })
        })
        .map(bounded_message)
        .unwrap_or_else(|| format!("Google API returned HTTP {}", status.as_u16()));
    let reason = error
        .and_then(|value| value.errors.first())
        .and_then(|detail| detail.reason.as_deref())
        .unwrap_or_default();
    let status_name = error
        .and_then(|value| value.status.as_deref())
        .unwrap_or_default();
    let category = match status {
        StatusCode::UNAUTHORIZED => GoogleErrorCategory::Authentication,
        StatusCode::FORBIDDEN if is_missing_scope(reason, status_name, &message) => {
            GoogleErrorCategory::MissingScope
        }
        StatusCode::FORBIDDEN => GoogleErrorCategory::PermissionDenied,
        StatusCode::NOT_FOUND => GoogleErrorCategory::NotFound,
        StatusCode::CONFLICT => GoogleErrorCategory::Conflict,
        StatusCode::TOO_MANY_REQUESTS => GoogleErrorCategory::RateLimited,
        status if status.is_server_error() => GoogleErrorCategory::Upstream,
        _ => GoogleErrorCategory::InvalidRequest,
    };
    let action = match category {
        GoogleErrorCategory::Authentication | GoogleErrorCategory::MissingScope => {
            GoogleErrorAction::Reauthorize
        }
        GoogleErrorCategory::PermissionDenied => GoogleErrorAction::CheckPermissions,
        GoogleErrorCategory::NotFound => GoogleErrorAction::CheckSpreadsheetId,
        GoogleErrorCategory::RateLimited | GoogleErrorCategory::Upstream => {
            GoogleErrorAction::Retry
        }
        _ => GoogleErrorAction::CheckRequest,
    };
    GoogleProviderError::Api {
        category,
        status: Some(
            error
                .and_then(|value| value.code)
                .unwrap_or(status.as_u16()),
        ),
        message,
        action,
        retry_after_seconds,
    }
}

fn is_missing_scope(reason: &str, status_name: &str, message: &str) -> bool {
    let haystack = format!("{reason} {status_name} {message}").to_ascii_lowercase();
    haystack.contains("insufficient") || haystack.contains("scope")
}

fn validate_api_url(url: &Url, label: &str) -> Result<(), GoogleProviderError> {
    let is_local_http = url.scheme() == "http"
        && url
            .host_str()
            .is_some_and(|host| matches!(host, "localhost" | "127.0.0.1" | "[::1]"));
    if (url.scheme() != "https" && !is_local_http) || url.host_str().is_none() {
        return Err(GoogleProviderError::InvalidResponse {
            message: format!("{label} must be an absolute HTTP(S) URL"),
        });
    }
    Ok(())
}

pub(crate) fn escape_drive_literal(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

pub(crate) fn invalid_provider_response(message: String) -> GoogleProviderError {
    GoogleProviderError::InvalidResponse { message }
}

fn bounded_message(value: impl Into<String>) -> String {
    value.into().chars().take(512).collect()
}
