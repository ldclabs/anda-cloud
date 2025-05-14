use anda_cloud_cdk::{
    PaymentProtocol, TEEInfo, TEEKind,
    agent::*,
    registry::{RegistryError, RegistryState},
};
use candid::{CandidType, Principal};
use ciborium::{from_reader, into_writer};
use ic_auth_types::ByteArrayB64;
use ic_http_certification::{
    HttpCertification, HttpCertificationPath, HttpCertificationTree, HttpCertificationTreeEntry,
    cel::{DefaultCelBuilder, create_cel_expr},
};
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

use crate::{call, notify, rand_bytes};

const MAX_LAST_CHALLENGED: usize = 10000;
const MAX_HEALTH_POWER_LIST: usize = 1000;

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Clone, CandidType, Default, Deserialize, Serialize)]
pub struct State {
    pub name: String,
    pub max_agent: u64,
    pub challenge_expires_in_ms: u64,
    pub peers: BTreeSet<Principal>,
    pub challengers: BTreeSet<Principal>,
    pub subscribers: BTreeSet<Principal>,
    pub name_canisters: BTreeSet<Principal>,
    pub governance_canister: Option<Principal>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct Indexes {
    // agent_id -> (agent_idx, challenged_at)
    id_map: BTreeMap<Principal, (u64, u64)>,

    // handle -> agent_idx
    by_handle: BTreeMap<String, u64>,

    // (health_power, agent_idx), size <= 1000
    by_health_power: BTreeSet<(u64, u64)>,

    // (challenged_at, agent_id), size <= 10000
    last_challenged: BTreeSet<(u64, Principal)>,

    health_power_threshold: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentLocal {
    id: Principal,

    #[serde(rename = "i")]
    info: AgentInfoLocal,

    #[serde(rename = "c")]
    created_at: u64,

    #[serde(rename = "a")]
    actived_start: u64,

    #[serde(rename = "hp")]
    health_power: u64,

    #[serde(rename = "cc")]
    challenge_code: ByteArrayB64<16>,

    #[serde(rename = "ca")]
    challenged_at: u64,

    #[serde(rename = "cb")]
    challenged_by: Principal,

    #[serde(rename = "ce")]
    challenged_expiration: u64,

    #[serde(rename = "t")]
    tee: Option<TEEInfoLocal>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentInfoLocal {
    #[serde(rename = "h")]
    handle: Option<(Principal, String)>,

    #[serde(rename = "n")]
    name: String,

    #[serde(rename = "d")]
    description: String,

    #[serde(rename = "e")]
    endpoint: String,

    #[serde(rename = "p")]
    protocols: BTreeMap<AgentProtocol, String>,

    #[serde(rename = "pm")]
    payments: BTreeSet<PaymentProtocol>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TEEInfoLocal {
    id: Principal,

    #[serde(rename = "k")]
    kind: TEEKind,

    #[serde(rename = "u")]
    url: String,
}

impl From<AgentInfo> for AgentInfoLocal {
    fn from(info: AgentInfo) -> Self {
        Self {
            handle: info.handle,
            name: info.name,
            description: info.description,
            endpoint: info.endpoint,
            protocols: info.protocols,
            payments: info.payments,
        }
    }
}

impl From<AgentInfoLocal> for AgentInfo {
    fn from(info: AgentInfoLocal) -> Self {
        Self {
            handle: info.handle,
            name: info.name,
            description: info.description,
            endpoint: info.endpoint,
            protocols: info.protocols,
            payments: info.payments,
        }
    }
}

impl From<TEEInfo> for TEEInfoLocal {
    fn from(info: TEEInfo) -> Self {
        Self {
            id: info.id,
            kind: info.kind,
            url: info.url,
        }
    }
}

impl From<TEEInfoLocal> for TEEInfo {
    fn from(info: TEEInfoLocal) -> Self {
        Self {
            id: info.id,
            kind: info.kind,
            url: info.url,
            attestation: None,
        }
    }
}

impl From<AgentLocal> for Agent {
    fn from(agent: AgentLocal) -> Self {
        Self {
            id: agent.id,
            info: agent.info.into(),
            created_at: agent.created_at,
            actived_start: agent.actived_start,
            health_power: agent.health_power,
            challenge_code: agent.challenge_code,
            challenged_at: agent.challenged_at,
            challenged_by: agent.challenged_by,
            challenged_expiration: agent.challenged_expiration,
            tee: agent.tee.map(|t| t.into()),
        }
    }
}

impl From<Agent> for AgentLocal {
    fn from(agent: Agent) -> Self {
        Self {
            id: agent.id,
            info: agent.info.into(),
            created_at: agent.created_at,
            actived_start: agent.actived_start,
            health_power: agent.health_power,
            challenge_code: agent.challenge_code,
            challenged_at: agent.challenged_at,
            challenged_by: agent.challenged_by,
            challenged_expiration: agent.challenged_expiration,
            tee: agent.tee.map(|t| t.into()),
        }
    }
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

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
const INDEX_MEMORY_ID: MemoryId = MemoryId::new(1);
const AGENT_MEMORY_ID: MemoryId = MemoryId::new(2);

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
    static INDEX : RefCell<Indexes> = RefCell::new(Indexes::default());
    static HTTP_TREE: RefCell<HttpCertificationTree> = RefCell::new(HttpCertificationTree::default());


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
    use lazy_static::lazy_static;
    use once_cell::sync::Lazy;

    lazy_static! {
        pub static ref DEFAULT_EXPR_PATH: HttpCertificationPath<'static> =
            HttpCertificationPath::wildcard("");
        pub static ref DEFAULT_CERTIFICATION: HttpCertification = HttpCertification::skip();
        pub static ref DEFAULT_CEL_EXPR: String =
            create_cel_expr(&DefaultCelBuilder::skip_certification());
    }

    // https://github.com/ldclabs/ic-panda/blob/main/src/ic_message_types/src/profile.rs#L15
    #[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
    struct UserInfo {
        pub id: Principal,
        pub name: String,
        pub image: String,
        pub profile_canister: Principal,
        pub cose_canister: Option<Principal>,
        pub username: Option<String>,
    }

    pub static DEFAULT_CERT_ENTRY: Lazy<HttpCertificationTreeEntry> =
        Lazy::new(|| HttpCertificationTreeEntry::new(&*DEFAULT_EXPR_PATH, *DEFAULT_CERTIFICATION));

    pub fn with<R>(f: impl FnOnce(&State) -> R) -> R {
        STATE.with_borrow(f)
    }

    pub fn with_mut<R>(f: impl FnOnce(&mut State) -> R) -> R {
        STATE.with_borrow_mut(f)
    }

    pub fn is_controller(caller: &Principal) -> bool {
        STATE.with_borrow(|s| s.governance_canister.as_ref() == Some(caller))
    }

    pub fn is_challenger(caller: &Principal) -> bool {
        STATE.with_borrow(|s| s.challengers.contains(caller))
    }

    pub fn get_state() -> RegistryState {
        STATE.with_borrow(|s| RegistryState {
            name: s.name.clone(),
            max_agent: s.max_agent,
            agents_total: INDEX.with_borrow(|rs| rs.id_map.len() as u64),
            challenge_expires_in_ms: s.challenge_expires_in_ms,
            governance_canister: s.governance_canister,
            challengers: s.challengers.clone(),
            peers: s.peers.clone(),
            name_canisters: s.name_canisters.clone(),
            subscribers: s.subscribers.clone(),
        })
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

    pub fn notify_subscribers(event: AgentEvent) {
        let subscribers = STATE.with_borrow(|s| s.subscribers.clone());
        if subscribers.is_empty() {
            return;
        }

        ic_cdk::futures::spawn(async move {
            for subscriber in subscribers {
                let _ = notify(subscriber, AGENT_EVENT_API, &event).await;
            }
        });
    }

    pub async fn check_handle(
        canister: Principal,
        handle: String,
        owner: Principal,
    ) -> Result<(), RegistryError> {
        STATE.with_borrow(|s| {
            if !s.name_canisters.contains(&canister) {
                return Err(RegistryError::BadRequest {
                    error: "invalid handle canister".to_string(),
                });
            }
            Ok(())
        })?;

        // https://github.com/ldclabs/ic-panda/blob/main/src/ic_message/src/api_query.rs#L83
        let rt: Result<UserInfo, String> = call(canister, "get_by_username", &(handle.clone(),), 0)
            .await
            .map_err(|error| RegistryError::Generic { error })?;

        if let Ok(user) = rt {
            if user.id == owner {
                return Ok(());
            }
        }

        Err(RegistryError::BadRequest {
            error: format!("handle {handle:?} is not belong to {owner}"),
        })
    }
}

pub mod agent {
    use super::*;

    pub async fn register(
        id: Principal,
        challenged_by: Principal,
        info: AgentInfo,
        tee: Option<TEEInfo>,
        now_ms: u64,
    ) -> Result<(), RegistryError> {
        let code = rand_bytes::<16>()
            .await
            .map_err(|error| RegistryError::Generic { error })?;
        INDEX.with_borrow_mut(|ri| {
            if ri.id_map.contains_key(&id) {
                return Err(RegistryError::AlreadyExists {
                    handle: id.to_string(),
                });
            }
            let (idx, challenge_expires_in_ms) = state::with_mut(|s| {
                let idx = s.max_agent;
                s.max_agent += 1;
                (idx, s.challenge_expires_in_ms)
            });

            ri.id_map.insert(id, (idx, now_ms));
            if let Some((_, handle)) = &info.handle {
                ri.by_handle.insert(handle.clone(), idx);
            }

            AGENT_STORE.with_borrow_mut(|ra| {
                let agent = AgentLocal {
                    id,
                    info: info.into(),
                    created_at: now_ms,
                    actived_start: now_ms,
                    health_power: 0,
                    challenge_code: code.into(),
                    challenged_at: now_ms,
                    challenged_by,
                    challenged_expiration: now_ms + challenge_expires_in_ms,
                    tee: tee.map(|t| t.into()),
                };
                ra.insert(idx, agent.clone());
            });

            Ok(())
        })
    }

    pub async fn challenge(
        id: Principal,
        challenged_by: Principal,
        code: ByteArrayB64<16>,
        info: AgentInfo,
        tee: Option<TEEInfo>,
        now_ms: u64,
    ) -> Result<(), RegistryError> {
        let new_code = rand_bytes::<16>()
            .await
            .map_err(|error| RegistryError::Generic { error })?;
        INDEX.with_borrow_mut(|ri| {
            let (idx, challenged_at) =
                ri.id_map
                    .get_mut(&id)
                    .ok_or_else(|| RegistryError::NotFound {
                        handle: id.to_string(),
                    })?;

            let challenge_expires_in_ms = state::with(|s| s.challenge_expires_in_ms);

            AGENT_STORE.with_borrow_mut(|ra| {
                let mut agent = ra.get(idx).ok_or_else(|| RegistryError::NotFound {
                    handle: id.to_string(),
                })?;
                if code != agent.challenge_code {
                    return Err(RegistryError::BadRequest {
                        error: format!(
                            "challenge code is not match, expect {}, got {}",
                            agent.challenge_code, code
                        ),
                    });
                }

                if now_ms <= agent.challenged_at {
                    return Ok(());
                }

                if let Some((_, handle)) = &info.handle {
                    ri.by_handle.insert(handle.clone(), *idx);
                }

                ri.by_health_power.remove(&(agent.health_power, *idx));
                if now_ms > agent.challenged_expiration {
                    // 之前的挑战已过期，进行惩罚
                    // 1. 重置激活时间
                    // 2. 减少健康权重
                    agent.actived_start = now_ms;
                    agent.health_power = agent
                        .health_power
                        .saturating_sub(now_ms - agent.challenged_at);
                } else {
                    agent.health_power += now_ms - agent.challenged_at;
                }

                if agent.health_power > ri.health_power_threshold {
                    ri.by_health_power.insert((agent.health_power, *idx));
                    if ri.by_health_power.len() > MAX_HEALTH_POWER_LIST {
                        for _ in 0..100 {
                            ri.by_health_power.pop_first();
                        }
                        ri.health_power_threshold = ri.by_health_power.first().unwrap().0;
                    }
                }

                ri.last_challenged.insert((now_ms, id));
                if ri.last_challenged.len() > MAX_LAST_CHALLENGED {
                    for _ in 0..100 {
                        // remove 100 oldest challenged agent
                        ri.last_challenged.pop_first();
                    }
                }

                *challenged_at = now_ms;
                agent.challenge_code = new_code.into();
                agent.info = info.into();
                agent.tee = tee.map(|t| t.into());
                agent.challenged_at = now_ms;
                agent.challenged_by = challenged_by;
                agent.challenged_expiration = now_ms + challenge_expires_in_ms;

                ra.insert(*idx, agent);

                Ok(())
            })
        })
    }

    pub fn get_agent(id: Principal) -> Result<Agent, RegistryError> {
        let agent = INDEX.with_borrow(|ri| {
            let (idx, _) = ri.id_map.get(&id).ok_or_else(|| RegistryError::NotFound {
                handle: id.to_string(),
            })?;
            AGENT_STORE.with_borrow(|ra| {
                ra.get(idx).ok_or_else(|| RegistryError::NotFound {
                    handle: id.to_string(),
                })
            })
        })?;

        Ok(agent.into())
    }

    pub fn get_agent_by_handle(handle: String) -> Result<Agent, RegistryError> {
        INDEX.with_borrow(|ri| {
            let idx = ri
                .by_handle
                .get(&handle)
                .ok_or_else(|| RegistryError::NotFound {
                    handle: handle.clone(),
                })?;
            AGENT_STORE
                .with_borrow(|ra| ra.get(idx).ok_or(RegistryError::NotFound { handle }))
                .map(|a| a.into())
        })
    }

    pub fn list(prev: Option<u64>, take: usize) -> Result<(u64, Vec<Agent>), RegistryError> {
        let max_id = state::with(|s| s.max_agent);
        let mut id = prev.unwrap_or(max_id).min(max_id);
        AGENT_STORE.with_borrow(|ra| {
            let mut agents = Vec::with_capacity(take);
            loop {
                if let Some(agent) = ra.get(&id) {
                    agents.push(agent.into());
                    if agents.len() >= take {
                        break;
                    }
                }
                if id == 0 {
                    break;
                }
                id -= 1;
            }

            Ok((id, agents))
        })
    }

    pub fn list_by_health_power(take: usize, now_ms: u64) -> Result<Vec<Agent>, RegistryError> {
        INDEX.with_borrow(|ri| {
            AGENT_STORE.with_borrow(|ra| {
                let mut agents = Vec::with_capacity(take);
                let iter = ri.by_health_power.iter().rev();
                for (_, idx) in iter {
                    if let Some(agent) = ra.get(idx) {
                        if agent.challenged_expiration <= now_ms {
                            continue;
                        }

                        agents.push(agent.into());
                        if agents.len() >= take {
                            break;
                        }
                    }
                }

                Ok(agents)
            })
        })
    }

    pub fn last_challenged(take: usize) -> Result<BTreeMap<Principal, u64>, RegistryError> {
        INDEX.with_borrow(|ri| {
            let mut rt = BTreeMap::new();
            let iter = ri.last_challenged.iter().rev();
            for (challenged_at, id) in iter.take(take) {
                rt.insert(*id, *challenged_at);
            }
            Ok(rt)
        })
    }
}
