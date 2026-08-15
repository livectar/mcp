# Compatibility policy

The initial MCP protocol revision is `2025-06-18`.

Changes to wire-level types, public traits, or required host services are
breaking changes and require a new minor release note. Additive tool fields
must remain optional when possible. Provider-specific behavior belongs in the
provider package and must not leak into the generic protocol contracts.

## Phase 1 transport additions

The Streamable HTTP transport now accepts one typed JSON-RPC request,
notification, or response per POST body, supports the `/mcp/{server_key}`
route, and exposes configurable body, tool, response, timeout, concurrency,
and Origin limits. `RequestId::Null`, `JsonRpcMessage`, the unauthorized error
code, and the default-disabled server-sent-events capability are additive
contracts; existing request and tool schemas remain compatible.

The transport construction API is intentionally breaking in this refactor:
`McpTransport` now accepts one `TransportConfig`, and the legacy `/mcp` route
and per-setting transport builders are removed. Callers must configure and
use `/mcp/{server_key}` explicitly.
