use mcp_protocol::schemas::{
    json_rpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, RequestId},
    lifecycle::InitializeParams,
    tools::{CallToolParams, ListToolsParams},
};
use serde::{de::DeserializeOwned, Deserialize};

#[derive(Debug)]
pub(crate) enum IncomingMessage {
    Request(DispatchRequest),
    Notification(IncomingNotification),
    Response(JsonRpcResponse),
}

#[derive(Debug)]
pub(crate) enum IncomingNotification {
    Initialized {
        jsonrpc: String,
    },
    CallTool {
        jsonrpc: String,
        request: DispatchRequest,
    },
    Other {
        jsonrpc: String,
        method: String,
    },
    Ignored {
        jsonrpc: String,
        method: String,
    },
}

#[derive(Debug)]
pub(crate) enum DispatchRequest {
    Initialize {
        jsonrpc: String,
        id: RequestId,
        params: InitializeParams,
    },
    ListTools {
        jsonrpc: String,
        id: RequestId,
        params: Option<ListToolsParams>,
    },
    CallTool {
        jsonrpc: String,
        id: RequestId,
        request: CallToolRequest,
    },
    Unknown {
        jsonrpc: String,
        id: RequestId,
        method: String,
    },
}

#[derive(Debug)]
pub(crate) struct CallToolRequest {
    pub(crate) params: CallToolParams,
    pub(crate) input_bytes: usize,
}

impl IncomingMessage {
    pub(crate) fn method(&self) -> Option<&str> {
        match self {
            Self::Request(request) => Some(request.method()),
            Self::Notification(notification) => Some(notification.method()),
            Self::Response(_) => None,
        }
    }

    pub(crate) fn initialize_params(&self) -> Option<&InitializeParams> {
        match self {
            Self::Request(request) => request.initialize_params(),
            Self::Notification(_) | Self::Response(_) => None,
        }
    }
}

impl IncomingNotification {
    pub(crate) fn jsonrpc(&self) -> &str {
        match self {
            Self::Initialized { jsonrpc }
            | Self::CallTool { jsonrpc, .. }
            | Self::Other { jsonrpc, .. }
            | Self::Ignored { jsonrpc, .. } => jsonrpc,
        }
    }

    pub(crate) fn method(&self) -> &str {
        match self {
            Self::Initialized { .. } => "notifications/initialized",
            Self::CallTool { .. } => "tools/call",
            Self::Other { method, .. } | Self::Ignored { method, .. } => method,
        }
    }
}

impl DispatchRequest {
    pub(crate) fn id(&self) -> &RequestId {
        match self {
            Self::Initialize { id, .. }
            | Self::ListTools { id, .. }
            | Self::CallTool { id, .. }
            | Self::Unknown { id, .. } => id,
        }
    }

    pub(crate) fn jsonrpc(&self) -> &str {
        match self {
            Self::Initialize { jsonrpc, .. }
            | Self::ListTools { jsonrpc, .. }
            | Self::CallTool { jsonrpc, .. }
            | Self::Unknown { jsonrpc, .. } => jsonrpc,
        }
    }

    pub(crate) fn method(&self) -> &str {
        match self {
            Self::Initialize { .. } => "initialize",
            Self::ListTools { .. } => "tools/list",
            Self::CallTool { .. } => "tools/call",
            Self::Unknown { method, .. } => method,
        }
    }

