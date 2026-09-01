use anda_cloud_cdk::x402::{SettleResponse, VerifyResponse, X402Error, X402Request};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use candid::{CandidType, Principal};
use cbor2::from_slice;
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

#[derive(Debug)]
struct HttpError {
    status_code: u16,
    message: String,
}

static CBOR: &str = "application/cbor";
static JSON: &str = "application/json";
static IC_CERTIFICATE_HEADER: &str = "ic-certificate";
static IC_CERTIFICATE_EXPRESSION_HEADER: &str = "ic-certificateexpression";
const MAX_PAYMENT_BODY_BYTES: usize = 64 * 1024;

/// Why a settlement was turned away on the free query path.
enum SettleRejection {
    TooLarge,
    Invalid((X402Error, Option<Principal>)),
}

/// Runs the whole settlement check on the free query path so requests that
/// cannot succeed never reach the paid update call. `None` means the request
/// still has to be settled by `http_request_update`.
fn settle_preflight(request: &HttpRequest<'static>) -> Option<SettleRejection> {
    if request.body().len() > MAX_PAYMENT_BODY_BYTES {
        return Some(SettleRejection::TooLarge);
    }

    let now_ms = ic_cdk::api::time() / 1_000_000;
    match decode_payment(request.body(), supports_cbor(request.headers())) {
        Ok(req) => verify_payment(ic_cdk::api::canister_self(), req, now_ms)
            .err()
            .map(SettleRejection::Invalid),
        Err(err) => Some(SettleRejection::Invalid((err, None))),
    }
}

#[ic_cdk::query(hidden = true)]
fn http_request(request: HttpRequest<'static>) -> HttpResponse {
    let req_path = request.get_path();
    let is_settle = request.method().as_str() == "POST"
        && matches!(req_path.as_ref(), Ok(path) if path == "/settle");
    let settle_rejection = if is_settle {
        settle_preflight(&request)
    } else {
        None
    };
    if is_settle && settle_rejection.is_none() {
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
        ("POST", "/verify") => post_verify(request.body(), in_cbor),
        ("POST", "/settle") => match settle_rejection {
            Some(SettleRejection::Invalid(error)) => encode_settle_result(Err(error), in_cbor),
            // A preflight that passed already returned the upgrade response above,
            // so the only case left here is an oversized body.
            _ => Err(HttpError {
                status_code: 413,
                message: "payment request body is too large".to_string(),
            }),
        },
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
        ("POST", "/settle") if request.body().len() <= MAX_PAYMENT_BODY_BYTES => {
            post_settle(request.body(), in_cbor).await
        }
        ("POST", "/settle") => Err(HttpError {
            status_code: 413,
            message: "payment request body is too large".to_string(),
        }),
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

fn post_verify(body: &[u8], in_cbor: bool) -> Result<Vec<u8>, HttpError> {
    let canister_self = ic_cdk::api::canister_self();
    let now_ms = ic_cdk::api::time() / 1_000_000;

    let req = match decode_payment(body, in_cbor) {
        Ok(req) => verify_payment(canister_self, req, now_ms),
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
    let req = match decode_payment(body, in_cbor) {
        Ok(req) => settle_payment(canister_self, req, now_ms).await,
        Err(err) => Err((err, None)),
    };
    encode_settle_result(req, in_cbor)
}

fn encode_settle_result(
    result: Result<(Principal, String), (X402Error, Option<Principal>)>,
    in_cbor: bool,
) -> Result<Vec<u8>, HttpError> {
    let network = "icp:1".to_string();
    let res = match result {
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
            message: format!("failed to serialize SettleResponse, error: {err}"),
        })
    } else {
        serde_json::to_vec(&res).map_err(|err| HttpError {
            status_code: 500,
            message: format!("failed to serialize SettleResponse, error: {err}"),
        })
    }
}

fn decode_payment(body: &[u8], in_cbor: bool) -> Result<X402Request, X402Error> {
    if body.len() > MAX_PAYMENT_BODY_BYTES {
        return Err(X402Error::InvalidPayload(
            "Payment request body is too large".to_string(),
        ));
    }

    let req: X402Request = if in_cbor {
        from_slice(body)
            .map_err(|err| X402Error::InvalidPayload(format!("failed to decode cbor: {err}")))?
    } else {
        serde_json::from_slice(body)
            .map_err(|err| X402Error::InvalidPayload(format!("failed to decode json: {err}")))?
    };

    req.validate()?;
    Ok(req)
}

fn verify_payment(
    canister_self: Principal,
    req: X402Request,
    now_ms: u64,
) -> Result<Principal, (X402Error, Option<Principal>)> {
    // Cheap, payer-independent checks first: verifying the payload signature can
    // cost several canister-signature verifications and also runs on the free
    // query preflight, so junk requests must not reach it.
    let asset_info = store::state::verify_requirements(&req.payment_payload, now_ms)
        .map_err(|err| (err, None))?;

    let payer = req
        .payment_payload
        .payload
        .verify_signature(now_ms, Some(canister_self))
        .map_err(|err| (X402Error::InvalidPayloadSignature(err), None))?;

    let _ = store::state::verify_payer(payer, &req.payment_payload, asset_info)
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
    // Same ordering as `verify_payment`: the payer-independent checks are cheap,
    // the signature verification is not.
    let asset_info = store::state::verify_requirements(&req.payment_payload, now_ms)
        .map_err(|err| (err, None))?;

    let payer = req
        .payment_payload
        .payload
        .verify_signature(now_ms, Some(canister_self))
        .map_err(|err| (X402Error::InvalidPayloadSignature(err), None))?;

    let asset_info = match store::state::verify_payer(payer, &req.payment_payload, asset_info)
        .map_err(|err| (err, Some(payer)))?
    {
        store::VerifiedPayment::Pending(asset_info) => asset_info,
        store::VerifiedPayment::Settled(transaction) => return Ok((payer, transaction)),
    };

    let created_at_time =
        payment_created_at_time(req.payment_payload.payload.authorization.expires_at)
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
        transaction: String::new(),
    };

    let tx =
        store::state::transfer_funds(canister_self, log, asset_info.transfer_fee, created_at_time)
            .await
            .map_err(|err| (err, Some(payer)))?;
    Ok((payer, tx))
}

