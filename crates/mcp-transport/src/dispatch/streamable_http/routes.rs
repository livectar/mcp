use super::{
    dispatch::{dispatch, resolve_server},
    headers::{header_accepts, validate_origin},
    response::{method_not_allowed, text_response},
    state::TransportState,
};
use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    http::{HeaderMap, Response, StatusCode},
};

pub(crate) async fn post(
    State(state): State<TransportState>,
    Path(server_key): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response<Body> {
    dispatch(state, server_key, headers, body).await
}

pub(crate) async fn get(
    State(state): State<TransportState>,
    Path(server_key): Path<String>,
    headers: HeaderMap,
) -> Response<Body> {
    let Some(server) = resolve_server(&state, &server_key) else {
        return text_response(StatusCode::NOT_FOUND, "unknown MCP server");
    };
    if let Err(response) = validate_origin(&state, &headers) {
        return response;
    }
    if !server.supports_server_sent_events() {
        return method_not_allowed("GET");
    }
    if !header_accepts(&headers, "text/event-stream") {
        return text_response(
            StatusCode::NOT_ACCEPTABLE,
            "MCP GET requires text/event-stream",
        );
    }
    method_not_allowed("GET")
}

pub(crate) async fn delete(
    State(state): State<TransportState>,
    Path(server_key): Path<String>,
    headers: HeaderMap,
) -> Response<Body> {
    if resolve_server(&state, &server_key).is_none() {
        return text_response(StatusCode::NOT_FOUND, "unknown MCP server");
    }
    if let Err(response) = validate_origin(&state, &headers) {
        return response;
    }
    method_not_allowed("DELETE")
}
