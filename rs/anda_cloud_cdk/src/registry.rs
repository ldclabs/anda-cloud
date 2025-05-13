use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, CandidType, Default, Deserialize, Serialize)]
pub struct RegistryState {
    pub name: String,
    pub max_agent: u64,
    pub agents_total: u64,
    pub challenge_expires_in_ms: u64,
    pub peers: BTreeSet<Principal>,
    pub challengers: BTreeSet<Principal>,
    pub subscribers: BTreeSet<Principal>,
    pub name_canisters: BTreeSet<Principal>,
    pub governance_canister: Option<Principal>,
}

#[derive(CandidType, Debug, Deserialize, Serialize, thiserror::Error)]
#[non_exhaustive]
pub enum RegistryError {
    #[error("Generic error: {:?}", error)]
    Generic { error: String },

    #[error("Agent {:?} not found", handle)]
    NotFound { handle: String },

    #[error("Agent {:?} already exists", handle)]
    AlreadyExists { handle: String },

    #[error("Bad request: {:?}", error)]
    BadRequest { error: String },

    #[error("Unauthorized: {:?}", error)]
    Unauthorized { error: String },

    #[error("No permission: {:?}", error)]
    Forbidden { error: String },

    #[error("Not supported: {:?}", error)]
    NotSupported { error: String },
}

impl RegistryError {
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
