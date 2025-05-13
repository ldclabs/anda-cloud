use candid::{CandidType, Principal};
use ic_auth_types::ByteArrayB64;
use ic_auth_verifier::envelope::SignedEnvelope;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::{TEEInfo, sha3_256, to_cbor_bytes};

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Agent {
    pub id: Principal,

    pub info: AgentInfo,

    pub created_at: u64,

    pub updated_at: u64,

    pub actived_at: u64,

    pub health_power: u64,

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

    // A endpoint URL for the agent. This is the URL that other agents and
    // users will use to communicate with the agent.
    pub endpoint: String,

    // The protocols the agent supports. It is a map of protocol name to
    // agent infomation.
    // (e.g. "ANDA" => "https://DOMAIN/.well-known/agents/{agent_id}.json"，
    //       "A2A" => "https://DOMAIN/.well-known/agent.json")
    pub protocols: BTreeMap<AgentProtocol, String>,

    // Payment protocols the agent supports.
    // (e.g. ["X402"])
    pub payments: BTreeSet<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ChallengeReqeust {
    pub registry: Principal,

    pub code: ByteArrayB64<32>,

    pub agent: AgentInfo,

    pub authentication: Option<SignedEnvelope>,
}

impl ChallengeReqeust {
    pub fn digest(&self) -> [u8; 32] {
        let data = to_cbor_bytes(&(&self.registry, &self.code, &self.agent));
        sha3_256(&data)
    }

    pub fn full_digest(&self) -> [u8; 32] {
        let data = to_cbor_bytes(&(
            &self.registry,
            &self.code,
            &self.agent,
            &self.authentication,
        ));
        sha3_256(&data)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct AgentEnvelope {
    pub challenge: ChallengeReqeust,
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
    

    #[test]
    fn test_agent() {}
}
