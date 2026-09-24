use candid::{CandidType, Principal};
use ic_auth_types::{ByteArrayB64, deterministic_cbor_into_vec};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{RegistryError, SignedEnvelope, TEEInfo, sha3_256};

/// An all-zero challenge code.
#[deprecated(
    note = "the registry does not require it: a first registration takes any challenge code, and challengers should pick a random one"
)]
pub const ZERO_CHALLENGE_CODE: ByteArrayB64<16> = ByteArrayB64([0u8; 16]);

/// Maximum allowed time drift in milliseconds for delegation verification.
/// This prevents replay attacks while allowing for reasonable clock differences.
pub const PERMITTED_DRIFT_MS: u64 = 60 * 1000;

/// How long a [`ChallengeRequest`] stays acceptable after its `created_at`,
/// in milliseconds, on top of [`PERMITTED_DRIFT_MS`].
///
/// Not to be confused with [`crate::RegistryState::challenge_expires_in_ms`],
/// the period within which an agent must be challenged again to stay active.
pub const CHALLENGE_EXPIRES_IN_MS: u64 = 1000 * 60 * 5; // 5 minutes

/// Maximum length in bytes of any URL in agent or TEE information.
pub const MAX_URL_BYTES: usize = 2048;

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
    /// Unique account identifier of the agent.
    pub handle: String,

    /// The dMsg.net canister where the agent profile is stored.
    pub handle_canister: Option<Principal>,

    /// Human readable name of the agent.
    /// (e.g. "Anda ICP")
    pub name: String,

    /// A URL to an image representing the agent, such as a logo or avatar.
    /// (e.g. "https://DOMAIN/path/to/image.png")
    pub image: String,

    /// A human-readable description of the agent. Used to assist users and
    /// other agents in understanding what the agent can do.
    /// (e.g. "Agent that helps users with recipes and cooking.")
    pub description: String,

    /// A endpoint URL for the agent. This is the URL that other agents and
    /// users will use to communicate with the agent.
    pub endpoint: String,

    /// Communication protocols the agent supports.
    pub protocols: Vec<AgentProtocol>,

    /// Information about the agent's service provider.
    pub provider: Option<AgentProvider>,
}

impl AgentInfo {
    /// Validates the agent information to ensure it meets system requirements.
    ///
    /// Checks for:
    /// - Handle format
    /// - Name presence and format
    /// - Description length
    /// - Image and endpoint URL validity
    /// - Protocol count and configuration validity
    /// - Provider validity
    ///
    /// # Returns
    /// - `Ok(())` if validation passes
    /// - `Err(String)` with a descriptive error message if validation fails
    pub fn validate(&self) -> Result<(), String> {
        validate_handle(&self.handle)?;

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

        if !self.image.is_empty() {
            check_https_url("image", &self.image)?;
        }

        check_https_url("endpoint", &self.endpoint)?;

        if self.protocols.len() > 16 {
            return Err("cannot declare more than 16 protocols".to_string());
        }

        let mut names = HashSet::with_capacity(self.protocols.len());
        for protocol in &self.protocols {
            protocol.validate()?;
            if !names.insert(protocol.name.as_str()) {
                return Err(format!("duplicate protocol name: {}", protocol.name));
            }
        }

        if let Some(provider) = &self.provider {
            provider.validate()?;
        }

        Ok(())
    }
}

/// Information about the agent's communication protocol.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct AgentProtocol {
    /// The name of the agent protocol. Should be uppercase.
    /// (e.g. "A2A", "MCP", "X402")
    pub name: String,
    /// A URI for the agent protocol's endpoint.
    /// (e.g. "https://DOMAIN/.well-known/agent.json")
    pub endpoint: String,
    /// The version of the agent protocol.
    /// (e.g. "v1")
    pub version: Option<String>,
}

