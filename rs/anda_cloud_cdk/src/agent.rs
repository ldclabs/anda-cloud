use candid::{CandidType, Principal};
use ic_auth_types::ByteArrayB64;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::{PaymentProtocol, RegistryError, SignedEnvelope, TEEInfo, sha3_256, to_cbor_bytes};

pub const ZERO_CHALLENGE_CODE: ByteArrayB64<16> = ByteArrayB64([0u8; 16]);

/// Maximum allowed time drift in milliseconds for delegation verification.
/// This prevents replay attacks while allowing for reasonable clock differences.
pub const PERMITTED_DRIFT_MS: u64 = 60 * 1000;

pub const CHALLENGE_EXPIRES_IN_MS: u64 = 1000 * 60 * 5; // 5 minute

/// Represents an AI agent registration information in the Anda network system.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Agent {
    /// The unique identifier of the agent.
    pub id: Principal,

    /// Information about the agent, including name, description, and supported protocols.
    pub info: AgentInfo,

    /// Timestamp when the agent was registered in milliseconds since the Unix epoch.
    pub created_at: u64,

    /// The most recent activation timestamp, updated when an agent successfully responds to a challenge
    /// after its previous challenge has expired.
    /// The agent's uptime can be calculated as: uptime = now_ms - actived_start
    pub actived_start: u64,

    /// Accumulated health value used to assess the agent's availability.
    /// This value increases with each successful challenge and decreases when challenges expire.
    ///
    /// Calculation example:
    /// 1. Assuming the challenge validity period is 1 hour (3,600,000 ms)
    /// 2. When an agent is successfully challenged, health_power increases by (now_ms - challenged_at)
    /// 3. When an agent hasn't been successfully challenged for over 1 hour (e.g., challenge after 1.5 hours),
    ///    health_power decreases by (now_ms - challenged_at), which would be 5,400,000
    ///    and actived_start is reset to now_ms
    pub health_power: u64,

    /// The challenge code for the next round, used to ensure the agent is in a healthy state.
    pub challenge_code: ByteArrayB64<16>,

    /// Timestamp when the agent was last challenged in milliseconds since the Unix epoch.
    pub challenged_at: u64,

    /// The principal that issued the challenge to this agent.
    pub challenged_by: Principal,

    /// Timestamp when the current challenge expires.
    pub challenged_expiration: u64,

    /// Optional Trusted Execution Environment information where the agent is running.
    pub tee: Option<TEEInfo>,
}

/// Contains descriptive and operational information about an AI agent.
///
/// This structure holds the metadata and configuration details that define
/// an agent's capabilities, endpoints, and supported protocols.
#[derive(Clone, Debug, Default, CandidType, Deserialize, Serialize)]
pub struct AgentInfo {
    /// Unique account identifier of the agent on dMsg.net.
    pub handle: Option<(Principal, String)>,

    /// Human readable name of the agent.
    /// (e.g. "Anda ICP")
    pub name: String,

    /// A human-readable description of the agent. Used to assist users and
    /// other agents in understanding what the agent can do.
    /// (e.g. "Agent that helps users with recipes and cooking.")
    pub description: String,

    /// A endpoint URL for the agent. This is the URL that other agents and
    /// users will use to communicate with the agent.
    pub endpoint: String,

    /// The protocols the agent supports. It is a map of protocol name to
    /// agent information.
    /// (e.g. "ANDA" => "https://DOMAIN/.well-known/agents/{agent_id}"，
    ///       "A2A" => "https://DOMAIN/.well-known/agent.json")
    pub protocols: BTreeMap<AgentProtocol, String>,

    /// Payment protocols the agent supports.
    /// (e.g. ["X402"])
    pub payments: BTreeSet<PaymentProtocol>,
}

