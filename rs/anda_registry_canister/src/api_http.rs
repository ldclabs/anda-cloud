use anda_cloud_cdk::{agent::ChallengeEnvelope, registry::RegistryError, to_cbor_bytes};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use candid::{CandidType, Principal};
use ciborium::from_reader;
use ic_http_certification::{HeaderField, HttpRequest, HttpUpdateRequest};
use serde::Deserialize;
use serde_bytes::ByteBuf;
use url::Url;

use crate::{api, store};

#[derive(CandidType, Deserialize, Clone, Default)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
}

static CBOR: &str = "application/cbor";
static JSON: &str = "application/json";
static IC_CERTIFICATE_HEADER: &str = "ic-certificate";
static IC_CERTIFICATE_EXPRESSION_HEADER: &str = "ic-certificateexpression";

// request url example:
// https://uxrrr-q7777-77774-qaaaq-cai.icp0.io/lookup?handle=abc123
// https://uxrrr-q7777-77774-qaaaq-cai.icp0.io/lookup?id=nprym-ylvyz-ig3fr-lgcmn-zzzt4-tyuix-3v6bm-fsel7-6lq6x-zh2w7-zqe
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
                BASE64.encode(to_cbor_bytes(&witness)),
                BASE64.encode(to_cbor_bytes(
                    &store::state::DEFAULT_EXPR_PATH.to_expr_path()
                ))
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
            }
        }
        Err(err) => {
            headers.push(("content-type".to_string(), "text/plain".to_string()));
            HttpResponse {
                status_code: err.status_code(),
                headers,
                body: err.to_string().into_bytes().into(),
            }
        }
    }
}

#[ic_cdk::update(hidden = true)]
async fn http_request_update(request: HttpUpdateRequest<'static>) -> HttpResponse {
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
                BASE64.encode(to_cbor_bytes(&witness)),
                BASE64.encode(to_cbor_bytes(
                    &store::state::DEFAULT_EXPR_PATH.to_expr_path()
                ))
            ),
        ),
    ];

    let req_url = match parse_url(request.url()) {
        Ok(url) => url,
        Err(err) => {
            return HttpResponse {
                status_code: 400,
                headers,
                body: err.into_bytes().into(),
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
            }
        }
        Err(err) => {
            headers.push(("content-type".to_string(), "text/plain".to_string()));
            HttpResponse {
                status_code: err.status_code(),
                headers,
                body: err.to_string().into_bytes().into(),
            }
        }
    }
}

fn get_state(in_cbor: bool) -> Result<Vec<u8>, RegistryError> {
    let body = store::state::get_state();
    if in_cbor {
        Ok(to_cbor_bytes(&body))
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
                    return Ok(to_cbor_bytes(&agent));
                } else {
                    return serde_json::to_vec(&agent).map_err(|err| RegistryError::Generic {
                        error: format!("failed to serialize agent, error: {err}"),
                    });
                }
            }
            "handle" => {
                let agent = store::agent::get_agent_by_handle(value.to_string())?;
                if in_cbor {
                    return Ok(to_cbor_bytes(&agent));
                } else {
                    return serde_json::to_vec(&agent).map_err(|err| RegistryError::Generic {
                        error: format!("failed to serialize agent, error: {err}"),
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
