//! # Break-Glass Contract
//!
//! Emergency access protocol that allows authorized hospital wallets
//! to access critical patient data when the patient is incapacitated.
//! Requires guardian co-signing for approval.
//!
//! ## Key Functions
//! - `initiate_break_glass`: Start an emergency access request
//! - `approve_break_glass`: Approve a request (guardian action)
//! - `deny_break_glass`: Deny a request (guardian action)
//! - `get_request`: Get details of a specific request
//! - `get_patient_requests`: List all requests for a patient
//! - `get_pending_requests`: List all pending requests

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Vec};

// ============================================
// Data Types
// ============================================

/// Status of a Break-Glass request
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum RequestStatus {
    Pending,
    Approved,
    Denied,
    Expired,
}

/// A Break-Glass emergency request stored on-chain
#[derive(Clone, Debug)]
#[contracttype]
pub struct BreakGlassRequest {
    pub id: String,
    pub patient: Address,
    pub requester: Address,
    pub hospital: Address,
    pub reason: String,
    pub status: RequestStatus,
    pub requested_at: u64,
    pub resolved_at: u64,
    pub guardian_approvals: Vec<Address>,
    pub required_approvals: u32,
}

// ============================================
// Storage Keys
// ============================================

/// Compound storage keys for parameterized lookups
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    BreakGlassRequest(String),
    PatientRequests(Address),
}

const PENDING_LIST: Symbol = symbol_short!("PENDING");
const REQUEST_COUNT: Symbol = symbol_short!("BG_CNT");

// Maximum time a request can stay pending (1 hour)
const MAX_PENDING_DURATION: u64 = 3600;

// ============================================
// Contract
// ============================================

#[contract]
pub struct BreakGlassContract;

#[contractimpl]
impl BreakGlassContract {
    /// Initiate a Break-Glass emergency request.
    ///
    /// # Arguments
    /// * `request_id` - Unique identifier for this request
    /// * `patient` - The patient who needs emergency access
    /// * `requester` - The emergency responder requesting access
    /// * `hospital` - The hospital making the request
    /// * `reason` - Reason for the emergency access
    /// * `required_approvals` - Number of guardian approvals required (default: 2)
    ///
    /// # Panics
    /// * If reason is empty
    /// * If required_approvals is 0
    pub fn initiate_break_glass(
        env: Env,
        request_id: String,
        patient: Address,
        requester: Address,
        hospital: Address,
        reason: String,
        required_approvals: u32,
    ) -> BreakGlassRequest {
        requester.require_auth();

        if reason.is_empty() {
            panic!("Reason for Break-Glass access is required");
        }

        if required_approvals == 0 {
            panic!("At least one guardian approval is required");
        }

        let now = env.ledger().timestamp();

        let request = BreakGlassRequest {
            id: request_id.clone(),
            patient: patient.clone(),
            requester: requester.clone(),
            hospital: hospital.clone(),
            reason: reason.clone(),
            status: RequestStatus::Pending,
            requested_at: now,
            resolved_at: 0,
            guardian_approvals: Vec::new(&env),
            required_approvals,
        };

        // Store the request
        env.storage().persistent().set(&DataKey::BreakGlassRequest(request_id.clone()), &request);

        // Add to patient's request list
        let mut patient_reqs: Vec<String> = env.storage()
            .persistent()
            .get(&DataKey::PatientRequests(patient.clone()))
            .unwrap_or(Vec::new(&env));
        patient_reqs.push_back(request_id.clone());
        env.storage().persistent().set(&DataKey::PatientRequests(patient.clone()), &patient_reqs);

        // Add to pending list
        let mut pending: Vec<String> = env.storage()
            .persistent()
            .get(&PENDING_LIST)
            .unwrap_or(Vec::new(&env));
        pending.push_back(request_id.clone());
        env.storage().persistent().set(&PENDING_LIST, &pending);

        // Increment count
        let count: u64 = env.storage().persistent().get(&REQUEST_COUNT).unwrap_or(0);
        env.storage().persistent().set(&REQUEST_COUNT, &(count + 1));

        request
    }

    /// Approve a Break-Glass request (guardian action).
    ///
    /// # Arguments
    /// * `request_id` - The request to approve
    /// * `guardian` - The guardian approving the request
    ///
    /// # Panics
    /// * If request is not in Pending status
    /// * If guardian has already approved
    /// * If request has expired
    pub fn approve_break_glass(
        env: Env,
        request_id: String,
        guardian: Address,
    ) -> BreakGlassRequest {
        guardian.require_auth();

        let mut request = Self::get_request_internal(&env, &request_id)
            .expect("Break-Glass request not found");

        if request.status != RequestStatus::Pending {
            panic!("Request is not in Pending status");
        }

        // Check if request has expired
        let now = env.ledger().timestamp();
        if now > request.requested_at + MAX_PENDING_DURATION {
            request.status = RequestStatus::Expired;
            request.resolved_at = now;
            env.storage().persistent().set(&DataKey::BreakGlassRequest(request_id.clone()), &request);
            panic!("Break-Glass request has expired");
        }

        // Check if guardian already approved
        for existing in request.guardian_approvals.iter() {
            if existing == guardian {
                panic!("Guardian has already approved this request");
            }
        }

        request.guardian_approvals.push_back(guardian);

        // Check if we have enough approvals
        if (request.guardian_approvals.len() as u32) >= request.required_approvals {
            request.status = RequestStatus::Approved;
            request.resolved_at = now;
        }

        env.storage().persistent().set(&DataKey::BreakGlassRequest(request_id.clone()), &request);

        request
    }

