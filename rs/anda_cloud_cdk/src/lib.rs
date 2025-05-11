use candid::{CandidType, Principal};
use ciborium::into_writer;
use ic_auth_types::ByteBufB64;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

pub mod agent;

pub fn to_cbor_bytes(obj: &impl Serialize) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    into_writer(obj, &mut buf).expect("failed to encode in CBOR format");
    buf
}

pub fn sha3_256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct TEEInfo {
    pub id: Principal,
    pub kind: String,
    // (e.g. https://DOMAIN/.well-known/tee.json)
    pub url: String,
    pub attestation: ByteBufB64,
}
