use std::sync::Arc;

use mcp_google::{providers::common::GoogleApiClient, server::GoogleServer};
use mcp_sdk::traits::server::McpServer;

#[test]
fn registers_read_and_mutation_tools() {
    let server = GoogleServer::new(Arc::new(GoogleApiClient::default())).unwrap();
    let mut names = server
        .tools()
        .into_iter()
        .map(|tool| tool.name)
        .collect::<Vec<_>>();
    names.sort();
    assert_eq!(
        names,
        vec![
            "sheets_append_rows",
            "sheets_clear_range",
            "sheets_create_spreadsheet",
            "sheets_get_spreadsheet",
            "sheets_list_spreadsheets",
            "sheets_read_cell_text",
            "sheets_read_range",
            "sheets_read_sheet_metadata",
            "sheets_write_range",
        ]
    );
}

#[test]
fn mutation_schemas_describe_nested_typed_cells() {
    let server = GoogleServer::new(Arc::new(GoogleApiClient::default())).unwrap();
    let tool = server
        .tools()
        .into_iter()
        .find(|tool| tool.name == "sheets_write_range")
        .unwrap();
    assert_eq!(tool.annotations.read_only_hint, Some(false));
    assert!(tool.description.contains("kind:text"));
    assert!(tool.description.contains("two-dimensional matrices"));
    let schema = tool.input_schema.to_json_payload().unwrap();
    assert!(schema.as_str().contains("\"type\":\"array\""));
    assert!(schema.as_str().contains("\"oneOf\""));
}
