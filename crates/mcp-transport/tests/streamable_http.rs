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
    tools::{CallToolParams, CallToolResult, ListToolsParams, ListToolsResult},
};
use mcp_sdk::traits::server::McpServer;
use mcp_testkit::fixtures::host::TestHost;
use mcp_transport::{
    dispatch::streamable_http::{
        config::TransportConfig, resolver::McpServerResolver, transport::McpTransport,
    },
    schemas::server_key::ServerKey,
};
use std::sync::Arc;
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
    TransportConfig::new(
        Arc::new(PingResolver {
            server: Arc::new(PingServer::new()),
        }),
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