impl AgentProtocol {
    /// Validates the agent protocol information to ensure it meets system requirements.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("protocol name is required".to_string());
        }

        if !self
            .name
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
        {
            return Err("protocol name must be uppercase letters, digits, '_' or '-'".to_string());
        }

        if self.name.len() > 12 {
            return Err("protocol name cannot be longer than 12 bytes".to_string());
        }

        if self.endpoint.is_empty() {
            return Err("protocol endpoint is required".to_string());
        }

        // HTTP endpoints must be https URLs; other URIs are accepted as they are.
        if self
            .endpoint
            .get(..4)
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("http"))
        {
            check_https_url("protocol endpoint", &self.endpoint)?;
        } else if self.endpoint.len() > MAX_URL_BYTES {
            return Err(format!(
                "protocol endpoint cannot be longer than {MAX_URL_BYTES} bytes"
            ));
        }

        if self.version.as_ref().is_some_and(|v| v.len() > 32) {
            return Err("protocol version cannot be longer than 32 bytes".to_string());
        }

        Ok(())
    }
}

/// Information about the agent's service provider.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct AgentProvider {
    /// The unique identifier of the agent provider.
    pub id: Principal,
    /// The name of the agent provider's organization.
    pub name: String,
    /// The agent provider's logo.
    pub logo: String,
    /// A URL for the agent provider's website or relevant documentation.
    pub url: String,
}

impl AgentProvider {
    /// Validates the agent provider information to ensure it meets system requirements.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("provider name is required".to_string());
        }

        if self.name.trim() != self.name {
            return Err("provider name cannot contain leading or trailing whitespace".to_string());
        }

        if self.name.len() > 32 {
            return Err("provider name cannot be longer than 32 bytes".to_string());
        }

        check_https_url("provider logo", &self.logo)?;
        check_https_url("provider url", &self.url)?;

        Ok(())
    }
}

/// Validates a agent handle to ensure it doesn't contain invalid characters
///
/// # Rules
/// - Must not be empty
/// - Must not exceed 64 characters
/// - Must start with a lowercase letter
/// - Can only contain: lowercase letters (a-z), digits (0-9), and underscores (_)
pub fn validate_handle(handle: &str) -> Result<(), String> {
    if handle.is_empty() {
        return Err("handle is required".into());
    }

    if handle.len() > 64 {
        return Err("handle cannot be longer than 64 bytes".into());
    }

    let mut iter = handle.chars();
    if !matches!(iter.next(), Some('a'..='z')) {
        return Err("handle must start with a lowercase letter".into());
    }

    for c in iter {
        if !matches!(c, 'a'..='z' | '0'..='9' | '_' ) {
            return Err(format!("handle contains invalid character: {c:?}"));
        }
    }
    Ok(())
}

