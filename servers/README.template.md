# MCP <Provider> Server

`mcp-<provider>` is the provider-facing MCP implementation for <scope>. It
does not depend on AI Social, its database, or private application services.

## Tools

- `<tool_name>` <does what it does and identifies whether it is read-only or a
  mutation>.
- `<tool_name>` <does what it does>.

Results are typed and describe <identity, pagination, formatting, or other
important result behavior>. <State what is bounded, lossless, or intentionally
unsupported.>

## Credentials and scopes

The host resolves a credential through `CredentialResolver` using provider
name `<provider-key>`. The credential is injected into the provider client and
never appears in tool arguments, schemas, results, prompts, logs, or audit
payloads.

Required provider scopes, permissions, or connection requirements:

```text
<scope-or-permission>
```

The host owns authorization, approval, connection ownership, refresh, and
secret storage. Never place real credentials or session material in this
README.

## Usage

Call tools through the MCP transport with business arguments only; do not pass
credentials or connection secrets in tool arguments.

Authorization and approval are evaluated by the host before provider calls.
Document which tools require approval and how provider failures are mapped to
MCP errors. Document timeout, rate-limit, pagination, freshness, and access
limitations when they apply.

## Tests

Unit, provider-mocked, and HTTP contract tests run without live provider
credentials:

```bash
cargo test -p mcp-<provider>
```
