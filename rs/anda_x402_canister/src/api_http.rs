use anda_cloud_cdk::x402::{SettleResponse, VerifyResponse, X402Error, X402Request};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use candid::{CandidType, Principal};
use ciborium::from_reader;
use ic_auth_types::cbor_into_vec;
use ic_http_certification::{HeaderField, HttpRequest, HttpUpdateRequest};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use crate::store;

#[derive(CandidType, Deserialize, Serialize, Clone, Default)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
    pub upgrade: Option<bool>,
}

struct HttpError {
    status_code: u16,
    message: String,
}

static CBOR: &str = "application/cbor";
static JSON: &str = "application/json";
static IC_CERTIFICATE_HEADER: &str = "ic-certificate";
static IC_CERTIFICATE_EXPRESSION_HEADER: &str = "ic-certificateexpression";

#[ic_cdk::query(hidden = true)]
async fn http_request(request: HttpRequest<'static>) -> HttpResponse {
    let req_path = request.get_path();
    if request.method().as_str() == "POST"
        && let Ok(path) = req_path.as_ref()
        && path == "/settle"
    {
        return HttpResponse {
            status_code: 200,
            headers: vec![],
            body: b"Upgrade".to_vec().into(),
            upgrade: Some(true),
        };
    }

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

    let req_path = match req_path {
        Ok(path) => path,
        Err(err) => {
            headers.push(("content-type".to_string(), "text/plain".to_string()));
            return HttpResponse {
                status_code: 400,
                headers,
                body: err.to_string().into_bytes().into(),
                upgrade: None,
            };
        }
    };

    let in_cbor = supports_cbor(request.headers());

    let rt = match (request.method().as_str(), req_path.as_str()) {
        ("HEAD", _) => Ok(Vec::new()),
        ("GET", "/") => get_info(in_cbor),
        ("GET", "/supported") => get_supported(in_cbor),
        ("POST", "/verify") => post_verify(request.body(), in_cbor).await,
        (method, path) => Err(HttpError {
            status_code: 404,
            message: format!("method {method}, path: {path}"),
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
                status_code: err.status_code,
                headers,
                body: err.message.into_bytes().into(),
                upgrade: None,
            }
        }
    }
}

#[ic_cdk::update(hidden = true)]
async fn http_request_update(request: HttpUpdateRequest<'static>) -> HttpResponse {
    let mut headers = vec![("x-content-type-options".to_string(), "nosniff".to_string())];

    let req_path = match request.get_path() {
        Ok(path) => path,
        Err(err) => {
            headers.push(("content-type".to_string(), "text/plain".to_string()));
            return HttpResponse {
                status_code: 400,
                headers,
                body: err.to_string().into_bytes().into(),
                upgrade: None,
            };
        }
    };

    let in_cbor = supports_cbor(request.headers());

    let rt = match (request.method().as_str(), req_path.as_str()) {
        ("POST", "/settle") => post_settle(request.body(), in_cbor).await,
        (method, path) => Err(HttpError {
            status_code: 404,
            message: format!("method {method}, path: {path}"),
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
                status_code: err.status_code,
                headers,
                body: err.message.into_bytes().into(),
                upgrade: None,
            }
        }
    }
}

fn get_supported(in_cbor: bool) -> Result<Vec<u8>, HttpError> {
    let body = store::state::supported();
    if in_cbor {
        cbor_into_vec(&body).map_err(|err| HttpError {
            status_code: 500,
            message: format!("failed to serialize supported payments, error: {err}"),
        })
    } else {
        serde_json::to_vec(&body).map_err(|err| HttpError {
            status_code: 500,
            message: format!("failed to serialize supported payments, error: {err}"),
        })
    }
}

fn get_info(in_cbor: bool) -> Result<Vec<u8>, HttpError> {
    let body = store::state::info();
    if in_cbor {
        cbor_into_vec(&body).map_err(|err| HttpError {
            status_code: 500,
            message: format!("failed to serialize state, error: {err}"),
        })
    } else {
        serde_json::to_vec(&body).map_err(|err| HttpError {
            status_code: 500,
            message: format!("failed to serialize state, error: {err}"),
        })
    }
}

async fn post_verify(body: &[u8], in_cbor: bool) -> Result<Vec<u8>, HttpError> {
    let canister_self = ic_cdk::api::canister_self();
    let now_ms = ic_cdk::api::time() / 1_000_000;

    let req = match decode_payment(body, in_cbor) {
        Ok(req) => verify_payment(canister_self, req, now_ms).await,
        Err(err) => Err((err, None)),
    };
    let res = match req {
        Ok(payer) => VerifyResponse::valid(payer.to_string()),
        Err((err, maybe_payer)) => {
            let payer_str = maybe_payer.map(|p| p.to_string());
            VerifyResponse::invalid(payer_str, err)
        }
    };

    if in_cbor {
        cbor_into_vec(&res).map_err(|err| HttpError {
            status_code: 500,
            message: format!("failed to serialize VerifyResponse, error: {err}"),
        })
    } else {
        serde_json::to_vec(&res).map_err(|err| HttpError {
            status_code: 500,
            message: format!("failed to serialize VerifyResponse, error: {err}"),
        })
    }
}

