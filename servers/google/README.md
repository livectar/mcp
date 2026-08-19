# MCP Google Server

`mcp-google` is the provider-facing Google MCP implementation for the Sheets
MVP. It exposes bounded Google Sheets reads and mutations through MCP, and uses
the Google Drive API only for spreadsheet discovery. It does not expose Google
REST endpoints to callers. Approval remains the responsibility of the main
application action policy.

## Tools

- `sheets_list_spreadsheets` lists accessible spreadsheets with bounded name
  and Drive-query filters and an opaque page cursor.
- `sheets_get_spreadsheet` returns spreadsheet metadata and tab identities
  without loading grid data.
- `sheets_read_range` reads a validated A1 range using formatted,
  unformatted, or formula rendering with lossless row/column cursor
  pagination.
- `sheets_read_cell_text` reads an unusually large text or formula cell in
  UTF-8-safe chunks with a continuation cursor.
- `sheets_read_sheet_metadata` returns tab IDs, titles, grid dimensions, and
  frozen-pane metadata.
- `sheets_create_spreadsheet` creates a spreadsheet with an optional initial
  tab configuration.
- `sheets_write_range` replaces an exact A1 range with a bounded rectangular
  matrix of typed cells.
- `sheets_append_rows` appends a bounded typed row matrix. It requires host
  authorization and does not automatically retry because an uncertain append
  may have been applied by Google.
- `sheets_clear_range` clears an exact A1 range.

Read results include spreadsheet identity. Range results additionally include
the resolved tab identity, the exact requested range, and `next_cursor` when
more rows or columns remain. Cell output is typed as empty, text, number,
boolean, or formula. No cell text is truncated; oversized text uses the
separate chunk tool. Mutation results include the spreadsheet ID, tab identity,
applied range when available, outcome, affected cell count, failed cell count,
and a bounded typed summary.

## Credentials and scopes

The host resolves a Google credential through `CredentialResolver` using
provider name `google`. The credential is injected into the provider client
and never appears in tool arguments, schemas, results, prompts, logs, or audit
payloads.

The read profile declares these scopes:

```text
https://www.googleapis.com/auth/spreadsheets.readonly
https://www.googleapis.com/auth/drive.readonly
```

Mutation tools use the separate
`https://www.googleapis.com/auth/spreadsheets` profile. `drive.file` is
intentionally not presented as an account-wide discovery scope.

The host owns OAuth authorization, refresh, connection ownership, and secret
storage. Never place a real token or client secret in this README.

When registering the built-in Google server as an OAuth2 MCP app, use the
following typed catalog metadata. This metadata describes the OAuth provider;
it is not a credential. Never put a client ID, client secret, access token, or
refresh token in the catalog metadata.

```json
{
  "config_key": "google-workspace",
  "issuer": "https://accounts.google.com",
  "authorization_endpoint": "https://accounts.google.com/o/oauth2/v2/auth",
  "token_endpoint": "https://oauth2.googleapis.com/token",
  "scopes": [
    "https://www.googleapis.com/auth/spreadsheets.readonly",
    "https://www.googleapis.com/auth/spreadsheets",
    "https://www.googleapis.com/auth/drive.readonly"
  ],
  "pkce_method": "S256",
  "pkce_required": true,
  "max_token_bytes": 16384,
  "max_metadata_bytes": 32768
}
```

The admin OAuth setup button stores the client credentials separately.

## Usage

Call tools through the MCP transport with business arguments only. For
example, a range read supplies `spreadsheet_id`, `range`, optional
`value_rendering`, optional `max_cells`, and then the returned cursor on each
continuation call; it does not supply a credential. The read is complete only
when `next_cursor` is absent. Mutation calls also use business arguments only;
callers do not supply an approval token or request reference. Write values use
the typed cell shape
`{"kind":"text","value":"hello"}` (or `empty`, `number`, `boolean`, and
`formula` variants).
The main application evaluates authorization and its existing workspace
approval policy before dispatching the MCP call. Auto-approved tools execute
immediately; confirmation-required tools follow that existing application flow.

Provider responses are bounded by request timeout, response bytes, page size,
cell count, mutation request bytes, and per-cell safety checks. Google `401`,
missing-scope `403`, ordinary permission `403`, `404`, `409`, `429`, and `5xx`
responses become safe typed categories with reauthorization, permission,
identifier, or retry guidance. Non-idempotent create and append requests do not
automatically retry; transport, oversized, and malformed success responses are
returned as an uncertain mutation. No response bound may silently discard data.

## Tests

The unit and mocked HTTP tests run without live Google credentials:

```bash
cargo test -p mcp-google
```
