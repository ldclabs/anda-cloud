use anda_cloud_cdk::{
    agent::{Agent, AgentInfo, AgentProtocol, ChallengeEnvelope, ChallengeRequest},
    registry::{RegistryError, RegistryState},
};
use candid::{
    CandidType, Principal, decode_one, encode_one,
    utils::{ArgumentEncoder, encode_args_ref},
};
use ed25519_consensus::SigningKey;
use ic_agent::{Identity, identity::BasicIdentity};
use ic_auth_verifier::envelope::SignedEnvelope;
use ic_http_certification::{HeaderField, HttpRequest, Method};
use ic_stable_structures::Storable;
use pocket_ic::{PocketIc, PocketIcBuilder};
use rand::{CryptoRng, Rng, RngCore};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::{collections::BTreeMap, env, ops::Add, path::Path, time::Duration};

#[derive(CandidType, Deserialize, Serialize, Clone, Default)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
    pub upgrade: Option<bool>,
}

// run `make build-wasm` to build the wasm
// run `cargo test --test anda_registry_canister` to run the test
#[test]
fn anda_registry_canister_should_work() {
    println!("POCKET_IC_BIN: {}", env!("POCKET_IC_BIN"));

    let challenger_id = new_identity();
    let agent_id = new_identity();
    let caller = challenger_id.sender().unwrap();
    let can = TestCanister::new::<()>("anda_registry_canister", None, Some(caller));
    let rt: Result<RegistryState, RegistryError> = can.query(caller, "get_state", &());
    println!("RegistryState: {:?}", rt.unwrap());

    let mut request = ChallengeRequest {
        registry: can.canister,
        code: [0u8; 16].into(),
        agent: AgentInfo {
            name: "test_agent".to_string(),
            description: "test agent".to_string(),
            endpoint: "https://test.agent/endpoint".to_string(),
            protocols: BTreeMap::from([(
                AgentProtocol::ANDA,
                format!(
                    "https://test.agent/.well-known/agents/{}",
                    agent_id.sender().unwrap()
                ),
            )]),
            ..Default::default()
        },
        authentication: None,
    };
    let digest = request.core_digest();
    request.authentication =
        Some(SignedEnvelope::sign_digest(&challenger_id, digest.into()).unwrap());
    let digest = request.digest();
    let envelope = ChallengeEnvelope {
        request,
        authentication: SignedEnvelope::sign_digest(&agent_id, digest.into()).unwrap(),
        tee: None,
    };

    let rt: Result<(), RegistryError> = can.update(caller, "register", &(&envelope,));
    assert!(rt.is_err());
    assert_eq!(
        rt.unwrap_err(),
        RegistryError::Forbidden {
            error: format!("challenger {} has no permission", caller),
        }
    );

    // Add the caller as a challenger
    let rt: Result<(), String> = can.update(caller, "admin_add_challengers", &(vec![caller],));
    assert!(rt.is_ok());

    let rt: Result<(), RegistryError> = can.update(caller, "register", &(&envelope,));
    assert!(rt.is_ok());

    let rt: Result<Agent, RegistryError> =
        can.query(caller, "get_agent", &(agent_id.sender().unwrap(),));
    let agent = rt.unwrap();
    println!("Agent: {:?}", agent);
    assert_eq!(agent.id, agent_id.sender().unwrap());
    assert_eq!(agent.info.name, "test_agent");
    assert!(agent.health_power == 0);

    let time = can.pic.get_time();
    let time = time.add(Duration::from_millis(1000));
    can.pic.set_time(time);

    // challenge with wrong code
    {
        let mut request = ChallengeRequest {
            registry: can.canister,
            code: [0u8; 16].into(),
            agent: AgentInfo {
                name: "Anda".to_string(),
                description: "test agent".to_string(),
                endpoint: "https://test.agent/endpoint".to_string(),
                protocols: BTreeMap::from([(
                    AgentProtocol::ANDA,
                    format!(
                        "https://test.agent/.well-known/agents/{}",
                        agent_id.sender().unwrap()
                    ),
                )]),
                ..Default::default()
            },
            authentication: None,
        };
        let digest = request.core_digest();
        request.authentication =
            Some(SignedEnvelope::sign_digest(&challenger_id, digest.into()).unwrap());
        let digest = request.digest();
        let envelope = ChallengeEnvelope {
            request,
            authentication: SignedEnvelope::sign_digest(&agent_id, digest.into()).unwrap(),
            tee: None,
        };

        let rt: Result<(), RegistryError> = can.update(caller, "challenge", &(&envelope,));
        assert!(rt.is_err());
        assert!(matches!(rt.unwrap_err(), RegistryError::BadRequest { .. }));
    }

    // challenge with correct code
    {
        let mut request = ChallengeRequest {
            registry: can.canister,
            code: agent.challenge_code,
            agent: AgentInfo {
                name: "Anda".to_string(),
                description: "test agent".to_string(),
                endpoint: "https://test.agent/endpoint".to_string(),
                protocols: BTreeMap::from([(
                    AgentProtocol::ANDA,
                    format!(
                        "https://test.agent/.well-known/agents/{}",
                        agent_id.sender().unwrap()
                    ),
                )]),
                ..Default::default()
            },
            authentication: None,
        };
        let digest = request.core_digest();
        request.authentication =
            Some(SignedEnvelope::sign_digest(&challenger_id, digest.into()).unwrap());
        let digest = request.digest();
        let envelope = ChallengeEnvelope {
            request,
            authentication: SignedEnvelope::sign_digest(&agent_id, digest.into()).unwrap(),
            tee: None,
        };

        let rt: Result<(), RegistryError> = can.update(caller, "challenge", &(&envelope,));
        assert!(rt.is_ok());

        let rt: Result<Agent, RegistryError> =
            can.query(caller, "get_agent", &(agent_id.sender().unwrap(),));
        let agent = rt.unwrap();
        assert_eq!(agent.info.name, "Anda");
        assert!(agent.health_power >= 1000);
    }

    // HTTP API
    {
        // in JSON format
        let req = HttpRequest::builder()
            .with_method(Method::GET)
            .with_url(format!("/lookup?id={}", agent_id.sender().unwrap()))
            .with_headers(vec![("accept".into(), "application/json".into())])
            .build();
        let rt: HttpResponse = can.query(caller, "http_request", &(req, true));
        assert_eq!(rt.status_code, 200);
        assert!(
            rt.headers
                .iter()
                .any(|h| { h.0 == "content-type" && h.1 == "application/json" })
        );
        let agent: Agent = serde_json::from_slice(&rt.body).unwrap();
        assert_eq!(agent.info.name, "Anda");
        assert!(agent.health_power >= 1000);

        let time = can.pic.get_time();
        let time = time.add(Duration::from_millis(1000));
        can.pic.set_time(time);

        // challenge in JSON format
        {
            let mut request = ChallengeRequest {
                registry: can.canister,
                code: agent.challenge_code,
                agent: AgentInfo {
                    name: "Anda 2".to_string(),
                    description: "test agent".to_string(),
                    endpoint: "https://test.agent/endpoint".to_string(),
                    protocols: BTreeMap::from([(
                        AgentProtocol::ANDA,
                        format!(
                            "https://test.agent/.well-known/agents/{}",
                            agent_id.sender().unwrap()
                        ),
                    )]),
                    ..Default::default()
                },
                authentication: None,
            };
            let digest = request.core_digest();
            request.authentication =
                Some(SignedEnvelope::sign_digest(&challenger_id, digest.into()).unwrap());
            let digest = request.digest();
            let envelope = ChallengeEnvelope {
                request,
                authentication: SignedEnvelope::sign_digest(&agent_id, digest.into()).unwrap(),
                tee: None,
            };

            let envelope = serde_json::to_string(&envelope).unwrap();
            println!("envelope: {}", envelope);

            let req = HttpRequest::builder()
                .with_method(Method::POST)
                .with_url("/challenge".to_string())
                .with_headers(vec![("content-type".into(), "application/json".into())])
                .with_body(envelope.to_bytes())
                .build_update();
            let rt: HttpResponse = can.update(caller, "http_request_update", &(&req,));
            assert_eq!(rt.status_code, 200);
        }

        // in CBOR format
        let req = HttpRequest::builder()
            .with_method(Method::GET)
            .with_url(format!("/lookup?id={}", agent_id.sender().unwrap()))
            .with_headers(vec![("accept".into(), "application/cbor".into())])
            .build();
        let rt: HttpResponse = can.query(caller, "http_request", &(req, true));
        assert_eq!(rt.status_code, 200);
        assert!(
            rt.headers
                .iter()
                .any(|h| { h.0 == "content-type" && h.1 == "application/cbor" })
        );
        let agent: Agent = ciborium::from_reader(&rt.body[..]).unwrap();
        assert_eq!(agent.info.name, "Anda 2");
        assert!(agent.health_power >= 2000);
    }
}

