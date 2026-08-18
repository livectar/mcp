use super::{
    headers::{validate_content_headers, validate_mcp_headers, validate_origin},
    requests::{
        parse_message, CallToolRequest, DispatchRequest, IncomingMessage, IncomingNotification,
        MessageParseError,
    },
    response::{accepted_response, rpc_error_response, rpc_result_response, text_response},
    state::TransportState,
};
use crate::schemas::{limits::TransportLimits, server_key::ServerKey};
use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, Response, StatusCode},
};
use mcp_protocol::schemas::json_payload::JsonPayload;
use mcp_protocol::{
    constants::PROTOCOL_REVISION,
    schemas::{
        json_rpc::{JsonRpcError, RequestId},
        lifecycle::{InitializeParams, InitializeResult},
        tools::{ListToolsParams, ListToolsResult},
    },
};
use mcp_sdk::{
    errors::{HostError, ServerError},
    schemas::context::RequestContext,
    traits::server::McpServer,
};
use std::sync::Arc;
use tokio::time::timeout;

pub(crate) async fn dispatch(
    state: TransportState,
    server_key: String,
    headers: HeaderMap,
    body: Bytes,
) -> Response<Body> {
    if body.len() > state.limits.max_request_bytes {
        return text_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            "MCP request exceeds the configured body limit",
        );
    }
    if let Err(response) = validate_content_headers(&headers) {
        return response;
    }

    let server = match resolve_server(&state, &server_key) {
        Some(server) => server,
        None => {
            return rpc_error_response(
                StatusCode::NOT_FOUND,
                RequestId::Null,
                JsonRpcError::invalid_request("unknown MCP server"),
                state.limits.max_response_bytes,
            )
        }
    };

    if let Err(response) = validate_origin(&state, &headers) {
        return response;
    }

    let caller = match resolve_caller(&state, &server_key, &headers).await {
        Ok(caller) => caller,
        Err(response) => return response,
    };

    let message = match parse_message(&body) {
        Ok(message) => message,
        Err(MessageParseError::InvalidRequest(error)) => {
            return rpc_error_response(
                StatusCode::BAD_REQUEST,
                RequestId::Null,
                error,
                state.limits.max_response_bytes,
            )
        }
        Err(MessageParseError::InvalidParams { id, error }) => {
            return rpc_error_response(StatusCode::OK, id, error, state.limits.max_response_bytes)
        }
    };

    if let Err(response) = validate_mcp_headers(
        &headers,
        &server,
        message.method(),
        message.initialize_params(),
    ) {
        return response;
    }

    match message {
        IncomingMessage::Request(request) => dispatch_request(state, caller, server, request).await,
        IncomingMessage::Notification(notification) => {
            dispatch_notification(state, caller, server, notification).await
        }
        IncomingMessage::Response(response) => {
            if response.jsonrpc != "2.0" {
                return rpc_error_response(
                    StatusCode::BAD_REQUEST,
                    response.id,
                    JsonRpcError::invalid_request("jsonrpc must be 2.0"),
                    state.limits.max_response_bytes,
                );
            }
            accepted_response()
        }
    }
}

async fn resolve_caller(
    state: &TransportState,
    server_key: &str,
    headers: &HeaderMap,
) -> Result<mcp_sdk::schemas::caller::CallerContext, Response<Body>> {
    let Some(resolver) = &state.caller_resolver else {
        return Ok(state.caller.clone());
    };
    let authorization = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    resolver
        .resolve(server_key, authorization)
        .await
        .map_err(|_| {
            rpc_error_response(
                StatusCode::UNAUTHORIZED,
                RequestId::Null,
                JsonRpcError::unauthorized("MCP caller authentication failed"),
                state.limits.max_response_bytes,
            )
        })
}

pub(crate) fn resolve_server(
    state: &TransportState,
    raw_server_key: &str,
) -> Option<Arc<dyn McpServer>> {
    let server_key = ServerKey::parse(raw_server_key.to_string())?;
    state.resolver.resolve(&server_key)
}

async fn dispatch_request(
    state: TransportState,
    caller: mcp_sdk::schemas::caller::CallerContext,
    server: Arc<dyn McpServer>,
    request: DispatchRequest,
) -> Response<Body> {
    let request_id = request.id().clone();
    let limits = Arc::clone(&state.limits);
    let concurrency = Arc::clone(&state.concurrency);
    let unavailable_request_id = request_id.clone();
    let request_timeout = limits.request_timeout;
    let max_response_bytes = limits.max_response_bytes;
    let dispatch = async move {
        let permit = match concurrency.acquire_owned().await {
            Ok(permit) => permit,
            Err(_) => {
                return rpc_error_response(
                    StatusCode::SERVICE_UNAVAILABLE,
                    unavailable_request_id,
                    JsonRpcError::internal("MCP transport concurrency is unavailable"),
                    max_response_bytes,
                )
            }
        };
        let response = dispatch_request_inner(state, caller, server, request).await;
        drop(permit);
        response
    };
    let result = timeout(request_timeout, dispatch).await;

    match result {
        Ok(response) => response,
        Err(_) => rpc_error_response(
            StatusCode::OK,
            request_id,
            JsonRpcError::internal("MCP request timed out"),
            limits.max_response_bytes,
        ),
    }
}

