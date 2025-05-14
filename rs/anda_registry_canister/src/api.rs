use anda_cloud_cdk::{
    agent::{Agent, ChallengeEnvelope, ZERO_CHALLENGE_CODE},
    registry::{RegistryError, RegistryState},
};
use candid::Principal;
use std::collections::BTreeMap;

use crate::{MILLISECONDS, store};

#[ic_cdk::query]
fn get_state() -> Result<RegistryState, RegistryError> {
    Ok(store::state::get_state())
}

#[ic_cdk::update]
pub async fn register(input: ChallengeEnvelope) -> Result<(), RegistryError> {
    input
        .validate()
        .map_err(|error| RegistryError::BadRequest { error })?;
    let now_ms = ic_cdk::api::time() / MILLISECONDS;
    let agent_id = input.authentication.sender();
    if input.request.code != ZERO_CHALLENGE_CODE {
        return Err(RegistryError::BadRequest {
            error: "challenge code is not empty".to_string(),
        });
    }

    let canister_self = ic_cdk::api::canister_self();
    if input.request.registry != canister_self {
        return Err(RegistryError::BadRequest {
            error: format!(
                "challenge registry is not this canister, expect {}, got {}",
                canister_self, input.request.registry
            ),
        });
    }
    let digest = input.request.core_digest();
    let full_digest = input.request.digest();
    let challenger_auth =
        input
            .request
            .authentication
            .ok_or_else(|| RegistryError::BadRequest {
                error: "challenger authentication is not provided".to_string(),
            })?;

    let challenger = challenger_auth.sender();
    if store::state::is_challenger(&challenger) {
        return Err(RegistryError::Forbidden {
            error: format!("challenger {} has no permission", challenger),
        });
    }

    challenger_auth
        .verify(now_ms, Some(canister_self), Some(&digest))
        .map_err(|error| RegistryError::Unauthorized { error })?;
    input
        .authentication
        .verify(now_ms, Some(canister_self), Some(&full_digest))
        .map_err(|error| RegistryError::Unauthorized { error })?;
    if let Some(_handle) = &input.request.agent.handle {
        // TODO: check handle
    }
    store::agent::register(agent_id, challenger, input.request.agent, input.tee, now_ms).await
}

#[ic_cdk::update]
pub async fn challenge(input: ChallengeEnvelope) -> Result<(), RegistryError> {
    input
        .validate()
        .map_err(|error| RegistryError::BadRequest { error })?;

    let now_ms = ic_cdk::api::time() / MILLISECONDS;

    let agent_id = input.authentication.sender();
    let canister_self = ic_cdk::api::canister_self();
    if input.request.registry != canister_self {
        return Err(RegistryError::BadRequest {
            error: format!(
                "challenge registry is not this canister, expect {}, got {}",
                canister_self, input.request.registry
            ),
        });
    }
    let digest = input.request.core_digest();
    let full_digest = input.request.digest();
    let challenger_auth =
        input
            .request
            .authentication
            .ok_or_else(|| RegistryError::BadRequest {
                error: "challenger authentication is not provided".to_string(),
            })?;

    let challenger = challenger_auth.sender();
    if store::state::is_challenger(&challenger) {
        return Err(RegistryError::Forbidden {
            error: format!("challenger {} has no permission", challenger),
        });
    }

    challenger_auth
        .verify(now_ms, Some(canister_self), Some(&digest))
        .map_err(|error| RegistryError::Unauthorized { error })?;
    input
        .authentication
        .verify(now_ms, Some(canister_self), Some(&full_digest))
        .map_err(|error| RegistryError::Unauthorized { error })?;
    if let Some(_handle) = &input.request.agent.handle {
        // TODO: check handle
    }

    store::agent::challenge(
        agent_id,
        challenger,
        input.request.code,
        input.request.agent,
        input.tee,
        now_ms,
    )
    .await
}

#[ic_cdk::query]
fn get_agent(id: Principal) -> Result<Agent, RegistryError> {
    store::agent::get_agent(id)
}

#[ic_cdk::query]
fn get_agent_by_handle(handle: String) -> Result<Agent, RegistryError> {
    store::agent::get_agent_by_handle(handle)
}

#[ic_cdk::query]
fn list(prev: Option<u64>, take: Option<u64>) -> Result<(u64, Vec<Agent>), RegistryError> {
    let take = take.unwrap_or(10).min(1000);
    store::agent::list(prev, take as usize)
}

#[ic_cdk::query]
fn list_by_health_power(take: Option<u64>) -> Result<Vec<Agent>, RegistryError> {
    let take = take.unwrap_or(10).min(1000);
    store::agent::list_by_health_power(take as usize)
}

#[ic_cdk::query]
fn last_challenged(take: Option<u64>) -> Result<BTreeMap<Principal, u64>, RegistryError> {
    let take = take.unwrap_or(100).min(10000);
    store::agent::last_challenged(take as usize)
}
