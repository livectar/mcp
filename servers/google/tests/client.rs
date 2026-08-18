use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    routing::any,
    Router,
};
use mcp_google::{
    errors::GoogleErrorCategory,
    providers::common::{GoogleApiClient, GoogleClientConfig, GoogleProvider},
    schemas::{
        cells::{CellTextKind, CellValue, ValueRenderMode},
        identifiers::{
            ids::SpreadsheetId,
            limits::{CellLimit, PageSize, TextChunkSize},
            ranges::A1Range,
        },
        requests::{ListSpreadsheetsRequest, ReadCellTextRequest, ReadRangeRequest},
    },
};
use mcp_sdk::schemas::{credentials::ProviderCredential, pagination::OpaqueCursor};
use std::time::Duration;
use tokio::task::JoinHandle;

struct FixtureServer {
    base_url: String,
    task: JoinHandle<()>,
}

impl FixtureServer {
    async fn start(status: StatusCode, body: &'static str) -> Self {
        let app = Router::new().fallback(any(move |request: Request<Body>| async move {
            let response_body = if status.is_success() {
                match request.uri().path() {
                    path if path.ends_with("/files") => DRIVE_BODY,
                    path if path.contains("A1:B1") => VALUES_PAGE_ONE_BODY,
                    path if path.contains("A2:B2") => VALUES_PAGE_TWO_BODY,
                    path if path.contains("A3") => CELL_TEXT_BODY,
                    _ => METADATA_BODY,
                }
            } else {
                body
            };
            Response::builder()
                .status(status)
                .header("content-type", "application/json")
                .body(Body::from(response_body))
                .expect("fixture response builds")
        }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("fixture listener binds");
        let address = listener.local_addr().expect("fixture address is available");
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("fixture serves");
        });
        Self {
            base_url: format!("http://{address}/"),
            task,
        }
    }

    fn config(&self) -> GoogleClientConfig {
        GoogleClientConfig {
            sheets_api_base_url: format!("{}sheets/v4/", self.base_url),
            drive_api_base_url: format!("{}drive/v3/", self.base_url),
            request_timeout: Duration::from_secs(2),
            max_response_bytes: 1024 * 1024,
            default_page_size: PageSize::new(25).unwrap(),
            max_page_size: PageSize::new(100).unwrap(),
            max_cells: CellLimit::new(500).unwrap(),
            max_retries: 0,
            retry_delay: Duration::from_millis(1),
        }
    }
}

impl Drop for FixtureServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

const METADATA_BODY: &str = r#"{"spreadsheetId":"spreadsheet","properties":{"title":"Demo"},"sheets":[{"properties":{"sheetId":0,"title":"Sheet 1","index":0,"sheetType":"GRID","gridProperties":{"rowCount":10,"columnCount":5,"frozenRowCount":1,"frozenColumnCount":2}}}]}"#;
const VALUES_PAGE_ONE_BODY: &str =
    r#"{"range":"Sheet 1!A1:B1","majorDimension":"ROWS","values":[["hello",4]]}"#;
const VALUES_PAGE_TWO_BODY: &str =
    r#"{"range":"Sheet 1!A2:B2","majorDimension":"ROWS","values":[[true,"=SUM(A1:B1)"]]}"#;
const CELL_TEXT_BODY: &str = concat!(
    r#"{"range":"Sheet 1!A3","majorDimension":"ROWS","values":[[""#,
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "\"]]}"
);
const DRIVE_BODY: &str = r#"{"nextPageToken":"opaque-next","files":[{"id":"spreadsheet","name":"Demo","webViewLink":"https://docs.google.com/spreadsheets/d/spreadsheet"}]}"#;

#[tokio::test]
async fn maps_drive_pagination_and_lossless_range_cursor() {
    let server = FixtureServer::start(StatusCode::OK, DRIVE_BODY).await;
    let client = GoogleApiClient::new(server.config()).unwrap();
    let credential = ProviderCredential::new("test-token").unwrap();

    let listed = client
        .list_spreadsheets(&credential, ListSpreadsheetsRequest::default())
        .await;
    assert!(listed.is_ok());
    let listed = listed.unwrap();
    assert_eq!(
        listed.spreadsheets[0].spreadsheet_id.as_str(),
        "spreadsheet"
    );
    assert_eq!(
        listed.next_cursor,
        Some(OpaqueCursor::new("opaque-next").unwrap())
    );
}

