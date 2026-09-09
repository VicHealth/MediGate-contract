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
    EpochEvents(u64),
}

const RECENT_EVENTS: Symbol = symbol_short!("RECENT");
const EVENT_COUNT: Symbol = symbol_short!("EVT_CNT");

// Maximum number of recent events to store
const MAX_RECENT_EVENTS: u32 = 100;
// Maximum number of entity-specific events to retain in circular buffer
const MAX_ENTITY_EVENTS: u32 = 50;

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

        // Add to actor's event list with ring-buffer capping
        let mut actor_events: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::ActorEvents(actor.clone()))
            .unwrap_or(Vec::new(&env));
        actor_events.push_back(event_id.clone());
        while actor_events.len() > MAX_ENTITY_EVENTS {
            actor_events.remove(0);
        }
        env.storage()
            .persistent()
            .set(&DataKey::ActorEvents(actor.clone()), &actor_events);

        // Add to target's event list with ring-buffer capping
        let mut target_events: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::TargetEvents(target.clone()))
            .unwrap_or(Vec::new(&env));
        target_events.push_back(event_id.clone());
        while target_events.len() > MAX_ENTITY_EVENTS {
            target_events.remove(0);
        }
        env.storage()
            .persistent()
            .set(&DataKey::TargetEvents(target.clone()), &target_events);

        // Daily epoch partition key (86,400 seconds per day)
        let epoch_day = timestamp / 86400;
        let mut epoch_events: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::EpochEvents(epoch_day))
            .unwrap_or(Vec::new(&env));
        epoch_events.push_back(event_id.clone());
        while epoch_events.len() > MAX_RECENT_EVENTS {
            epoch_events.remove(0);
        }
        env.storage()
            .persistent()
            .set(&DataKey::EpochEvents(epoch_day), &epoch_events);

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

    /// Batch retrieve multiple audit events by their IDs.
    ///
    /// # Arguments
    /// * `event_ids` - The list of event IDs to fetch
    ///
    /// # Returns
    /// * `Vec<Option<AuditEvent>>` - Ordered list of results matching the requested IDs
    pub fn get_events_batch(env: Env, event_ids: Vec<String>) -> Vec<Option<AuditEvent>> {
        let mut results = Vec::new(&env);
        for id in event_ids.iter() {
            results.push_back(env.storage().persistent().get(&DataKey::AuditEvent(id)));
        }
        results
    }

    /// Get all event IDs for a specific actor (bounded by MAX_ENTITY_EVENTS ring buffer).
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

    /// Get all event IDs for a specific target (bounded by MAX_ENTITY_EVENTS ring buffer).
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

    /// Get event IDs logged within a specific daily epoch partition.
    ///
    /// # Arguments
    /// * `epoch_day` - Unix timestamp divided by 86,400 (day number)
    ///
    /// # Returns
    /// * `Vec<String>` - List of event IDs for that day
    pub fn get_epoch_events(env: Env, epoch_day: u64) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&DataKey::EpochEvents(epoch_day))
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
    use soroban_sdk::{
        testutils::{Address as _, Ledger as _},
        Env,
    };

    #[test]
    fn test_log_and_get_event() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, AuditLoggerContract);
        let client = AuditLoggerContractClient::new(&env, &contract_id);

        let actor = Address::generate(&env);
        let target = Address::generate(&env);
        let event_id = String::from_str(&env, "evt-001");
        let hash = String::from_str(&env, "abc123def456");

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
            &String::from_str(&env, "evt-a"),
            &EventType::KeyGranted,
            &actor,
            &target1,
            &String::from_str(&env, "hash1"),
        );
        client.log_event(
            &String::from_str(&env, "evt-b"),
            &EventType::DataViewed,
            &actor,
            &target2,
            &String::from_str(&env, "hash2"),
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
        let event_id = String::from_str(&env, "evt-verify");
        let hash = String::from_str(&env, "correct-hash");

        client.log_event(
            &event_id,
            &EventType::PatientRegistered,
            &actor,
            &target,
            &hash,
        );

        assert!(client.verify_event(&event_id, &String::from_str(&env, "correct-hash")));
        assert!(!client.verify_event(&event_id, &String::from_str(&env, "wrong-hash")));
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
            &String::from_str(&env, "e1"),
            &EventType::KeyGranted,
            &actor,
            &target,
            &String::from_str(&env, "h1"),
        );
        assert_eq!(client.total_events(), 1);

        client.log_event(
            &String::from_str(&env, "e2"),
            &EventType::KeyRevoked,
            &actor,
            &target,
            &String::from_str(&env, "h2"),
        );
        assert_eq!(client.total_events(), 2);
    }

    #[test]
    fn test_ring_buffer_pruning_and_epoch_partitioning() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1700000000); // Day: 1700000000 / 86400 = 19675
        let contract_id = env.register_contract(None, AuditLoggerContract);
        let client = AuditLoggerContractClient::new(&env, &contract_id);

        let actor = Address::generate(&env);
        let target = Address::generate(&env);

        // Log 55 events for the same actor (MAX_ENTITY_EVENTS = 50)
        for i in 0..55 {
            let mut buf = [0u8; 10];
            let id_str = match i {
                0..=9 => {
                    buf[0] = b'e';
                    buf[1] = b'0' + i as u8;
                    core::str::from_utf8(&buf[0..2]).unwrap()
                }
                _ => {
                    buf[0] = b'e';
                    buf[1] = b'0' + (i / 10) as u8;
                    buf[2] = b'0' + (i % 10) as u8;
                    core::str::from_utf8(&buf[0..3]).unwrap()
                }
            };
            let event_id = String::from_str(&env, id_str);
            client.log_event(
                &event_id,
                &EventType::DataViewed,
                &actor,
                &target,
                &String::from_str(&env, "hash"),
            );
        }

        // Bounded to 50 events
        let actor_events = client.get_actor_events(&actor);
        assert_eq!(actor_events.len(), 50);

        // Epoch events check
        let epoch_events = client.get_epoch_events(&(1700000000 / 86400));
        assert_eq!(epoch_events.len(), 55);

        // Batch retrieval check
        let mut query_batch = soroban_sdk::Vec::new(&env);
        query_batch.push_back(String::from_str(&env, "e50"));
        query_batch.push_back(String::from_str(&env, "non_existent"));
        let batch_res = client.get_events_batch(&query_batch);
        assert_eq!(batch_res.len(), 2);
        assert!(batch_res.get(0).unwrap().is_some());
        assert!(batch_res.get(1).unwrap().is_none());
    }
}
