# MCP pagination and lossless reads

This is a cross-server contract for every MCP server in this repository. A
server may limit a response for transport, memory, provider, or model-context
safety, but it must never silently discard user data.

## Completion rule

Every paginated result uses an optional `next_cursor` field:

- `next_cursor` present: the result is incomplete; call the same tool again
  with the cursor in the request's continuation-cursor field.
- `next_cursor` absent: the result is complete for the requested operation.

Clients and agents must not infer completeness from the number of returned
items, a page-size limit, a response summary, or prose claims of completeness.

## Cursor requirements

Cursors must be opaque, bounded, typed values. A cursor must be statelessly
decodable by the server and must contain enough scope to reject accidental
reuse, including:

- the original logical request and resource identity;
- the page or row/column offset;
- rendering, filtering, or ordering options that affect the result;
- the effective page-size or byte limit when changing it would skip data.

The server must reject a cursor whose request scope does not match the new
request. Cursors must not contain credentials, access tokens, or unbounded
provider response data.

Rust MCP servers should use `mcp_sdk::schemas::pagination::OpaqueCursor` for
cursor transport and `Paginated` for the completion contract. Servers may
define provider-specific cursor payload structs, but cursor validation,
encoding, decoding, and the shared size limit belong to the SDK.

## Range reads

Range-like tools must paginate the provider request itself. The server should
request only the current row/column window from the upstream API. The range
cursor must retain the original range and row/column offsets. A read is
complete only after the final page returns no `next_cursor`.

Transport response limits remain mandatory. If a provider page cannot fit
within the configured transport limit, return a typed error with an action to
narrow the page; never truncate rows, columns, or cell values to make it fit.

## Individual large values

Text and formula cells are never truncated in place. If a value cannot fit in
the normal result, the normal read returns a typed, actionable error and the
server exposes a separate text-chunk operation. Text-chunk results contain:

- the exact resource and cell identity;
- the text or formula kind;
- one UTF-8-safe chunk;
- `next_cursor` until the complete value has been read.

Chunk cursors retain the original cell, rendering mode, chunk size, and byte
offset. Clients concatenate chunks in cursor order and consider the value
complete only when `next_cursor` is absent.

## Agent behavior

When using any MCP tool, the agent must follow the same completion rule. It
must continue through `next_cursor` when the user requested all results, and
must state the exact limitation or ask to continue when a tool returns a
typed size error. It must never claim that a single page is complete.
