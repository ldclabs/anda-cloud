use anda_cloud_cdk::x402::{Scheme, SupportedPaymentKind, X402Version};
use candid::{Nat, Principal};
use icrc_ledger_types::icrc1::account::Account;
use std::str::FromStr;

use crate::{
    helper::{pretty_format, token_info, transfer_token_to},
    store,
};

#[ic_cdk::update(guard = "is_controller")]
fn admin_add_supported_payment(x402_version: u8, scheme: String) -> Result<(), String> {
    let payment = SupportedPaymentKind {
        x402_version: X402Version::try_from(x402_version).map_err(|err| err.to_string())?,
        scheme: Scheme::from_str(&scheme).map_err(|err| err.to_string())?,
        network: "icp".to_string(),
    };

    if payment.scheme == Scheme::Upto {
        return Err("Scheme::Upto is not supported".to_string());
    }
    store::state::with_mut(|s| {
        s.supported_payments.insert(payment);
        Ok(())
    })
}

#[ic_cdk::update]
fn validate_admin_add_supported_payment(
    x402_version: u8,
    scheme: String,
) -> Result<String, String> {
    if scheme == Scheme::Upto.to_string() {
        return Err("Scheme::Upto is not supported".to_string());
    }

    pretty_format(&(x402_version, scheme))
}

#[ic_cdk::update(guard = "is_controller")]
fn admin_remove_supported_payment(x402_version: u8, scheme: String) -> Result<(), String> {
    let x402_version = X402Version::try_from(x402_version).map_err(|err| err.to_string())?;
    let scheme = Scheme::from_str(&scheme).map_err(|err| err.to_string())?;
    store::state::with_mut(|s| {
        s.supported_payments
            .retain(|p| p.x402_version != x402_version || p.scheme != scheme);
        Ok(())
    })
}

#[ic_cdk::update]
fn validate_admin_remove_supported_payment(
    x402_version: u8,
    scheme: String,
) -> Result<String, String> {
    pretty_format(&(x402_version, scheme))
}

#[ic_cdk::update(guard = "is_controller")]
async fn admin_update_supported_asset(asset: Principal, payment_fee: u128) -> Result<(), String> {
    let asset_info = check_supported_asset(asset, payment_fee).await?;
    store::state::with_mut(|s| {
        s.supported_assets.insert(asset, asset_info);
        Ok(())
    })
}

#[ic_cdk::update]
async fn validate_admin_update_supported_asset(
    asset: Principal,
    payment_fee: u128,
) -> Result<String, String> {
    let _ = check_supported_asset(asset, payment_fee).await?;
    pretty_format(&(asset, payment_fee))
}

#[ic_cdk::update(guard = "is_controller")]
async fn admin_remove_supported_asset(asset: Principal) -> Result<(), String> {
    store::state::with_mut(|s| {
        s.supported_assets.remove(&asset);
        Ok(())
    })
}

#[ic_cdk::update]
async fn validate_remove_update_supported_asset(asset: Principal) -> Result<String, String> {
    pretty_format(&(asset,))
}

async fn check_supported_asset(
    asset: Principal,
    payment_fee: u128,
) -> Result<store::AssetInfo, String> {
    let mut info = token_info(asset).await?;
    if payment_fee < info.transfer_fee {
        return Err(format!(
            "payment fee {} is less than transfer fee {}",
            payment_fee, info.transfer_fee
        ));
    }

    info.payment_fee = payment_fee;
    Ok(info)
}

#[ic_cdk::update(guard = "is_controller")]
async fn admin_collect_fees(asset: Principal, to: Principal, amount: u128) -> Result<Nat, String> {
    let value = store::state::with(|s| {
        let info = s
            .supported_assets
            .get(&asset)
            .ok_or_else(|| format!("asset {} is not supported", asset))?;
        if amount <= info.transfer_fee {
            return Err(format!("amount must be greater than {}", info.transfer_fee));
        }

        let balance = s
            .total_collected_fees
            .get(&asset)
            .cloned()
            .unwrap_or_default()
            .saturating_sub(
                s.total_withdrawn_fees
                    .get(&asset)
                    .cloned()
                    .unwrap_or_default(),
            );
        if amount > balance {
            return Err(format!(
                "amount {} exceeds available fees {}",
                amount, balance
            ));
        }
        Ok(amount - info.transfer_fee)
    })?;

    let idx = transfer_token_to(
        asset,
        Account {
            owner: to,
            subaccount: None,
        },
        value,
        None,
    )
    .await?;

    store::state::with_mut(|s| {
        s.total_withdrawn_fees
            .entry(asset)
            .and_modify(|v| *v = v.saturating_add(amount))
            .or_insert(amount);
    });
    Ok(idx)
}

#[ic_cdk::update]
async fn validate_admin_collect_fees(
    asset: Principal,
    to: Principal,
    amount: u128,
) -> Result<String, String> {
    store::state::with(|s| {
        let info = s
            .supported_assets
            .get(&asset)
            .ok_or_else(|| format!("asset {} is not supported", asset))?;
        if amount <= info.transfer_fee {
            return Err(format!("amount must be greater than {}", info.transfer_fee));
        }

        let balance = s
            .total_collected_fees
            .get(&asset)
            .cloned()
            .unwrap_or_default()
            .saturating_sub(
                s.total_withdrawn_fees
                    .get(&asset)
                    .cloned()
                    .unwrap_or_default(),
            );
        if amount > balance {
            return Err(format!(
                "amount {} exceeds available fees {}",
                amount, balance
            ));
        }
        Ok(())
    })?;
    pretty_format(&(asset, to, amount))
}

fn is_controller() -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();
    if ic_cdk::api::is_controller(&caller)
        || store::state::with(|s| s.governance_canister == Some(caller))
    {
        Ok(())
    } else {
        Err("user is not a controller".to_string())
    }
}
