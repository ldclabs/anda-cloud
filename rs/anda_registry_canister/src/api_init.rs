use candid::{CandidType, Principal};
use serde::Deserialize;

use crate::{CHALLENGE_EXPIRES_IN_MS, store};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum ChainArgs {
    Init(InitArgs),
    Upgrade(UpgradeArgs),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct InitArgs {
    name: String,
    challenge_expires_in_ms: u64,
    governance_canister: Option<Principal>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct UpgradeArgs {
    name: Option<String>,
    challenge_expires_in_ms: Option<u64>,
    governance_canister: Option<Principal>,
}

#[ic_cdk::init]
fn init(args: Option<ChainArgs>) {
    match args.unwrap_or(ChainArgs::Init(InitArgs {
        name: "Anda AI Agents Registry Service".to_string(),
        challenge_expires_in_ms: CHALLENGE_EXPIRES_IN_MS, // 1 day
        governance_canister: None,
    })) {
        ChainArgs::Init(args) => {
            store::state::with_mut(|s| {
                s.name = args.name;
                s.challenge_expires_in_ms = args.challenge_expires_in_ms;
                s.governance_canister = args.governance_canister;
            });
        }
        ChainArgs::Upgrade(_) => {
            ic_cdk::trap(
                "cannot initialize the canister with an Upgrade args. Please provide an Init args.",
            );
        }
    }

    store::state::init_http_certified_data();
}

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    store::state::save();
}

#[ic_cdk::post_upgrade]
fn post_upgrade(args: Option<ChainArgs>) {
    store::state::load();

    match args {
        Some(ChainArgs::Upgrade(args)) => {
            store::state::with_mut(|s| {
                if let Some(name) = args.name {
                    s.name = name;
                }
                if let Some(challenge_expires_in_ms) = args.challenge_expires_in_ms {
                    s.challenge_expires_in_ms = challenge_expires_in_ms;
                }
                if let Some(governance_canister) = args.governance_canister {
                    s.governance_canister = Some(governance_canister);
                }
            });
        }
        Some(ChainArgs::Init(_)) => {
            ic_cdk::trap(
                "cannot upgrade the canister with an Init args. Please provide an Upgrade args.",
            );
        }
        _ => {}
    }

    store::state::init_http_certified_data();
}
