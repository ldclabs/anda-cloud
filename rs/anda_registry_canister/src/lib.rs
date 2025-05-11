mod api;
mod api_init;
mod store;

use api_init::ChainArgs;

const CHALLENGE_EXPIRES_IN_MS: u64 = 1000 * 60 * 60; // 1 hour
const MILLISECONDS: u64 = 1000000;

ic_cdk::export_candid!();
