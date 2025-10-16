use candid::{CandidType, Principal};
use core::fmt::Display;
use ic_auth_types::ByteBufB64;
use serde::{Deserialize, Serialize};

pub use ic_auth_verifier::envelope::SignedEnvelope;

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

impl Display for TEEKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TEEKind::NITRO => write!(f, "NITRO"),
        }
    }
}

impl TryFrom<&str> for TEEKind {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_uppercase().as_str() {
            "NITRO" => Ok(TEEKind::NITRO),
            _ => Err(format!("Unknown TEE kind: {}", s)),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use ciborium::Value;

    #[test]
    fn test_tee_protocol() {
        let val = TEEKind::try_from("NITRO").unwrap();
        assert_eq!(val, TEEKind::NITRO);
        let val = TEEKind::try_from("nitro").unwrap();
        assert_eq!(val, TEEKind::NITRO);
        assert_eq!(val.to_string(), "NITRO");

        let got = serde_json::to_string(&val).unwrap();
        assert_eq!(got, "\"NITRO\"");

        let got: TEEKind = serde_json::from_str(&got).unwrap();
        assert_eq!(got, val);

        let got = Value::serialized(&val).unwrap();
        let expected = Value::Text("NITRO".to_string());
        assert_eq!(got, expected);
        let got: TEEKind = Value::deserialized(&expected).unwrap();
        assert_eq!(got, val);
    }
}
