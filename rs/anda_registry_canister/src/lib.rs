use anda_cloud_cdk::{
    agent::{Agent, ChallengeEnvelope},
    registry::{RegistryError, RegistryState},
};
use candid::Principal;
use std::collections::{BTreeMap, BTreeSet};

mod api;
mod api_admin;
mod api_http;
mod api_init;
mod store;

use api_init::ChainArgs;

const CHALLENGE_EXPIRES_IN_MS: u64 = 1000 * 60 * 60; // 1 hour
const MILLISECONDS: u64 = 1000000;
const ANONYMOUS: Principal = Principal::anonymous();

fn is_controller() -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();
    if ic_cdk::api::is_controller(&caller) || store::state::is_controller(&caller) {
        Ok(())
    } else {
        Err("user is not a controller".to_string())
    }
}

fn validate_principals(principals: &BTreeSet<Principal>) -> Result<(), String> {
    if principals.is_empty() {
        return Err("principals cannot be empty".to_string());
    }
    if principals.contains(&ANONYMOUS) {
        return Err("anonymous user is not allowed".to_string());
    }
    Ok(())
}

async fn rand_bytes<const N: usize>() -> Result<[u8; N], String> {
    let mut data = ic_cdk::management_canister::raw_rand()
        .await
        .map_err(|err| format!("{err:?}"))?;
    data.truncate(N);
    data.try_into().map_err(|err| format!("{err:?}"))
}

ic_cdk::export_candid!();
