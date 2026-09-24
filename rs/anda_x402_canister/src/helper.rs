use candid::{
    CandidType, IDLValue, Nat, Principal, pretty::candid::value::pp_value, utils::ArgumentEncoder,
};
use ic_cdk::call::{Call, CallFailed, Response};
use icrc_ledger_types::{
    icrc::generic_metadata_value::MetadataValue,
    icrc1::{
        account::Account,
        transfer::{Memo, TransferArg, TransferError},
    },
    icrc2::transfer_from::{TransferFromArgs, TransferFromError},
};

use crate::store;

pub fn msg_caller() -> Result<Principal, String> {
    let caller = ic_cdk::api::msg_caller();
    if caller == Principal::anonymous() {
        return Err("anonymous user is not allowed".to_string());
    }
    Ok(caller)
}

/// Calls a ledger and waits unboundedly, so the outcome of a transfer is
/// always known.
async fn call<In, Out>(id: Principal, method: &str, args: In) -> Result<Out, String>
where
    In: ArgumentEncoder + Send,
    Out: candid::CandidType + for<'a> candid::Deserialize<'a>,
{
    let res = Call::unbounded_wait(id, method).with_args(&args).await;
    decode_reply(id, method, res)
}

fn decode_reply<Out>(
    id: Principal,
    method: &str,
    res: Result<Response, CallFailed>,
) -> Result<Out, String>
where
    Out: candid::CandidType + for<'a> candid::Deserialize<'a>,
{
    res.map_err(|err| format!("failed to call {method} on {id}, error: {err:?}"))?
        .candid()
        .map_err(|err| format!("failed to decode response from {method} on {id}, error: {err:?}"))
}

pub fn pretty_format<T>(data: &T) -> Result<String, String>
where
    T: CandidType,
{
    let val = IDLValue::try_from_candid_type(data).map_err(|err| format!("{err:?}"))?;
    let doc = pp_value(7, &val);

    Ok(format!("{}", doc.pretty(120)))
}

pub async fn transfer_token_to(
    asset: Principal,
    to: Account,
    amount: u128,
    memo: Option<Memo>,
) -> Result<Nat, String> {
    let res: Result<Nat, TransferError> = call(
        asset,
        "icrc1_transfer",
        (TransferArg {
            from_subaccount: None,
            to,
            fee: None,
            created_at_time: None,
            memo,
            amount: amount.into(),
        },),
    )
    .await?;
    normalize_transfer_result(res)
}

fn normalize_transfer_result(res: Result<Nat, TransferError>) -> Result<Nat, String> {
    match res {
        Ok(index)
        | Err(TransferError::Duplicate {
            duplicate_of: index,
        }) => Ok(index),
        Err(err) => Err(format!("failed to transfer tokens, error: {err:?}")),
    }
}

pub async fn transfer_token_from(
    asset: Principal,
    from: Principal,
    to: Principal,
    amount: u128,
    memo: Option<Memo>,
    created_at_time: Option<u64>,
) -> Result<Nat, String> {
    let res: Result<Nat, TransferFromError> = call(
        asset,
        "icrc2_transfer_from",
        (TransferFromArgs {
            spender_subaccount: None,
            from: Account {
                owner: from,
                subaccount: None,
            },
            to: Account {
                owner: to,
                subaccount: None,
            },
            fee: None,
            created_at_time,
            memo,
            amount: amount.into(),
        },),
    )
    .await?;
    normalize_transfer_from_result(res)
}

fn normalize_transfer_from_result(res: Result<Nat, TransferFromError>) -> Result<Nat, String> {
    match res {
        Ok(index)
        | Err(TransferFromError::Duplicate {
            duplicate_of: index,
        }) => Ok(index),
        Err(err) => Err(format!(
            "failed to transfer tokens from user, error: {err:?}"
        )),
    }
}

pub async fn token_info(asset: Principal) -> Result<store::AssetInfo, String> {
    // A plain read, so a bounded wait is safe: a canister that never answers
    // cannot keep this one from stopping.
    let method = "icrc1_metadata";
    let res: Vec<(String, MetadataValue)> =
        decode_reply(asset, method, Call::bounded_wait(asset, method).await)?;
    let mut info = store::AssetInfo::default();
    for (key, value) in res {
        match (key.as_str(), value) {
            ("icrc1:name", MetadataValue::Text(val)) => info.name = val,
            ("icrc1:symbol", MetadataValue::Text(val)) => info.symbol = val,
            ("icrc1:logo", MetadataValue::Text(val)) => info.logo = Some(val),
            ("icrc1:decimals", MetadataValue::Nat(val)) => {
                info.decimals = u8::try_from(&val.0).unwrap_or_default()
            }
            ("icrc1:fee", MetadataValue::Nat(val)) => {
                info.transfer_fee = u128::try_from(&val.0).unwrap_or_default()
            }
            _ => {}
        }
    }

    if info.symbol.is_empty() {
        return Err("asset symbol is missing".to_string());
    }
    if info.decimals == 0 {
        return Err("asset decimals is missing".to_string());
    }
    if info.transfer_fee == 0 {
        return Err("asset transfer fee is missing".to_string());
    }
    if info.name.is_empty() {
        info.name = info.symbol.clone();
    }
    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_icrc1_transfer_is_success() {
        let index = Nat::from(42_u64);
        assert_eq!(
            normalize_transfer_result(Err(TransferError::Duplicate {
                duplicate_of: index.clone(),
            }))
            .unwrap(),
            index
        );
    }

    #[test]
    fn duplicate_icrc2_transfer_from_is_success() {
        let index = Nat::from(42_u64);
        assert_eq!(
            normalize_transfer_from_result(Err(TransferFromError::Duplicate {
                duplicate_of: index.clone(),
            }))
            .unwrap(),
            index
        );
    }

    #[test]
    fn non_duplicate_transfer_error_is_preserved() {
        let error = normalize_transfer_from_result(Err(TransferFromError::TemporarilyUnavailable))
            .unwrap_err();
        assert!(error.contains("TemporarilyUnavailable"));
    }
}
