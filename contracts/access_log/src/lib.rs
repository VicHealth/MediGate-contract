//! # AccessLog Contract (Minimal Viable Contract)
//!
//! A minimal Soroban smart contract that records who accessed a patient's
//! metadata (no PHI). This is the minimal viable contract for demonstrating
//! MediGate's on-chain audit capability on Stellar testnet.
//!
//! ## Key Functions
//! - `log_access`: Record a new access event (patient_id, provider_id, timestamp)
//! - `get_logs`: Retrieve all access logs for a given patient
//! - `get_recent_logs`: Get the most recent access logs
//! - `total_accesses`: Get the total number of access records
//!
//! ## Design Principle
//! No PHI/PII is ever stored on-chain — only patient and provider identifiers.
//! This contract is intentionally minimal to prove deployability on Stellar testnet.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, String, Symbol, Vec};

// ============================================
// Data Types
// ============================================

/// An access log entry stored on-chain
#[derive(Clone, Debug)]
#[contracttype]
pub struct AccessLogEntry {
    pub patient_id: String,
    pub provider_id: String,
    pub timestamp: u64,
}

// ============================================
// Storage Keys
// ============================================

/// Compound storage keys for parameterized lookups
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    LogEntry(u64),
    PatientLogs(String),
}

const RECENT_LOGS: Symbol = symbol_short!("RECENT");
const LOG_COUNT: Symbol = symbol_short!("LOG_CNT");

/// Maximum number of recent global logs to keep
const MAX_RECENT: u32 = 50;

// ============================================
// Contract
// ============================================

#[contract]
pub struct AccessLogContract;

#[contractimpl]
impl AccessLogContract {
    /// Record a new access event.
    ///
    /// # Arguments
    /// * `patient_id` - The patient's identifier (DID or Stellar address)
    /// * `provider_id` - The provider's identifier (DID or Stellar address)
    ///
    /// # Returns
    /// * `AccessLogEntry` - The recorded access log entry
    ///
    /// # Panics
    /// * If patient_id is empty
    /// * If provider_id is empty
    pub fn log_access(env: Env, patient_id: String, provider_id: String) -> AccessLogEntry {
        if patient_id.is_empty() {
            panic!("patient_id cannot be empty");
        }
        if provider_id.is_empty() {
            panic!("provider_id cannot be empty");
        }

        let timestamp = env.ledger().timestamp();

        let entry = AccessLogEntry {
            patient_id: patient_id.clone(),
            provider_id: provider_id.clone(),
            timestamp,
        };

        // Generate a unique log ID from counter
        let count: u64 = env.storage().persistent().get(&LOG_COUNT).unwrap_or(0);
        let log_id = count + 1;

        // Store the log entry by its numeric ID
        env.storage()
            .persistent()
            .set(&DataKey::LogEntry(log_id), &entry);

        // Add to patient's log list
        let mut patient_logs: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::PatientLogs(patient_id.clone()))
            .unwrap_or(Vec::new(&env));
        patient_logs.push_back(log_id);
        env.storage()
            .persistent()
            .set(&DataKey::PatientLogs(patient_id.clone()), &patient_logs);

        // Add to recent global logs
        let mut recent: Vec<u64> = env
            .storage()
            .persistent()
            .get(&RECENT_LOGS)
            .unwrap_or(Vec::new(&env));
        recent.push_back(log_id);
        if recent.len() > MAX_RECENT {
            recent.remove(0);
        }
        env.storage().persistent().set(&RECENT_LOGS, &recent);

        // Increment log count
        env.storage().persistent().set(&LOG_COUNT, &(count + 1));

