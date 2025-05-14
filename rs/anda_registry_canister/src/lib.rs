use anda_cloud_cdk::{
    agent::{Agent, ChallengeEnvelope},
    registry::{RegistryError, RegistryState},
};
use candid::{CandidType, Principal, utils::ArgumentEncoder};
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

async fn call<In, Out>(id: Principal, method: &str, args: &In, cycles: u128) -> Result<Out, String>
where
    In: ArgumentEncoder + Send,
    Out: candid::CandidType + for<'a> candid::Deserialize<'a>,
{
    let res = ic_cdk::call::Call::bounded_wait(id, method)
        .with_args(args)
        .with_cycles(cycles)
        .await
        .map_err(|err| format!("failed to call {} on {:?}, error: {:?}", method, &id, err))?;
    res.candid().map_err(|err| {
        format!(
            "failed to decode response from {} on {:?}, error: {:?}",
            method, &id, err
        )
    })
}

async fn notify<In>(id: Principal, method: &str, arg: &In) -> Result<(), String>
where
    In: CandidType,
{
    ic_cdk::call::Call::unbounded_wait(id, method)
        .with_arg(arg)
        .oneway()
        .map_err(|err| format!("failed to call {} on {:?}, error: {:?}", method, &id, err))
}

async fn rand_bytes<const N: usize>() -> Result<[u8; N], String> {
    let mut data = ic_cdk::management_canister::raw_rand()
        .await
        .map_err(|err| format!("{err:?}"))?;
    data.truncate(N);
    data.try_into().map_err(|err| format!("{err:?}"))
}

ic_cdk::export_candid!();
