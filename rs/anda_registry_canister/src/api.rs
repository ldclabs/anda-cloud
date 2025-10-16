use anda_cloud_cdk::{
    agent::{Agent, AgentEvent, AgentEventKind, ChallengeEnvelope},
    registry::{RegistryError, RegistryState},
};
use candid::Principal;
use ic_tee_nitro_attestation::parse_and_verify;
use std::collections::BTreeMap;

use crate::{MILLISECONDS, rand_bytes, store};

#[ic_cdk::query]
fn get_state() -> Result<RegistryState, RegistryError> {
    Ok(store::state::get_state())
}

#[ic_cdk::update]
pub async fn register(input: ChallengeEnvelope) -> Result<(), RegistryError> {
    let now_ms = ic_cdk::api::time() / MILLISECONDS;
    let canister_self = ic_cdk::api::canister_self();
    input.verify(now_ms, canister_self)?;

    let agent = input.authentication.sender();
    let challenger = input.request.authentication.unwrap().sender();
    if !store::state::is_challenger(&challenger) {
        return Err(RegistryError::Forbidden {
            error: format!("challenger {} has no permission", challenger),
        });
    }

    if let Some(tee) = &input.tee {
        let attestation = parse_and_verify(tee.attestation.as_ref().ok_or_else(|| {
            RegistryError::BadRequest {
                error: "attestation is not provided".to_string(),
            }
        })?)
        .map_err(|error| RegistryError::BadRequest {
            error: format!("attestation is not valid: {}", error),
        })?;

        if attestation.public_key.as_ref().map(|v| v.as_slice())
            != Some(input.authentication.pubkey.as_slice())
        {
            return Err(RegistryError::BadRequest {
                error: "attestation public key is not equal to agent public key".to_string(),
            });
        }
        if attestation.nonce.as_ref().map(|v| v.as_slice()) != Some(input.request.code.as_slice()) {
            return Err(RegistryError::BadRequest {
                error: "attestation nonce is not equal to chanllenge code".to_string(),
            });
        }
    }

    if let Some(canister) = &input.request.agent.handle_canister {
        store::state::check_handle(*canister, input.request.agent.handle.clone(), agent).await?;
    }

    let code = rand_bytes::<16>()
        .await
        .map_err(|error| RegistryError::Generic { error })?;
    store::agent::register(
        agent,
        challenger,
        input.request.agent,
        input.tee,
        code.into(),
        now_ms,
    )?;

    store::state::notify_subscribers(AgentEvent {
        id: agent,
        kind: AgentEventKind::Registered,
        ts: now_ms,
    });

    Ok(())
}

#[ic_cdk::update]
pub async fn challenge(input: ChallengeEnvelope) -> Result<(), RegistryError> {
    let now_ms = ic_cdk::api::time() / MILLISECONDS;
    let canister_self = ic_cdk::api::canister_self();
    input.verify(now_ms, canister_self)?;

    let agent = input.authentication.sender();
    let challenger = input.request.authentication.unwrap().sender();
    if !store::state::is_challenger(&challenger) {
        return Err(RegistryError::Forbidden {
            error: format!("challenger {} has no permission", challenger),
        });
    }

    if let Some(tee) = &input.tee {
        let attestation = parse_and_verify(tee.attestation.as_ref().ok_or_else(|| {
            RegistryError::BadRequest {
                error: "attestation is not provided".to_string(),
            }
        })?)
        .map_err(|error| RegistryError::BadRequest {
            error: format!("attestation is not valid: {}", error),
        })?;

        if attestation.public_key.as_ref().map(|v| v.as_slice())
            != Some(input.authentication.pubkey.as_slice())
        {
            return Err(RegistryError::BadRequest {
                error: "attestation public key is not equal to agent public key".to_string(),
            });
        }
        if attestation.nonce.as_ref().map(|v| v.as_slice()) != Some(input.request.code.as_slice()) {
            return Err(RegistryError::BadRequest {
                error: "attestation nonce is not equal to chanllenge code".to_string(),
            });
        }
    }

    if let Some(canister) = &input.request.agent.handle_canister {
        store::state::check_handle(*canister, input.request.agent.handle.clone(), agent).await?;
    }

    let new_code = rand_bytes::<16>()
        .await
        .map_err(|error| RegistryError::Generic { error })?;
    store::agent::challenge(
        agent,
        challenger,
        input.request.agent,
        input.tee,
        input.request.code,
        new_code.into(),
        now_ms,
    )?;

    store::state::notify_subscribers(AgentEvent {
        id: agent,
        kind: AgentEventKind::Challenged,
        ts: now_ms,
    });

    Ok(())
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
    let now_ms = ic_cdk::api::time() / MILLISECONDS;
    store::agent::list_by_health_power(take as usize, now_ms)
}

#[ic_cdk::query]
fn last_challenged(take: Option<u64>) -> Result<BTreeMap<Principal, u64>, RegistryError> {
    let take = take.unwrap_or(100).min(10000);
    store::agent::last_challenged(take as usize)
}