    pub(crate) fn initialize_params(&self) -> Option<&InitializeParams> {
        match self {
            Self::Initialize { params, .. } => Some(params),
            Self::ListTools { .. } | Self::CallTool { .. } | Self::Unknown { .. } => None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct MessageHeader {
    jsonrpc: String,
    #[serde(default)]
    id: Option<RequestId>,
    #[serde(default)]
    method: Option<String>,
}

#[derive(Debug)]
pub(crate) enum MessageParseError {
    InvalidRequest(JsonRpcError),
    InvalidParams { id: RequestId, error: JsonRpcError },
}

pub(crate) fn parse_message(body: &[u8]) -> Result<IncomingMessage, MessageParseError> {
    let header = serde_json::from_slice::<MessageHeader>(body).map_err(|error| {
        MessageParseError::InvalidRequest(JsonRpcError::invalid_request(format!(
            "invalid JSON-RPC request: {error}"
        )))
    })?;

    match (header.method, header.id) {
        (Some(method), Some(id)) => parse_request(body, header.jsonrpc, id, method),
        (Some(method), None) => parse_notification(body, header.jsonrpc, method),
        (None, _) => serde_json::from_slice::<JsonRpcResponse>(body)
            .map(IncomingMessage::Response)
            .map_err(|error| {
                MessageParseError::InvalidRequest(JsonRpcError::invalid_request(format!(
                    "invalid JSON-RPC message: {error}"
                )))
            }),
    }
}

fn parse_request(
    body: &[u8],
    jsonrpc: String,
    id: RequestId,
    method: String,
) -> Result<IncomingMessage, MessageParseError> {
    match method.as_str() {
        "initialize" => {
            let request = decode_request::<InitializeParams>(body, &id)?;
            let params = request
                .params
                .ok_or_else(|| MessageParseError::InvalidParams {
                    id: id.clone(),
                    error: JsonRpcError::invalid_params("initialize params are required"),
                })?;
            Ok(IncomingMessage::Request(DispatchRequest::Initialize {
                jsonrpc,
                id,
                params,
            }))
        }
        "tools/list" => {
            let request = decode_request::<ListToolsParams>(body, &id)?;
            Ok(IncomingMessage::Request(DispatchRequest::ListTools {
                jsonrpc,
                id,
                params: request.params,
            }))
        }
        "tools/call" => {
            let request = decode_request::<CallToolParams>(body, &id)?;
            let params = request
                .params
                .ok_or_else(|| MessageParseError::InvalidParams {
                    id: id.clone(),
                    error: JsonRpcError::invalid_params("tools/call params are required"),
                })?;
            let request =
                call_tool_request(params).map_err(|error| MessageParseError::InvalidParams {
                    id: id.clone(),
                    error,
                })?;
            Ok(IncomingMessage::Request(DispatchRequest::CallTool {
                jsonrpc,
                id,
                request,
            }))
        }
        _ => Ok(IncomingMessage::Request(DispatchRequest::Unknown {
            jsonrpc,
            id,
            method,
        })),
    }
}

fn parse_notification(
    body: &[u8],
    jsonrpc: String,
    method: String,
) -> Result<IncomingMessage, MessageParseError> {
    match method.as_str() {
        "notifications/initialized" => Ok(IncomingMessage::Notification(
            IncomingNotification::Initialized { jsonrpc },
        )),
        "tools/call" => {
            let request = match serde_json::from_slice::<
                mcp_protocol::schemas::json_rpc::JsonRpcNotification<CallToolParams>,
            >(body)
            {
                Ok(request) => request,
                Err(_) => {
                    return Ok(IncomingMessage::Notification(
                        IncomingNotification::Ignored { jsonrpc, method },
                    ))
                }
            };
            let Some(params) = request.params else {
                return Ok(IncomingMessage::Notification(
                    IncomingNotification::Ignored { jsonrpc, method },
                ));
            };
            let request = match call_tool_request(params) {
                Ok(request) => request,
                Err(_) => {
                    return Ok(IncomingMessage::Notification(
                        IncomingNotification::Ignored { jsonrpc, method },
                    ))
                }
            };
            Ok(IncomingMessage::Notification(
                IncomingNotification::CallTool {
                    jsonrpc,
                    request: DispatchRequest::CallTool {
                        jsonrpc: "2.0".to_string(),
                        id: RequestId::Null,
                        request,
                    },
                },
            ))
        }
        _ => Ok(IncomingMessage::Notification(IncomingNotification::Other {
            jsonrpc,
            method,
        })),
    }
}

fn decode_request<Params: DeserializeOwned>(
    body: &[u8],
    id: &RequestId,
) -> Result<JsonRpcRequest<Params>, MessageParseError> {
    serde_json::from_slice(body).map_err(|error| MessageParseError::InvalidParams {
        id: id.clone(),
        error: JsonRpcError::invalid_params(error.to_string()),
    })
}

fn call_tool_request(params: CallToolParams) -> Result<CallToolRequest, JsonRpcError> {
    let input_bytes = serde_json::to_vec(&params)
        .map_err(|error| JsonRpcError::internal(format!("failed to size tool input: {error}")))?
        .len();
    Ok(CallToolRequest {
        params,
        input_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        parse_message, CallToolRequest, DispatchRequest, IncomingMessage, MessageParseError,
    };
    use mcp_protocol::schemas::{
        json_rpc::{JsonRpcRequest, RequestId},
        lifecycle::{ClientCapabilities, ImplementationInfo, InitializeParams},
        tools::{CallToolParams, ListToolsParams},
    };
    use serde::Serialize;

    fn request<Params: Serialize>(method: &str, params: Option<Params>) -> Vec<u8> {
        serde_json::to_vec(&JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(1),
            method: method.to_string(),
            params,
        })
        .unwrap()
    }

    fn initialize_params() -> InitializeParams {
        InitializeParams {
            protocol_version: "2025-06-18".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ImplementationInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
        }
    }

    #[test]
    fn parses_initialize_directly_into_a_typed_dispatch_request() {
        let message = parse_message(&request("initialize", Some(initialize_params()))).unwrap();

        assert!(matches!(
            message,
            IncomingMessage::Request(DispatchRequest::Initialize { .. })
        ));
    }

    #[test]
    fn missing_optional_list_params_are_typed_as_none() {
        let message = parse_message(&request::<ListToolsParams>("tools/list", None)).unwrap();

        assert!(matches!(
            message,
            IncomingMessage::Request(DispatchRequest::ListTools { params: None, .. })
        ));
    }

    #[test]
    fn malformed_or_missing_params_fail_at_the_boundary() {
        let missing =
            parse_message(br#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#).unwrap_err();
        assert!(matches!(missing, MessageParseError::InvalidParams { .. }));

        let malformed =
            parse_message(br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#)
                .unwrap_err();
        assert!(matches!(malformed, MessageParseError::InvalidParams { .. }));
    }

    #[test]
    fn call_request_keeps_dynamic_tool_arguments_and_input_size() {
        let message = parse_message(&request(
            "tools/call",
            Some(CallToolParams {
                name: "ping".to_string(),
                arguments: None,
            }),
        ))
        .unwrap();

        assert!(matches!(
            message,
            IncomingMessage::Request(DispatchRequest::CallTool {
                request: CallToolRequest { input_bytes, .. },
                ..
            }) if input_bytes > 0
        ));
    }

    #[test]
    fn list_params_are_decoded_into_the_protocol_type() {
        let message = parse_message(&request(
            "tools/list",
            Some(ListToolsParams {
                cursor: Some("3".to_string()),
            }),
        ))
        .unwrap();

        assert!(matches!(
            message,
            IncomingMessage::Request(DispatchRequest::ListTools {
                params: Some(ListToolsParams { cursor: Some(cursor) }),
                ..
            }) if cursor == "3"
        ));
    }
}
