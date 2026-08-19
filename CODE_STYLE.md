# Code Style

This repository favors small, inspectable Rust modules. File boundaries are
part of the design: a reader should be able to open an implementation file and
understand its ownership without discovering unrelated helpers at the bottom.

## One primary implementation per file

Each Rust file should have one primary implementation owner:

- one main struct, trait, or enum and its implementation methods; or
- one focused helper type and its implementation; or
- one focused group of free helper functions with a single purpose.

Do not place paging structs, conversion functions, validation functions, or
response-body types in the provider's core implementation file. Supporting
types should live beside their own implementation, and helper functions should
live in purpose-named modules such as `paging.rs`, `conversion.rs`,
`validation.rs`, or `cursor.rs`.

When a service grows, use a module directory:

```text
providers/
├── common.rs                  # shared transport/configuration
├── drive.rs                   # GoogleDriveProvider
└── sheets/
    ├── mod.rs                 # module declarations only
    ├── provider.rs            # GoogleSheetsProvider
    ├── conversion.rs         # provider response conversion
    ├── cursor.rs              # cursor decoding
    ├── paging.rs              # RangePage
    ├── types.rs               # supporting response/body types
    └── validation.rs          # request/range validation helpers
```

`mod.rs` files declare modules only. They must not become a second provider
implementation or a place for broad re-exports.

## Core implementation files

A provider implementation file may contain private methods needed directly by
that provider, but it should not contain unrelated free functions or helper
structs. If a method is only conversion, validation, paging, or cursor logic,
move it to the corresponding focused module and call it explicitly from the
provider.

Avoid generic catch-all files such as `utils.rs`, `helpers.rs`, or
`mutations.rs` when a narrower name describes the responsibility.

## Validation

Before submitting a Rust change, run:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```
