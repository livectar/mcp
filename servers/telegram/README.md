# MCP Telegram Server

## Goal

`mcp-telegram` defines the public, provider-facing boundary for Telegram MCP
tools. The current Phase 0 package validates tool contracts and credential
injection; it does not yet contain a live Telegram Bot API or user-client
implementation.

## Tool

### `telegram_get_identity`

Returns the identity for the configured Telegram connection.

Arguments:

```json
{}
```

The structured result is:

```json
{
  "username": "example_account"
}
```

The concrete provider implementation supplies the identity through the
`TelegramProvider` trait.

## Credentials and access

The host must resolve a Telegram credential through `CredentialResolver` using
provider name `telegram`. The credential is injected into the provider client
internally and never appears in tool arguments, schemas, prompts, logs, or
audit payloads.

The concrete bot token or user-client session, authentication flow, required
permissions, connection ownership, and refresh policy belong to the
host/provider adapter and are not implemented in this Phase 0 package. Never
place a real token, phone code, 2FA password, or session data in this README.

## Usage

Call the tool with business arguments only:

```json
{
  "jsonrpc": "2.0",
  "id": "telegram-example",
  "method": "tools/call",
  "params": {
    "name": "telegram_get_identity",
    "arguments": {}
  }
}
```

Authorization and approval are evaluated by the host before the provider
call. Provider failures are returned as typed server errors.

## Limitations and tests

Live Telegram access, Bot API/user-client authentication, update handling,
rate limits, and concrete permission validation are deferred to the provider
implementation phase. The contract test uses a mocked provider and
credential resolver:

```bash
cargo test -p mcp-telegram
```
