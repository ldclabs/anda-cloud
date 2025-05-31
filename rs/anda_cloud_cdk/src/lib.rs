use candid::{CandidType, Principal};
use ciborium::into_writer;
use ic_auth_types::ByteBufB64;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

pub use ic_auth_verifier::envelope::SignedEnvelope;

/// Agent module containing structures and implementations for agent registration and verification.
pub mod agent;
/// Registry module containing structures and implementations for the Anda Registry Canister.
pub mod registry;

pub use agent::*;
pub use registry::*;

/// Serializes an object to CBOR (Concise Binary Object Representation) format.
///
/// CBOR is a binary data serialization format that is designed to be compact and efficient.
/// This function converts a Rust object to its CBOR representation as a byte vector.
///
/// # Arguments
/// * `obj` - A reference to an object that implements the Serialize trait
///
/// # Returns
/// A Vec<u8> containing the CBOR-encoded representation of the object
///
/// # Panics
/// Panics if the object cannot be encoded in CBOR format
pub fn to_cbor_bytes(obj: &impl Serialize) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    into_writer(obj, &mut buf).expect("failed to encode in CBOR format");
    buf
}

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

/// Represents information about a Trusted Execution Environment (TEE) where an agent is running.
///
/// TEEs provide hardware-based isolation and security guarantees for code execution.
/// This structure contains identification and attestation information for a TEE instance.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct TEEInfo {
    /// The principal identifier of the TEE instance.
    pub id: Principal,

    /// The type of TEE technology being used.
    pub kind: TEEKind,

    /// URL where additional TEE information can be retrieved.
    /// (e.g. https://DOMAIN/.well-known/tee)
    pub url: String,

    /// Optional attestation data that proves the TEE's authenticity.
    /// This typically contains cryptographic evidence that the TEE is genuine
    /// and running the expected code.
    pub attestation: Option<ByteBufB64>,
}

/// Enumerates the supported types of Trusted Execution Environments.
///
/// Different TEE technologies provide varying levels of security guarantees
/// and are supported by different hardware platforms.
#[derive(
    Clone, Debug, CandidType, Deserialize, Serialize, Eq, PartialEq, Hash, Ord, PartialOrd,
)]
pub enum TEEKind {
    /// AWS Nitro Enclaves, a TEE technology provided by Amazon Web Services.
    /// Nitro Enclaves provide isolated compute environments for sensitive workloads.
    NITRO,
}

impl TEEInfo {
    /// Validates the TEE information to ensure it meets system requirements.
    ///
    /// Checks that the URL is properly formatted and valid.
    ///
    /// # Returns
    /// - `Ok(())` if validation passes
    /// - `Err(String)` with a descriptive error message if validation fails
    pub fn validate(&self) -> Result<(), String> {
        if !self.url.starts_with("https://") {
            return Err("url should start with https://".to_string());
        }

        if url::Url::parse(&self.url).is_err() {
            return Err(format!("{:?} is not a valid URL", self.url));
        }

        if self.attestation.is_none() {
            return Err("attestation is required".to_string());
        }
        Ok(())
    }
}

/// Defines the supported payment protocols for agent services.
///
/// These protocols determine how payments can be processed
/// for services provided by agents.
#[derive(
    Clone, Debug, CandidType, Deserialize, Serialize, Eq, PartialEq, Hash, Ord, PartialOrd,
)]
pub enum PaymentProtocol {
    /// X402 payment protocol
    X402,
}
