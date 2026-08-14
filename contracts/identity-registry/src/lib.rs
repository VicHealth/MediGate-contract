//! # Identity Registry Contract
//!
//! Manages Decentralized Identifiers (DIDs) for patients and providers
//! on the Stellar network. This contract stores only metadata hashes
//! and DID mappings — never PHI/PII.
//!
//! ## Key Functions
//! - `register_did`: Register a new DID for a patient or provider
//! - `resolve_did`: Look up a DID to get the associated Stellar public key
//! - `update_did`: Update DID metadata (e.g., after SEP-30 recovery)
//! - `deactivate_did`: Deactivate a DID (mark as revoked)
//! - `is_active`: Check if a DID is currently active

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol,
};

// ============================================
// Data Types
// ============================================

/// Role of the DID holder
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum UserRole {
    Patient,
    Provider,
    EmergencyResponder,
}

/// Status of a DID
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum DidStatus {
    Active,
    Deactivated,
}

/// DID metadata stored on-chain
#[derive(Clone, Debug)]
#[contracttype]
pub struct DidEntry {
    pub stellar_address: Address,
    pub role: UserRole,
    pub status: DidStatus,
    pub registered_at: u64,
    pub updated_at: u64,
}

/// Registration event data
#[derive(Clone, Debug)]
#[contracttype]
pub struct RegistrationEvent {
    pub did: String,
    pub stellar_address: Address,
    pub role: UserRole,
    pub timestamp: u64,
}

// ============================================
// Storage Keys
// ============================================

/// Compound storage keys for parameterized lookups
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    DidEntry(String),
    RegistrationEvent(String),
}

const DID_COUNT: Symbol = symbol_short!("DID_COUNT");

// ============================================
// Contract
// ============================================

#[contract]
pub struct IdentityRegistryContract;

#[contractimpl]
impl IdentityRegistryContract {
    /// Register a new DID for a user.
    ///
    /// # Arguments
    /// * `did` - The Decentralized Identifier string (e.g., "did:stellar:GB1234...")
    /// * `stellar_address` - The Stellar account address
    /// * `role` - The user's role (Patient, Provider, or EmergencyResponder)
    ///
    /// # Panics
    /// * If the DID is already registered and active
    pub fn register_did(
        env: Env,
        did: String,
        stellar_address: Address,
        role: UserRole,
    ) -> DidEntry {
        // Check if DID already exists and is active
        let existing = Self::get_did_entry(&env, &did);
        if let Some(entry) = existing {
            if entry.status == DidStatus::Active {
                panic!("DID already registered and active");
            }
        }

        let timestamp = env.ledger().timestamp();

        let entry = DidEntry {
            stellar_address: stellar_address.clone(),
            role: role.clone(),
            status: DidStatus::Active,
            registered_at: timestamp,
            updated_at: timestamp,
        };

        // Store the DID entry
        env.storage()
            .persistent()
            .set(&DataKey::DidEntry(did.clone()), &entry);

        // Increment DID count
        let count: u64 = env.storage().persistent().get(&DID_COUNT).unwrap_or(0);
        env.storage().persistent().set(&DID_COUNT, &(count + 1));

        // Emit registration event
        let event = RegistrationEvent {
            did: did.clone(),
            stellar_address,
            role,
            timestamp,
        };
        env.storage()
            .persistent()
            .set(&DataKey::RegistrationEvent(did.clone()), &event);

        entry
    }

    /// Resolve a DID to get the associated Stellar address and metadata.
    ///
    /// # Arguments
    /// * `did` - The DID to resolve
    ///
    /// # Returns
    /// * `Option<DidEntry>` - The DID entry if found
    pub fn resolve_did(env: Env, did: String) -> Option<DidEntry> {
        Self::get_did_entry(&env, &did)
    }

