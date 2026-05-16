//! # Access Key Manager Contract
//!
//! Manages time-bound access keys that patients grant to healthcare providers.
//! Keys have a Time-to-Live (TTL) and automatically expire on the ledger.
//! Patients can also revoke keys before their TTL expires.
//!
//! ## Key Functions
//! - `grant_access`: Create a new time-bound access key
//! - `revoke_access`: Revoke an active access key before expiration
//! - `validate_key`: Check if a key is valid for a specific data category
//! - `get_key`: Get details of a specific access key
//! - `get_patient_keys`: List all keys for a patient
//! - `get_provider_keys`: List all keys for a provider

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Vec, Map};

// ============================================
// Data Types
// ============================================

/// Status of an access key
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum KeyStatus {
    Active,
    Revoked,
    Expired,
}

/// Data categories that can be permitted
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum DataCategory {
    Allergies,
    Medications,
    LabResults,
    Radiology,
    Cardiology,
    MentalHealth,
    Immunizations,
    Vitals,
    Procedures,
    Genetics,
}

/// An access key entry stored on-chain
#[derive(Clone, Debug)]
#[contracttype]
pub struct AccessKey {
    pub id: String,
    pub patient: Address,
    pub provider: Address,
    pub permission_mask: Vec<DataCategory>,
    pub issued_at: u64,
    pub expires_at: u64,
    pub status: KeyStatus,
}

// ============================================
// Storage Keys
// ============================================

/// Compound storage keys for parameterized lookups
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    AccessKey(String),
    PatientKeys(Address),
    ProviderKeys(Address),
}

const KEY_COUNT: Symbol = symbol_short!("KEY_COUNT");

// ============================================
// Contract
// ============================================

#[contract]
pub struct AccessKeyManagerContract;

#[contractimpl]
impl AccessKeyManagerContract {
    /// Grant a new time-bound access key to a provider.
    ///
    /// # Arguments
    /// * `key_id` - Unique identifier for this key
    /// * `patient` - The patient granting access
    /// * `provider` - The provider receiving access
    /// * `permission_mask` - List of permitted data categories
    /// * `ttl` - Time-to-live in seconds
    ///
    /// # Panics
    /// * If TTL is less than 300 seconds (5 minutes) or more than 1 year
    /// * If permission mask is empty
    pub fn grant_access(
        env: Env,
        key_id: String,
        patient: Address,
        provider: Address,
        permission_mask: Vec<DataCategory>,
        ttl: u64,
    ) -> AccessKey {
        patient.require_auth();

        // Validate TTL
        if ttl < 300 {
            panic!("TTL must be at least 300 seconds (5 minutes)");
        }
        if ttl > 31_536_000 {
            panic!("TTL cannot exceed 31,536,000 seconds (1 year)");
        }

        // Validate permission mask
        if permission_mask.is_empty() {
            panic!("Permission mask cannot be empty");
        }

        let now = env.ledger().timestamp();
        let expires_at = now + ttl;

        let key = AccessKey {
            id: key_id.clone(),
            patient: patient.clone(),
            provider: provider.clone(),
            permission_mask: permission_mask.clone(),
            issued_at: now,
            expires_at,
            status: KeyStatus::Active,
        };

        // Store the key
        env.storage().persistent().set(&DataKey::AccessKey(key_id.clone()), &key);

        // Add to patient's key list
        let mut patient_keys: Vec<String> = env.storage()
            .persistent()
            .get(&DataKey::PatientKeys(patient.clone()))
            .unwrap_or(Vec::new(&env));
        patient_keys.push_back(key_id.clone());
        env.storage().persistent().set(&DataKey::PatientKeys(patient.clone()), &patient_keys);

        // Add to provider's key list
        let mut provider_keys: Vec<String> = env.storage()
            .persistent()
            .get(&DataKey::ProviderKeys(provider.clone()))
            .unwrap_or(Vec::new(&env));
        provider_keys.push_back(key_id.clone());
        env.storage().persistent().set(&DataKey::ProviderKeys(provider.clone()), &provider_keys);

        // Increment key count
        let count: u64 = env.storage().persistent().get(&KEY_COUNT).unwrap_or(0);
        env.storage().persistent().set(&KEY_COUNT, &(count + 1));

        key
    }

    /// Revoke an active access key.
    ///
    /// # Arguments
    /// * `key_id` - The key to revoke
    /// * `caller` - The address requesting revocation (must be the patient)
    pub fn revoke_access(
        env: Env,
        key_id: String,
        caller: Address,
    ) -> AccessKey {
        caller.require_auth();

        let mut key = Self::get_key_internal(&env, &key_id)
            .expect("Access key not found");

        if key.patient != caller {
            panic!("Only the patient who issued the key can revoke it");
        }

        if key.status != KeyStatus::Active {
            panic!("Key is not active");
        }

        key.status = KeyStatus::Revoked;
        env.storage().persistent().set(&DataKey::AccessKey(key_id.clone()), &key);

        key
    }

