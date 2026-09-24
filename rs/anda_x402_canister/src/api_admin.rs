use anda_cloud_cdk::x402::{NETWORK_ICP, SupportedKind, X402_VERSION};
use candid::{Nat, Principal};
use icrc_ledger_types::icrc1::account::Account;

use crate::{
    helper::{pretty_format, token_info, transfer_token_to},
    store,
};

// Validators run the same checks as the methods they vouch for, and are
// guarded like them: the governance canister calls both. Unguarded, a
// validator would let anyone spend this canister's cycles, and have it call
// arbitrary canisters.

#[ic_cdk::update(guard = "is_controller")]
fn admin_add_supported_payment(x402_version: u8, scheme: String) -> Result<(), String> {
    check_payment_kind(x402_version, &scheme)?;
    let payment = SupportedKind {
        x402_version,
        scheme,
        network: NETWORK_ICP.to_string(),
        extra: None,
    };

    store::state::with_mut(|s| {
        if !s.supported_payments.contains(&payment) {
            s.supported_payments.push(payment);
        }
        Ok(())
    })
}

#[ic_cdk::update(guard = "is_controller")]
fn validate_admin_add_supported_payment(
    x402_version: u8,
    scheme: String,
) -> Result<String, String> {
    check_payment_kind(x402_version, &scheme)?;
    pretty_format(&(x402_version, scheme))
}

/// Only the kind this canister implements can be offered: any other version or
/// scheme would be settled as if it were `exact` on x402 v2.
fn check_payment_kind(x402_version: u8, scheme: &str) -> Result<(), String> {
    if x402_version != X402_VERSION || scheme != store::SCHEME_EXACT {
        return Err(format!(
            "unsupported payment kind: x402 v{x402_version} {scheme}, only v{X402_VERSION} {} is implemented",
            store::SCHEME_EXACT
        ));
    }
    Ok(())
}

#[ic_cdk::update(guard = "is_controller")]
fn admin_remove_supported_payment(x402_version: u8, scheme: String) -> Result<(), String> {
    store::state::with_mut(|s| {
        s.supported_payments
            .retain(|p| p.x402_version != x402_version || p.scheme != scheme);
        Ok(())
    })
}

#[ic_cdk::update(guard = "is_controller")]
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

#[ic_cdk::update(guard = "is_controller")]
async fn validate_admin_update_supported_asset(
    asset: Principal,
    payment_fee: u128,
) -> Result<String, String> {
    let _ = check_supported_asset(asset, payment_fee).await?;
    pretty_format(&(asset, payment_fee))
}

#[ic_cdk::update(guard = "is_controller")]
fn admin_remove_supported_asset(asset: Principal) -> Result<(), String> {
    store::state::with_mut(|s| {
        s.supported_assets.remove(&asset);
        Ok(())
    })
}

#[ic_cdk::update(guard = "is_controller")]
fn validate_admin_remove_supported_asset(asset: Principal) -> Result<String, String> {
    pretty_format(&(asset,))
}

/// Deprecated name of `validate_admin_remove_supported_asset`, kept for
/// governance functions registered with it.
#[ic_cdk::update(guard = "is_controller")]
fn validate_remove_update_supported_asset(asset: Principal) -> Result<String, String> {
    validate_admin_remove_supported_asset(asset)
}

async fn check_supported_asset(
    asset: Principal,
    payment_fee: u128,
) -> Result<store::AssetInfo, String> {
    let mut info = token_info(asset).await?;
    if payment_fee > 0 && payment_fee <= info.transfer_fee {
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
    // Debit before the ledger call, so a concurrent withdrawal sees what is left.
    let value = store::state::with_mut(|s| {
        let value = check_collect_fees(s, asset, amount)?;
        let withdrawn = s.total_withdrawn_fees.entry(asset).or_insert(0);
        *withdrawn = withdrawn.saturating_add(amount);
        Ok::<_, String>(value)
    })?;

    let res = transfer_token_to(
        asset,
        Account {
            owner: to,
            subaccount: None,
        },
        value,
        None,
    )
    .await;
    if res.is_err() {
        store::state::with_mut(|s| {
            if let Some(withdrawn) = s.total_withdrawn_fees.get_mut(&asset) {
                *withdrawn = withdrawn.saturating_sub(amount);
            }
        });
    }
    res
}

#[ic_cdk::update(guard = "is_controller")]
fn validate_admin_collect_fees(
    asset: Principal,
    to: Principal,
    amount: u128,
) -> Result<String, String> {
    store::state::with(|s| check_collect_fees(s, asset, amount))?;
    pretty_format(&(asset, to, amount))
}

/// Checks a withdrawal of `amount` collected fees, and returns what reaches the
/// recipient once the ledger has taken its transfer fee.
fn check_collect_fees(s: &store::State, asset: Principal, amount: u128) -> Result<u128, String> {
    let info = s
        .supported_assets
        .get(&asset)
        .ok_or_else(|| format!("asset {} is not supported", asset))?;
    if amount <= info.transfer_fee {
        return Err(format!("amount must be greater than {}", info.transfer_fee));
    }

    let collected = s.total_collected_fees.get(&asset).copied().unwrap_or(0);
    let withdrawn = s.total_withdrawn_fees.get(&asset).copied().unwrap_or(0);
    let balance = collected.saturating_sub(withdrawn);
    if amount > balance {
        return Err(format!(
            "amount {} exceeds available fees {}",
            amount, balance
        ));
    }
    Ok(amount - info.transfer_fee)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_implemented_payment_kind_can_be_offered() {
        check_payment_kind(X402_VERSION, store::SCHEME_EXACT).unwrap();
        assert!(check_payment_kind(1, store::SCHEME_EXACT).is_err());
        assert!(check_payment_kind(X402_VERSION, "upto").is_err());
    }

    #[test]
    fn fee_withdrawals_are_bounded_by_the_collected_balance() {
        let asset = Principal::from_slice(&[1]);
        let mut state = store::State::default();
        state.supported_assets.insert(
            asset,
            store::AssetInfo {
                transfer_fee: 10,
                ..Default::default()
            },
        );
        state.total_collected_fees.insert(asset, 1_000);
        state.total_withdrawn_fees.insert(asset, 400);

        assert_eq!(check_collect_fees(&state, asset, 600), Ok(590));
        assert!(check_collect_fees(&state, asset, 601).is_err());
        assert!(check_collect_fees(&state, asset, 10).is_err());
        assert!(check_collect_fees(&state, Principal::from_slice(&[2]), 100).is_err());
    }
}
