use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Represents the state of an Anda Registry Canister.
///
/// The Registry Canister is responsible for managing agent registrations,
/// maintaining a list of authorized challengers, and coordinating with
/// other registry canisters in a distributed network.
#[derive(Clone, CandidType, Default, Debug, Deserialize, Serialize)]
pub struct RegistryState {
    /// The name of this registry instance.
    pub name: String,

    /// Maximum number of agents that can be registered in this registry.
    pub max_agent: u64,

    /// Total number of agents currently registered in this registry.
    pub agents_total: u64,

    /// Duration in milliseconds after which a challenge expires.
    /// Agents must respond to challenges within this timeframe to maintain their active status.
    pub challenge_expires_in_ms: u64,

    /// Set of principal IDs of peer registry canisters in the network.
    /// These peers can synchronize agent information across the network.
    pub peers: BTreeSet<Principal>,

    /// Set of principal IDs authorized to issue challenges to agents.
    /// Only principals in this set can initiate the challenge process.
    pub challengers: BTreeSet<Principal>,

    /// Set of principal IDs subscribed to registry events.
    /// These subscribers receive notifications about agent registration changes.
    pub subscribers: BTreeSet<Principal>,

    /// Set of principal IDs of name service canisters associated with this registry.
    /// These canisters handle agent name resolution and discovery.
    pub name_canisters: BTreeSet<Principal>,

    /// Optional principal ID of the governance canister that controls this registry.
    /// If set, certain administrative operations require approval from this canister.
    pub governance_canister: Option<Principal>,
}

/// Represents errors that can occur during registry operations.
///
/// This enum provides specific error types with associated messages
/// to help diagnose and handle different failure scenarios.
#[derive(CandidType, Debug, Deserialize, Serialize, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum RegistryError {
    /// A generic error with a descriptive message.
    #[error("Generic error: {:?}", error)]
    Generic { error: String },

    /// Error when an agent with the specified handle cannot be found.
    #[error("Agent {:?} not found", handle)]
    NotFound { handle: String },

    /// Error when attempting to register an agent with a handle that already exists.
    #[error("Agent {:?} already exists", handle)]
    AlreadyExists { handle: String },

    /// Error when the request is malformed or contains invalid parameters.
    #[error("Bad request: {:?}", error)]
    BadRequest { error: String },

    /// Error when the caller is not authenticated or lacks valid credentials.
    #[error("Unauthorized: {:?}", error)]
    Unauthorized { error: String },

    /// Error when the caller is authenticated but lacks permission for the operation.
    #[error("No permission: {:?}", error)]
    Forbidden { error: String },

    /// Error when the requested operation is not supported by the registry.
    #[error("Not supported: {:?}", error)]
    NotSupported { error: String },
}

impl RegistryError {
    /// Returns the HTTP status code corresponding to this error type.
    ///
    /// This method maps registry errors to standard HTTP status codes,
    /// making it easier to integrate with HTTP-based APIs.
    ///
    /// # Returns
    /// - An HTTP status code as a u16 value
    pub fn status_code(&self) -> u16 {
        match self {
            RegistryError::BadRequest { .. } => 400,
            RegistryError::NotSupported { .. } => 400,
            RegistryError::Unauthorized { .. } => 401,
            RegistryError::Forbidden { .. } => 403,
            RegistryError::NotFound { .. } => 404,
            RegistryError::AlreadyExists { .. } => 409,
            RegistryError::Generic { .. } => 500,
        }
    }
}
