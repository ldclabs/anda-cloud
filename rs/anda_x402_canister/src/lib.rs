use anda_cloud_cdk::x402::{Scheme, X402Version};
use candid::{Nat, Principal};

use crate::api_init::CanisterArgs;

mod api;
mod api_admin;
mod api_http;
mod api_init;
mod helper;
mod store;

ic_cdk::export_candid!();
