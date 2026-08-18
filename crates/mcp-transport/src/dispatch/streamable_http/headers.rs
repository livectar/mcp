use super::{response::text_response, state::TransportState};
use axum::{
    body::Body,
    http::{header, HeaderMap, Response, StatusCode},
};
use mcp_protocol::{constants::PROTOCOL_REVISION, schemas::lifecycle::InitializeParams};
use mcp_sdk::traits::server::McpServer;
use std::sync::Arc;

const SSE_CONTENT_TYPE: &str = "text/event-stream";

pub(crate) fn validate_content_headers(headers: &HeaderMap) -> Result<(), Response<Body>> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.split(';').next().unwrap_or_default().trim());
    if content_type != Some("application/json") {
        return Err(text_response(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "MCP POST requests must use application/json",
        ));
    }

    let accepted_types = headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(',')
                .filter_map(|item| item.split(';').next())
                .map(str::trim)
                .collect::<Vec<_>>()
        });
    let Some(accepted_types) = accepted_types else {
        return Err(text_response(
            StatusCode::NOT_ACCEPTABLE,
            "MCP clients must accept application/json and text/event-stream",
        ));
    };
    if !accepted_types.contains(&"application/json") || !accepted_types.contains(&SSE_CONTENT_TYPE)
    {
        return Err(text_response(
            StatusCode::NOT_ACCEPTABLE,
            "MCP clients must accept application/json and text/event-stream",
        ));
    }
    Ok(())
}

pub(crate) fn validate_mcp_headers(
    headers: &HeaderMap,
    server: &Arc<dyn McpServer>,
    method: Option<&str>,
    initialize_params: Option<&InitializeParams>,
) -> Result<(), Response<Body>> {
    if let Some(header_method) = header_string(headers, "Mcp-Method")? {
        if method != Some(header_method.as_str()) {
            return Err(text_response(
                StatusCode::BAD_REQUEST,
                "Mcp-Method does not match the JSON-RPC message",
            ));
        }
    }

    let protocol_header = header_string(headers, "MCP-Protocol-Version")?;
    if method != Some("initialize") && protocol_header.as_deref() != Some(PROTOCOL_REVISION) {
        return Err(text_response(
            StatusCode::BAD_REQUEST,
            "MCP-Protocol-Version is required for this request",
        ));
    }
    if let Some(protocol_header) = protocol_header {
        if method == Some("initialize") {
            if initialize_params.map(|params| params.protocol_version.as_str())
                != Some(protocol_header.as_str())
            {
                return Err(text_response(
                    StatusCode::BAD_REQUEST,
                    "MCP-Protocol-Version does not match initialize params",
                ));
            }
        } else if protocol_header != PROTOCOL_REVISION {
            return Err(text_response(
                StatusCode::BAD_REQUEST,
                "unsupported MCP protocol version",
            ));
        }
    }

    if let Some(name) = header_string(headers, "Mcp-Name")? {
        if name != server.info().name {
            return Err(text_response(
                StatusCode::BAD_REQUEST,
                "Mcp-Name does not match the selected MCP server",
            ));
        }
    }
    Ok(())
}

pub(crate) fn header_string(
    headers: &HeaderMap,
    name: &str,
) -> Result<Option<String>, Response<Body>> {
    let Some(value) = headers.get(name) else {
        return Ok(None);
    };
    value
        .to_str()
        .map(|value| Some(value.to_string()))
        .map_err(|_| text_response(StatusCode::BAD_REQUEST, "MCP header contains invalid UTF-8"))
}

pub(crate) fn validate_origin(
    state: &TransportState,
    headers: &HeaderMap,
) -> Result<(), Response<Body>> {
    if let Some(origin) = header_string(headers, "Origin")? {
        if !state
            .allowed_origins
            .iter()
            .any(|allowed| allowed == &origin)
        {
            return Err(text_response(
                StatusCode::FORBIDDEN,
                "MCP request origin is not allowed",
            ));
        }
    }
    Ok(())
}

pub(crate) fn header_accepts(headers: &HeaderMap, media_type: &str) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(',')
                .filter_map(|item| item.split(';').next())
                .map(str::trim)
                .any(|item| item == media_type)
        })
        .unwrap_or(false)
}
