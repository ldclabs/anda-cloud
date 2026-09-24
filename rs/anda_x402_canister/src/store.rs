use anda_cloud_cdk::x402::*;
use candid::{CandidType, Principal};
use cbor2::{from_slice, to_vec as cbor_to_vec};
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
    sync::LazyLock,
};

/// The only payment scheme this facilitator implements.
pub const SCHEME_EXACT: &str = "exact";
pub(crate) const MAX_AUTHORIZATION_TTL_MS: u64 = 60 * 60 * 1_000; // 1 hour
/// How many of the latest payment log IDs `my_info` lists. The full history is
/// paged by `my_payment_logs`.
const PAYER_INFO_MAX_LOGS: usize = 100;
// Keep new log IDs disjoint from the read-only append-log namespace.
const V2_LOG_ID_OFFSET: u64 = 1 << 63;

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct State {
    pub name: String, // facilitator name
    #[serde(default)]
    pub key_name: String,
    pub supported_payments: Vec<SupportedKind>,
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
    pub supported_payments: Vec<SupportedKindCan>,
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
            supported_payments: state.supported_payments.iter().map(|k| k.into()).collect(),
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

/// The part of an [`AssetInfo`] a payment needs.
#[derive(Clone, Copy)]
pub struct AssetFees {
    pub payment_fee: u128,
    pub transfer_fee: u128,
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

/// Compact payer state used by new writes.
///
/// The legacy [`PayerState`] stores every payment log ID in one CBOR value.
/// Updating a payer therefore rewrites their complete payment history.  Keep
/// the legacy map read-only for upgrade compatibility and lazily seed this
/// bounded-growth state when the payer next settles a payment.
#[derive(Clone, Deserialize, Serialize)]
struct PayerStateV2 {
    #[serde(rename = "n")]
    next_nonce: u64,
    #[serde(rename = "t")]
    total_sent: BTreeMap<Principal, u128>,
    #[serde(default, rename = "l", skip_serializing_if = "Option::is_none")]
    last_log_id: Option<u64>,
}

impl Default for PayerStateV2 {
    fn default() -> Self {
        Self {
            next_nonce: 1,
            total_sent: BTreeMap::new(),
            last_log_id: None,
        }
    }
}

impl From<PayerState> for PayerStateV2 {
    fn from(state: PayerState) -> Self {
        Self {
            next_nonce: state.next_nonce,
            total_sent: state.total_sent,
            last_log_id: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaymentLog {
    #[serde(rename = "s")]
    pub scheme: String,
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
    /// Ledger transaction ID. Missing from legacy records.
    #[serde(default, rename = "x", skip_serializing_if = "String::is_empty")]
    pub transaction: String,
}

#[derive(Clone, CandidType, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentLogInfo {
    pub id: u64,
    pub scheme: String,
    pub asset: Principal,
    pub from: Principal,
    pub to: Principal,
    pub value: String,
    pub fee: String,
    pub expires_at: u64,
    pub nonce: u64,
    pub timestamp: u64, // in milliseconds
    /// Ledger transaction ID. Empty for legacy records.
    pub transaction: String,
}

impl PaymentLog {
    fn into_info(self, id: u64) -> PaymentLogInfo {
        PaymentLogInfo {
            id,
            scheme: self.scheme,
            asset: self.asset,
            from: self.from,
            to: self.to,
            value: self.value.to_string(),
            fee: self.fee.to_string(),
            expires_at: self.expires_at,
            nonce: self.nonce,
            timestamp: self.timestamp,
            transaction: self.transaction,
        }
    }
}

macro_rules! impl_cbor_storable {
    ($ty:ty) => {
        impl Storable for $ty {
            const BOUND: Bound = Bound::Unbounded;

            fn to_bytes(&self) -> Cow<'_, [u8]> {
                Cow::Owned(
                    cbor_to_vec(self).unwrap_or_else(|err| {
                        panic!("failed to encode {}: {err}", stringify!($ty))
                    }),
                )
            }

            fn into_bytes(self) -> Vec<u8> {
                cbor_to_vec(&self)
                    .unwrap_or_else(|err| panic!("failed to encode {}: {err}", stringify!($ty)))
            }

            fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
                from_slice(&bytes)
                    .unwrap_or_else(|err| panic!("failed to decode {}: {err}", stringify!($ty)))
            }
        }
    };
}

impl_cbor_storable!(PayerState);
impl_cbor_storable!(PayerStateV2);
impl_cbor_storable!(PaymentLog);

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
const PAYER_STATE_MEMORY_ID: MemoryId = MemoryId::new(1);
const LOGS_INDEX_MEMORY_ID: MemoryId = MemoryId::new(2);
const LOGS_DATA_MEMORY_ID: MemoryId = MemoryId::new(3);
const PAYER_STATE_V2_MEMORY_ID: MemoryId = MemoryId::new(4);
const PAYMENT_LOGS_V2_MEMORY_ID: MemoryId = MemoryId::new(5);

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

    static PAYER_STATE_V2: RefCell<StableBTreeMap<Principal, PayerStateV2, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(PAYER_STATE_V2_MEMORY_ID)),
        )
    );

    static LOGS: RefCell<StableLog<PaymentLog, Memory, Memory>> = RefCell::new(
        StableLog::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(LOGS_INDEX_MEMORY_ID)),
            MEMORY_MANAGER.with_borrow(|m| m.get(LOGS_DATA_MEMORY_ID)),
        )
    );

    static PAYMENT_LOGS_V2: RefCell<StableBTreeMap<(Principal, u64), PaymentLog, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(PAYMENT_LOGS_V2_MEMORY_ID)),
        ));

    // Payers with a settlement awaiting the token ledger, see `PayerGuard`.
    static PAYERS_IN_FLIGHT: RefCell<BTreeSet<Principal>> = const { RefCell::new(BTreeSet::new()) };
}

