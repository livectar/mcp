# MCP Ping Server

## Goal

`mcp-ping` is the provider-independent reference server for validating the
public MCP protocol, SDK, transport, authorization hooks, and testkit. It is
not an external-provider integration.

## Tool

### `ping`

Returns a deterministic response.

Arguments are optional:

```json
{
  "message": "hello"
}
```

The structured result is:

```json
{
  "message": "hello",
  "protocol_version": "2025-06-18"
}
```

If `message` is omitted, the server returns `pong`.

## Credentials and access

No provider credential is required. The host still supplies authorization and
approval services through `mcp-sdk`; the server does not store credentials or
select connections.

## Usage

Call the tool through MCP with business arguments only:

```json
{
  "jsonrpc": "2.0",
  "id": "ping-example",
  "method": "tools/call",
  "params": {
    "name": "ping",
    "arguments": { "message": "hello" }
  }
}
```

## Limitations and tests

This server has no external side effects, provider rate limits, pagination,
or freshness concerns. Run its tests from the public workspace:

```bash
cargo test -p mcp-ping
```