impl AgentInfo {
    /// Validates the agent information to ensure it meets system requirements.
    ///
    /// Checks for:
    /// - Name presence and format
    /// - Description length
    /// - Endpoint URL validity
    /// - Protocol configuration validity
    ///
    /// # Returns
    /// - `Ok(())` if validation passes
    /// - `Err(String)` with a descriptive error message if validation fails
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("name is required".to_string());
        }

        if self.name.trim() != self.name {
            return Err("name cannot contain leading or trailing whitespace".to_string());
        }

        if self.name.len() > 32 {
            return Err("name cannot be longer than 32 bytes".to_string());
        }

        if self.description.len() > 512 {
            return Err("description cannot be longer than 512 bytes".to_string());
        }

        if !self.endpoint.starts_with("https://") {
            return Err("endpoint should start with https://".to_string());
        }

        if url::Url::parse(&self.endpoint).is_err() {
            return Err("endpoint is not a valid URL".to_string());
        }

        if self.protocols.is_empty() {
            return Err("protocols is required".to_string());
        }

        for (protocol, url) in &self.protocols {
            if !url.starts_with("https://") {
                return Err(format!("protocol {protocol:?} should start with https://",));
            }

            if url::Url::parse(url).is_err() {
                return Err(format!("protocol {protocol:?} is not a valid URL"));
            }
        }
        Ok(())
    }
}

/// Represents a challenge request initiated by a challenger to an agent.
///
/// This structure contains all the necessary information for a challenger
/// to verify an agent's health and update its information in the registry.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ChallengeRequest {
    /// The registry canister where the agent is registered.
    pub registry: Principal,

    /// The challenge code that must be responded to.
    pub code: ByteArrayB64<16>,

    /// Latest information about the agent being challenged.
    /// When the challenge is successful, this information will be updated in the Anda Registry Canister,
    /// effectively synchronizing the agent's information through the challenge process.
    pub agent: AgentInfo,

    /// Creation timestamp of the challenge request in milliseconds since the Unix epoch.
    pub created_at: u64,

    /// Authentication signature information from the challenger.
    /// The challenger must be registered in the Anda Registry Canister, otherwise the challenge will fail.
    pub authentication: Option<SignedEnvelope>,
}

#[derive(Debug, Serialize)]
struct ChallengeRequestCoreRef<'a> {
    registry: &'a Principal,
    code: &'a ByteArrayB64<16>,
    agent: &'a AgentInfo,
    created_at: u64,
}

impl ChallengeRequest {
    /// Computes a digest (hash) of the challenge request's core components.
    ///
    /// The digest includes the registry, code, and agent information.
    ///
    /// # Returns
    /// - A 32-byte array containing the SHA3-256 hash of the serialized data
    pub fn core_digest(&self) -> [u8; 32] {
        let data = to_cbor_bytes(&ChallengeRequestCoreRef {
            registry: &self.registry,
            code: &self.code,
            agent: &self.agent,
            created_at: self.created_at,
        });
        sha3_256(&data)
    }

    /// Computes a full digest including all components of the challenge request.
    ///
    /// The full digest includes the registry, code, agent information, and authentication.
    ///
    /// # Returns
    /// - A 32-byte array containing the SHA3-256 hash of the serialized data
    pub fn digest(&self) -> [u8; 32] {
        let data = to_cbor_bytes(&self);
        sha3_256(&data)
    }

    /// Validates the challenge request by ensuring the agent information is valid.
    ///
    /// # Returns
    /// - `Ok(())` if validation passes
    /// - `Err(String)` with a descriptive error message if validation fails
    pub fn validate(&self, now_ms: u64, registry: &Principal) -> Result<(), String> {
        self.agent.validate()?;
        if self.created_at + CHALLENGE_EXPIRES_IN_MS + PERMITTED_DRIFT_MS < now_ms {
            return Err(format!(
                "challenge request is too old, created_at: {}, now: {}",
                self.created_at, now_ms
            ));
        }
        if self.created_at > now_ms + PERMITTED_DRIFT_MS {
            return Err(format!(
                "challenge request is in the future, created_at: {}, now: {}",
                self.created_at, now_ms
            ));
        }
        if self.registry != *registry {
            return Err(format!(
                "challenge request is for a different registry, expected: {}, got: {}",
                registry, self.registry
            ));
        }
        Ok(())
    }

    /// Verifies the challenge request by validating its components and authentication.
    pub fn verify(&self, now_ms: u64, registry: Principal) -> Result<(), RegistryError> {
        self.validate(now_ms, &registry)
            .map_err(|error| RegistryError::BadRequest { error })?;

        let digest = self.core_digest();
        if let Some(auth) = &self.authentication {
            auth.verify(now_ms, Some(registry), Some(&digest))
                .map_err(|error| RegistryError::Unauthorized { error })?;
        } else {
            return Err(RegistryError::BadRequest {
                error: "challenger authentication is not provided".to_string(),
            });
        }
        Ok(())
    }
}

