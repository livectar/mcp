use axum::{
    body::Body,
    http::{header, Response, StatusCode},
};
use mcp_protocol::schemas::{
    json_payload::JsonPayload,
    json_rpc::{JsonRpcError, JsonRpcResponse, RequestId},
};

pub(crate) const JSON_CONTENT_TYPE: &str = "application/json";

pub(crate) fn accepted_response() -> Response<Body> {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::ACCEPTED;
    response
}

pub(crate) fn method_not_allowed(method: &str) -> Response<Body> {
    let mut response = text_response(
        StatusCode::METHOD_NOT_ALLOWED,
        if method == "GET" {
            "the selected MCP server does not support server-sent events"
        } else {
            "the selected MCP server is stateless"
        },
    );
    response.headers_mut().insert(
        header::ALLOW,
        if method == "GET" {
            header::HeaderValue::from_static("POST")
        } else {
            header::HeaderValue::from_static("POST, GET")
        },
    );
    response
}

pub(crate) fn rpc_success_response(
    status: StatusCode,
    id: RequestId,
    result: JsonPayload,
    max_response_bytes: usize,
) -> Response<Body> {
    serialize_rpc_response(
        status,
        JsonRpcResponse::success(id, result),
        max_response_bytes,
    )
}

pub(crate) fn rpc_error_response(
    status: StatusCode,
    id: RequestId,
    error: JsonRpcError,
    max_response_bytes: usize,
) -> Response<Body> {
    serialize_rpc_response(
        status,
        JsonRpcResponse::failure(id, error),
        max_response_bytes,
    )
}

fn serialize_rpc_response(
    status: StatusCode,
    response: JsonRpcResponse,
    max_response_bytes: usize,
) -> Response<Body> {
    let bytes = match serde_json::to_vec(&response) {
        Ok(bytes) if bytes.len() <= max_response_bytes => bytes,
        _ => {
            let fallback = serde_json::to_vec(&JsonRpcResponse::failure(
                response.id,
                JsonRpcError::internal("MCP response exceeds the configured size limit"),
            ))
            .unwrap_or_default();
            if fallback.len() <= max_response_bytes {
                fallback
            } else {
                Vec::new()
            }
        }
    };
    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = status;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static(JSON_CONTENT_TYPE),
    );
    response
}

pub(crate) fn text_response(status: StatusCode, message: &str) -> Response<Body> {
    let mut response = Response::new(Body::from(message.to_string()));
    *response.status_mut() = status;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    response
}
