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
// Bitwise Permission Masks
// ============================================

pub const MASK_LABS: u32 = 0b0000_0001; // 1
pub const MASK_VITALS: u32 = 0b0000_0010; // 2
pub const MASK_BILLING: u32 = 0b0000_0100; // 4
pub const MASK_CLINICAL: u32 = 0b0000_1000; // 8
pub const MASK_ALLERGIES: u32 = 0b0001_0000; // 16
pub const MASK_MEDICATIONS: u32 = 0b0010_0000; // 32
pub const MASK_RADIOLOGY: u32 = 0b0100_0000; // 64
pub const MASK_MENTAL_HEALTH: u32 = 0b1000_0000; // 128

// ============================================
// Storage Keys
// ============================================

/// Compound storage keys for parameterized lookups
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Permission(Address, Address),
    BitwisePermission(Address, Address),
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

    /// Set bitwise permission mask directly for a patient-provider pair.
    pub fn set_bitwise_mask(env: Env, patient: Address, provider: Address, mask: u32) -> u32 {
        patient.require_auth();
        env.storage().persistent().set(
            &DataKey::BitwisePermission(patient.clone(), provider.clone()),
            &mask,
        );
        mask
    }

    /// Get bitwise permission mask for a patient-provider pair.
    pub fn get_bitwise_mask(env: Env, patient: Address, provider: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::BitwisePermission(patient, provider))
            .unwrap_or(0)
    }

    /// Check if a provider has the specified bitwise permission mask for a patient.
    /// Returns true if all bits in `required_mask` are present in the provider's mask.
    pub fn has_bitwise_permission(
        env: Env,
        patient: Address,
        provider: Address,
        required_mask: u32,
    ) -> bool {
        let current_mask: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::BitwisePermission(patient, provider))
            .unwrap_or(0);
        (current_mask & required_mask) == required_mask
    }

    /// Add permission bits to an existing bitwise mask using bitwise OR.
    pub fn add_bitwise_permission(
        env: Env,
        patient: Address,
        provider: Address,
        mask_to_add: u32,
    ) -> u32 {
        patient.require_auth();
        let current_mask: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::BitwisePermission(
                patient.clone(),
                provider.clone(),
            ))
            .unwrap_or(0);
        let new_mask = current_mask | mask_to_add;
        env.storage()
            .persistent()
            .set(&DataKey::BitwisePermission(patient, provider), &new_mask);
        new_mask
    }

    /// Remove permission bits from an existing bitwise mask using bitwise AND NOT.
    pub fn remove_bitwise_permission(
        env: Env,
        patient: Address,
        provider: Address,
        mask_to_remove: u32,
    ) -> u32 {
        patient.require_auth();
        let current_mask: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::BitwisePermission(
                patient.clone(),
                provider.clone(),
            ))
            .unwrap_or(0);
        let new_mask = current_mask & !mask_to_remove;
        env.storage()
            .persistent()
            .set(&DataKey::BitwisePermission(patient, provider), &new_mask);
        new_mask
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
        env.storage()
            .persistent()
            .remove(&DataKey::BitwisePermission(patient, provider));
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

    #[test]
    fn test_bitwise_permission_operations() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PermissionMaskContract);
        let client = PermissionMaskContractClient::new(&env, &contract_id);

        let patient = Address::generate(&env);
        let provider = Address::generate(&env);

        // Initially zero mask
        assert_eq!(client.get_bitwise_mask(&patient, &provider), 0);
        assert!(!client.has_bitwise_permission(&patient, &provider, &MASK_LABS));

        // Set mask: Labs (1) + Vitals (2) = 3
        let initial_mask = MASK_LABS | MASK_VITALS;
        let mask = client.set_bitwise_mask(&patient, &provider, &initial_mask);
        assert_eq!(mask, 3);
        assert!(client.has_bitwise_permission(&patient, &provider, &MASK_LABS));
        assert!(client.has_bitwise_permission(&patient, &provider, &MASK_VITALS));
        assert!(client.has_bitwise_permission(&patient, &provider, &(MASK_LABS | MASK_VITALS)));
        assert!(!client.has_bitwise_permission(&patient, &provider, &MASK_CLINICAL));

        // Add Clinical notes (8)
        let mask_after_add = client.add_bitwise_permission(&patient, &provider, &MASK_CLINICAL);
        assert_eq!(mask_after_add, 11); // 1 + 2 + 8 = 11
        assert!(client.has_bitwise_permission(&patient, &provider, &MASK_CLINICAL));

        // Remove Labs (1)
        let mask_after_rem = client.remove_bitwise_permission(&patient, &provider, &MASK_LABS);
        assert_eq!(mask_after_rem, 10); // 2 + 8 = 10
        assert!(!client.has_bitwise_permission(&patient, &provider, &MASK_LABS));
        assert!(client.has_bitwise_permission(&patient, &provider, &MASK_VITALS));
        assert!(client.has_bitwise_permission(&patient, &provider, &MASK_CLINICAL));

        // Revoke all
        client.revoke_all(&patient, &provider);
        assert_eq!(client.get_bitwise_mask(&patient, &provider), 0);
    }
}
