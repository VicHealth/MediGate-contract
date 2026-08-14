//! # Permission Mask Contract
//!
//! Enforces granular data category permissions for access keys.
//! This contract works alongside the Access Key Manager to provide
//! fine-grained control over which specific data categories a provider
//! can access for a given patient.
//!
//! ## Key Functions
//! - `set_permission`: Set or update a permission mask for a patient-provider pair
//! - `get_permission`: Get the permission mask for a patient-provider pair
//! - `has_permission`: Check if a provider has permission for a specific category
//! - `revoke_permission`: Remove a specific category from a permission mask
//! - `revoke_all`: Remove all permissions for a patient-provider pair

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

// ============================================
// Data Types
// ============================================

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

/// A permission mask entry stored on-chain
#[derive(Clone, Debug)]
#[contracttype]
pub struct PermissionMask {
    pub patient: Address,
    pub provider: Address,
    pub categories: Vec<DataCategory>,
    pub granted_at: u64,
    pub updated_at: u64,
}

// ============================================
// Storage Keys
// ============================================

/// Compound storage keys for parameterized lookups
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Permission(Address, Address),
}

const PERMISSION_COUNT: Symbol = symbol_short!("PERM_CNT");

// ============================================
// Contract
// ============================================

#[contract]
pub struct PermissionMaskContract;

#[contractimpl]
impl PermissionMaskContract {
    /// Set or update a permission mask for a patient-provider pair.
    /// Replaces any existing permissions with the new set.
    ///
    /// # Arguments
    /// * `patient` - The patient granting permissions
    /// * `provider` - The provider receiving permissions
    /// * `categories` - The list of permitted data categories
    ///
    /// # Panics
    /// * If categories list is empty
    pub fn set_permission(
        env: Env,
        patient: Address,
        provider: Address,
        categories: Vec<DataCategory>,
    ) -> PermissionMask {
        patient.require_auth();

        if categories.is_empty() {
            panic!("At least one category must be specified");
        }

        let now = env.ledger().timestamp();

        let mask = PermissionMask {
            patient: patient.clone(),
            provider: provider.clone(),
            categories: categories.clone(),
            granted_at: now,
            updated_at: now,
        };

        // Store using a composite key: patient + provider
        env.storage().persistent().set(
            &DataKey::Permission(patient.clone(), provider.clone()),
            &mask,
        );

        // Increment count if new
        let count: u64 = env
            .storage()
            .persistent()
            .get(&PERMISSION_COUNT)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&PERMISSION_COUNT, &(count + 1));

        mask
    }

    /// Get the permission mask for a patient-provider pair.
    ///
    /// # Arguments
    /// * `patient` - The patient's address
    /// * `provider` - The provider's address
    ///
    /// # Returns
    /// * `Option<PermissionMask>` - The permission mask if it exists
    pub fn get_permission(env: Env, patient: Address, provider: Address) -> Option<PermissionMask> {
        env.storage()
            .persistent()
            .get(&DataKey::Permission(patient.clone(), provider.clone()))
    }

    /// Check if a provider has permission for a specific data category.
    ///
    /// # Arguments
    /// * `patient` - The patient's address
    /// * `provider` - The provider's address
    /// * `category` - The data category to check
    ///
    /// # Returns
    /// * `bool` - Whether the provider has permission
    pub fn has_permission(
        env: Env,
        patient: Address,
        provider: Address,
        category: DataCategory,
    ) -> bool {
        let mask = env
            .storage()
            .persistent()
            .get::<_, PermissionMask>(&DataKey::Permission(patient.clone(), provider.clone()));

        match mask {
            Some(m) => m.categories.contains(&category),
            None => false,
        }
    }

    /// Remove a specific category from a permission mask.
    ///
    /// # Arguments
    /// * `patient` - The patient's address
    /// * `provider` - The provider's address
    /// * `category` - The category to remove
    ///
    /// # Panics
    /// * If no permission mask exists for the pair
    pub fn revoke_permission(
        env: Env,
        patient: Address,
        provider: Address,
        category: DataCategory,
    ) -> PermissionMask {
        patient.require_auth();

        let mut mask = env
            .storage()
            .persistent()
            .get::<_, PermissionMask>(&DataKey::Permission(patient.clone(), provider.clone()))
            .expect("No permission mask found for this patient-provider pair");

        // Remove the category
        let mut new_categories: Vec<DataCategory> = Vec::new(&env);
        for cat in mask.categories.iter() {
            if cat != category {
                new_categories.push_back(cat);
            }
        }

        mask.categories = new_categories;
        mask.updated_at = env.ledger().timestamp();

        env.storage().persistent().set(
            &DataKey::Permission(patient.clone(), provider.clone()),
            &mask,
        );

        mask
    }

    /// Remove all permissions for a patient-provider pair.
    ///
    /// # Arguments
    /// * `patient` - The patient's address
    /// * `provider` - The provider's address
    pub fn revoke_all(env: Env, patient: Address, provider: Address) {
        patient.require_auth();

        env.storage()
            .persistent()
            .remove(&DataKey::Permission(patient.clone(), provider.clone()));
    }

    /// Get the total number of permission masks stored.
    pub fn total_permissions(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&PERMISSION_COUNT)
            .unwrap_or(0)
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
    fn test_set_and_get_permission() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PermissionMaskContract);
        let client = PermissionMaskContractClient::new(&env, &contract_id);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        let mut categories = Vec::new(&env);
        categories.push_back(DataCategory::Allergies);
        categories.push_back(DataCategory::Medications);

        let mask = client.set_permission(&patient, &provider, &categories);

        assert_eq!(mask.patient, patient);
        assert_eq!(mask.provider, provider);
        assert_eq!(mask.categories.len(), 2);

        // Retrieve and verify
        let retrieved = client.get_permission(&patient, &provider);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().categories.len(), 2);
    }

    #[test]
    fn test_has_permission() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PermissionMaskContract);
        let client = PermissionMaskContractClient::new(&env, &contract_id);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        let mut categories = Vec::new(&env);
        categories.push_back(DataCategory::LabResults);
        categories.push_back(DataCategory::Radiology);

        client.set_permission(&patient, &provider, &categories);

        assert!(client.has_permission(&patient, &provider, &DataCategory::LabResults));
        assert!(client.has_permission(&patient, &provider, &DataCategory::Radiology));
        assert!(!client.has_permission(&patient, &provider, &DataCategory::Genetics));
    }

    #[test]
    fn test_revoke_permission() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PermissionMaskContract);
        let client = PermissionMaskContractClient::new(&env, &contract_id);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        let mut categories = Vec::new(&env);
        categories.push_back(DataCategory::Allergies);
        categories.push_back(DataCategory::Medications);
        categories.push_back(DataCategory::Vitals);

        client.set_permission(&patient, &provider, &categories);

        assert!(client.has_permission(&patient, &provider, &DataCategory::Vitals));

        client.revoke_permission(&patient, &provider, &DataCategory::Vitals);

        assert!(!client.has_permission(&patient, &provider, &DataCategory::Vitals));
        assert!(client.has_permission(&patient, &provider, &DataCategory::Allergies));
    }

    #[test]
    fn test_revoke_all() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PermissionMaskContract);
        let client = PermissionMaskContractClient::new(&env, &contract_id);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        let mut categories = Vec::new(&env);
        categories.push_back(DataCategory::Allergies);

        client.set_permission(&patient, &provider, &categories);
        assert!(client.get_permission(&patient, &provider).is_some());

        client.revoke_all(&patient, &provider);
        assert!(client.get_permission(&patient, &provider).is_none());
    }
}
