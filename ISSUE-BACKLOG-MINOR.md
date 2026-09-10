# MediGate-contract — Minor Issues Backlog (9 Issues)

Sized strictly as **100 pts (Trivial / Good First Issue)** in Drips Wave criteria.

---

## #1: Add docstrings explaining emergency break-glass permission bitmasks in permission-mask

- **Labels**: `complexity:trivial, contract, documentation, good first issue`

- **Complexity**: `100 pts` (Trivial)


### Summary
Add comprehensive docstrings explaining the bitwise permission flag operators (`0b0001` read, `0b0010` write, `0b0100` emergency break-glass) in `permission-mask/src/lib.rs`.

### Requirements
- Document bitwise operations (`has_permission`, `grant_permission`, `revoke_permission`).
- Detail security boundaries and audit implications of bitmask alterations.
- Ensure `cargo doc --no-deps` completes without warnings.

---

## #2: Add CI build, Soroban SDK version, and Apache-2.0 badges to README.md

- **Labels**: `complexity:trivial, contract, documentation`

- **Complexity**: `100 pts` (Trivial)


### Summary
Update contract repository `README.md` with official Shields.io badges for CI status, Soroban SDK version, and open source license.

### Requirements
- Add status badges linked to GitHub Actions workflow.
- Include badges for Soroban SDK v22 and Rust edition 2021.
- Verify clean markdown formatting on GitHub.

---

## #3: Add unit test verifying rejection of expired break-glass access tokens

- **Labels**: `complexity:trivial, contract, testing, security`

- **Complexity**: `100 pts` (Trivial)


### Summary
Ensure the `break-glass` smart contract strictly rejects emergency access requests when the token timestamp has passed expiration.

### Requirements
- Write unit test in `contracts/break-glass/src/test.rs` advancing ledger timestamp beyond the 4-hour window.
- Verify contract call fails with `Error::TokenExpired`.
- Confirm full test suite passes with `cargo test`.

---

## #4: Add unit test validating audit logger rejects duplicate log event sequence numbers

- **Labels**: `complexity:trivial, contract, testing, good first issue`

- **Complexity**: `100 pts` (Trivial)


### Summary
Validate that the `audit-logger` contract prevents replay of already logged event sequences.

### Requirements
- Add unit test submitting duplicate event sequence IDs.
- Assert contract returns `Error::DuplicateSequence`.
- Verify test runs deterministically.

---

## #5: Extract permission bitmask constants into dedicated types.rs module

- **Labels**: `complexity:trivial, contract, code-hygiene`

- **Complexity**: `100 pts` (Trivial)


### Summary
Extract raw numeric bitwise literals (`1 << 0`, `1 << 1`, `1 << 2`) into documented named constants in `types.rs`.

### Requirements
- Define `pub const MASK_READ: u32 = 1 << 0;`, `pub const MASK_WRITE: u32 = 1 << 1;`, `pub const MASK_EMERGENCY: u32 = 1 << 2;`.
- Replace raw bitwise literals across all contract modules.
- Ensure 100% test pass rate.

---

## #6: Add error documentation table with human-readable error descriptions in errors.rs

- **Labels**: `complexity:trivial, contract, documentation, good first issue`

- **Complexity**: `100 pts` (Trivial)


### Summary
Provide clear documentation for every error variant defined in `errors.rs` to streamline healthcare API integration.

### Requirements
- Annotate `UnauthorizedPhysician`, `TokenExpired`, `AuditLogCorrupted`, and `ConsentWithdrawn`.
- Include recommended remediation steps in docstrings for frontends.
- Check generated docs for clean formatting.

---

## #7: Add cargo clippy and fmt lint script for all contract crates in the workspace

- **Labels**: `complexity:trivial, contract, ci, tooling`

- **Complexity**: `100 pts` (Trivial)


### Summary
Add a shell script and Makefile target to run `cargo fmt --check` and `cargo clippy -- -D warnings` across the workspace.

### Requirements
- Create `scripts/lint.sh` executable.
- Ensure zero warnings are produced on current contracts.
- Document command in `CONTRIBUTING.md`.

---

## #8: Add execution gas benchmark report for patient record registration and audit logging

- **Labels**: `complexity:trivial, contract, documentation`

- **Complexity**: `100 pts` (Trivial)


### Summary
Measure and record gas and ledger footprint for emergency break-glass invocations and audit logging.

### Requirements
- Run budget meter tests recording CPU instructions and RAM usage.
- Create `docs/benchmarks.md` with tabular results.
- Provide instructions for running benchmark tests locally.

---

## #9: Add contract state lifecycle diagram and interface overview table to README.md

- **Labels**: `complexity:trivial, contract, documentation`

- **Complexity**: `100 pts` (Trivial)


### Summary
Provide visual architecture and interface documentation for MediGate smart contracts.

### Requirements
- Add Mermaid diagram illustrating state progression: Access Request -> Break-Glass Approval -> Audit Log -> Expiration.
- Add summary table of public functions for `break-glass`, `consent-delegation`, and `audit-logger`.
- Verify GitHub renders Mermaid diagram cleanly.

---
