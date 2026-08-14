//! # Audit Logger Contract
//!
//! Records immutable audit events for all access-related operations.
//! Only metadata (event type, actor, target, timestamp) is stored on-chain —
//! never PHI/PII. This provides a verifiable, tamper-proof audit trail.
//!
//! ## Key Functions
//! - `log_event`: Record a new audit event
//! - `get_event`: Get details of a specific event
//! - `get_actor_events`: List all events for a specific actor
//! - `get_target_events`: List all events for a specific target
//! - `get_recent_events`: List the most recent events
//! - `total_events`: Get the total number of events logged

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Vec,
};

// ============================================
// Data Types
// ============================================

/// Types of audit events
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum EventType {
    KeyGranted,
    KeyRevoked,
    AccessGranted,
    AccessDenied,
    BreakGlassInitiated,
    BreakGlassResolved,
    DataViewed,
    DataUpdated,
    PatientRegistered,
    ProviderRegistered,
}

/// An audit event stored on-chain
#[derive(Clone, Debug)]
#[contracttype]
pub struct AuditEvent {
    pub id: String,
    pub event_type: EventType,
    pub actor: Address,
    pub target: Address,
    pub timestamp: u64,
    pub metadata_hash: String, // Hash of off-chain metadata (not the data itself)
}

// ============================================
// Storage Keys
// ============================================

/// Compound storage keys for parameterized lookups
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    AuditEvent(String),
    ActorEvents(Address),
    TargetEvents(Address),
}

const RECENT_EVENTS: Symbol = symbol_short!("RECENT");
const EVENT_COUNT: Symbol = symbol_short!("EVT_CNT");

// Maximum number of recent events to store
const MAX_RECENT_EVENTS: u32 = 100;

// ============================================
// Contract
// ============================================

#[contract]
pub struct AuditLoggerContract;

#[contractimpl]
impl AuditLoggerContract {
    /// Record a new audit event.
    ///
    /// # Arguments
    /// * `event_id` - Unique identifier for this event
    /// * `event_type` - The type of event
    /// * `actor` - The address that performed the action
    /// * `target` - The address that was acted upon
    /// * `metadata_hash` - Hash of the off-chain metadata (for verification)
    ///
    /// # Panics
    /// * If event_id is empty
    /// * If metadata_hash is empty
    pub fn log_event(
        env: Env,
        event_id: String,
        event_type: EventType,
        actor: Address,
        target: Address,
        metadata_hash: String,
    ) -> AuditEvent {
        if event_id.is_empty() {
            panic!("Event ID cannot be empty");
        }

        if metadata_hash.is_empty() {
            panic!("Metadata hash cannot be empty");
        }

        let timestamp = env.ledger().timestamp();

        let event = AuditEvent {
            id: event_id.clone(),
            event_type: event_type.clone(),
            actor: actor.clone(),
            target: target.clone(),
            timestamp,
            metadata_hash: metadata_hash.clone(),
        };

        // Store the event
        env.storage()
            .persistent()
            .set(&DataKey::AuditEvent(event_id.clone()), &event);

        // Add to actor's event list
        let mut actor_events: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::ActorEvents(actor.clone()))
            .unwrap_or(Vec::new(&env));
        actor_events.push_back(event_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::ActorEvents(actor.clone()), &actor_events);

