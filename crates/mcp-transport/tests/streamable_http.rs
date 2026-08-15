use async_trait::async_trait;
use axum::{
    body::{to_bytes, Body},
    http::{header, Request, Response, StatusCode},
    Router,
};
use mcp_ping::{schemas::ping::PingResult, server::PingServer};
use mcp_protocol::constants::{MAX_JSON_PAYLOAD_BYTES, PROTOCOL_REVISION};
use mcp_protocol::schemas::{
    json_payload::JsonPayload,
    json_rpc::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId},
    lifecycle::{ClientCapabilities, ImplementationInfo, InitializeParams, InitializeResult},
    tools::{
        CallToolParams, CallToolResult, ContentBlock, ListToolsParams, ListToolsResult,
        ToolAnnotations, ToolDefinition,
    },
};
use mcp_sdk::{errors::ServerError, schemas::context::RequestContext, traits::server::McpServer};
use mcp_testkit::fixtures::host::TestHost;
use mcp_transport::{
    dispatch::streamable_http::{
        config::TransportConfig, resolver::McpServerResolver, transport::McpTransport,
    },
    schemas::server_key::ServerKey,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;
use tower::ServiceExt;

struct PingResolver {
    server: Arc<dyn McpServer>,
}

impl McpServerResolver for PingResolver {
    fn resolve(&self, server_key: &ServerKey) -> Option<Arc<dyn McpServer>> {
        (server_key.as_str() == "ping").then(|| Arc::clone(&self.server))
    }
}

fn transport_config(host: TestHost) -> TransportConfig {
    transport_config_with_server(host, Arc::new(PingServer::new()))
}

fn transport_config_with_server(host: TestHost, server: Arc<dyn McpServer>) -> TransportConfig {
    TransportConfig::new(
        Arc::new(PingResolver { server }),
        host.caller,
        host.services,
    )
}

fn transport() -> Router {
    McpTransport::new(transport_config(TestHost::new())).router()
}

fn headers(request: Request<Body>, method: &str) -> Request<Body> {
    let mut request = request;
    request
        .headers_mut()
        .insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    request.headers_mut().insert(
        header::ACCEPT,
        "application/json, text/event-stream".parse().unwrap(),
    );
    request
        .headers_mut()
        .insert("MCP-Protocol-Version", PROTOCOL_REVISION.parse().unwrap());
    request
        .headers_mut()
        .insert("Mcp-Method", method.parse().unwrap());
    request
}

async fn post(router: &Router, method: &str, body: Vec<u8>) -> Response<Body> {
    let request = headers(
        Request::post("/mcp/ping")
            .body(Body::from(body))
            .expect("request can be built"),
        method,
    );
    router
        .clone()
        .oneshot(request)
        .await
        .expect("router returns a response")
}

async fn body<T: serde::de::DeserializeOwned>(response: Response<Body>) -> T {
    let bytes = to_bytes(response.into_body(), MAX_JSON_PAYLOAD_BYTES)
        .await
        .expect("response body can be read");
    serde_json::from_slice(&bytes).expect("response body is valid JSON")
}

#[tokio::test]
async fn serves_initialize_notification_list_and_call_lifecycle() {
    let router = transport();
    let initialize_params = JsonPayload::from_serializable(&InitializeParams {
        protocol_version: PROTOCOL_REVISION.to_string(),
        capabilities: ClientCapabilities {
            tools: Some(Default::default()),
        },
        client_info: ImplementationInfo {
            name: "transport-test".to_string(),
            version: "0.1.0".to_string(),
        },
    })
    .expect("initialize params serialize");
    let initialize = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::Number(1),
        method: "initialize".to_string(),
        params: Some(initialize_params),
    };
    let response = post(
        &router,
        "initialize",
        serde_json::to_vec(&initialize).expect("initialize serializes"),
    )
    .await;
    let response: JsonRpcResponse = body(response).await;
    let result: InitializeResult = response
        .result
        .expect("initialize returns a result")
        .decode()
        .expect("initialize result decodes");
    assert_eq!(result.protocol_version, PROTOCOL_REVISION);

    let notification = JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "notifications/initialized".to_string(),
        params: None,
    };
    let notification_response = post(
        &router,
        "notifications/initialized",
        serde_json::to_vec(&notification).expect("notification serializes"),
    )
    .await;
    assert_eq!(notification_response.status(), StatusCode::ACCEPTED);

    let list = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::Number(2),
        method: "tools/list".to_string(),
        params: Some(
            JsonPayload::from_serializable(&ListToolsParams::default())
                .expect("list params serialize"),
        ),
    };
    let response = post(
        &router,
        "tools/list",
        serde_json::to_vec(&list).expect("list serializes"),
    )
    .await;
    let response: JsonRpcResponse = body(response).await;
    let result: ListToolsResult = response
        .result
        .expect("list returns a result")
        .decode()
        .expect("list result decodes");
    assert_eq!(result.tools.len(), 1);
    assert_eq!(result.tools[0].name, "ping");

    let call = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::Number(3),
        method: "tools/call".to_string(),
        params: Some(
            JsonPayload::from_serializable(&CallToolParams {
                name: "ping".to_string(),
                arguments: Some(
                    JsonPayload::parse(r#"{"message":"hello"}"#).expect("tool arguments are valid"),
                ),
            })
            .expect("call params serialize"),
        ),
    };
    let response = post(
        &router,
        "tools/call",
        serde_json::to_vec(&call).expect("call serializes"),
    )
    .await;
    let response: JsonRpcResponse = body(response).await;
    let result: CallToolResult = response
        .result
        .expect("call returns a result")
        .decode()
        .expect("call result decodes");
    let structured: PingResult = result
        .structured_content
        .expect("ping returns structured content")
        .decode()
        .expect("structured result decodes");
    assert_eq!(structured.message, "hello");
}