fn payment_created_at_time(expires_at: u64) -> Result<u64, X402Error> {
    // ICRC ledgers deduplicate transfers by their complete arguments,
    // including `created_at_time`. Derive it only from the signed expiry so a
    // retry cannot change it by mutating unsigned payment requirements.
    let created_at_ms = expires_at
        .checked_sub(store::MAX_AUTHORIZATION_TTL_MS)
        .ok_or_else(|| X402Error::InvalidPayload("Authorization expiry is invalid".to_string()))?;

    created_at_ms.checked_mul(1_000_000).ok_or_else(|| {
        X402Error::InvalidPayload("Authorization creation time is too large".to_string())
    })
}

fn supports_cbor(headers: &[HeaderField]) -> bool {
    headers
        .iter()
        .any(|(name, value)| (name == "accept" || name == "content-type") && value.contains(CBOR))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_timestamp_is_deterministic_and_in_nanoseconds() {
        let expires_at = 1_800_000_000_000;
        assert_eq!(
            payment_created_at_time(expires_at).unwrap(),
            (expires_at - store::MAX_AUTHORIZATION_TTL_MS) * 1_000_000
        );
    }

    #[test]
    fn ledger_timestamp_rejects_underflow_and_overflow() {
        assert!(payment_created_at_time(1).is_err());
        assert!(payment_created_at_time(u64::MAX).is_err());
    }

    #[test]
    fn oversized_payment_is_rejected_before_decoding() {
        let body = vec![0; MAX_PAYMENT_BODY_BYTES + 1];
        assert!(decode_payment(&body, false).is_err());
    }

    #[test]
    fn preflight_failure_keeps_the_settle_response_shape() {
        let body = encode_settle_result(
            Err((X402Error::InvalidPayload("bad request".to_string()), None)),
            false,
        )
        .unwrap();
        let response: SettleResponse = serde_json::from_slice(&body).unwrap();

        assert!(!response.success);
        assert_eq!(response.network, "icp:1");
        assert_eq!(
            response.error_reason.as_deref(),
            Some("invalid_payload: bad request")
        );
    }
}
