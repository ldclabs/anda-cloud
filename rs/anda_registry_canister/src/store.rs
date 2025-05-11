use anda_cloud_cdk::{TEEInfo, agent::*};
use candid::{CandidType, Principal};
use ciborium::{from_reader, into_writer};
use ic_auth_types::{ByteArrayB64, ByteBufB64, SignedDelegation};
use ic_canister_sig_creation::{
    DELEGATION_SIG_DOMAIN,
    signature_map::{CanisterSigInputs, LABEL_SIG, SignatureMap},
};
use ic_cdk::api::certified_data_set;
use ic_certification::labeled_hash;
use ic_stable_structures::{
    DefaultMemoryImpl, StableBTreeMap, StableCell, Storable,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Clone, CandidType, Default, Deserialize, Serialize)]
pub struct State {
    pub name: String,
    pub max_agent_id: u64,
    pub challenge_expires_in_ms: u64,
    pub peers: BTreeSet<Principal>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct Indexes {
    // handle -> agent_id
    by_handle: BTreeMap<String, u64>,

    // (stake_power, agent_id)
    by_stake_power: BTreeSet<(u64, u64)>,

    // (health_power, agent_id)
    by_health_power: BTreeSet<(u64, u64)>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentLocal {
    id: Principal,
    #[serde(rename = "i")]
    info: AgentInfoLocal,
    #[serde(rename = "c")]
    created_at: u64,
    #[serde(rename = "u")]
    updated_at: u64,
    #[serde(rename = "a")]
    actived_at: u64,
    #[serde(rename = "hp")]
    health_power: u64,
    #[serde(rename = "sp")]
    stake_power: u64,
    #[serde(rename = "cc")]
    challenge_code: ByteArrayB64<32>,
    #[serde(rename = "ca")]
    challenged_at: u64,
    #[serde(rename = "cb")]
    challenged_by: Principal,
    #[serde(rename = "ce")]
    challenged_expiration: u64,
    #[serde(rename = "t")]
    tee: Option<TEEInfoLocal>,
}

impl Storable for AgentLocal {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buf = vec![];
        into_writer(self, &mut buf).expect("failed to encode AgentLocal data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode AgentLocal data")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentInfoLocal {
    #[serde(rename = "h")]
    handle: Option<String>,
    #[serde(rename = "n")]
    name: String,
    #[serde(rename = "d")]
    description: String,
    #[serde(rename = "u")]
    url: String,
    #[serde(rename = "p")]
    protocols: BTreeMap<AgentProtocol, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TEEInfoLocal {
    id: Principal,
    #[serde(rename = "k")]
    kind: String,
    #[serde(rename = "u")]
    url: String,
    #[serde(rename = "a")]
    attestation: ByteBufB64,
}

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
const INDEX_MEMORY_ID: MemoryId = MemoryId::new(1);
const AGENT_MEMORY_ID: MemoryId = MemoryId::new(2);

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
    static INDEX : RefCell<Indexes> = RefCell::new(Indexes::default());


    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static STATE_STORE: RefCell<StableCell<Vec<u8>, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(STATE_MEMORY_ID)),
            Vec::new()
        ).expect("failed to init STATE store")
    );

    static INDEX_STORE: RefCell<StableCell<Vec<u8>, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(INDEX_MEMORY_ID)),
            Vec::new()
        ).expect("failed to init Indexes store")
    );

    static AGENT_STORE: RefCell<StableBTreeMap<u64, AgentLocal, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(AGENT_MEMORY_ID)),
        )
    );
}

pub mod state {
    use super::*;

    pub fn with<R>(f: impl FnOnce(&State) -> R) -> R {
        STATE.with_borrow(f)
    }

    pub fn with_mut<R>(f: impl FnOnce(&mut State) -> R) -> R {
        STATE.with_borrow_mut(f)
    }

    pub fn load() {
        STATE_STORE.with_borrow(|rs| {
            STATE.with_borrow_mut(|r| {
                let v: State =
                    from_reader(&rs.get()[..]).expect("failed to decode STATE_STORE data");
                *r = v;
            });
        });
        INDEX_STORE.with_borrow(|rs| {
            INDEX.with_borrow_mut(|r| {
                let v: Indexes =
                    from_reader(&rs.get()[..]).expect("failed to decode INDEX_STORE data");
                *r = v;
            });
        });
    }

    pub fn save() {
        STATE.with_borrow(|r| {
            STATE_STORE.with_borrow_mut(|rs| {
                let mut buf = vec![];
                into_writer(r, &mut buf).expect("failed to encode STATE data");
                rs.set(buf).expect("failed to set STATE_STORE data");
            });
        });
        INDEX.with_borrow(|r| {
            INDEX_STORE.with_borrow_mut(|rs| {
                let mut buf = vec![];
                into_writer(r, &mut buf).expect("failed to encode INDEX data");
                rs.set(buf).expect("failed to set INDEX_STORE data");
            });
        });
    }
}