async fn dispatch_request_inner(
    state: TransportState,
    caller: mcp_sdk::schemas::caller::CallerContext,
    server: Arc<dyn McpServer>,
    request: DispatchRequest,
) -> Response<Body> {
    let request_id = request.id().clone();
    if request.jsonrpc() != "2.0" {
        return rpc_error_response(
            StatusCode::OK,
            request_id,
            JsonRpcError::invalid_request("jsonrpc must be 2.0"),
            state.limits.max_response_bytes,
        );
    }

    let context = RequestContext::new(request_id.clone(), caller, state.services);
    match request {
        DispatchRequest::Initialize { params, .. } => rpc_result_response(
            StatusCode::OK,
            request_id,
            initialize(&server, params),
            state.limits.max_response_bytes,
        ),
        DispatchRequest::ListTools { params, .. } => rpc_result_response(
            StatusCode::OK,
            request_id,
            list_tools(&server, params, &state.limits),
            state.limits.max_response_bytes,
        ),
        DispatchRequest::CallTool { request, .. } => {
            let result = call_tool(&server, &context, request, &state.limits).await;
            rpc_result_response(
                StatusCode::OK,
                request_id,
                result,
                state.limits.max_response_bytes,
            )
        }
        DispatchRequest::Unknown { method, .. } => rpc_error_response(
            StatusCode::OK,
            request_id,
            JsonRpcError::method_not_found(&method),
            state.limits.max_response_bytes,
        ),
    }
}

async fn dispatch_notification(
    state: TransportState,
    caller: mcp_sdk::schemas::caller::CallerContext,
    server: Arc<dyn McpServer>,
    notification: IncomingNotification,
) -> Response<Body> {
    if notification.jsonrpc() != "2.0" {
        return text_response(StatusCode::BAD_REQUEST, "jsonrpc must be 2.0");
    }

    match notification {
        IncomingNotification::Initialized { .. }
        | IncomingNotification::Other { .. }
        | IncomingNotification::Ignored { .. } => accepted_response(),
        IncomingNotification::CallTool { request, .. } => {
            let _ = dispatch_request(state, caller, server, request).await;
            accepted_response()
        }
    }
}

fn initialize(
    server: &Arc<dyn McpServer>,
    params: InitializeParams,
) -> Result<InitializeResult, JsonRpcError> {
    if params.protocol_version != PROTOCOL_REVISION {
        return Err(JsonRpcError::invalid_params(format!(
            "unsupported MCP protocol version: {}",
            params.protocol_version
        )));
    }
    Ok(InitializeResult {
        protocol_version: PROTOCOL_REVISION.to_string(),
        capabilities: server.capabilities(),
        server_info: server.info(),
    })
}

fn list_tools(
    server: &Arc<dyn McpServer>,
    params: Option<ListToolsParams>,
    limits: &TransportLimits,
) -> Result<ListToolsResult, JsonRpcError> {
    let mut tools = server.tools();
    tools.sort_by(|left, right| left.name.cmp(&right.name));
    let start = params
        .as_ref()
        .and_then(|params| params.cursor.as_deref())
        .map(ToolCursor::decode)
        .transpose()?
        .map(|cursor| cursor.offset)
        .unwrap_or(0);
    if start > tools.len() {
        return Err(JsonRpcError::invalid_params(
            "tools/list cursor is outside the available tool pages",
        ));
    }
    let end = start
        .saturating_add(limits.max_tools_per_page)
        .min(tools.len());
    let next_cursor = (end < tools.len()).then(|| ToolCursor { offset: end }.encode());
    let tools = tools
        .into_iter()
        .skip(start)
        .take(end - start)
        .map(|tool| tool.into_protocol())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| JsonRpcError::internal(format!("invalid tool schema: {error}")))?;
    Ok(ListToolsResult { tools, next_cursor })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ToolCursor {
    offset: usize,
}

impl ToolCursor {
    fn decode(value: &str) -> Result<Self, JsonRpcError> {
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(JsonRpcError::invalid_params("tools/list cursor is invalid"));
        }
        let offset = value.parse::<usize>().map_err(|_| {
            JsonRpcError::invalid_params("tools/list cursor is outside the supported range")
        })?;
        Ok(Self { offset })
    }

    fn encode(self) -> String {
        self.offset.to_string()
    }
}

async fn call_tool(
    server: &Arc<dyn McpServer>,
    context: &RequestContext,
    request: CallToolRequest,
    limits: &TransportLimits,
) -> Result<JsonPayload, JsonRpcError> {
    if request.input_bytes > limits.max_tool_input_bytes {
        return Err(JsonRpcError::invalid_params(
            "tool input exceeds the configured size limit",
        ));
    }
    let result = server.call_tool(context, request.params).await;
    let result = match result {
        Ok(result) => result,
        Err(error @ ServerError::ToolNotFound(_))
        | Err(error @ ServerError::InvalidArguments(_)) => {
            return Err(server_error(error));
        }
        Err(error @ ServerError::Host(HostError::AuthorizationDenied))
        | Err(error @ ServerError::Host(HostError::ApprovalDenied)) => {
            return Err(JsonRpcError::unauthorized(error.to_string()));
        }
        Err(error) => error.error_result(),
    };
    let payload = JsonPayload::from_serializable(&result)
        .map_err(|error| JsonRpcError::internal(error.to_string()))?;
    if payload.as_str().len() > limits.max_tool_output_bytes {
        return Err(JsonRpcError::internal(
            "tool output exceeds the configured size limit",
        ));
    }
    Ok(payload)
}

fn server_error(error: ServerError) -> JsonRpcError {
    match error {
        ServerError::ToolNotFound(message) => JsonRpcError::invalid_params(message),
        ServerError::InvalidArguments(message) => JsonRpcError::invalid_params(message),
        ServerError::Host(message) => JsonRpcError::internal(message.to_string()),
        ServerError::Provider(message) => JsonRpcError::internal(message),
        ServerError::Protocol(message) => JsonRpcError::internal(message),
    }
}
