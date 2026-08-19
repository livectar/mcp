# Contributing

Keep this repository host-neutral. Public crates must not import AI Social
application crates, database entities, assistant workflows, or private
provider modules.

Read [`CODE_STYLE.md`](CODE_STYLE.md) before adding or expanding a provider.

Before opening a change, run:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Protocol and trait changes require tests for both the wire contract and the
affected server or transport behavior. Provider tests must use mocks; live
credentials do not belong in this repository or its CI.
