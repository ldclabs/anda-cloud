use anda_cloud_cdk::x402::{Scheme, SupportedPaymentKind, X402Version};
use candid::{CandidType, Principal};
use serde::Deserialize;
use std::collections::BTreeSet;

use crate::store;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum CanisterArgs {
    Init(InitArgs),
    Upgrade(UpgradeArgs),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct InitArgs {
    pub name: String,
    pub governance_canister: Option<Principal>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct UpgradeArgs {
    pub name: Option<String>,
    pub governance_canister: Option<Principal>,
}

#[ic_cdk::init]
fn init(args: Option<CanisterArgs>) {
    if let Some(CanisterArgs::Init(args)) = args {
        store::state::with_mut(|s| {
            s.name = args.name;

            let network = "icp".to_string();
            s.supported_payments = BTreeSet::from([SupportedPaymentKind {
                x402_version: X402Version::V1,
                scheme: Scheme::Exact,
                network,
            }]);
            s.governance_canister = args.governance_canister;
        });
    } else if let Some(CanisterArgs::Upgrade(_)) = args {
        ic_cdk::trap("cannot init the canister with an Upgrade args. Please provide an Init args.");
    }

    store::state::init_http_certified_data();
}

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    store::state::save();
}

#[ic_cdk::post_upgrade]
fn post_upgrade(args: Option<CanisterArgs>) {
    store::state::load();

    match args {
        Some(CanisterArgs::Upgrade(args)) => store::state::with_mut(|s| {
            if let Some(name) = args.name {
                s.name = name;
            }

            if let Some(governance_canister) = args.governance_canister {
                s.governance_canister = Some(governance_canister);
            }
        }),
        Some(CanisterArgs::Init(_)) => {
            ic_cdk::trap(
                "cannot upgrade the canister with an Init args. Please provide an Upgrade args.",
            );
        }
        _ => {}
    }

    store::state::init_http_certified_data();
}
