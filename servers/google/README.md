# MCP Google Server

## Goal

`mcp-google` defines the public, provider-facing boundary for Google MCP
tools. The current Phase 0 package validates tool contracts and credential
injection; it does not yet contain a live Google API client.

## Tool

### `google_get_identity`

Returns the identity for the configured Google connection.

Arguments:

```json
{}
```

The structured result is:

```json
{
  "display_name": "Example account"
}
```

The concrete provider implementation supplies the identity through the
`GoogleProvider` trait.

## Credentials and access

The host must resolve a Google credential through `CredentialResolver` using
provider name `google`. The credential is injected into the provider client
internally and never appears in tool arguments, schemas, prompts, logs, or
audit payloads.

The concrete OAuth/API credential type, scopes, connection ownership, and
refresh policy belong to the host/provider adapter and are not implemented in
this Phase 0 package. Never place a real token or client secret in this
README.

## Usage

Call the tool with business arguments only:

```json
{
  "jsonrpc": "2.0",
  "id": "google-example",
  "method": "tools/call",
  "params": {
    "name": "google_get_identity",
    "arguments": {}
  }
}
```

Authorization and approval are evaluated by the host before the provider
call. Provider failures are returned as typed server errors.

## Limitations and tests

Live Google API access, pagination, refresh, rate-limit handling, and concrete
scope validation are deferred to the provider implementation phase. The
contract test uses a mocked provider and credential resolver:

```bash
cargo test -p mcp-google
```
