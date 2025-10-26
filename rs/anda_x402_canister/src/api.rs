use crate::{helper::msg_caller, store};

#[ic_cdk::query]
fn info() -> Result<store::State, String> {
    Ok(store::state::info())
}

#[ic_cdk::query]
fn next_nonce() -> Result<u64, String> {
    let caller = msg_caller()?;
    Ok(store::state::next_nonce(caller))
}

#[ic_cdk::query]
fn my_payment_logs(take: u32, prev: Option<u64>) -> Result<Vec<store::PaymentLogInfo>, String> {
    let caller = msg_caller()?;
    let take = take.clamp(2, 100) as usize;
    let rt = store::state::user_logs(caller, take, prev);
    Ok(rt)
}
