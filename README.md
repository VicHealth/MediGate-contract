<div align="center">
  <img src="MediGate_logo.png" alt="MediGate Logo" width="200"/>

  # 🏥 MediGate Smart Contracts

  **Soroban Smart Contracts — Rust + Stellar**

  [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
  [![Stellar](https://img.shields.io/badge/Stellar-Soroban-7B1FA2)](https://stellar.org)
  [![Rust](https://img.shields.io/badge/Rust-1.78+-DEA584)](https://rust-lang.org)
  [![Soroban SDK](https://img.shields.io/badge/Soroban_SDK-21.7.7-FF6B6B)](https://soroban.stellar.org)

  *"Putting the keys to health data back in the hands of the patient."*

</div>

---

## 📋 Overview

This repository contains all **6 Soroban smart contracts** for the MediGate ecosystem — deployed and verified on **Stellar Testnet (Protocol 26)**. These contracts manage decentralized authorization for Electronic Health Records without ever storing PHI/PII on-chain.

### Core Principle

> **No Protected Health Information (PHI) or Personally Identifiable Information (PII) is ever stored on the blockchain.** Only hashes, Decentralized Identifiers (DIDs), permission metadata, and timestamps are recorded on-chain. The actual medical data remains encrypted in HIPAA-compliant off-chain storage.

---

## ✅ Deployed Contracts on Testnet

All contracts are live on **Stellar Testnet** and can be inspected on the Stellar Expert explorer.

| # | Contract | C-Address | Description | Explorer |
|---|----------|-----------|-------------|----------|
| 1 | **AccessLog** | `CBDNQRGC3FWUI3TFMF3AIUJ4Y4AYTPRM2ZNKJLHDOWEZOXAPAPVN4WOQ` | Records patient-provider access events with timestamps | [🔗 View](https://stellar.expert/explorer/testnet/contract/CBDNQRGC3FWUI3TFMF3AIUJ4Y4AYTPRM2ZNKJLHDOWEZOXAPAPVN4WOQ) |
| 2 | **Identity Registry** | `CDDR6NMKYDHL2NWK2HXQ55SRWXIWCVAZ3ZR36OE3RUWR2JUH643FK72T` | Manages Decentralized Identifiers (DIDs) for patients and providers | [🔗 View](https://stellar.expert/explorer/testnet/contract/CDDR6NMKYDHL2NWK2HXQ55SRWXIWCVAZ3ZR36OE3RUWR2JUH643FK72T) |
| 3 | **Access Key Manager** | `CAAJPBEWOVUN2VCY4AK3X354IHSLGICMJYO5WBH6ZPRPYAFOD4EXMK4P` | Time-bound access key generation, validation, and revocation | [🔗 View](https://stellar.expert/explorer/testnet/contract/CAAJPBEWOVUN2VCY4AK3X354IHSLGICMJYO5WBH6ZPRPYAFOD4EXMK4P) |
| 4 | **Permission Mask** | `CCF6NM6FRXQ3HFAHR76OWILYKSO3GKDAS4FY5SX5PXBTUB7GFWVFPWJA` | Granular data category permissions (Allergies, Lab Results, etc.) | [🔗 View](https://stellar.expert/explorer/testnet/contract/CCF6NM6FRXQ3HFAHR76OWILYKSO3GKDAS4FY5SX5PXBTUB7GFWVFPWJA) |
| 5 | **Break-Glass** | `CBUC6GCBXH3DJEV4DFRILRS5FDI4ZW4XXAKITP6DHMUITPDILW5CE7CX` | Emergency access protocol with guardian co-signing | [🔗 View](https://stellar.expert/explorer/testnet/contract/CBUC6GCBXH3DJEV4DFRILRS5FDI4ZW4XXAKITP6DHMUITPDILW5CE7CX) |
| 6 | **Audit Logger** | `CAOL7SIZ6LJDCPD7FMEPQ6ATBZOJQATIFUGIMAP6ADSBEAY36BZ7HAYO` | Immutable on-chain audit trail for all access events | [🔗 View](https://stellar.expert/explorer/testnet/contract/CAOL7SIZ6LJDCPD7FMEPQ6ATBZOJQATIFUGIMAP6ADSBEAY36BZ7HAYO) |

> **Deploy Identity:** `medigate-dev` (`GAM7HQSORJO3XOTORAXN4ODV5XWJBG5BLSWQNERYV2IS2P25AUXZFLWY`)
>
> **Deployed at:** 2026-05-16T15:01:51Z | **Network:** Stellar Testnet (Protocol 26) | **Soroban SDK:** 21.7.7

---

## 📜 Contract Details

### 1. AccessLog (`access_log`)
Records every patient-provider access event with timestamps.

- **Functions:** `log_access`, `get_logs`, `get_recent_logs`, `total_accesses`
- **[Source](./contracts/access_log/)**

### 2. Identity Registry (`identity-registry`)
Manages Decentralized Identifiers (DIDs) for patients and providers.

- **Functions:** `register_did`, `resolve_did`, `update_did`, `deactivate_did`, `is_active`, `total_dids`
- **[Source](./contracts/identity-registry/)**

### 3. Access Key Manager (`access-key-manager`)
Manages time-bound access keys with TTL and automatic expiration.

- **Functions:** `grant_access`, `revoke_access`, `validate_key`, `get_key`, `get_patient_keys`, `get_provider_keys`, `total_keys`
- **[Source](./contracts/access-key-manager/)**

### 4. Permission Mask (`permission-mask`)
Enforces granular data category permissions.

- **Categories:** Allergies, Medications, LabResults, Radiology, Cardiology, MentalHealth, Immunizations, Vitals, Procedures, Genetics
- **Functions:** `set_permission`, `get_permission`, `has_permission`, `revoke_permission`, `revoke_all`, `total_permissions`
- **[Source](./contracts/permission-mask/)**

### 5. Break-Glass (`break-glass`)
Emergency access protocol with guardian co-signing.

- **Functions:** `initiate_break_glass`, `approve_break_glass`, `deny_break_glass`, `get_request`, `get_patient_requests`, `get_pending_requests`, `total_requests`
- **[Source](./contracts/break-glass/)**

### 6. Audit Logger (`audit-logger`)
Immutable on-chain audit trail for all access operations.

- **Event Types:** KeyGranted, KeyRevoked, AccessGranted, AccessDenied, BreakGlassInitiated, BreakGlassResolved, DataViewed, DataUpdated, PatientRegistered, ProviderRegistered
- **Functions:** `log_event`, `get_event`, `get_actor_events`, `get_target_events`, `get_recent_events`, `total_events`, `verify_event`
- **[Source](./contracts/audit-logger/)**

---

## 🚀 Building & Deploying

### Prerequisites

- [Rust](https://www.rust-lang.org/) (1.78+)
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli) (v26+)
- [binaryen](https://github.com/WebAssembly/binaryen) (for `wasm-opt`, recommended)

### Build All Contracts

```bash
RUSTFLAGS="-Ctarget-feature=-reference-types" cargo build --target wasm32-unknown-unknown --release

# Or using npm script
npm run build
```

> **Note:** The `RUSTFLAGS` flag is needed when using Rust 1.82+ to ensure WASM compatibility with Stellar's VM.

### Deploy to Testnet

```bash
npm run deploy:testnet

# Or manually:
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/access_log.wasm \
  --network testnet \
  --source <your-identity>
```

### Run Tests

```bash
cargo test
```

---

## 🏗️ Design Principles

- **No PHI/PII On-Chain** — Only hashes, DIDs, and permission metadata
- **Time-Bound Access** — All keys have automatic expiration (TTL)
- **Granular Control** — Permission masks for specific data categories
- **Emergency Override** — Break-Glass with guardian co-signing
- **Immutable Audit Trail** — Every access event logged on-chain

---

## 📁 Project Structure

```
contracts/
├── access_log/                # AccessLog contract (MVP)
├── access-key-manager/        # TTL-based key management
├── audit-logger/              # Immutable audit trail
├── break-glass/               # Emergency access protocol
├── identity-registry/         # DID management
└── permission-mask/           # Granular data permissions
scripts/
├── deploy.sh                  # Bash deployment script
└── deploy.mjs                 # Node.js deployment script
Cargo.toml                     # Workspace config
Cargo.lock                     # Dependency lockfile
package.json                   # NPM scripts
.contract-ids                  # Deployed contract IDs
```

---

## 🔗 Related Repositories

| Repository | Description |
|-----------|-------------|
| [MediGate-frontend](https://github.com/VicHealth/MediGate-frontend) | React UI |
| [MediGate-Backend](https://github.com/VicHealth/MediGate-Backend) | Express orchestrator API |

---

## 🤝 Contributing

1. **Fork** the repository
2. **Create a feature branch:** `git checkout -b feature/amazing-feature`
3. **Commit your changes:** `git commit -m 'Add amazing feature'`
4. **Push to the branch:** `git push origin feature/amazing-feature`
5. **Open a Pull Request**

---

## 📄 License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

---

<div align="center">
  <strong>MediGate</strong> — *Decentralizing the gateway to medical records, one key at a time.*
  <br/><br/>
  [![Stellar](https://img.shields.io/badge/Built_on-Stellar-7B1FA2)](https://stellar.org)
  [![Soroban](https://img.shields.io/badge/Powered_by-Soroban-FF6B6B)](https://soroban.stellar.org)
</div>

>
> **Contract IDs saved in:** [`.contract-ids`](./.contract-ids)