/// Holds a payer's settlement lock, so that a nonce moves funds at most once
/// while the ledger call is awaited.
///
/// The lock is released on drop, including when a trap after an `await`
/// cancels the call: the executor then drops the call's locals, so the lock
/// cannot be stranded.
pub struct PayerGuard(Principal);

impl PayerGuard {
    pub fn acquire(payer: Principal) -> Result<Self, X402Error> {
        PAYERS_IN_FLIGHT.with_borrow_mut(|in_flight| {
            if in_flight.insert(payer) {
                Ok(Self(payer))
            } else {
                Err(X402Error::SettleError(
                    "Another payment from this payer is already in progress".to_string(),
                ))
            }
        })
    }
}

impl Drop for PayerGuard {
    fn drop(&mut self) {
        PAYERS_IN_FLIGHT.with_borrow_mut(|in_flight| {
            in_flight.remove(&self.0);
        });
    }
}

pub mod state {
    use super::*;
    use std::ops::Bound::{Excluded, Included};

    pub static DEFAULT_EXPR_PATH: LazyLock<HttpCertificationPath<'static>> =
        LazyLock::new(|| HttpCertificationPath::wildcard(""));
    pub static DEFAULT_CEL_EXPR: LazyLock<String> =
        LazyLock::new(|| create_cel_expr(&DefaultCelBuilder::skip_certification()));
    pub static DEFAULT_CERT_ENTRY: LazyLock<HttpCertificationTreeEntry<'static>> =
        LazyLock::new(|| {
            HttpCertificationTreeEntry::new(&*DEFAULT_EXPR_PATH, HttpCertification::skip())
        });

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
                let v: State = from_slice(bytes).expect("failed to decode STATE_STORE data");
                *h = v;
            });
        });
    }

    pub fn save() {
        STATE.with_borrow(|h| {
            STATE_STORE.with_borrow_mut(|r| {
                r.set(cbor_to_vec(h).expect("failed to encode STATE_STORE data"));
            });
        });
    }

    pub fn info() -> StateInfo {
        with(|s| s.into())
    }

    pub fn supported() -> SupportedResponse {
        // The canister is the ICRC-2 spender that pulls every payment, so it is
        // the address payers must approve. Publishing it under the `icp:*`
        // CAIP-2 pattern is what lets a client discover that without having to
        // be told the facilitator's canister ID out of band.
        let signers = BTreeMap::from([(
            "icp:*".to_string(),
            vec![ic_cdk::api::canister_self().to_string()],
        )]);

        with(|state| SupportedResponse {
            kinds: state.supported_payments.clone().into_iter().collect(),
            extensions: vec![],
            signers,
        })
    }

    fn legacy_payer_state(caller: Principal) -> PayerState {
        PAYER_STATE.with_borrow(|store| store.get(&caller).unwrap_or_default())
    }

    fn payer_state(caller: Principal) -> PayerStateV2 {
        PAYER_STATE_V2
            .with_borrow(|store| store.get(&caller))
            .unwrap_or_else(|| legacy_payer_state(caller).into())
    }

    pub fn next_nonce(caller: Principal) -> u64 {
        payer_state(caller).next_nonce
    }

    pub fn payer_info(caller: Principal) -> PayerStateInfo {
        let state = payer_state(caller);
        // Read keys only: the log bodies are not needed to list their IDs.
        let mut logs: BTreeSet<u64> = PAYMENT_LOGS_V2.with_borrow(|store| {
            store
                .keys_range((Included((caller, 0)), Included((caller, u64::MAX))))
                .rev()
                .take(PAYER_INFO_MAX_LOGS)
                .map(|(_, id)| id)
                .collect()
        });
        // Legacy IDs are all older than the compact ones.
        let missing = PAYER_INFO_MAX_LOGS - logs.len();
        if missing > 0 {
            logs.extend(
                legacy_payer_state(caller)
                    .logs
                    .into_iter()
                    .rev()
                    .take(missing),
            );
        }

        PayerStateInfo {
            next_nonce: state.next_nonce,
            total_sent: state.total_sent,
            logs,
        }
    }

    pub fn user_logs(user: Principal, take: usize, prev: Option<u64>) -> Vec<PaymentLogInfo> {
        let upper = prev.unwrap_or(u64::MAX);
        if upper == 0 || take == 0 {
            return vec![];
        }

        let mut result = PAYMENT_LOGS_V2.with_borrow(|store| {
            store
                .range((Included((user, 0)), Excluded((user, upper))))
                .rev()
                .take(take)
                .map(|entry| {
                    let id = entry.key().1;
                    entry.value().into_info(id)
                })
                .collect::<Vec<_>>()
        });

        if result.len() == take {
            return result;
        }

        let legacy_ids = legacy_payer_state(user)
            .logs
            .range(..upper)
            .rev()
            .take(take - result.len())
            .copied()
            .collect::<Vec<_>>();

        LOGS.with_borrow(|store| {
            result.extend(
                legacy_ids
                    .into_iter()
                    .filter_map(|id| store.get(id).map(|log| log.into_info(id))),
            );
        });
        result
    }

    /// Validates everything that does not depend on the payer's identity.
    ///
    /// Callers run this *before* verifying the payload signature: a signature
    /// check costs up to [`MAX_DELEGATION_CHAIN_LENGTH`](ic_auth_verifier)
    /// canister-signature verifications and also runs on the free query
    /// preflight, so junk requests should be turned away by these cheap checks
    /// first.
    pub fn verify_requirements(
        payload: &PaymentPayload,
        now_ms: u64,
    ) -> Result<AssetFees, X402Error> {
        if payload.accepted.network != NETWORK_ICP {
            return Err(X402Error::InvalidNetwork(format!(
                "{}, expected: {NETWORK_ICP}",
                payload.accepted.network
            )));
        }

        // The reserved `extra` keys decide *how* value moves and *when*, so an
        // unrecognized value must be refused rather than silently served as if
        // it were the ICRC-2 default.
        payload.accepted.validate()?;

        let expires_at = payload.payload.authorization.expires_at;
        if expires_at <= now_ms {
            return Err(X402Error::InvalidPayloadAuthorizationValidBefore(format!(
                "Expired authorization: {expires_at}, current time: {now_ms}"
            )));
        }
        if expires_at > now_ms.saturating_add(MAX_AUTHORIZATION_TTL_MS) {
            return Err(X402Error::InvalidPayload(
                "Authorization lifetime exceeds one hour".to_string(),
            ));
        }

        with(|state| {
            let mut version_supported = false;
            let scheme_supported = state.supported_payments.iter().any(|kind| {
                if kind.x402_version == payload.x402_version {
                    version_supported = true;
                    kind.scheme == payload.accepted.scheme
                } else {
                    false
                }
            });
            if !version_supported {
                return Err(X402Error::InvalidX402Version(payload.x402_version));
            }
            if !scheme_supported {
                return Err(X402Error::UnsupportedScheme(
                    payload.accepted.scheme.to_string(),
                ));
            }

            // Copy out the fees only: an asset's logo can be a sizeable data URI.
            let fees = state
                .supported_assets
                .get(&payload.accepted.asset)
                .map(|info| AssetFees {
                    payment_fee: info.payment_fee,
                    transfer_fee: info.transfer_fee,
                })
                .ok_or_else(|| {
                    X402Error::InvalidPaymentRequirements(format!(
                        "Unsupported asset: {}",
                        payload.accepted.asset
                    ))
                })?;
            if payload.payload.authorization.value.0 <= fees.payment_fee {
                return Err(X402Error::InvalidPayloadAuthorizationValue(format!(
                    "Authorization value {} is not sufficient to cover payment fee {}",
                    payload.payload.authorization.value.0, fees.payment_fee
                )));
            }
            Ok(fees)
        })
    }

    /// Checks that `nonce` is the payer's next one. Only callable once the
    /// payload signature has established `payer`.
    ///
    /// An authorization settles at most once, so repeating one is refused.
    /// For the payment settled last, the error names its ledger transaction,
    /// so a resource server that lost the settle response can reconcile it.
    pub fn verify_payer(payer: Principal, nonce: u64) -> Result<(), X402Error> {
        let state = payer_state(payer);
        if state.next_nonce == nonce {
            return Ok(());
        }
        if state.next_nonce.checked_sub(1) == Some(nonce)
            && let Some(tx) = last_transaction(payer, &state)
        {
            return Err(X402Error::VerifyError(format!(
                "Nonce {nonce} was already settled in transaction {tx}"
            )));
        }
        Err(X402Error::VerifyError(format!(
            "Invalid nonce: {nonce}, expected: {}",
            state.next_nonce
        )))
    }

    /// The ledger transaction of the payment the payer settled last. Payers
    /// migrated from the legacy layout have none until they settle again.
    fn last_transaction(payer: Principal, state: &PayerStateV2) -> Option<String> {
        let log_id = state.last_log_id?;
        PAYMENT_LOGS_V2
            .with_borrow(|store| store.get(&(payer, log_id)))
            .map(|log| log.transaction)
            .filter(|tx| !tx.is_empty())
    }

    /// Records a payment the ledger has moved and advances the payer's nonce.
    /// The caller holds the payer's [`PayerGuard`].
    pub fn record_payment(log: PaymentLog) -> Result<u64, X402Error> {
        let payer = log.from;
        let mut state = payer_state(payer);
        if state.next_nonce != log.nonce {
            return Err(X402Error::SettleError(format!(
                "Payer nonce changed while settling: {}, expected {}",
                state.next_nonce, log.nonce
            )));
        }

        let log_id = V2_LOG_ID_OFFSET
            .checked_add(PAYMENT_LOGS_V2.with_borrow(|store| store.len()))
            .ok_or_else(|| X402Error::SettleError("Payment log ID space is exhausted".into()))?;
        state.next_nonce += 1;
        let total_sent = state.total_sent.entry(log.asset).or_insert(0);
        *total_sent = total_sent.saturating_add(log.value);
        state.last_log_id = Some(log_id);

        PAYMENT_LOGS_V2.with_borrow_mut(|store| {
            store.insert((payer, log_id), log);
        });
        // The first write also migrates a legacy payer to the compact layout.
        PAYER_STATE_V2.with_borrow_mut(|store| {
            store.insert(payer, state);
        });
        Ok(log_id)
    }

    pub fn add_collected_fee(asset: Principal, fee: u128) {
        with_mut(|state| {
            let total = state.total_collected_fees.entry(asset).or_insert(0);
            *total = total.saturating_add(fee);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_auth_types::ByteBufB64;

    fn principal(id: u8) -> Principal {
        Principal::from_slice(&[id])
    }

    fn payment_payload() -> PaymentPayload {
        PaymentPayload {
            x402_version: 2,
            resource: None,
            accepted: PaymentRequirements {
                scheme: "exact".to_string(),
                network: "icp:1".to_string(),
                amount: TokenAmount(1_000),
                asset: principal(2),
                pay_to: principal(3),
                max_timeout_seconds: 300,
                extra: None,
            },
            payload: IcpPayload {
                signature: ByteBufB64::new(),
                authorization: IcpPayloadAuthorization {
                    to: principal(3),
                    asset: principal(2),
                    value: TokenAmount(1_000),
                    expires_at: 1_800_000_000_000,
                    nonce: 7,
                },
            },
            extensions: None,
        }
    }

    fn payment_log() -> PaymentLog {
        PaymentLog {
            scheme: "exact".to_string(),
            asset: principal(2),
            from: principal(1),
            to: principal(3),
            value: 1_000,
            fee: 10,
            expires_at: 1_800_000_000_000,
            nonce: 7,
            timestamp: 1_799_999_700_000,
            transaction: "42".to_string(),
        }
    }

    #[test]
    fn compact_payer_state_does_not_grow_with_log_history() {
        let mut legacy = PayerState::default();
        legacy.total_sent.insert(principal(2), 123_456);
        legacy.logs.extend(0..10_000);

        let legacy_size = cbor_to_vec(&legacy).unwrap().len();
        let compact = PayerStateV2::from(legacy.clone());
        let compact_bytes = compact.to_bytes();
        let decoded = PayerStateV2::from_bytes(compact_bytes.clone());

        assert_eq!(decoded.next_nonce, legacy.next_nonce);
        assert_eq!(decoded.total_sent, legacy.total_sent);
        assert_eq!(decoded.last_log_id, None);
        assert!(legacy_size > compact_bytes.len() * 100);
    }

    #[test]
    fn legacy_payment_log_decodes_without_transaction() {
        #[derive(Serialize)]
        struct LegacyPaymentLog {
            #[serde(rename = "s")]
            scheme: String,
            #[serde(rename = "a")]
            asset: Principal,
            #[serde(rename = "f")]
            from: Principal,
            #[serde(rename = "t")]
            to: Principal,
            #[serde(rename = "v")]
            value: u128,
            #[serde(rename = "fe")]
            fee: u128,
            #[serde(rename = "e")]
            expires_at: u64,
            #[serde(rename = "n")]
            nonce: u64,
            #[serde(rename = "ts")]
            timestamp: u64,
        }

        let log = payment_log();
        let bytes = cbor_to_vec(&LegacyPaymentLog {
            scheme: log.scheme.clone(),
            asset: log.asset,
            from: log.from,
            to: log.to,
            value: log.value,
            fee: log.fee,
            expires_at: log.expires_at,
            nonce: log.nonce,
            timestamp: log.timestamp,
        })
        .unwrap();
        let decoded: PaymentLog = from_slice(&bytes).unwrap();

        assert_eq!(decoded.nonce, log.nonce);
        assert!(decoded.transaction.is_empty());
    }

    fn record(payer: Principal, nonce: u64) -> u64 {
        let mut log = payment_log();
        log.from = payer;
        log.nonce = nonce;
        state::record_payment(log).unwrap()
    }

    #[test]
    fn a_settled_authorization_is_refused_and_names_its_transaction() {
        let payer = principal(9);
        let log_id = record(payer, 1);

        assert!(log_id >= V2_LOG_ID_OFFSET);
        assert_eq!(state::next_nonce(payer), 2);
        let logs = state::user_logs(payer, 10, None);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].id, log_id);
        assert_eq!(logs[0].nonce, 1);
        assert_eq!(logs[0].transaction, "42");

        state::verify_payer(payer, 2).unwrap();
        // Repeating the settled authorization, on /verify or /settle, is refused.
        assert_eq!(
            state::verify_payer(payer, 1),
            Err(X402Error::VerifyError(
                "Nonce 1 was already settled in transaction 42".to_string()
            ))
        );
        assert!(matches!(
            state::verify_payer(payer, 0),
            Err(X402Error::VerifyError(msg)) if msg.starts_with("Invalid nonce")
        ));

        let mut again = payment_log();
        again.from = payer;
        again.nonce = 1;
        assert!(state::record_payment(again).is_err());
    }

    #[test]
    fn legacy_payer_is_lazily_migrated_without_copying_log_ids() {
        let payer = principal(10);
        let mut legacy = PayerState {
            next_nonce: 8,
            ..PayerState::default()
        };
        legacy.logs.extend([2, 4, 6]);
        PAYER_STATE.with_borrow_mut(|store| {
            store.insert(payer, legacy);
        });

        // Reads fall back to the legacy record until the payer settles again.
        assert_eq!(state::next_nonce(payer), 8);
        assert!(matches!(
            state::verify_payer(payer, 7),
            Err(X402Error::VerifyError(msg)) if msg.starts_with("Invalid nonce")
        ));
        assert!(
            PAYER_STATE_V2
                .with_borrow(|store| store.get(&payer))
                .is_none()
        );

        let log_id = record(payer, 8);
        let compact = PAYER_STATE_V2
            .with_borrow(|store| store.get(&payer))
            .unwrap();
        assert_eq!(compact.next_nonce, 9);
        assert_eq!(compact.last_log_id, Some(log_id));
        assert_eq!(
            state::payer_info(payer).logs,
            BTreeSet::from([2, 4, 6, log_id])
        );
    }

    #[test]
    fn payer_info_lists_only_the_latest_logs() {
        let payer = principal(13);
        let mut legacy = PayerState::default();
        legacy.logs.extend([1, 2, 3]);
        PAYER_STATE.with_borrow_mut(|store| {
            store.insert(payer, legacy);
        });

        let mut ids: BTreeSet<u64> = (1..PAYER_INFO_MAX_LOGS as u64)
            .map(|nonce| record(payer, nonce))
            .collect();
        // One slot is left, taken by the newest legacy log.
        let mut expected = ids.clone();
        expected.insert(3);
        assert_eq!(state::payer_info(payer).logs, expected);

        ids.insert(record(payer, PAYER_INFO_MAX_LOGS as u64));
        let info = state::payer_info(payer);
        assert_eq!(info.logs, ids);
        assert_eq!(info.next_nonce, PAYER_INFO_MAX_LOGS as u64 + 1);
    }

    #[test]
    fn cheap_checks_reject_without_touching_payer_state() {
        // These all resolve without a verified payer, so the caller can run them
        // before paying for signature verification.
        let now_ms = 1_799_999_700_000;

        let mut wrong_network = payment_payload();
        wrong_network.accepted.network = "eip155:1".to_string();
        assert!(matches!(
            state::verify_requirements(&wrong_network, now_ms),
            Err(X402Error::InvalidNetwork(_))
        ));

        let expired = payment_payload();
        assert!(matches!(
            state::verify_requirements(&expired, expired.payload.authorization.expires_at),
            Err(X402Error::InvalidPayloadAuthorizationValidBefore(_))
        ));

        // Any remaining lifetime passes the expiry check, however short. The
        // checks then stop at the payment kinds this test state lacks.
        let short_lived = payment_payload();
        assert!(matches!(
            state::verify_requirements(
                &short_lived,
                short_lived.payload.authorization.expires_at - 1
            ),
            Err(X402Error::InvalidX402Version(_))
        ));

        let too_long_lived = payment_payload();
        assert!(matches!(
            state::verify_requirements(
                &too_long_lived,
                too_long_lived.payload.authorization.expires_at - MAX_AUTHORIZATION_TTL_MS - 1,
            ),
            Err(X402Error::InvalidPayload(_))
        ));

        // No supported payment kinds are configured in this test state.
        assert!(matches!(
            state::verify_requirements(&payment_payload(), now_ms),
            Err(X402Error::InvalidX402Version(_))
        ));
    }

    #[test]
    fn reserved_extra_keys_gate_the_mechanism() {
        let now_ms = 1_799_999_700_000;

        // `escrow` needs two settles, which the single ICRC-2 pull cannot serve.
        let mut escrow = payment_payload();
        escrow.accepted.extra = Some(Map::from_iter([(
            EXTRA_PAYMENT_FLOW.to_string(),
            Value::String("escrow".to_string()),
        )]));
        assert!(matches!(
            state::verify_requirements(&escrow, now_ms),
            Err(X402Error::InvalidPaymentRequirements(_))
        ));

        let mut unknown_method = payment_payload();
        unknown_method.accepted.extra = Some(Map::from_iter([(
            EXTRA_ASSET_TRANSFER_METHOD.to_string(),
            Value::String("eip3009".to_string()),
        )]));
        assert!(matches!(
            state::verify_requirements(&unknown_method, now_ms),
            Err(X402Error::InvalidPaymentRequirements(_))
        ));

        // Explicit mechanism defaults get past this gate and fail later, on the
        // payment kinds this test state does not configure.
        let mut defaults = payment_payload();
        defaults.accepted.extra = Some(Map::from_iter([
            (
                EXTRA_ASSET_TRANSFER_METHOD.to_string(),
                Value::String(ASSET_TRANSFER_METHOD_ICRC2.to_string()),
            ),
            (
                EXTRA_PAYMENT_FLOW.to_string(),
                Value::String("upfront".to_string()),
            ),
        ]));
        assert!(matches!(
            state::verify_requirements(&defaults, now_ms),
            Err(X402Error::InvalidX402Version(_))
        ));
    }

    #[test]
    fn configured_requirements_yield_the_asset_fees() {
        let now_ms = 1_799_999_700_000;
        state::with_mut(|s| {
            s.supported_payments.push(SupportedKind {
                x402_version: X402_VERSION,
                scheme: SCHEME_EXACT.to_string(),
                network: NETWORK_ICP.to_string(),
                extra: None,
            });
            s.supported_assets.insert(
                principal(2),
                AssetInfo {
                    symbol: "TKN".to_string(),
                    decimals: 8,
                    transfer_fee: 10,
                    payment_fee: 100,
                    logo: Some("data:image/png;base64,AAAA".to_string()),
                    ..Default::default()
                },
            );
        });

        let fees = state::verify_requirements(&payment_payload(), now_ms).unwrap();
        assert_eq!((fees.payment_fee, fees.transfer_fee), (100, 10));

        let mut fee_only = payment_payload();
        fee_only.payload.authorization.value = TokenAmount(100);
        assert!(matches!(
            state::verify_requirements(&fee_only, now_ms),
            Err(X402Error::InvalidPayloadAuthorizationValue(_))
        ));
    }

    #[test]
    fn payer_guard_admits_one_settlement_per_payer() {
        let payer = principal(11);
        let guard = PayerGuard::acquire(payer).unwrap();
        assert!(PayerGuard::acquire(payer).is_err());
        // Other payers are not held up.
        let _other = PayerGuard::acquire(principal(12)).unwrap();

        drop(guard);
        let _guard = PayerGuard::acquire(payer).unwrap();
    }
}