#[tokio::test]
async fn rejects_unknown_servers_and_stateless_session_methods() {
    let router = transport();
    let request = Request::get("/mcp").body(Body::empty()).unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let request = Request::get("/mcp/unknown").body(Body::empty()).unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let request = Request::get("/mcp/ping")
        .header(header::ACCEPT, "text/event-stream")
        .body(Body::empty())
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);

    let request = Request::delete("/mcp/ping").body(Body::empty()).unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn validates_method_and_origin_headers() {
    let host = TestHost::new();
    let mut config = transport_config(host.clone());
    config.allowed_origins = vec!["https://client.example".to_string()];
    let router = McpTransport::new(config).router();
    let request = headers(
        Request::post("/mcp/ping")
            .header("Origin", "https://evil.example")
            .body(Body::from(b"{}".to_vec()))
            .unwrap(),
        "initialize",
    );
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let request = headers(
        Request::post("/mcp/ping")
            .body(Body::from(b"{}".to_vec()))
            .unwrap(),
        "tools/list",
    );
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

struct PagedServer {
    calls: Arc<AtomicUsize>,
}

impl PagedServer {
    fn new() -> (Self, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        (
            Self {
                calls: Arc::clone(&calls),
            },
            calls,
        )
    }
}

#[async_trait]
impl McpServer for PagedServer {
    fn info(&self) -> ImplementationInfo {
        ImplementationInfo {
            name: "paged-test-server".to_string(),
            version: "0.1.0".to_string(),
        }
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        ["zeta", "alpha", "direct", "middle"]
            .into_iter()
            .map(|name| ToolDefinition {
                name: name.to_string(),
                description: "transport test tool".to_string(),
                input_schema: JsonPayload::parse(r#"{"type":"object"}"#)
                    .expect("fixture schema is valid"),
                annotations: ToolAnnotations::default(),
            })
            .collect()
    }

    async fn call_tool(
        &self,
        _context: &RequestContext,
        request: CallToolParams,
    ) -> Result<CallToolResult, ServerError> {
        if request.name != "direct" {
            return Err(ServerError::ToolNotFound(request.name));
        }
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(CallToolResult {
            content: vec![ContentBlock::Text {
                text: "called in process".to_string(),
            }],
            structured_content: None,
            is_error: false,
        })
    }
}

fn list_request(id: i64, cursor: Option<&str>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::Number(id),
        method: "tools/list".to_string(),
        params: Some(
            JsonPayload::from_serializable(&ListToolsParams {
                cursor: cursor.map(ToOwned::to_owned),
            })
            .expect("list params serialize"),
        ),
    }
}

fn call_request(name: &str) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::Number(1),
        method: "tools/call".to_string(),
        params: Some(
            JsonPayload::from_serializable(&CallToolParams {
                name: name.to_string(),
                arguments: None,
            })
            .expect("call params serialize"),
        ),
    }
}

#[tokio::test]
async fn sorts_tool_discovery_and_paginates_with_an_opaque_cursor() {
    let (server, _) = PagedServer::new();
    let mut config = transport_config_with_server(TestHost::new(), Arc::new(server));
    config.limits.max_tools_per_page = 2;
    let router = McpTransport::new(config).router();

    let response = post(
        &router,
        "tools/list",
        serde_json::to_vec(&list_request(1, None)).expect("list serializes"),
    )
    .await;
    let response: JsonRpcResponse = body(response).await;
    let first_page: ListToolsResult = response
        .result
        .expect("first list result exists")
        .decode()
        .expect("first list result decodes");
    assert_eq!(
        first_page
            .tools
            .iter()
            .map(|tool| tool.name.as_str())
            .collect::<Vec<_>>(),
        vec!["alpha", "direct"]
    );
    assert_eq!(first_page.next_cursor.as_deref(), Some("2"));

    let response = post(
        &router,
        "tools/list",
        serde_json::to_vec(&list_request(2, first_page.next_cursor.as_deref()))
            .expect("next list serializes"),
    )
    .await;
    let response: JsonRpcResponse = body(response).await;
    let second_page: ListToolsResult = response
        .result
        .expect("second list result exists")
        .decode()
        .expect("second list result decodes");
    assert_eq!(
        second_page
            .tools
            .iter()
            .map(|tool| tool.name.as_str())
            .collect::<Vec<_>>(),
        vec!["middle", "zeta"]
    );
    assert!(second_page.next_cursor.is_none());

    let response = post(
        &router,
        "tools/list",
        serde_json::to_vec(&list_request(3, Some("invalid-cursor")))
            .expect("invalid list serializes"),
    )
    .await;
    let response: JsonRpcResponse = body(response).await;
    assert_eq!(
        response.error.expect("invalid cursor error exists").code,
        -32602
    );
}

