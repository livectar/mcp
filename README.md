# Livectar MCP

Reusable, provider-independent Rust crates for Model Context Protocol servers.

The repository owns MCP wire contracts, server traits, Streamable HTTP
dispatch, test utilities, and provider implementations. It deliberately has
no dependency on AI Social, its database, or its workspace and assistant
models. Hosts inject authorization, credentials, approvals, and audit sinks
through the interfaces in `mcp-sdk`.

## Workspace layout

- `crates/mcp-protocol` — typed JSON-RPC and MCP lifecycle contracts.
- `crates/mcp-sdk` — server, tool, and host-service traits.
- `crates/mcp-transport` — reusable HTTP dispatch plumbing.
- `crates/mcp-testkit` — deterministic host-service test doubles.
- `servers/` — MCP server implementations. Each server has its own README with
  its purpose, tools, credentials, setup, usage, and limitations.

## Development

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo build --workspace
```

The protocol revision for this initial workspace is `2025-06-18`. Changes to
wire contracts or public traits require a compatibility note and a versioned
release.

All server implementations must follow the repository pagination and lossless
read contract in [`docs/pagination.md`](docs/pagination.md).