    /// Update the Stellar address associated with a DID.
    /// Used after SEP-30 social recovery to point to a new key.
    ///
    /// # Arguments
    /// * `did` - The DID to update
    /// * `new_address` - The new Stellar address
    /// * `caller` - The address making the request (must match current address)
    pub fn update_did(env: Env, did: String, new_address: Address, caller: Address) -> DidEntry {
        caller.require_auth();

        let mut entry = Self::get_did_entry(&env, &did).expect("DID not found");

        // Verify caller is the current owner
        if entry.stellar_address != caller {
            panic!("Only the DID owner can update the address");
        }

        if entry.status != DidStatus::Active {
            panic!("Cannot update a deactivated DID");
        }

        entry.stellar_address = new_address;
        entry.updated_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::DidEntry(did.clone()), &entry);

        entry
    }

    /// Deactivate a DID.
    ///
    /// # Arguments
    /// * `did` - The DID to deactivate
    /// * `caller` - The address making the request (must match current address)
    pub fn deactivate_did(env: Env, did: String, caller: Address) {
        caller.require_auth();

        let mut entry = Self::get_did_entry(&env, &did).expect("DID not found");

        if entry.stellar_address != caller {
            panic!("Only the DID owner can deactivate");
        }

        entry.status = DidStatus::Deactivated;
        entry.updated_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::DidEntry(did.clone()), &entry);
    }

    /// Check if a DID is active.
    ///
    /// # Arguments
    /// * `did` - The DID to check
    ///
    /// # Returns
    /// * `bool` - Whether the DID is active
    pub fn is_active(env: Env, did: String) -> bool {
        let entry = Self::get_did_entry(&env, &did);
        match entry {
            Some(e) => e.status == DidStatus::Active,
            None => false,
        }
    }

    /// Get the total number of registered DIDs.
    pub fn total_dids(env: Env) -> u64 {
        env.storage().persistent().get(&DID_COUNT).unwrap_or(0)
    }

    // ============================================
    // Internal Helpers
    // ============================================

    fn get_did_entry(env: &Env, did: &String) -> Option<DidEntry> {
        env.storage()
            .persistent()
            .get(&DataKey::DidEntry(did.clone()))
    }
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_register_and_resolve_did() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, IdentityRegistryContract);
        let client = IdentityRegistryContractClient::new(&env, &contract_id);

        let stellar_address = Address::generate(&env);
        let did = String::from_slice(&env, "did:stellar:GB1234ABCD");

        let entry = client.register_did(&did, &stellar_address, &UserRole::Patient);

        assert_eq!(entry.stellar_address, stellar_address);
        assert_eq!(entry.role, UserRole::Patient);
        assert_eq!(entry.status, DidStatus::Active);

        // Resolve the DID
        let resolved = client.resolve_did(&did);
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().stellar_address, stellar_address);
    }

    #[test]
    fn test_is_active() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, IdentityRegistryContract);
        let client = IdentityRegistryContractClient::new(&env, &contract_id);

        let stellar_address = Address::generate(&env);
        let did = String::from_slice(&env, "did:stellar:GB5678EFGH");

        assert!(!client.is_active(&did));

        client.register_did(&did, &stellar_address, &UserRole::Provider);
        assert!(client.is_active(&did));
    }

    #[test]
    fn test_deactivate_did() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, IdentityRegistryContract);
        let client = IdentityRegistryContractClient::new(&env, &contract_id);

        let stellar_address = Address::generate(&env);
        let did = String::from_slice(&env, "did:stellar:GB9012IJKL");

        client.register_did(&did, &stellar_address, &UserRole::Patient);
        assert!(client.is_active(&did));

        client.deactivate_did(&did, &stellar_address);
        assert!(!client.is_active(&did));
    }

    #[test]
    fn test_total_dids() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, IdentityRegistryContract);
        let client = IdentityRegistryContractClient::new(&env, &contract_id);

        assert_eq!(client.total_dids(), 0);

        let addr1 = Address::generate(&env);
        let addr2 = Address::generate(&env);

        client.register_did(
            &String::from_slice(&env, "did:stellar:AAA"),
            &addr1,
            &UserRole::Patient,
        );
        assert_eq!(client.total_dids(), 1);

        client.register_did(
            &String::from_slice(&env, "did:stellar:BBB"),
            &addr2,
            &UserRole::Provider,
        );
        assert_eq!(client.total_dids(), 2);
    }
}