struct TestCanister {
    pic: PocketIc,
    canister: Principal,
}

impl TestCanister {
    fn new<In>(name: &str, init_arg: Option<In>, controller: Option<Principal>) -> Self
    where
        In: CandidType,
    {
        let pic = PocketIcBuilder::new()
            .with_application_subnet()
            .with_ii_subnet()
            .with_fiduciary_subnet()
            .with_nonmainnet_features(true)
            .build();

        let canister = pic.create_canister();
        pic.add_cycles(canister, 2_000_000_000_000);
        if let Some(controller) = controller {
            let mut controllers = pic.get_controllers(canister);
            controllers.push(controller);
            pic.set_controllers(canister, None, controllers).unwrap();
        }

        let wasm_bytes = load_canister_wasm(name);
        let arg = encode_one(init_arg).expect("failed to encode init args");
        pic.install_canister(canister, wasm_bytes, arg, None);

        // Make sure the canister is properly initialized
        fast_forward(&pic, 5);

        Self { pic, canister }
    }

    fn update<In, Out>(&self, caller: Principal, method_name: &str, args: &In) -> Out
    where
        In: ArgumentEncoder + Send,
        Out: candid::CandidType + for<'a> candid::Deserialize<'a>,
    {
        let args = encode_args_ref(args).expect("failed to encode args");
        let reply = self
            .pic
            .update_call(self.canister, caller, method_name, args);
        match reply {
            Ok(data) => decode_one(&data).expect("failed to decode reply"),
            Err(user_error) => panic!("canister returned a user error: {user_error}"),
        }
    }

