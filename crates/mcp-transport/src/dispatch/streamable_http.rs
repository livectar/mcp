use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use mcp_protocol::schemas::json_payload::JsonPayload;
use mcp_protocol::{
    constants::{MAX_JSON_PAYLOAD_BYTES, PROTOCOL_REVISION},
    schemas::{
        json_rpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse},
        lifecycle::{InitializeParams, InitializeResult},
        tools::{CallToolParams, ListToolsParams, ListToolsResult},
    },
};
use mcp_sdk::{
    errors::ServerError,
    schemas::{
        caller::CallerContext,
        context::{HostServices, RequestContext},
    },
    traits::server::McpServer,
};
use std::sync::Arc;

#[derive(Clone)]
struct TransportState {
    server: Arc<dyn McpServer>,
    caller: CallerContext,
    services: HostServices,
}

#[derive(Clone)]
pub struct McpTransport {
    state: TransportState,
    max_request_bytes: usize,
}

impl McpTransport {
    pub fn new(server: Arc<dyn McpServer>, caller: CallerContext, services: HostServices) -> Self {
        Self {
            state: TransportState {
                server,
                caller,
                services,
            },
            max_request_bytes: MAX_JSON_PAYLOAD_BYTES,
        }
    }

    pub fn with_max_request_bytes(mut self, max_request_bytes: usize) -> Self {
        self.max_request_bytes = max_request_bytes;
        self
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/mcp", post(dispatch))
            .layer(axum::extract::DefaultBodyLimit::max(self.max_request_bytes))
            .with_state(self.state)
    }
}

async fn dispatch(
    State(state): State<TransportState>,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    if request.jsonrpc != "2.0" {
        return response(JsonRpcResponse::failure(
            request.id,
            JsonRpcError::invalid_request("jsonrpc must be 2.0"),
        ));
    }

    let context = RequestContext::new(
        request.id.clone(),
        state.caller.clone(),
        state.services.clone(),
    );
    let result = match request.method.as_str() {
        "initialize" => initialize(&state.server, request.params),
        "tools/list" => list_tools(&state.server, request.params),
        "tools/call" => call_tool(&state.server, &context, request.params).await,
        method => Err(JsonRpcError::method_not_found(method)),
    };

    let rpc_response = match result {
        Ok(result) => JsonRpcResponse::success(request.id, result),
        Err(error) => JsonRpcResponse::failure(request.id, error),
    };
    response(rpc_response)
}

fn initialize(
    server: &Arc<dyn McpServer>,
    params: Option<JsonPayload>,
) -> Result<JsonPayload, JsonRpcError> {
    let params =
        params.ok_or_else(|| JsonRpcError::invalid_params("initialize params are required"))?;
    let _: InitializeParams = params
        .decode()
        .map_err(|error| JsonRpcError::invalid_params(error.to_string()))?;
    JsonPayload::from_serializable(&InitializeResult {
        protocol_version: PROTOCOL_REVISION.to_string(),
        capabilities: server.capabilities(),
        server_info: server.info(),
    })
    .map_err(|error| JsonRpcError::internal(error.to_string()))
}

fn list_tools(
    server: &Arc<dyn McpServer>,
    params: Option<JsonPayload>,
) -> Result<JsonPayload, JsonRpcError> {
    if let Some(params) = params {
        let _: ListToolsParams = params
            .decode()
            .map_err(|error| JsonRpcError::invalid_params(error.to_string()))?;
    }
    JsonPayload::from_serializable(&ListToolsResult {
        tools: server.tools(),
        next_cursor: None,
    })
    .map_err(|error| JsonRpcError::internal(error.to_string()))
}

async fn call_tool(
    server: &Arc<dyn McpServer>,
    context: &RequestContext,
    params: Option<JsonPayload>,
) -> Result<JsonPayload, JsonRpcError> {
    let params =
        params.ok_or_else(|| JsonRpcError::invalid_params("tools/call params are required"))?;
    let params: CallToolParams = params
        .decode()
        .map_err(|error| JsonRpcError::invalid_params(error.to_string()))?;
    let result = server
        .call_tool(context, params)
        .await
        .map_err(server_error)?;
    JsonPayload::from_serializable(&result)
        .map_err(|error| JsonRpcError::internal(error.to_string()))
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

fn response(response: JsonRpcResponse) -> (StatusCode, Json<JsonRpcResponse>) {
    (StatusCode::OK, Json(response))
}
