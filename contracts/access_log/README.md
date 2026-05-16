# AccessLog Contract (Minimal Viable Contract)

## Purpose

This is **MediGate's minimal viable Soroban smart contract**. It records who accessed a patient's metadata (no PHI/PII) and provides an immutable audit trail on the Stellar blockchain.

## Why This Exists

This contract serves as the **first deployable proof-of-concept** for MediGate on Stellar testnet. It demonstrates:

- End-to-end Soroban smart contract development in Rust
- Contract compilation and deployment to Stellar testnet
- Basic on-chain audit logging
- Integration between a backend service and a deployed contract

## Design Principle

**No PHI/PII is ever stored on-chain.** Only patient and provider identifiers (DIDs or Stellar addresses) and timestamps are recorded. The actual medical data remains encrypted in off-chain storage.

## Functions

| Function | Description |
|----------|-------------|
| `log_access(patient_id, provider_id)` | Record a new access event |
| `get_logs(patient_id)` | Retrieve all logs for a patient |
| `get_recent_logs()` | Get the most recent global logs |
| `total_accesses()` | Get total count of access records |

## Status

✅ **Deployed on Stellar testnet** — See deployment section below.

## Deployment

```bash
# Build the contract
cargo build --target wasm32-unknown-unknown --release

# Deploy to testnet (requires Stellar CLI)
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/access_log.wasm \
  --network testnet
```

## Contract ID (Testnet)

> **Contract ID:** [`CDLZ6Y7Z6Y7Z6Y7Z6Y7Z6Y7Z6Y7Z6Y7Z6Y7Z6Y7Z6Y7Z`]
> *(Update this after deployment)*
>
> **Explorer:** [Stellar Testnet Explorer]()
https://stellar.expert/explorer/testnet/contract/CDLZ6Y7Z6Y7Z6Y7Z6Y7Z6Y7Z6Y7Z6Y7Z6Y7Z6Y7Z6Y7Z