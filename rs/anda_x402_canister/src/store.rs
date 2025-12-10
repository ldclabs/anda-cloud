use anda_cloud_cdk::x402::*;
use candid::{CandidType, Principal};
use ciborium::{from_reader, into_writer};
use ic_http_certification::{
    HttpCertification, HttpCertificationPath, HttpCertificationTree, HttpCertificationTreeEntry,
    cel::{DefaultCelBuilder, create_cel_expr},
};
use ic_stable_structures::{
    DefaultMemoryImpl, StableBTreeMap, StableCell, StableLog, Storable,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

use crate::helper::{token_allowance, transfer_token_from};

const CLOCK_SKEW_MS: u64 = 1000 * 60; // 1 minute

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct State {
    pub name: String, // facilitator name
    #[serde(default)]
    pub key_name: String,
    pub supported_payments: BTreeSet<SupportedPaymentKind>,
    pub supported_assets: BTreeMap<Principal, AssetInfo>,
    pub total_collected_fees: BTreeMap<Principal, u128>,
    pub total_withdrawn_fees: BTreeMap<Principal, u128>,
    pub governance_canister: Option<Principal>,
}

#[derive(Clone, CandidType, Default, Deserialize, Serialize)]
pub struct StateInfo {
    pub name: String, // facilitator name
    #[serde(default)]
    pub key_name: String,
    pub supported_payments: BTreeSet<SupportedPaymentKind>,
    pub supported_assets: BTreeMap<Principal, AssetInfo>,
    pub total_collected_fees: BTreeMap<Principal, u128>,
    pub total_withdrawn_fees: BTreeMap<Principal, u128>,
    pub governance_canister: Option<Principal>,
}

impl From<&State> for StateInfo {
    fn from(state: &State) -> Self {
        StateInfo {
            name: state.name.clone(),
            key_name: state.key_name.clone(),
            supported_payments: state.supported_payments.clone(),
            supported_assets: state
                .supported_assets
                .iter()
                .map(|(k, v)| {
                    (
                        *k,
                        AssetInfo {
                            name: v.name.clone(),
                            symbol: v.symbol.clone(),
                            decimals: v.decimals,
                            transfer_fee: v.transfer_fee,
                            payment_fee: v.payment_fee,
                            logo: None, // do not expose logo in info, reduce data size
                        },
                    )
                })
                .collect(),
            total_collected_fees: state.total_collected_fees.clone(),
            total_withdrawn_fees: state.total_withdrawn_fees.clone(),
            governance_canister: state.governance_canister,
        }
    }
}

#[derive(Clone, CandidType, Default, Deserialize, Serialize)]
pub struct AssetInfo {
    #[serde(default)]
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub transfer_fee: u128,
    pub payment_fee: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
struct PayerState {
    #[serde(rename = "n")]
    pub next_nonce: u64,
    #[serde(rename = "t")]
    pub total_sent: BTreeMap<Principal, u128>,
    #[serde(rename = "l")]
    pub logs: BTreeSet<u64>,
}

#[derive(Clone, CandidType, Deserialize, Serialize)]
pub struct PayerStateInfo {
    pub next_nonce: u64,
    pub total_sent: BTreeMap<Principal, u128>,
    pub logs: BTreeSet<u64>,
}

impl Default for PayerState {
    fn default() -> Self {
        PayerState {
            next_nonce: 1,
            total_sent: BTreeMap::new(),
            logs: BTreeSet::new(),
        }
    }
}

impl From<PayerState> for PayerStateInfo {
    fn from(state: PayerState) -> Self {
        PayerStateInfo {
            next_nonce: state.next_nonce,
            total_sent: state.total_sent,
            logs: state.logs,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaymentLog {
    #[serde(rename = "s")]
    pub scheme: Scheme,
    #[serde(rename = "a")]
    pub asset: Principal,
    #[serde(rename = "f")]
    pub from: Principal,
    #[serde(rename = "t")]
    pub to: Principal,
    #[serde(rename = "v")]
    pub value: u128,
    #[serde(rename = "fe")]
    pub fee: u128,
    #[serde(rename = "e")]
    pub expires_at: u64,
    #[serde(rename = "n")]
    pub nonce: u64,
    #[serde(rename = "ts")]
    pub timestamp: u64, // in milliseconds
}

#[derive(Clone, CandidType, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentLogInfo {
    pub id: u64,
    pub scheme: Scheme,
    pub asset: Principal,
    pub from: Principal,
    pub to: Principal,
    pub value: String,
    pub fee: String,
    pub expires_at: u64,
    pub nonce: u64,
    pub timestamp: u64, // in milliseconds
}

impl From<PaymentLog> for PaymentLogInfo {
    fn from(log: PaymentLog) -> Self {
        PaymentLogInfo {
            id: 0,
            scheme: log.scheme,
            asset: log.asset,
            from: log.from,
            to: log.to,
            value: log.value.to_string(),
            fee: log.fee.to_string(),
            expires_at: log.expires_at,
            nonce: log.nonce,
            timestamp: log.timestamp,
        }
    }
}

impl Storable for PayerState {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![];
        into_writer(self, &mut buf).expect("failed to encode PayerState data");
        Cow::Owned(buf)
    }

    fn into_bytes(self) -> Vec<u8> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode PayerState data");
        buf
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode PayerState data")
    }
}

impl Storable for PaymentLog {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![];
        into_writer(self, &mut buf).expect("failed to encode PaymentLog data");
        Cow::Owned(buf)
    }

    fn into_bytes(self) -> Vec<u8> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode PaymentLog data");
        buf
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode PaymentLog data")
    }
}

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
const PAYER_STATE_MEMORY_ID: MemoryId = MemoryId::new(1);
const LOGS_INDEX_MEMORY_ID: MemoryId = MemoryId::new(2);
const LOGS_DATA_MEMORY_ID: MemoryId = MemoryId::new(3);

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
    static HTTP_TREE: RefCell<HttpCertificationTree> = RefCell::new(HttpCertificationTree::default());

    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static STATE_STORE: RefCell<StableCell<Vec<u8>, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(STATE_MEMORY_ID)),
            Vec::new()
        )
    );

    static PAYER_STATE: RefCell<StableBTreeMap<Principal, PayerState, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(PAYER_STATE_MEMORY_ID)),
        )
    );

    static LOGS: RefCell<StableLog<PaymentLog, Memory, Memory>> = RefCell::new(
        StableLog::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(LOGS_INDEX_MEMORY_ID)),
            MEMORY_MANAGER.with_borrow(|m| m.get(LOGS_DATA_MEMORY_ID)),
        )
    );
}

