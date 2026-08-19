use crate::{
    errors::{GoogleErrorAction, GoogleErrorCategory, GoogleProviderError},
    schemas::{identifiers::ranges::A1Range, results::sheets::SheetTabMetadata},
};

use super::super::common::invalid_provider_response;

pub(super) fn parse_response_range(value: String) -> Result<A1Range, GoogleProviderError> {
    A1Range::new(value).map_err(invalid_provider_response)
}

pub(super) fn range_cell_count(range: &A1Range) -> Result<u32, GoogleProviderError> {
    let bounds = range
        .bounds()
        .map_err(invalid_provider_response)?
        .resolve(None, None)
        .map_err(invalid_provider_response)?;
    bounds
        .row_count()
        .checked_mul(bounds.column_count())
        .ok_or_else(|| {
            invalid_provider_response("mutation range cell count overflowed".to_string())
        })
}

pub(super) fn resolve_tab(
    tabs: &[SheetTabMetadata],
    requested_title: Option<&str>,
) -> Result<SheetTabMetadata, GoogleProviderError> {
    let tab = requested_title
        .and_then(|title| tabs.iter().find(|tab| tab.identity.title == title))
        .or_else(|| tabs.first())
        .ok_or_else(|| GoogleProviderError::Api {
            category: GoogleErrorCategory::NotFound,
            status: None,
            message: "spreadsheet has no readable sheet tabs".to_string(),
            action: GoogleErrorAction::CheckSpreadsheetId,
            retry_after_seconds: None,
        })?;
    if requested_title.is_some_and(|title| tab.identity.title != title) {
        return Err(GoogleProviderError::Api {
            category: GoogleErrorCategory::NotFound,
            status: None,
            message: "requested sheet tab was not found".to_string(),
            action: GoogleErrorAction::CheckSpreadsheetId,
            retry_after_seconds: None,
        });
    }
    Ok(tab.clone())
}

pub(super) fn invalid_request(message: impl Into<String>) -> GoogleProviderError {
    GoogleProviderError::Api {
        category: GoogleErrorCategory::InvalidRequest,
        status: None,
        message: message.into(),
        action: GoogleErrorAction::CheckRequest,
        retry_after_seconds: None,
    }
}