#[tokio::test]
async fn reads_formula_boolean_number_without_data_loss() {
    let server = FixtureServer::start(StatusCode::OK, VALUES_PAGE_ONE_BODY).await;
    let client = GoogleApiClient::new(server.config()).unwrap();
    let credential = ProviderCredential::new("test-token").unwrap();
    let request = ReadRangeRequest {
        spreadsheet_id: SpreadsheetId::new("spreadsheet").unwrap(),
        range: A1Range::new("'Sheet 1'!A1:B2").unwrap(),
        value_rendering: ValueRenderMode::Formula,
        max_cells: Some(CellLimit::new(3).unwrap()),
        continuation_cursor: None,
    };

    let first = client
        .read_range(&credential, request.clone())
        .await
        .unwrap();
    assert_eq!(first.identity.tab.title, "Sheet 1");
    assert_eq!(first.identity.requested_range.as_str(), "'Sheet 1'!A1:B2");
    assert_eq!(first.page_range.as_str(), "'Sheet 1'!A1:B1");
    assert_eq!(first.returned_cell_count, 2);
    assert!(first.next_cursor.is_some());
    assert_eq!(first.values[0][0], CellValue::Text("hello".to_string()));
    assert_eq!(first.values[0][1], CellValue::Number(4.0));

    let second = client
        .read_range(
            &credential,
            ReadRangeRequest {
                continuation_cursor: first.next_cursor,
                ..request
            },
        )
        .await
        .unwrap();
    assert_eq!(second.returned_cell_count, 2);
    assert_eq!(second.page_range.as_str(), "'Sheet 1'!A2:B2");
    assert!(second.next_cursor.is_none());
    assert_eq!(second.values[0][0], CellValue::Boolean(true));
    assert_eq!(
        second.values[0][1],
        CellValue::Formula("=SUM(A1:B1)".to_string())
    );
}

#[tokio::test]
async fn reads_large_cell_text_with_lossless_chunks() {
    let server = FixtureServer::start(StatusCode::OK, CELL_TEXT_BODY).await;
    let client = GoogleApiClient::new(server.config()).unwrap();
    let credential = ProviderCredential::new("test-token").unwrap();
    let request = ReadCellTextRequest {
        spreadsheet_id: SpreadsheetId::new("spreadsheet").unwrap(),
        cell: A1Range::new("'Sheet 1'!A3").unwrap(),
        value_rendering: ValueRenderMode::Formatted,
        chunk_bytes: Some(TextChunkSize::new(256).unwrap()),
        continuation_cursor: None,
    };

    let first = client
        .read_cell_text(&credential, request.clone())
        .await
        .unwrap();
    assert_eq!(first.kind, CellTextKind::Text);
    assert_eq!(first.text.len(), 256);
    assert!(first.next_cursor.is_some());

    let second = client
        .read_cell_text(
            &credential,
            ReadCellTextRequest {
                continuation_cursor: first.next_cursor,
                ..request.clone()
            },
        )
        .await
        .unwrap();
    assert_eq!(second.text.len(), 256);
    assert!(second.next_cursor.is_some());

    let third = client
        .read_cell_text(
            &credential,
            ReadCellTextRequest {
                continuation_cursor: second.next_cursor,
                ..request.clone()
            },
        )
        .await
        .unwrap();
    assert_eq!(third.text.len(), 128);
    assert!(third.next_cursor.is_none());
}

#[tokio::test]
async fn maps_missing_scope_without_leaking_a_token() {
    let server = FixtureServer::start(
        StatusCode::FORBIDDEN,
        r#"{"error":{"code":403,"message":"Request had insufficient authentication scopes for secret-token.","status":"PERMISSION_DENIED","errors":[{"reason":"insufficientPermissions","message":"Request had insufficient authentication scopes."}]}}"#,
    )
    .await;
    let client = GoogleApiClient::new(server.config()).unwrap();
    let error = client
        .list_spreadsheets(
            &ProviderCredential::new("secret-token").unwrap(),
            ListSpreadsheetsRequest::default(),
        )
        .await
        .unwrap_err();
    let text = error.to_string();
    assert!(matches!(
        error,
        mcp_google::errors::GoogleProviderError::Api {
            category: GoogleErrorCategory::MissingScope,
            ..
        }
    ));
    assert!(text.contains("missing_scope"));
    assert!(text.contains("reauthorize_google_connection"));
    assert!(!text.contains("secret-token"));
}

#[test]
fn rejects_invalid_spreadsheet_and_a1_identifiers() {
    assert!(SpreadsheetId::new("spreadsheet/id").is_err());
    assert!(A1Range::new("Sheet 1!A1;DROP").is_err());
    assert!(A1Range::new("'Unclosed!A1").is_err());
}
