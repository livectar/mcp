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

## AI Social plugin JSON

Import this JSON into the AI Social plugin catalog, publish it, and install
it from workspace settings:

```json
{
  "schema_version": 1,
  "apps": [
    {
      "app_key": "ai-social-ping",
      "name": "AI Social Ping",
      "description": "Local MCP ping server for development checks.",
      "icon_url": null,
      "category": "development",
      "transport": "streamable_http",
      "server_url": "http://127.0.0.1:4200/mcp/ping",
      "auth_type": "none",
      "oauth": null,
      "config": {}
    }
  ]
}
```

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
