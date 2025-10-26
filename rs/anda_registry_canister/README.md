# `anda_registry_canister`

A fully on-chain AI agents registry & discovery service on the Internet Computer, part of [Anda Cloud](https://github.com/ldclabs/anda-cloud).

## Overview

`anda_registry_canister` is an ICP smart contract that functions as an AI agents registry & discovery service in Anda Cloud. It enables the registration, discovery, and health monitoring of AI agents across multiple protocols.

## Features

- Support for multiple agent protocols including MCP (Model Context Protocol), A2A (Agent2Agent protocol), ANDA (Autonomous Networked Decentralized Agent protocol) and others in the future
- Support for X402 payment protocol and other payment protocols in the future
- Trusted Execution Environment (TEE) attestation verification support for agents running in TEE
- Global unique handle registration and discovery for agents, with name service provided by [dMsg.net](https://dMsg.net)
- Challenge-based health detection mechanism built on the [Internet Identity](https://internetcomputer.org/docs/references/ii-spec) protocol
- Support for both ICP Canister API and HTTP API, with HTTP API supporting both JSON and CBOR formats
- Fully deployed as a smart contract on the decentralized ICP blockchain, governed by ICPanda DAO

## Demo

Try it online: https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=lfcwh-piaaa-aaaap-an2fa-cai

## Quick Start

### Local Deployment

Deploy the canister:
```bash
# dfx canister create --specified-id lfcwh-piaaa-aaaap-an2fa-cai anda_registry_canister
dfx deploy anda_registry_canister --argument "(opt variant {Init =
  record {
    name = \"LDC Labs\";
    challenge_expires_in_ms = 3600000;
    governance_canister = null;
  }
})"
```

### Candid API

The canister exposes a comprehensive Candid API. Key endpoints include:

```did
# Agent Registration and Challenge
register : (ChallengeEnvelope) -> (Result_1)
challenge : (ChallengeEnvelope) -> (Result_1)

# Agent Discovery
get_agent : (principal) -> (Result_2) query
get_agent_by_handle : (text) -> (Result_2) query
list : (opt nat64, opt nat64) -> (Result_5) query
list_by_health_power : (opt nat64) -> (Result_6) query
last_challenged : (opt nat64) -> (Result_4) query

# Registry State
get_state : () -> (Result_3) query

# Administration

admin_add_challengers : (vec principal) -> (Result)
admin_add_name_canisters : (vec principal) -> (Result)
admin_add_peers : (vec principal) -> (Result)
admin_add_subscribers : (vec principal) -> (Result)
admin_remove_challengers : (vec principal) -> (Result)
admin_remove_name_canisters : (vec principal) -> (Result)
admin_remove_peers : (vec principal) -> (Result)
admin_remove_subscribers : (vec principal) -> (Result)
```

Full Candid API definition: [anda_registry_canister.did](https://github.com/ldclabs/anda-cloud/tree/main/rs/anda_registry_canister/anda_registry_canister.did)

### HTTP API

The canister supports HTTP requests for agent registration, challenge, and discovery:

#### Endpoints

- `POST /register`: Register a new agent
- `POST /challenge`: Challenge an existing agent
- `GET /lookup?id={principal}`: Get agent by principal ID
- `GET /lookup?handle={handle}`: Get agent by handle
- `GET /state`: Get registry state

#### Content Types

The HTTP API supports both JSON and CBOR formats. The content type is determined by the `Accept` and `Content-Type` headers:

- For JSON: `application/json`
- For CBOR: `application/cbor`

## Data Types

### Agent

The core data structure representing an AI agent:

```rs
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
```

### AgentInfo

Contains metadata about the agent:

```rs
/// Contains descriptive and operational information about an AI agent.
///
/// This structure holds the metadata and configuration details that define
/// an agent's capabilities, endpoints, and supported protocols.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
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
```

## License
Copyright © 2024-2025 [LDC Labs](https://github.com/ldclabs).

`ldclabs/anda-cloud` is licensed under the MIT License. See [LICENSE](../../LICENSE) for the full license text.