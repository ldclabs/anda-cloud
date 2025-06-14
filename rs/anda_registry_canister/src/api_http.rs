use anda_cloud_cdk::{agent::ChallengeEnvelope, registry::RegistryError};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use candid::{CandidType, Principal};
use ciborium::from_reader;
use ic_auth_types::cbor_into_vec;
use ic_http_certification::{HeaderField, HttpRequest, HttpUpdateRequest};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use url::Url;

use crate::{api, store};

#[derive(CandidType, Deserialize, Serialize, Clone, Default)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
    pub upgrade: Option<bool>,
}

// type HttpResponse = record {
//     status_code: nat16;
//     headers: vec HeaderField;
//     body: blob;
//     upgrade : opt bool;
//     streaming_strategy: opt StreamingStrategy;
// };

static CBOR: &str = "application/cbor";
static JSON: &str = "application/json";
static IC_CERTIFICATE_HEADER: &str = "ic-certificate";
static IC_CERTIFICATE_EXPRESSION_HEADER: &str = "ic-certificateexpression";

// request url example:
// https://lfcwh-piaaa-aaaap-an2fa-cai.icp0.io/lookup?handle=abc123
// https://lfcwh-piaaa-aaaap-an2fa-cai.icp0.io/lookup?id=nprym-ylvyz-ig3fr-lgcmn-zzzt4-tyuix-3v6bm-fsel7-6lq6x-zh2w7-zqe
#[ic_cdk::query(hidden = true)]
async fn http_request(request: HttpRequest<'static>) -> HttpResponse {
    let witness = store::state::http_tree_with(|t| {
        t.witness(&store::state::DEFAULT_CERT_ENTRY, request.url())
            .expect("get witness failed")
    });

    let certified_data = ic_cdk::api::data_certificate().expect("no data certificate available");

    let mut headers = vec![
        ("x-content-type-options".to_string(), "nosniff".to_string()),
        (
            IC_CERTIFICATE_EXPRESSION_HEADER.to_string(),
            store::state::DEFAULT_CEL_EXPR.clone(),
        ),
        (
            IC_CERTIFICATE_HEADER.to_string(),
            format!(
                "certificate=:{}:, tree=:{}:, expr_path=:{}:, version=2",
                BASE64.encode(certified_data),
                BASE64.encode(cbor_into_vec(&witness).expect("failed to serialize witness")),
                BASE64.encode(
                    cbor_into_vec(&store::state::DEFAULT_EXPR_PATH.to_expr_path())
                        .expect("failed to serialize expr path")
                )
            ),
        ),
    ];

    let req_url = match parse_url(request.url()) {
        Ok(url) => url,
        Err(err) => {
            headers.push(("content-type".to_string(), "text/plain".to_string()));
            return HttpResponse {
                status_code: 400,
                headers,
                body: err.into_bytes().into(),
                upgrade: None,
            };
        }
    };

    let in_cbor = supports_cbor(request.headers());

    let rt = match (request.method().as_str(), req_url.path()) {
        ("HEAD", _) => Ok(Vec::new()),
        ("GET", "/state") => get_state(in_cbor),
        ("GET", "/lookup") => lookup(req_url, in_cbor),
        (method, path) => Err(RegistryError::NotSupported {
            error: format!("method {method}, path: {path}"),
        }),
    };

    match rt {
        Ok(body) => {
            if in_cbor {
                headers.push(("content-type".to_string(), CBOR.to_string()));
            } else {
                headers.push(("content-type".to_string(), JSON.to_string()));
            }
            headers.push(("content-length".to_string(), body.len().to_string()));
            HttpResponse {
                status_code: 200,
                headers,
                body: body.into(),
                upgrade: None,
            }
        }
        Err(err) => {
            headers.push(("content-type".to_string(), "text/plain".to_string()));
            HttpResponse {
                status_code: err.status_code(),
                headers,
                body: err.to_string().into_bytes().into(),
                upgrade: None,
            }
        }
    }
}