/// Checks that `value` is a valid https URL of at most [`MAX_URL_BYTES`] bytes.
/// `field` names the value in the error message.
pub(crate) fn check_https_url(field: &str, value: &str) -> Result<(), String> {
    if value.len() > MAX_URL_BYTES {
        return Err(format!(
            "{field} cannot be longer than {MAX_URL_BYTES} bytes"
        ));
    }

    let url = url::Url::parse(value).map_err(|_| format!("{field} is not a valid URL"))?;
    if url.scheme() != "https" {
        return Err(format!("{field} should start with https://"));
    }
    Ok(())
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
    /// The digest includes the registry, code, agent information and creation time,
    /// but not the authentication, which is the challenger's signature over it.
    ///
    /// # Returns
    /// - A 32-byte array containing the SHA3-256 hash of the serialized data
    pub fn core_digest(&self) -> [u8; 32] {
        let data = deterministic_cbor_into_vec(&ChallengeRequestCoreRef {
            registry: &self.registry,
            code: &self.code,
            agent: &self.agent,
            created_at: self.created_at,
        })
        .expect("failed to serialize ChallengeRequestCoreRef");
        sha3_256(&data)
    }

    /// Computes a full digest including all components of the challenge request.
    ///
    /// The full digest includes the registry, code, agent information, and authentication.
    ///
    /// # Returns
    /// - A 32-byte array containing the SHA3-256 hash of the serialized data
    pub fn digest(&self) -> [u8; 32] {
        let data =
            deterministic_cbor_into_vec(&self).expect("failed to serialize ChallengeRequest");
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
    ///
    /// # Returns
    /// - The principal of the challenger that signed the request
    pub fn verify(&self, now_ms: u64, registry: Principal) -> Result<Principal, RegistryError> {
        self.validate(now_ms, &registry)
            .map_err(|error| RegistryError::BadRequest { error })?;

        let auth = self
            .authentication
            .as_ref()
            .ok_or_else(|| RegistryError::BadRequest {
                error: "challenger authentication is not provided".to_string(),
            })?;
        auth.verify(now_ms, Some(registry), Some(&self.core_digest()))
            .map_err(|error| RegistryError::Unauthorized { error })?;
        Ok(auth.sender())
    }
}

/// A complete envelope containing a challenge request and its authentication.
///
/// This structure is used for secure agent registration and verification,
/// combining the challenge data with authentication proof and optional TEE information.
///
/// The complete challenge process is as follows:
/// 1. The challenger obtains the agent's challenge code from the Registry Canister.
///    For first-time registration, the challenger picks a random 16-byte challenge code.
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
    ///
    /// # Returns
    /// - `(agent, challenger)`: the principals that signed the envelope and the request
    pub fn verify(
        &self,
        now_ms: u64,
        registry: Principal,
    ) -> Result<(Principal, Principal), RegistryError> {
        if let Some(tee) = &self.tee {
            tee.validate()
                .map_err(|error| RegistryError::BadRequest { error })?;
        }

        let challenger = self.request.verify(now_ms, registry)?;
        self.authentication
            .verify(now_ms, Some(registry), Some(&self.request.digest()))
            .map_err(|error| RegistryError::Unauthorized { error })?;

        Ok((self.authentication.sender(), challenger))
    }
}

