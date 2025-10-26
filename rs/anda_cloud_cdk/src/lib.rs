use sha3::{Digest, Sha3_256};

pub use ic_auth_verifier::envelope::SignedEnvelope;

/// Agent module containing structures and implementations for agent registration and verification.
pub mod agent;
/// Registry module containing structures and implementations for the Anda Registry Canister.
pub mod registry;

pub mod tee;
pub mod x402;

pub use agent::*;
pub use registry::*;
pub use tee::*;

/// Computes the SHA3-256 hash of the provided data.
///
/// SHA3-256 is a cryptographic hash function that produces a 256-bit (32-byte) hash value.
/// This function is commonly used for data integrity verification and digital signatures.
///
/// # Arguments
/// * `data` - A byte slice containing the data to be hashed
///
/// # Returns
/// A 32-byte array containing the SHA3-256 hash of the input data
pub fn sha3_256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().into()
}