    /// Validate an access key for a specific data category.
    /// Returns true if the key is active, not expired, and has the required permission.
    ///
    /// # Arguments
    /// * `key_id` - The key to validate
    /// * `category` - The data category to check
    ///
    /// # Returns
    /// * `bool` - Whether the key is valid for the given category
    pub fn validate_key(
        env: Env,
        key_id: String,
        category: DataCategory,
    ) -> bool {
        let key = match Self::get_key_internal(&env, &key_id) {
            Some(k) => k,
            None => return false,
        };

        // Check if key is active
        if key.status != KeyStatus::Active {
            return false;
        }

        // Check if key has expired
        let now = env.ledger().timestamp();
        if now >= key.expires_at {
            // Auto-expire the key
            let mut expired_key = key;
            expired_key.status = KeyStatus::Expired;
            env.storage().persistent().set(&DataKey::AccessKey(key_id.clone()), &expired_key);
            return false;
        }

        // Check if the category is in the permission mask
        key.permission_mask.contains(&category)
    }

    /// Get details of a specific access key.
    ///
    /// # Arguments
    /// * `key_id` - The key to look up
    ///
    /// # Returns
    /// * `Option<AccessKey>` - The key if found
    pub fn get_key(env: Env, key_id: String) -> Option<AccessKey> {
        Self::get_key_internal(&env, &key_id)
    }

    /// Get all key IDs for a patient.
    ///
    /// # Arguments
    /// * `patient` - The patient's address
    ///
    /// # Returns
    /// * `Vec<String>` - List of key IDs
    pub fn get_patient_keys(env: Env, patient: Address) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&DataKey::PatientKeys(patient.clone()))
            .unwrap_or(Vec::new(&env))
    }

    /// Get all key IDs for a provider.
    ///
    /// # Arguments
    /// * `provider` - The provider's address
    ///
    /// # Returns
    /// * `Vec<String>` - List of key IDs
    pub fn get_provider_keys(env: Env, provider: Address) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&DataKey::ProviderKeys(provider.clone()))
            .unwrap_or(Vec::new(&env))
    }

    /// Get the total number of access keys issued.
    pub fn total_keys(env: Env) -> u64 {
        env.storage().persistent().get(&KEY_COUNT).unwrap_or(0)
    }

    // ============================================
    // Internal Helpers
    // ============================================

    fn get_key_internal(env: &Env, key_id: &String) -> Option<AccessKey> {
        env.storage().persistent().get(&DataKey::AccessKey(key_id.clone()))
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
    fn test_grant_and_validate_key() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AccessKeyManagerContract);
        let client = AccessKeyManagerContractClient::new(&env, &contract_id);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let key_id = String::from_slice(&env, "key-001");

        let mut mask = Vec::new(&env);
        mask.push_back(DataCategory::Allergies);
        mask.push_back(DataCategory::Medications);

        let key = client.grant_access(&key_id, &patient, &provider, &mask, &3600);

        assert_eq!(key.status, KeyStatus::Active);
        assert_eq!(key.patient, patient);
        assert_eq!(key.provider, provider);

        // Validate the key
        assert!(client.validate_key(&key_id, &DataCategory::Allergies));
        assert!(client.validate_key(&key_id, &DataCategory::Medications));
        assert!(!client.validate_key(&key_id, &DataCategory::LabResults));
    }

    #[test]
    fn test_revoke_key() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AccessKeyManagerContract);
        let client = AccessKeyManagerContractClient::new(&env, &contract_id);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let key_id = String::from_slice(&env, "key-002");

        let mut mask = Vec::new(&env);
        mask.push_back(DataCategory::Vitals);

        client.grant_access(&key_id, &patient, &provider, &mask, &3600);
        assert!(client.validate_key(&key_id, &DataCategory::Vitals));

        client.revoke_access(&key_id, &patient);
        assert!(!client.validate_key(&key_id, &DataCategory::Vitals));
    }

    #[test]
    fn test_patient_and_provider_keys() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AccessKeyManagerContract);
        let client = AccessKeyManagerContractClient::new(&env, &contract_id);

        let patient = Address::generate(&env);
        let provider1 = Address::generate(&env);
        let provider2 = Address::generate(&env);

        let mut mask = Vec::new(&env);
        mask.push_back(DataCategory::Allergies);

        client.grant_access(
            &String::from_slice(&env, "key-a"),
            &patient,
            &provider1,
            &mask,
            &3600,
        );
        client.grant_access(
            &String::from_slice(&env, "key-b"),
            &patient,
            &provider2,
            &mask,
            &3600,
        );

        let patient_keys = client.get_patient_keys(&patient);
        assert_eq!(patient_keys.len(), 2);

        let provider1_keys = client.get_provider_keys(&provider1);
        assert_eq!(provider1_keys.len(), 1);
    }

    #[test]
    fn test_total_keys() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AccessKeyManagerContract);
        let client = AccessKeyManagerContractClient::new(&env, &contract_id);

        assert_eq!(client.total_keys(), 0);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);
        let mut mask = Vec::new(&env);
        mask.push_back(DataCategory::Allergies);

        client.grant_access(
            &String::from_slice(&env, "key-1"),
            &patient,
            &provider,
            &mask,
            &3600,
        );
        assert_eq!(client.total_keys(), 1);
    }
}