pub const AGENT_EVENT_API: &str = "on_agent_event";

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

    #[test]
    fn agent_protocol_validate_rejects_lowercase_name() {
        let protocol = AgentProtocol {
            name: "mcp".into(),
            endpoint: "https://agent.example/protocol".into(),
            version: Some("v1".into()),
        };

        assert!(matches!(protocol.validate(), Err(message) if message.contains("uppercase")));
    }

    #[test]
    fn agent_info_validate_accepts_valid_configuration() {
        let info = sample_agent_info();

        assert!(info.validate().is_ok());
    }

    #[test]
    fn agent_info_validate_detects_duplicate_protocol_names() {
        let mut info = sample_agent_info();
        info.protocols.push(AgentProtocol {
            name: "MCP".into(),
            endpoint: "https://agent.example/dup".into(),
            version: Some("v1".into()),
        });

        assert!(
            matches!(info.validate(), Err(message) if message.contains("duplicate protocol name"))
        );
    }

    #[test]
    fn agent_provider_validate_rejects_insecure_logo_url() {
        let provider = AgentProvider {
            id: sample_principal(3),
            name: "Anda Labs".into(),
            logo: "http://example.com/logo.png".into(),
            url: "https://example.com".into(),
        };

        assert!(
            matches!(provider.validate(), Err(message) if message.contains("should start with https"))
        );
    }

    #[test]
    fn validate_handle_enforces_rules() {
        assert!(validate_handle("agent_1").is_ok());
        assert!(validate_handle("").is_err());
        assert!(validate_handle("1agent").is_err());
        assert!(validate_handle("agent-1").is_err());
    }

    #[test]
    fn agent_info_validate_bounds_urls_and_protocols() {
        let mut info = sample_agent_info();
        info.image = String::new();
        assert!(info.validate().is_ok());

        info.image = "http://agent.example/image.png".into();
        assert!(matches!(info.validate(), Err(message) if message.starts_with("image")));

        let mut info = sample_agent_info();
        info.endpoint = format!("https://agent.example/{}", "a".repeat(MAX_URL_BYTES));
        assert!(matches!(info.validate(), Err(message) if message.contains("cannot be longer")));

        let mut info = sample_agent_info();
        info.protocols = (0..17)
            .map(|i| AgentProtocol {
                name: format!("P{i}"),
                endpoint: "https://agent.example/protocol".into(),
                version: None,
            })
            .collect();
        assert!(matches!(info.validate(), Err(message) if message.contains("more than 16")));
    }

    #[test]
    fn agent_protocol_validate_checks_name_endpoint_and_version() {
        let protocol = |name: &str, endpoint: &str, version: Option<&str>| AgentProtocol {
            name: name.into(),
            endpoint: endpoint.into(),
            version: version.map(Into::into),
        };

        assert!(
            protocol("X402", "https://agent.example/x402", Some("v2"))
                .validate()
                .is_ok()
        );
        // Non-HTTP URIs are accepted as they are.
        assert!(protocol("ANDA", "icp:aaaaa-aa", None).validate().is_ok());

        assert!(
            protocol("M CP", "https://agent.example", None)
                .validate()
                .is_err()
        );
        assert!(
            protocol("MCP", "HTTP://agent.example", None)
                .validate()
                .is_err()
        );
        assert!(
            protocol("MCP", "https://agent.example", Some(&"v".repeat(33)))
                .validate()
                .is_err()
        );
    }

    #[test]
    fn challenge_request_validate_accepts_recent_requests() {
        let registry = sample_principal(10);
        let now_ms = 1_000_000;
        let request = sample_challenge_request(now_ms - 1_000, registry);

        assert!(request.validate(now_ms, &registry).is_ok());
    }

    #[test]
    fn challenge_request_validate_rejects_expired_requests() {
        let registry = sample_principal(11);
        let now_ms = 2_000_000;
        let created_at = now_ms - CHALLENGE_EXPIRES_IN_MS - PERMITTED_DRIFT_MS - 1;
        let request = sample_challenge_request(created_at, registry);

        assert!(
            matches!(request.validate(now_ms, &registry), Err(message) if message.contains("too old"))
        );
    }

    #[test]
    fn challenge_request_validate_rejects_future_requests() {
        let registry = sample_principal(12);
        let now_ms = 3_000_000;
        let created_at = now_ms + PERMITTED_DRIFT_MS + 1;
        let request = sample_challenge_request(created_at, registry);

        assert!(
            matches!(request.validate(now_ms, &registry), Err(message) if message.contains("in the future"))
        );
    }

    #[test]
    fn challenge_request_validate_rejects_wrong_registry() {
        let registry = sample_principal(13);
        let wrong_registry = sample_principal(14);
        let now_ms = 4_000_000;
        let request = sample_challenge_request(now_ms, wrong_registry);

        assert!(
            matches!(request.validate(now_ms, &registry), Err(message) if message.contains("different registry"))
        );
    }

    fn sample_principal(seed: u8) -> Principal {
        Principal::self_authenticating([seed; 32])
    }

    fn sample_agent_info() -> AgentInfo {
        AgentInfo {
            handle: "agent_one".into(),
            handle_canister: Some(sample_principal(1)),
            name: "Agent One".into(),
            image: "https://agent.example/image.png".into(),
            description: "Helpful agent".into(),
            endpoint: "https://agent.example/api".into(),
            protocols: vec![AgentProtocol {
                name: "MCP".into(),
                endpoint: "https://agent.example/protocol".into(),
                version: Some("v1".into()),
            }],
            provider: Some(AgentProvider {
                id: sample_principal(2),
                name: "Anda Labs".into(),
                logo: "https://example.com/logo.png".into(),
                url: "https://example.com".into(),
            }),
        }
    }

    fn sample_challenge_request(created_at: u64, registry: Principal) -> ChallengeRequest {
        ChallengeRequest {
            registry,
            code: ByteArrayB64([1u8; 16]),
            agent: sample_agent_info(),
            created_at,
            authentication: None,
        }
    }
}