pub mod state {
    use super::*;
    use lazy_static::lazy_static;
    use once_cell::sync::Lazy;

    lazy_static! {
        pub static ref DEFAULT_EXPR_PATH: HttpCertificationPath<'static> =
            HttpCertificationPath::wildcard("");
        pub static ref DEFAULT_CERTIFICATION: HttpCertification = HttpCertification::skip();
        pub static ref DEFAULT_CEL_EXPR: String =
            create_cel_expr(&DefaultCelBuilder::skip_certification());
    }

    pub static DEFAULT_CERT_ENTRY: Lazy<HttpCertificationTreeEntry> =
        Lazy::new(|| HttpCertificationTreeEntry::new(&*DEFAULT_EXPR_PATH, *DEFAULT_CERTIFICATION));

    pub fn with<R>(f: impl FnOnce(&State) -> R) -> R {
        STATE.with_borrow(f)
    }

    pub fn with_mut<R>(f: impl FnOnce(&mut State) -> R) -> R {
        STATE.with_borrow_mut(f)
    }

    pub fn http_tree_with<R>(f: impl FnOnce(&HttpCertificationTree) -> R) -> R {
        HTTP_TREE.with(|r| f(&r.borrow()))
    }

    pub fn init_http_certified_data() {
        HTTP_TREE.with(|r| {
            let mut tree = r.borrow_mut();
            tree.insert(&DEFAULT_CERT_ENTRY);
            ic_cdk::api::certified_data_set(tree.root_hash())
        });
    }

    pub fn load() {
        STATE_STORE.with_borrow(|r| {
            STATE.with_borrow_mut(|h| {
                let bytes = r.get();
                if bytes.is_empty() {
                    return;
                }
                let v: State = from_reader(&bytes[..]).expect("failed to decode STATE_STORE data");
                *h = v;
            });
        });
    }

    pub fn save() {
        STATE.with_borrow(|h| {
            STATE_STORE.with_borrow_mut(|r| {
                let mut buf = vec![];
                into_writer(h, &mut buf).expect("failed to encode STATE_STORE data");
                r.set(buf);
            });
        });
    }

    pub fn info() -> StateInfo {
        with(|s| s.into())
    }

    pub fn supported() -> SupportedPaymentKindsResponse {
        with(|state| SupportedPaymentKindsResponse {
            kinds: state.supported_payments.clone().into_iter().collect(),
        })
    }

    pub fn next_nonce(caller: Principal) -> u64 {
        PAYER_STATE.with_borrow(|r| {
            let s = r.get(&caller).unwrap_or_default();
            s.next_nonce
        })
    }

    pub fn payer_info(caller: Principal) -> PayerStateInfo {
        PAYER_STATE.with_borrow(|r| {
            let s = r.get(&caller).unwrap_or_default();
            s.into()
        })
    }

    pub fn user_logs(user: Principal, take: usize, prev: Option<u64>) -> Vec<PaymentLogInfo> {
        PAYER_STATE.with_borrow(|r| {
            let item = r.get(&user).unwrap_or_default();
            if item.logs.is_empty() {
                return vec![];
            }
            let ids = item
                .logs
                .range(..prev.unwrap_or(u64::MAX))
                .rev()
                .take(take)
                .cloned()
                .collect::<Vec<u64>>();

            if ids.is_empty() {
                return vec![];
            }

            LOGS.with_borrow(|log_store| {
                let mut logs: Vec<PaymentLogInfo> = Vec::with_capacity(ids.len());
                for id in ids {
                    if let Some(log) = log_store.get(id) {
                        let mut log: PaymentLogInfo = log.into();
                        log.id = id;
                        logs.push(log);
                    }
                }
                logs
            })
        })
    }

