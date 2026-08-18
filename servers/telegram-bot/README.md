# MCP Telegram Bot Server

`mcp-telegram-bot` is the provider-facing Telegram Bot API implementation for
MCP. It uses `teloxide` and has no dependency on AI Social, its database, or
private application services.

## Tools

- `telegram_get_me` returns the configured bot identity and Bot API capability
  flags. This is read-only.
- `telegram_get_chat` returns metadata for a chat accessible to the bot. This
  is read-only.
- `telegram_send_message` sends a text message to an accessible chat. This is
  a mutation and requires host approval.

Message results include the Telegram message ID, chat ID, Unix timestamp, and
text when Telegram returns it. Message text is limited to Telegram’s 4,096
character limit.

## Credentials and permissions

The host resolves a credential through `CredentialResolver` using provider name
`telegram-bot`. The encrypted BotFather token is injected into the teloxide
provider and never appears in tool arguments, schemas, results, prompts, logs,
or audit payloads.

Telegram enforces the bot’s membership, privacy mode, chat permissions, and
access to each requested chat. The host owns authorization, approval,
connection ownership, and secret storage. Never place a real bot token in this
README.

## Usage

Call `telegram_send_message` through the MCP transport with `chat_id`, `text`,
and optional `parse_mode` business arguments. `parse_mode` accepts
`markdown_v2` or `html`. If omitted, Telegram receives plain text. MarkdownV2
escaping and HTML supported-tag rules remain the caller’s responsibility.

The host evaluates authorization before every provider call and approval before
`telegram_send_message`. Telegram API failures are mapped to MCP provider
errors. This package currently provides synchronous Bot API tools only; update
polling, webhooks, message history, media, listener persistence, notification
routing, and Telegram user-client authentication are out of scope.

## Tests

Unit, provider-mocked, and mocked HTTP contract tests run without live Telegram
credentials:

```bash
cargo test -p mcp-telegram-bot
```
