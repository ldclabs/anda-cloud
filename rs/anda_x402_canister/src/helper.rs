use candid::{
    CandidType, IDLValue, Nat, Principal, pretty::candid::value::pp_value, utils::ArgumentEncoder,
};
use icrc_ledger_types::{
    icrc::generic_metadata_value::MetadataValue,
    icrc1::{
        account::Account,
        transfer::{Memo, TransferArg, TransferError},
    },
    icrc2::{
        allowance::{Allowance, AllowanceArgs},
        transfer_from::{TransferFromArgs, TransferFromError},
    },
};
use num_traits::cast::ToPrimitive;

use crate::store;

const ANONYMOUS: Principal = Principal::anonymous();
pub fn msg_caller() -> Result<Principal, String> {
    let caller = ic_cdk::api::msg_caller();
    check_auth(&caller)?;
    Ok(caller)
}

pub fn check_auth(user: &Principal) -> Result<(), String> {
    if user == &ANONYMOUS {
        Err("anonymous user is not allowed".to_string())
    } else {
        Ok(())
    }
}

pub async fn call<In, Out>(
    id: Principal,
    method: &str,
    args: In,
    cycles: u128,
) -> Result<Out, String>
where
    In: ArgumentEncoder + Send,
    Out: candid::CandidType + for<'a> candid::Deserialize<'a>,
{
    let res = ic_cdk::call::Call::bounded_wait(id, method)
        .with_args(&args)
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
        0,
    )
    .await?;
    res.map_err(|err| format!("failed to transfer tokens, error: {:?}", err))
}

pub async fn transfer_token_from(
    asset: Principal,
    from: Principal,
    to: Principal,
    amount: u128,
    memo: Option<Memo>,
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
            created_at_time: None,
            memo,
            amount: amount.into(),
        },),
        0,
    )
    .await?;
    res.map_err(|err| format!("failed to transfer tokens from user, error: {:?}", err))
}

pub async fn token_allowance(
    asset: Principal,
    from: Principal,
    spender: Principal,
) -> Result<Allowance, String> {
    let res: Allowance = call(
        asset,
        "icrc2_allowance",
        (AllowanceArgs {
            account: Account {
                owner: from,
                subaccount: None,
            },
            spender: Account {
                owner: spender,
                subaccount: None,
            },
        },),
        0,
    )
    .await?;
    Ok(res)
}

pub async fn token_info(asset: Principal) -> Result<store::AssetInfo, String> {
    let res: Vec<(String, MetadataValue)> = call(asset, "icrc1_metadata", (), 0).await?;
    let mut info = store::AssetInfo::default();
    for (key, value) in res {
        match (key.as_str(), value) {
            ("icrc1:symbol", MetadataValue::Text(val)) => info.symbol = val,
            ("icrc1:decimals", MetadataValue::Nat(val)) => {
                info.decimals = val.0.to_u8().unwrap_or_default()
            }
            ("icrc1:fee", MetadataValue::Nat(val)) => {
                info.transfer_fee = val.0.to_u128().unwrap_or_default()
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
    Ok(info)
}