    fn query<In, Out>(&self, caller: Principal, method_name: &str, args: &In) -> Out
    where
        In: ArgumentEncoder + Send,
        Out: candid::CandidType + for<'a> candid::Deserialize<'a>,
    {
        let args = encode_args_ref(args).expect("failed to encode args");
        let reply = self
            .pic
            .query_call(self.canister, caller, method_name, args);
        match reply {
            Ok(data) => decode_one(&data).expect("failed to decode reply"),
            Err(user_error) => panic!("canister returned a user error: {user_error}"),
        }
    }
}

fn load_canister_wasm(name: &str) -> Vec<u8> {
    let wasm_path_string = format!(
        "{}/target/wasm32-unknown-unknown/release/{name}.wasm",
        git_root_dir()
    );
    let wasm_path = Path::new(&wasm_path_string);
    std::fs::read(wasm_path)
        .expect("wasm does not exist - run `cargo build --release --target wasm32-unknown-unknown`")
}

pub fn random_self_authenticating_principal<R: Rng + CryptoRng>(rng: &mut R) -> Principal {
    let mut fake_public_key = vec![0u8; 32];
    rng.fill_bytes(&mut fake_public_key);
    Principal::self_authenticating::<&[u8]>(fake_public_key.as_ref())
}

fn fast_forward(ic: &PocketIc, ticks: u64) {
    for _ in 0..ticks - 1 {
        ic.tick();
    }
}

fn git_root_dir() -> String {
    // let manifest_dir = env!("CARGO_MANIFEST_DIR");
    use std::process::Command;
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout)
        .expect("Invalid UTF-8 output")
        .trim()
        .to_string()
}

fn rand_bytes<const N: usize>() -> [u8; N] {
    let mut rng = rand::rng();
    let mut bytes = [0u8; N];
    rng.fill_bytes(&mut bytes);
    bytes
}

fn new_identity() -> impl Identity {
    let secret: [u8; 32] = rand_bytes();
    let signing_key = SigningKey::from(secret);
    BasicIdentity::from_signing_key(signing_key)
}