    pub fn verify_payload(
        payer: Principal,
        payload: &PaymentPayload,
        now_ms: u64,
    ) -> Result<AssetInfo, X402Error> {
        if payload.payload.authorization.expires_at < now_ms + CLOCK_SKEW_MS {
            return Err(X402Error::InvalidPayload(format!(
                "Expired authorization: {}, current time: {}",
                payload.payload.authorization.expires_at, now_ms
            )));
        }

        let asset_info = with(|state| {
            if &payload.network != "icp" {
                return Err(X402Error::InvalidNetwork(format!(
                    "{}, expected: icp",
                    payload.network
                )));
            }

            let supported_payments = state
                .supported_payments
                .iter()
                .filter(|kind| kind.x402_version == payload.x402_version)
                .collect::<Vec<&SupportedPaymentKind>>();
            if supported_payments.is_empty() {
                return Err(X402Error::InvalidX402Version(payload.x402_version.into()));
            }
            if !supported_payments
                .iter()
                .any(|&kind| kind.scheme == payload.scheme)
            {
                return Err(X402Error::UnsupportedScheme(payload.scheme.to_string()));
            }

            let asset_info = state
                .supported_assets
                .get(&payload.payload.authorization.asset)
                .cloned()
                .ok_or_else(|| {
                    X402Error::InvalidPayloadAuthorizationValidAsset(format!(
                        "Unsupported asset: {}",
                        payload.payload.authorization.asset
                    ))
                })?;
            if payload.payload.authorization.value.0 <= asset_info.payment_fee {
                return Err(X402Error::InvalidPayloadAuthorizationValue(format!(
                    "Authorization value {} is not sufficient to cover payment fee {}",
                    payload.payload.authorization.value.0, asset_info.payment_fee
                )));
            }
            Ok(asset_info)
        })?;

        PAYER_STATE.with_borrow(|r| {
            let s = r.get(&payer).unwrap_or_default();
            if s.next_nonce != payload.payload.authorization.nonce {
                return Err(X402Error::VerifyError(format!(
                    "Invalid nonce: {}, expected: {}",
                    payload.payload.authorization.nonce, s.next_nonce
                )));
            }
            Ok(())
        })?;

        Ok(asset_info)
    }

    #[allow(dead_code)]
    pub async fn check_funds(
        payer: Principal,
        canister_self: Principal,
        asset: Principal,
        amount: u128,
        now_ms: u64,
    ) -> Result<(), X402Error> {
        let res = token_allowance(asset, payer, canister_self)
            .await
            .map_err(|err| {
                X402Error::VerifyError(format!("Failed to get token allowance: {}", err))
            })?;
        if let Some(expires_at) = res.expires_at
            && expires_at <= now_ms
        {
            return Err(X402Error::VerifyError(format!(
                "Token allowance expired at {}, current time: {}",
                expires_at, now_ms
            )));
        }

        if res.allowance < amount {
            return Err(X402Error::InsufficientFunds(format!(
                "{}, required: {}",
                res.allowance, amount
            )));
        }

        Ok(())
    }

    pub async fn transfer_funds(
        canister_self: Principal,
        log: PaymentLog,
        transfer_fee: u128,
    ) -> Result<String, X402Error> {
        let idx = transfer_token_from(
            log.asset,
            log.from,
            log.to,
            log.value.saturating_sub(log.fee),
            Some(log.nonce.into()),
        )
        .await
        .map_err(|err| {
            X402Error::SettleError(format!("Failed to transfer payment fee: {}", err))
        })?;

        let log_id = LOGS
            .with_borrow_mut(|r| r.append(&log))
            .expect("failed to append to LOGS");

        let tx = idx.to_string();
        PAYER_STATE.with_borrow_mut(|r| {
            let mut s = r.get(&log.from).unwrap_or_default();
            s.next_nonce = s.next_nonce.saturating_add(1);
            let total_sent = s.total_sent.entry(log.asset).or_insert(0);
            *total_sent = total_sent.saturating_add(log.value);
            s.logs.insert(log_id);
            r.insert(log.from, s);
        });

        if log.fee > 0 {
            // run in background
            let asset = log.asset;
            let payer = log.from;
            let fee = log.fee.saturating_sub(transfer_fee);
            let nonce = log.nonce;
            ic_cdk_timers::set_timer(Duration::from_secs(0), async move {
                let res =
                    transfer_token_from(asset, payer, canister_self, fee, Some(nonce.into())).await;

                if res.is_ok() {
                    STATE.with_borrow_mut(|state| {
                        let total = state.total_collected_fees.entry(asset).or_insert(0);
                        *total = total.saturating_add(fee);
                    });
                }
            });
        }

        Ok(tx)
    }
}