#[ic_cdk::update(hidden = true)]
async fn http_request_update(request: HttpUpdateRequest<'static>) -> HttpResponse {
    let mut headers = vec![("x-content-type-options".to_string(), "nosniff".to_string())];

    let req_url = match parse_url(request.url()) {
        Ok(url) => url,
        Err(err) => {
            return HttpResponse {
                status_code: 400,
                headers,
                body: err.into_bytes().into(),
                upgrade: None,
            };
        }
    };

    let in_cbor = supports_cbor(request.headers());

    let rt = match (request.method().as_str(), req_url.path()) {
        ("POST", "/register") => register(request.body(), in_cbor).await,
        ("POST", "/challenge") => challenge(request.body(), in_cbor).await,
        (method, path) => Err(RegistryError::NotSupported {
            error: format!("method {method}, path: {path}"),
        }),
    };

    match rt {
        Ok(body) => {
            if in_cbor {
                headers.push(("content-type".to_string(), CBOR.to_string()));
            } else {
                headers.push(("content-type".to_string(), JSON.to_string()));
            }
            headers.push(("content-length".to_string(), body.len().to_string()));
            HttpResponse {
                status_code: 200,
                headers,
                body: body.into(),
                upgrade: None,
            }
        }
        Err(err) => {
            headers.push(("content-type".to_string(), "text/plain".to_string()));
            HttpResponse {
                status_code: err.status_code(),
                headers,
                body: err.to_string().into_bytes().into(),
                upgrade: None,
            }
        }
    }
}

fn get_state(in_cbor: bool) -> Result<Vec<u8>, RegistryError> {
    let body = store::state::get_state();
    if in_cbor {
        Ok(cbor_into_vec(&body).map_err(|err| RegistryError::Generic {
            error: format!("failed to serialize state, error: {err}"),
        })?)
    } else {
        serde_json::to_vec(&body).map_err(|err| RegistryError::Generic {
            error: format!("failed to serialize state, error: {err}"),
        })
    }
}

fn lookup(url: Url, in_cbor: bool) -> Result<Vec<u8>, RegistryError> {
    if let Some((key, value)) = url.query_pairs().next() {
        match key.as_ref() {
            "id" => {
                let id = Principal::from_text(value.as_ref()).map_err(|err| {
                    RegistryError::BadRequest {
                        error: format!("invalid id: {value}, error: {err}"),
                    }
                })?;
                let agent = store::agent::get_agent(id)?;
                if in_cbor {
                    return cbor_into_vec(&agent).map_err(|err| RegistryError::Generic {
                        error: format!("failed to serialize agent in CBOR, error: {err}"),
                    });
                } else {
                    return serde_json::to_vec(&agent).map_err(|err| RegistryError::Generic {
                        error: format!("failed to serialize agent in JSON, error: {err}"),
                    });
                }
            }
            "handle" => {
                let agent = store::agent::get_agent_by_handle(value.to_string())?;
                if in_cbor {
                    return cbor_into_vec(&agent).map_err(|err| RegistryError::Generic {
                        error: format!("failed to serialize agent in CBOR, error: {err}"),
                    });
                } else {
                    return serde_json::to_vec(&agent).map_err(|err| RegistryError::Generic {
                        error: format!("failed to serialize agent in JSON, error: {err}"),
                    });
                }
            }
            other => {
                Err(RegistryError::BadRequest {
                    error: format!("invalid query parameter: {other}={value}"),
                })?;
            }
        }
    }

    Err(RegistryError::BadRequest {
        error: "missing query parameter".to_string(),
    })
}

async fn register(body: &[u8], in_cbor: bool) -> Result<Vec<u8>, RegistryError> {
    let envelope: ChallengeEnvelope = if in_cbor {
        from_reader(body).map_err(|err| RegistryError::BadRequest {
            error: format!("failed to decode AgentEnvelope from CBOR, error: {err}"),
        })?
    } else {
        serde_json::from_slice(body).map_err(|err| RegistryError::BadRequest {
            error: format!("failed to decode AgentEnvelope from JSON, error: {err}"),
        })?
    };

    api::register(envelope).await?;
    Ok(Vec::new())
}

async fn challenge(body: &[u8], in_cbor: bool) -> Result<Vec<u8>, RegistryError> {
    let envelope: ChallengeEnvelope = if in_cbor {
        from_reader(body).map_err(|err| RegistryError::BadRequest {
            error: format!("failed to decode AgentEnvelope from CBOR, error: {err}"),
        })?
    } else {
        serde_json::from_slice(body).map_err(|err| RegistryError::BadRequest {
            error: format!("failed to decode AgentEnvelope from JSON, error: {err}"),
        })?
    };

    api::challenge(envelope).await?;
    Ok(Vec::new())
}

fn parse_url(s: &str) -> Result<Url, String> {
    let url = if s.starts_with('/') {
        Url::parse(format!("http://localhost{}", s).as_str())
    } else {
        Url::parse(s)
    };
    url.map_err(|err| format!("failed to parse url {s}, error: {err}"))
}

fn supports_cbor(headers: &[HeaderField]) -> bool {
    headers
        .iter()
        .any(|(name, value)| (name == "accept" || name == "content-type") && value.contains(CBOR))
}