/// A complete envelope containing a challenge request and its authentication.
///
/// This structure is used for secure agent registration and verification,
/// combining the challenge data with authentication proof and optional TEE information.
///
/// The complete challenge process is as follows:
/// 1. The challenger obtains the agent's challenge code from the Registry Canister.
///    For first-time registration, the challenge code is [0u8; 32].
/// 2. The challenger obtains the agent's latest information through its protocol
///    and generates a ChallengeRequest.
/// 3. The challenger signs the ChallengeRequest with their ICP identity and sends it to the agent.
/// 4. Upon receiving the challenger's request, the agent confirms the information
///    and signs it with its own ICP identity.
/// 5. If the agent is running in a TEE environment, it also generates TEE information,
///    including an attestation containing this challenge information.
/// 6. The challenger sends the ChallengeEnvelope returned by the agent to the Registry Canister.
/// 7. The Registry Canister verifies the ChallengeEnvelope and updates the agent's status.
/// 8. Since the challenge code is updated after each successful challenge, only the first
///    challenger for a given challenge code can succeed. Requests from other challengers
///    for the same challenge code will be invalid.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ChallengeEnvelope {
    /// The challenge request initiated by the challenger.
    pub request: ChallengeRequest,

    /// The agent's signature on the challenge request.
    pub authentication: SignedEnvelope,

    /// TEE information if the agent is running in a Trusted Execution Environment.
    pub tee: Option<TEEInfo>,
}

impl ChallengeEnvelope {
    /// Verifies the challenge envelope by validating its components and authentication.
    /// The tee attestation is not verified if present. It should be verified separately.
    pub fn verify(&self, now_ms: u64, registry: Principal) -> Result<(), RegistryError> {
        if let Some(tee) = &self.tee {
            tee.validate()
                .map_err(|error| RegistryError::BadRequest { error })?;
        }

        self.request.verify(now_ms, registry)?;

        let digest = self.request.digest();
        self.authentication
            .verify(now_ms, Some(registry), Some(&digest))
            .map_err(|error| RegistryError::Unauthorized { error })?;

        Ok(())
    }
}

/// Defines the supported agent communication protocols.
///
/// These protocols determine how agents can interact with each other
/// and with the Anda Cloud system.
#[derive(
    Clone, Debug, CandidType, Deserialize, Serialize, Eq, PartialEq, Hash, Ord, PartialOrd,
)]
pub enum AgentProtocol {
    /// Autonomous Networked Decentralized Agent protocol, https://github.com/ldclabs/anda
    ANDA,
    /// Agent2Agent protocol, https://github.com/google/A2A
    A2A,
    /// Model Context Protocol, https://github.com/modelcontextprotocol
    MCP,
}

pub static AGENT_EVENT_API: &str = "on_agent_event";

/// Represents an event related to an agent's registration or status change.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct AgentEvent {
    /// The principal ID of the agent.
    pub id: Principal,

    /// The event type.
    pub kind: AgentEventKind,

    /// The timestamp when the event occurred in milliseconds since the Unix epoch.
    pub ts: u64,
}

/// Enumerates the types of events that can occur for an agent.
#[derive(
    Clone, Debug, CandidType, Deserialize, Serialize, Eq, PartialEq, Hash, Ord, PartialOrd,
)]
pub enum AgentEventKind {
    Registered,
    Challenged,
    Unregistered,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ciborium::Value;

    #[test]
    fn test_agent_protocol() {
        let protocols = vec![AgentProtocol::ANDA, AgentProtocol::A2A, AgentProtocol::MCP];
        let got = serde_json::to_string(&protocols).unwrap();
        let expected = r#"["ANDA","A2A","MCP"]"#;
        assert_eq!(got, expected);

        let got: Vec<AgentProtocol> = serde_json::from_str(&got).unwrap();
        assert_eq!(got, protocols);

        let got = Value::serialized(&protocols).unwrap();
        let expected = Value::Array(vec![
            Value::Text("ANDA".to_string()),
            Value::Text("A2A".to_string()),
            Value::Text("MCP".to_string()),
        ]);
        assert_eq!(got, expected);
        let got: Vec<AgentProtocol> = Value::deserialized(&expected).unwrap();
        assert_eq!(got, protocols);
    }
}