    /// Deny a Break-Glass request (guardian action).
    ///
    /// # Arguments
    /// * `request_id` - The request to deny
    /// * `guardian` - The guardian denying the request
    ///
    /// # Panics
    /// * If request is not in Pending status
    pub fn deny_break_glass(
        env: Env,
        request_id: String,
        guardian: Address,
    ) -> BreakGlassRequest {
        guardian.require_auth();

        let mut request = Self::get_request_internal(&env, &request_id)
            .expect("Break-Glass request not found");

        if request.status != RequestStatus::Pending {
            panic!("Request is not in Pending status");
        }

        request.status = RequestStatus::Denied;
        request.resolved_at = env.ledger().timestamp();

        env.storage().persistent().set(&DataKey::BreakGlassRequest(request_id.clone()), &request);

        request
    }

    /// Get details of a specific Break-Glass request.
    ///
    /// # Arguments
    /// * `request_id` - The request to look up
    ///
    /// # Returns
    /// * `Option<BreakGlassRequest>` - The request if found
    pub fn get_request(env: Env, request_id: String) -> Option<BreakGlassRequest> {
        Self::get_request_internal(&env, &request_id)
    }

    /// Get all Break-Glass request IDs for a patient.
    ///
    /// # Arguments
    /// * `patient` - The patient's address
    ///
    /// # Returns
    /// * `Vec<String>` - List of request IDs
    pub fn get_patient_requests(env: Env, patient: Address) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&DataKey::PatientRequests(patient.clone()))
            .unwrap_or(Vec::new(&env))
    }

    /// Get all pending Break-Glass request IDs.
    ///
    /// # Returns
    /// * `Vec<String>` - List of pending request IDs
    pub fn get_pending_requests(env: Env) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&PENDING_LIST)
            .unwrap_or(Vec::new(&env))
    }

    /// Get the total number of Break-Glass requests.
    pub fn total_requests(env: Env) -> u64 {
        env.storage().persistent().get(&REQUEST_COUNT).unwrap_or(0)
    }

    // ============================================
    // Internal Helpers
    // ============================================

    fn get_request_internal(env: &Env, request_id: &String) -> Option<BreakGlassRequest> {
        env.storage().persistent().get(&DataKey::BreakGlassRequest(request_id.clone()))
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
    fn test_initiate_and_get_request() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BreakGlassContract);
        let client = BreakGlassContractClient::new(&env, &contract_id);

        let patient = Address::generate(&env);
        let requester = Address::generate(&env);
        let hospital = Address::generate(&env);
        let request_id = String::from_slice(&env, "bg-001");
        let reason = String::from_slice(&env, "Patient unconscious, needs allergy info");

        let request = client.initiate_break_glass(
            &request_id,
            &patient,
            &requester,
            &hospital,
            &reason,
            &2,
        );

        assert_eq!(request.status, RequestStatus::Pending);
        assert_eq!(request.patient, patient);
        assert_eq!(request.requester, requester);

        // Retrieve
        let retrieved = client.get_request(&request_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().status, RequestStatus::Pending);
    }

    #[test]
    fn test_approve_break_glass() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BreakGlassContract);
        let client = BreakGlassContractClient::new(&env, &contract_id);

        let patient = Address::generate(&env);
        let requester = Address::generate(&env);
        let hospital = Address::generate(&env);
        let guardian1 = Address::generate(&env);
        let guardian2 = Address::generate(&env);
        let request_id = String::from_slice(&env, "bg-002");
        let reason = String::from_slice(&env, "Emergency surgery, need blood type");

        client.initiate_break_glass(
            &request_id,
            &patient,
            &requester,
            &hospital,
            &reason,
            &2,
        );

        // First approval
        let request = client.approve_break_glass(&request_id, &guardian1);
        assert_eq!(request.status, RequestStatus::Pending);
        assert_eq!(request.guardian_approvals.len(), 1);

        // Second approval - should trigger approval
        let request = client.approve_break_glass(&request_id, &guardian2);
        assert_eq!(request.status, RequestStatus::Approved);
        assert_eq!(request.guardian_approvals.len(), 2);
    }

    #[test]
    fn test_deny_break_glass() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BreakGlassContract);
        let client = BreakGlassContractClient::new(&env, &contract_id);

        let patient = Address::generate(&env);
        let requester = Address::generate(&env);
        let hospital = Address::generate(&env);
        let guardian = Address::generate(&env);
        let request_id = String::from_slice(&env, "bg-003");
        let reason = String::from_slice(&env, "Patient unresponsive");

        client.initiate_break_glass(
            &request_id,
            &patient,
            &requester,
            &hospital,
            &reason,
            &2,
        );

        let request = client.deny_break_glass(&request_id, &guardian);
        assert_eq!(request.status, RequestStatus::Denied);
    }

    #[test]
    fn test_patient_requests() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BreakGlassContract);
        let client = BreakGlassContractClient::new(&env, &contract_id);

        let patient = Address::generate(&env);
        let requester = Address::generate(&env);
        let hospital = Address::generate(&env);

        client.initiate_break_glass(
            &String::from_slice(&env, "bg-a"),
            &patient,
            &requester,
            &hospital,
            &String::from_slice(&env, "Reason 1"),
            &2,
        );
        client.initiate_break_glass(
            &String::from_slice(&env, "bg-b"),
            &patient,
            &requester,
            &hospital,
            &String::from_slice(&env, "Reason 2"),
            &2,
        );

        let patient_reqs = client.get_patient_requests(&patient);
        assert_eq!(patient_reqs.len(), 2);
    }
}
