# Contributing to MediGate Contracts

Thanks for considering a contribution. This workspace contains the Soroban smart contracts for MediGate (identity registry, access-key manager, permission mask, break glass, and audit logging).

## Code of Conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md). By participating you agree to uphold it.

## Development Setup

Prerequisites: Rust (stable) with the `wasm32-unknown-unknown` target.

```bash
rustup target add wasm32-unknown-unknown
cd MediGate-contract
cargo build --workspace
cargo test --workspace
cargo build --workspace --release --target wasm32-unknown-unknown
```

## Workspace Layout

| Path | Purpose |
|---|---|
| `contracts/identity-registry/` | identity registry contract |
| `contracts/access-key-manager/` | access key management |
| `contracts/permission-mask/` | permission control |
| `contracts/break-glass/` | emergency access |
| `contracts/audit-logger/` + `contracts/access_log/` | audit logging |

## Making Changes

1. Branch per issue: `git checkout -b <issue-number>-<short-slug>`
2. Add unit tests for new contract functions.
3. Run `cargo fmt`, `cargo clippy --workspace`, and `cargo test --workspace` before pushing.

## Pull Requests

Reference the issue you're closing, keep the change focused, and ensure CI (fmt, clippy, tests, WASM build) is green.

## Security

Do not open public issues for security problems — follow the process in [SECURITY.md](SECURITY.md).