        entry
    }

    /// Get all access logs for a specific patient.
    ///
    /// # Arguments
    /// * `patient_id` - The patient's identifier
    ///
    /// # Returns
    /// * `Vec<AccessLogEntry>` - List of all access log entries for the patient
    pub fn get_logs(env: Env, patient_id: String) -> Vec<AccessLogEntry> {
        let log_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::PatientLogs(patient_id.clone()))
            .unwrap_or(Vec::new(&env));

        let mut logs: Vec<AccessLogEntry> = Vec::new(&env);
        for id in log_ids.iter() {
            if let Some(entry) = env
                .storage()
                .persistent()
                .get::<_, AccessLogEntry>(&DataKey::LogEntry(id))
            {
                logs.push_back(entry);
            }
        }
        logs
    }

    /// Get the most recent access logs globally.
    ///
    /// # Returns
    /// * `Vec<AccessLogEntry>` - List of recent access log entries
    pub fn get_recent_logs(env: Env) -> Vec<AccessLogEntry> {
        let recent_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&RECENT_LOGS)
            .unwrap_or(Vec::new(&env));

        let mut logs: Vec<AccessLogEntry> = Vec::new(&env);
        for id in recent_ids.iter() {
            if let Some(entry) = env
                .storage()
                .persistent()
                .get::<_, AccessLogEntry>(&DataKey::LogEntry(id))
            {
                logs.push_back(entry);
            }
        }
        logs
    }

    /// Get the total number of access log entries.
    ///
    /// # Returns
    /// * `u64` - Total access count
    pub fn total_accesses(env: Env) -> u64 {
        env.storage().persistent().get(&LOG_COUNT).unwrap_or(0)
    }
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Ledger as _, Env};

    #[test]
    fn test_log_and_get_access() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_700_000_000);
        let contract_id = env.register_contract(None, AccessLogContract);
        let client = AccessLogContractClient::new(&env, &contract_id);

        let patient_id = String::from_slice(&env, "patient:stellar:GB1234");
        let provider_id = String::from_slice(&env, "provider:stellar:GA5678");

        let entry = client.log_access(&patient_id, &provider_id);

        assert_eq!(entry.patient_id, patient_id);
        assert_eq!(entry.provider_id, provider_id);
        assert!(entry.timestamp > 0);

        let logs = client.get_logs(&patient_id);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs.get(0).unwrap().provider_id, provider_id);
    }

    #[test]
    fn test_multiple_logs_for_patient() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, AccessLogContract);
        let client = AccessLogContractClient::new(&env, &contract_id);

        let patient = String::from_slice(&env, "patient:stellar:P001");
        let provider1 = String::from_slice(&env, "provider:stellar:PR001");
        let provider2 = String::from_slice(&env, "provider:stellar:PR002");

        client.log_access(&patient, &provider1);
        client.log_access(&patient, &provider2);

        let logs = client.get_logs(&patient);
        assert_eq!(logs.len(), 2);
        assert_eq!(logs.get(0).unwrap().provider_id, provider1);
        assert_eq!(logs.get(1).unwrap().provider_id, provider2);
    }

    #[test]
    fn test_get_empty_logs() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, AccessLogContract);
        let client = AccessLogContractClient::new(&env, &contract_id);

        let unknown = String::from_slice(&env, "patient:stellar:UNKNOWN");
        let logs = client.get_logs(&unknown);
        assert_eq!(logs.len(), 0);
    }

    #[test]
    fn test_total_accesses() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, AccessLogContract);
        let client = AccessLogContractClient::new(&env, &contract_id);

        assert_eq!(client.total_accesses(), 0);

        let p = String::from_slice(&env, "patient:P");
        let d = String::from_slice(&env, "provider:D");
        client.log_access(&p, &d);
        assert_eq!(client.total_accesses(), 1);
    }

    #[test]
    fn test_get_recent_logs() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, AccessLogContract);
        let client = AccessLogContractClient::new(&env, &contract_id);

        let p1 = String::from_slice(&env, "p1");
        let p2 = String::from_slice(&env, "p2");
        let d = String::from_slice(&env, "d");

        client.log_access(&p1, &d);
        client.log_access(&p2, &d);

        let recent = client.get_recent_logs();
        assert_eq!(recent.len(), 2);
    }

    #[test]
    #[should_panic(expected = "patient_id cannot be empty")]
    fn test_empty_patient_id() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, AccessLogContract);
        let client = AccessLogContractClient::new(&env, &contract_id);

        client.log_access(
            &String::from_slice(&env, ""),
            &String::from_slice(&env, "provider:test"),
        );
    }

    #[test]
    #[should_panic(expected = "provider_id cannot be empty")]
    fn test_empty_provider_id() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, AccessLogContract);
        let client = AccessLogContractClient::new(&env, &contract_id);

        client.log_access(
            &String::from_slice(&env, "patient:test"),
            &String::from_slice(&env, ""),
        );
    }
}