async fn post_settle(body: &[u8], in_cbor: bool) -> Result<Vec<u8>, HttpError> {
    let canister_self = ic_cdk::api::canister_self();
    let now_ms = ic_cdk::api::time() / 1_000_000;
    let network = "icp".to_string();

    let req = match decode_payment(body, in_cbor) {
        Ok(req) => settle_payment(canister_self, req, now_ms).await,
        Err(err) => Err((err, None)),
    };
    let res = match req {
        Ok((payer, transaction)) => SettleResponse {
            success: true,
            error_reason: None,
            payer: Some(payer.to_string()),
            transaction,
            network,
        },
        Err((err, maybe_payer)) => {
            let payer = maybe_payer.map(|p| p.to_string());
            SettleResponse {
                success: false,
                error_reason: Some(err.to_string()),
                payer,
                transaction: "".to_string(),
                network,
            }
        }
    };

    if in_cbor {
        cbor_into_vec(&res).map_err(|err| HttpError {
            status_code: 500,
            message: format!("failed to serialize VerifyResponse, error: {err}"),
        })
    } else {
        serde_json::to_vec(&res).map_err(|err| HttpError {
            status_code: 500,
            message: format!("failed to serialize VerifyResponse, error: {err}"),
        })
    }
}

fn decode_payment(body: &[u8], in_cbor: bool) -> Result<X402Request, X402Error> {
    let req: X402Request = if in_cbor {
        from_reader(body)
            .map_err(|err| X402Error::InvalidPayload(format!("failed to decode cbor: {err}")))?
    } else {
        serde_json::from_slice(body)
            .map_err(|err| X402Error::InvalidPayload(format!("failed to decode json: {err}")))?
    };

    req.validate()?;
    Ok(req)
}

async fn verify_payment(
    canister_self: Principal,
    req: X402Request,
    now_ms: u64,
) -> Result<Principal, (X402Error, Option<Principal>)> {
    let payer = req
        .payment_payload
        .payload
        .verify_signature(now_ms, Some(canister_self))
        .map_err(|err| (X402Error::InvalidPayloadSignature(err), None))?;

    let _ = store::state::verify_payload(payer, &req.payment_payload, now_ms)
        .map_err(|err| (err, Some(payer)))?;
    // `check_funds` relies on Inter-canister calls, which leads to expensive update calls.
    // To optimize the verification process, we skip the fund checking here.
    // let required_amount = req
    //     .payment_payload
    //     .payload
    //     .authorization
    //     .value
    //     .0
    //     .checked_add(asset_info.transfer_fee)
    //     .ok_or_else(|| {
    //         (
    //             X402Error::InvalidPayload("payment amount overflow".to_string()),
    //             Some(payer),
    //         )
    //     })?;
    //
    // store::state::check_funds(
    //     payer,
    //     canister_self,
    //     req.payment_requirements.asset,
    //     required_amount,
    //     now_ms,
    // )
    // .await
    // .map_err(|err| (err, Some(payer)))?;
    Ok(payer)
}

async fn settle_payment(
    canister_self: Principal,
    req: X402Request,
    now_ms: u64,
) -> Result<(Principal, String), (X402Error, Option<Principal>)> {
    let payer = req
        .payment_payload
        .payload
        .verify_signature(now_ms, Some(canister_self))
        .map_err(|err| (X402Error::InvalidPayloadSignature(err), None))?;

    let asset_info = store::state::verify_payload(payer, &req.payment_payload, now_ms)
        .map_err(|err| (err, Some(payer)))?;

    let log = store::PaymentLog {
        scheme: req.payment_payload.accepted.scheme,
        asset: req.payment_payload.accepted.asset,
        from: payer,
        to: req.payment_payload.accepted.pay_to,
        value: req.payment_payload.payload.authorization.value.0,
        fee: asset_info.payment_fee,
        expires_at: req.payment_payload.payload.authorization.expires_at,
        nonce: req.payment_payload.payload.authorization.nonce,
        timestamp: now_ms,
    };

    let tx = store::state::transfer_funds(canister_self, log, asset_info.transfer_fee)
        .await
        .map_err(|err| (err, Some(payer)))?;
    Ok((payer, tx))
}

fn supports_cbor(headers: &[HeaderField]) -> bool {
    headers
        .iter()
        .any(|(name, value)| (name == "accept" || name == "content-type") && value.contains(CBOR))
}
