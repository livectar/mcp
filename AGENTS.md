# Livectar MCP Agent Instructions

These instructions apply to every package in this public repository. The
repository is host-neutral: it must build and run without AI Social, its
database, workspace authorization, assistant workflows, or private provider
modules.

## Package ownership

Keep the dependency direction one-way:

```text
mcp-protocol
      ↑
mcp-sdk ← mcp-testkit
      ↑
mcp-transport
      ↑
servers/*
```

The protocol crate owns wire and lifecycle contracts. The SDK owns public
server/tool traits and host-service interfaces. Transport owns HTTP dispatch.
Provider servers own provider API clients and provider-specific tools. Testkit
owns test doubles only. A provider package must never query AI Social tables
or define a second version of an SDK trait.

## Required package layout

Every package must have a focused crate entrypoint:

```text
package/
├── Cargo.toml
└── src/
    ├── lib.rs                 # public module declarations and small API surface
    ├── errors.rs              # typed package errors, when errors are shared
    ├── schemas/               # typed requests, responses, enums, and wire contracts
    │   ├── mod.rs
    │   └── *.rs
    ├── models/                # optional provider/domain models
    │   ├── mod.rs
    │   └── *.rs
    ├── services/              # optional use-case orchestration
    │   ├── mod.rs
    │   └── *.rs
    ├── handlers/              # optional tool/request handlers
    │   ├── mod.rs
    │   └── *.rs
    ├── providers/             # optional provider clients and error mapping
    │   ├── mod.rs
    │   └── *.rs
    └── tests/                 # optional integration tests
```

Do not create empty directories just to match this template. `schemas/` is
required whenever a package owns shared request, response, tool, event, or
status contracts. `models/`, `services/`, `handlers/`, `providers/`, and
`tests/` are added only when the package has that responsibility.

Use these package-specific variants:

```text
mcp-protocol/
└── src/
    ├── lib.rs
    ├── schemas/
    └── errors.rs

mcp-sdk/
└── src/
    ├── lib.rs
    ├── schemas/
    ├── traits/
    └── errors.rs

mcp-transport/
└── src/
    ├── lib.rs
    ├── schemas/
    ├── dispatch/
    └── errors.rs

servers/<provider>/
├── README.md                 # server purpose, credentials, and usage
└── src/
    ├── lib.rs
    ├── schemas/
    ├── models/                # provider/domain response models, optional
    ├── providers/
    ├── handlers/
    ├── services/              # orchestration only, optional
    ├── errors.rs
    └── tests/

mcp-testkit/
└── src/
    ├── lib.rs
    ├── fixtures/
    ├── doubles/
    └── builders/
```

If a package is later converted to a binary, keep `main.rs` limited to
configuration, dependency construction, route/registry assembly, and startup.
Put the implementation in the same modules listed above. A binary may use
`config.rs`, `state.rs`, `routes.rs`, and `registry.rs` at its root when those
are composition concerns.

## `lib.rs` and `mod.rs` rules

- Every library package has a `src/lib.rs` entrypoint.
- `lib.rs` declares top-level modules and may expose a small, deliberate
  public surface with explicit re-exports, for example:

  ```rust
  mod errors;
  pub mod schemas;
  pub mod traits;
  ```

- Do not add `pub use` re-exports. Import public contracts, traits, and
  constructors from their defining modules. Never use `pub use module::*`.
- Nested `mod.rs` files contain module declarations and, when needed, a small
  explicit public surface. They must not contain business logic, large type
  definitions, or initialization code.
- Keep implementation types private unless another package genuinely needs
  them. Prefer imports from the defining module over a deep re-export tree.
- Keep `From`, `TryFrom`, and related conversion implementations beside the
  type they implement.

## Typed contracts

- Put every shared request, response, tool input/output, lifecycle message,
  provider status, error code, and finite mode in `schemas/`.
- Use Serde structs and enums. Known finite values must be enums, not strings.
- Use typed identifiers and bounded named wrappers for opaque text.
- Do not use `serde_json::Value`, JSON maps, `json!`, `to_value`, or
  `from_value`. Do not pass dynamic JSON between internal functions.
- MCP wire payloads may use a named bounded JSON wrapper at the final protocol
  boundary when the protocol is intentionally generic. Decode it immediately
  into the package's typed schema before business logic runs.
- Serialize typed contracts once at the transport boundary and reject invalid
  or oversized payloads explicitly.
- Keep provider credentials out of tool arguments, schemas, prompts, logs,
  audit payloads, and routing configuration. Credentials enter only through
  the SDK host-service traits.

## Responsibilities by module

### Schemas

Own public contracts and validation-relevant types. Schema modules must not
make network calls, access host storage, or perform orchestration.

### Models

Models are optional and represent provider/domain data or typed API responses.
They are not a database layer in this repository. Do not add SeaORM entities,
AI Social persistence models, or provider-specific storage here.

### Providers

Own provider clients, request construction, pagination, provider error
mapping, and credential use. Providers receive credentials from the SDK
context and must not resolve or persist them themselves.

### Handlers

Own tool-level validation and dispatch. Handlers call provider/services and
return typed protocol results. They do not define shared contracts inline.

### Services

Own multi-step use cases that compose handlers, providers, and host services.
Keep them stateless unless durable state is explicitly part of a public
runtime package.

### Errors

Use typed errors with `thiserror`. Preserve enough context for callers to map
errors to protocol responses, but never include secrets or unbounded provider
responses.

## Testing

- Keep unit tests beside the module they exercise.
- Put cross-module and HTTP contract tests under `tests/` or a dedicated test
  module.
- Provider tests must use mocked provider clients and host services; live
  credentials are never required.
- Test tool schemas, typed decoding, authorization/approval behavior,
  credential injection, bounded errors, and protocol serialization.
- Each package must be independently testable from the AI Social repository.

## Formatting and validation

Run from `external/mcp` before submitting a change:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Keep `Cargo.toml` package boundaries explicit. Add a dependency only to the
lowest-level package that owns the behavior, and avoid pulling provider or
application dependencies into protocol and SDK crates.

## Per-server documentation

Every package under `servers/` must contain a `README.md`. Keep it updated in
the same change as the server implementation. The README must explain:

- the server's goal, scope, and explicit non-goals;
- the provider or external system it integrates with;
- the tools it exposes, including typed business arguments and result shape;
- required credentials, provider scopes, permissions, connection setup, and
  how the host injects credentials through `CredentialResolver`;
- a safe local setup and usage example using placeholders, never real tokens;
- authorization, approval, rate-limit, pagination, freshness, and access
  limitations;
- provider error behavior and the package's mocked test command.

Credential values, OAuth tokens, phone codes, 2FA passwords, and session data
must never be committed to a server README. Tool examples must contain only
business arguments; credential selection and secret injection remain internal
host behavior.
