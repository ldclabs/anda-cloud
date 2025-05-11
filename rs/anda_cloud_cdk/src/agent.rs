use candid::{CandidType, Principal};
use ic_auth_types::{ByteArrayB64, ByteBufB64, SignedDelegation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{TEEInfo, sha3_256, to_cbor_bytes};

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Agent {
    pub id: Principal,

    pub info: AgentInfo,

    pub created_at: u64,

    pub updated_at: u64,

    pub actived_at: u64,

    pub health_power: u64,

    pub stake_power: u64,

    pub challenge_code: ByteArrayB64<32>,

    pub challenged_at: u64,

    pub challenged_by: Principal,

    pub challenged_expiration: u64,

    pub tee: Option<TEEInfo>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct AgentInfo {
    // Unique account identifier of the agent on dMsg.net.
    pub handle: Option<String>,

    // Human readable name of the agent.
    // (e.g. "Anda ICP")
    pub name: String,

    // A human-readable description of the agent. Used to assist users and
    // other agents in understanding what the agent can do.
    // (e.g. "Agent that helps users with recipes and cooking.")
    pub description: String,

    // A URL to the address the agent is hosted at.
    pub url: String,

    // The protocols the agent supports. It is a map of protocol name to
    // agent infomation.
    // (e.g. "ANDA" => "https://DOMAIN/.well-known/agents/{agent_id}.json"，
    //       "A2A" => "https://DOMAIN/.well-known/agent.json")
    pub protocols: BTreeMap<AgentProtocol, String>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ChallengeReqeust {
    pub code: ByteArrayB64<32>,

    pub registry: Principal,

    pub info: AgentInfo,

    pub authentication: Option<SignedEnvelope>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct SignedEnvelope {
    /// The public key of the self-authenticating principal this request is from.
    /// This is the head of the delegation chain (if any) and is used to derive
    /// the principal ID of the sender.
    pub pubkey: ByteBufB64,

    /// A cryptographic signature authorizing the request.
    /// When delegations are involved, this is the signature from the tail of the
    /// delegation chain, not necessarily made by the owner of `pubkey`.
    pub signature: ByteBufB64,

    /// The chain of delegations connecting `pubkey` to `signature`, in order.
    /// Each delegation authorizes the next entity in the chain to sign on behalf
    /// of the previous entity, forming a chain of trust from the original identity
    /// to the actual signer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation: Option<Vec<SignedDelegation>>,
}

impl ChallengeReqeust {
    pub fn digest(&self) -> [u8; 32] {
        let data = to_cbor_bytes(&(&self.code, &self.registry, &self.info, &self.authentication));
        sha3_256(&data)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ChallengeResponse {
    pub request: ChallengeReqeust,
    pub authentication: SignedEnvelope,
    pub tee: Option<TEEInfo>,
}

#[derive(
    Clone, Debug, CandidType, Deserialize, Serialize, Eq, PartialEq, Hash, Ord, PartialOrd,
)]
#[serde(untagged)]
pub enum AgentProtocol {
    ANDA,
    A2A,
    MCP,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent() {}
}