#[tokio::test]
async fn calls_the_registered_server_without_an_internal_http_hop() {
    let (server, calls) = PagedServer::new();
    let router = McpTransport::new(transport_config_with_server(
        TestHost::new(),
        Arc::new(server),
    ))
    .router();
    let call = call_request("direct");

    let response = post(
        &router,
        "tools/call",
        serde_json::to_vec(&call).expect("call serializes"),
    )
    .await;
    let response: JsonRpcResponse = body(response).await;
    let result: CallToolResult = response
        .result
        .expect("direct call result exists")
        .decode()
        .expect("direct call result decodes");
    assert!(!result.is_error);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[derive(Clone, Copy)]
enum FailureScenario {
    Provider,
    Timeout,
    LargeResponse,
    InvalidArguments,
}

struct FailureServer {
    scenario: FailureScenario,
}

#[async_trait]
impl McpServer for FailureServer {
    fn info(&self) -> ImplementationInfo {
        ImplementationInfo {
            name: "failure-test-server".to_string(),
            version: "0.1.0".to_string(),
        }
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![ToolDefinition {
            name: "scenario".to_string(),
            description: "transport failure fixture".to_string(),
            input_schema: JsonPayload::parse(r#"{"type":"object"}"#)
                .expect("fixture schema is valid"),
            annotations: ToolAnnotations::default(),
        }]
    }

    async fn call_tool(
        &self,
        _context: &RequestContext,
        _request: CallToolParams,
    ) -> Result<CallToolResult, ServerError> {
        match self.scenario {
            FailureScenario::Provider => Err(ServerError::Provider(
                "simulated provider failure".to_string(),
            )),
            FailureScenario::Timeout => {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok(CallToolResult {
                    content: vec![ContentBlock::Text {
                        text: "unreachable".to_string(),
                    }],
                    structured_content: None,
                    is_error: false,
                })
            }
            FailureScenario::LargeResponse => Ok(CallToolResult {
                content: vec![ContentBlock::Text {
                    text: "x".repeat(512),
                }],
                structured_content: None,
                is_error: false,
            }),
            FailureScenario::InvalidArguments => Err(ServerError::InvalidArguments(
                "simulated invalid arguments".to_string(),
            )),
        }
    }
}

fn failure_transport(scenario: FailureScenario) -> TransportConfig {
    transport_config_with_server(TestHost::new(), Arc::new(FailureServer { scenario }))
}

#[tokio::test]
async fn preserves_typed_failures_at_the_mcp_boundary() {
    let router = McpTransport::new(failure_transport(FailureScenario::Provider)).router();
    let response = post(
        &router,
        "tools/call",
        serde_json::to_vec(&call_request("scenario")).expect("provider call serializes"),
    )
    .await;
    let response: JsonRpcResponse = body(response).await;
    let result: CallToolResult = response
        .result
        .expect("provider failures are returned as tool results")
        .decode()
        .expect("provider failure result decodes");
    assert!(result.is_error);

    let router = McpTransport::new(failure_transport(FailureScenario::InvalidArguments)).router();
    let response = post(
        &router,
        "tools/call",
        serde_json::to_vec(&call_request("scenario")).expect("invalid call serializes"),
    )
    .await;
    let response: JsonRpcResponse = body(response).await;
    assert_eq!(
        response.error.expect("invalid argument error exists").code,
        -32602
    );
}

#[tokio::test]
async fn bounds_tool_output_and_request_execution_time() {
    let mut config = failure_transport(FailureScenario::LargeResponse);
    config.limits.max_tool_output_bytes = 64;
    let router = McpTransport::new(config).router();
    let response = post(
        &router,
        "tools/call",
        serde_json::to_vec(&call_request("scenario")).expect("large call serializes"),
    )
    .await;
    let response: JsonRpcResponse = body(response).await;
    assert_eq!(
        response.error.expect("large response error exists").code,
        -32603
    );

    let mut config = failure_transport(FailureScenario::Timeout);
    config.limits.request_timeout = Duration::from_millis(10);
    let router = McpTransport::new(config).router();
    let response = post(
        &router,
        "tools/call",
        serde_json::to_vec(&call_request("scenario")).expect("timeout call serializes"),
    )
    .await;
    let response: JsonRpcResponse = body(response).await;
    assert_eq!(response.error.expect("timeout error exists").code, -32603);
}
