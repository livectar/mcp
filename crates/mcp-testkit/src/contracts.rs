use mcp_sdk::traits::server::McpServer;

/// Assert the metadata invariants required of every compiled MCP server.
///
/// The helper intentionally does not invoke tools: a public server may expose
/// side-effecting operations, and execution is covered by that server's own
/// typed contract tests.
pub fn assert_server_contract(server: &dyn McpServer) {
    let info = server.info();
    assert!(
        !info.name.trim().is_empty(),
        "server name must not be empty"
    );
    assert!(
        !info.version.trim().is_empty(),
        "server version must not be empty"
    );
    assert!(
        server.capabilities().tools.is_some(),
        "servers in the MCP testkit must advertise tools"
    );

    let mut tools = server.tools();
    tools.sort_by(|left, right| left.name.cmp(&right.name));
    assert!(!tools.is_empty(), "server must expose at least one tool");
    for pair in tools.windows(2) {
        assert_ne!(pair[0].name, pair[1].name, "tool names must be unique");
    }
    for tool in tools {
        assert!(!tool.name.trim().is_empty(), "tool name must not be empty");
        assert!(
            !tool.description.trim().is_empty(),
            "tool description must not be empty"
        );
    }
}