        // Add to target's event list
        let mut target_events: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::TargetEvents(target.clone()))
            .unwrap_or(Vec::new(&env));
        target_events.push_back(event_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::TargetEvents(target.clone()), &target_events);

        // Add to recent events list
        let mut recent: Vec<String> = env
            .storage()
            .persistent()
            .get(&RECENT_EVENTS)
            .unwrap_or(Vec::new(&env));
        recent.push_back(event_id.clone());

        // Trim recent events to max size
        while recent.len() > MAX_RECENT_EVENTS {
            recent.remove(0);
        }
        env.storage().persistent().set(&RECENT_EVENTS, &recent);

        // Increment event count
        let count: u64 = env.storage().persistent().get(&EVENT_COUNT).unwrap_or(0);
        env.storage().persistent().set(&EVENT_COUNT, &(count + 1));

        event
    }

    /// Get details of a specific audit event.
    ///
    /// # Arguments
    /// * `event_id` - The event to look up
    ///
    /// # Returns
    /// * `Option<AuditEvent>` - The event if found
    pub fn get_event(env: Env, event_id: String) -> Option<AuditEvent> {
        env.storage()
            .persistent()
            .get(&DataKey::AuditEvent(event_id.clone()))
    }

    /// Get all event IDs for a specific actor.
    ///
    /// # Arguments
    /// * `actor` - The address that performed actions
    ///
    /// # Returns
    /// * `Vec<String>` - List of event IDs
    pub fn get_actor_events(env: Env, actor: Address) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&DataKey::ActorEvents(actor.clone()))
            .unwrap_or(Vec::new(&env))
    }

    /// Get all event IDs for a specific target.
    ///
    /// # Arguments
    /// * `target` - The address that was acted upon
    ///
    /// # Returns
    /// * `Vec<String>` - List of event IDs
    pub fn get_target_events(env: Env, target: Address) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&DataKey::TargetEvents(target.clone()))
            .unwrap_or(Vec::new(&env))
    }

    /// Get the most recent event IDs.
    ///
    /// # Returns
    /// * `Vec<String>` - List of recent event IDs
    pub fn get_recent_events(env: Env) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&RECENT_EVENTS)
            .unwrap_or(Vec::new(&env))
    }

    /// Get the total number of audit events logged.
    pub fn total_events(env: Env) -> u64 {
        env.storage().persistent().get(&EVENT_COUNT).unwrap_or(0)
    }

    /// Verify that an event exists and has the expected metadata hash.
    ///
    /// # Arguments
    /// * `event_id` - The event to verify
    /// * `expected_hash` - The expected metadata hash
    ///
    /// # Returns
    /// * `bool` - Whether the event exists and matches the hash
    pub fn verify_event(env: Env, event_id: String, expected_hash: String) -> bool {
        let event = env
            .storage()
            .persistent()
            .get::<_, AuditEvent>(&DataKey::AuditEvent(event_id.clone()));
        match event {
            Some(e) => e.metadata_hash == expected_hash,
            None => false,
        }
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
    fn test_log_and_get_event() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, AuditLoggerContract);
        let client = AuditLoggerContractClient::new(&env, &contract_id);

        let actor = Address::generate(&env);
        let target = Address::generate(&env);
        let event_id = String::from_slice(&env, "evt-001");
        let hash = String::from_slice(&env, "abc123def456");

        let event = client.log_event(&event_id, &EventType::AccessGranted, &actor, &target, &hash);

        assert_eq!(event.event_type, EventType::AccessGranted);
        assert_eq!(event.actor, actor);
        assert_eq!(event.target, target);
        assert_eq!(event.metadata_hash, hash);

        // Retrieve
        let retrieved = client.get_event(&event_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().event_type, EventType::AccessGranted);
    }

    #[test]
    fn test_actor_and_target_events() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, AuditLoggerContract);
        let client = AuditLoggerContractClient::new(&env, &contract_id);

        let actor = Address::generate(&env);
        let target1 = Address::generate(&env);
        let target2 = Address::generate(&env);

        client.log_event(
            &String::from_slice(&env, "evt-a"),
            &EventType::KeyGranted,
            &actor,
            &target1,
            &String::from_slice(&env, "hash1"),
        );
        client.log_event(
            &String::from_slice(&env, "evt-b"),
            &EventType::DataViewed,
            &actor,
            &target2,
            &String::from_slice(&env, "hash2"),
        );

        let actor_events = client.get_actor_events(&actor);
        assert_eq!(actor_events.len(), 2);

        let target1_events = client.get_target_events(&target1);
        assert_eq!(target1_events.len(), 1);
    }

    #[test]
    fn test_verify_event() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, AuditLoggerContract);
        let client = AuditLoggerContractClient::new(&env, &contract_id);

        let actor = Address::generate(&env);
        let target = Address::generate(&env);
        let event_id = String::from_slice(&env, "evt-verify");
        let hash = String::from_slice(&env, "correct-hash");

        client.log_event(
            &event_id,
            &EventType::PatientRegistered,
            &actor,
            &target,
            &hash,
        );

        assert!(client.verify_event(&event_id, &String::from_slice(&env, "correct-hash")));
        assert!(!client.verify_event(&event_id, &String::from_slice(&env, "wrong-hash")));
    }

    #[test]
    fn test_total_events() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, AuditLoggerContract);
        let client = AuditLoggerContractClient::new(&env, &contract_id);

        assert_eq!(client.total_events(), 0);

        let actor = Address::generate(&env);
        let target = Address::generate(&env);

        client.log_event(
            &String::from_slice(&env, "e1"),
            &EventType::KeyGranted,
            &actor,
            &target,
            &String::from_slice(&env, "h1"),
        );
        assert_eq!(client.total_events(), 1);

        client.log_event(
            &String::from_slice(&env, "e2"),
            &EventType::KeyRevoked,
            &actor,
            &target,
            &String::from_slice(&env, "h2"),
        );
        assert_eq!(client.total_events(), 2);
    }
}
