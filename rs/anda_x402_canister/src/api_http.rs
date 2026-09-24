use anda_cloud_cdk::x402::{
    NETWORK_ICP, PaymentPayload, SettleResponse, TokenAmount, VerifyResponse, X402Error,
    X402Request,
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use candid::{CandidType, Principal};
use cbor2::from_slice;
use ic_auth_types::cbor_into_vec;
use ic_http_certification::{HeaderField, HttpRequest, HttpUpdateRequest};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use crate::{helper::transfer_token_from, store};

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

impl HttpError {
    fn new(status_code: u16, message: impl Into<String>) -> Self {
        Self {
            status_code,
            message: message.into(),
        }
    }
}

/// A payment turned away, with its payer once the signature has established one.
type Rejection = (X402Error, Option<Principal>);

static CBOR: &str = "application/cbor";
static JSON: &str = "application/json";
static IC_CERTIFICATE_HEADER: &str = "ic-certificate";
static IC_CERTIFICATE_EXPRESSION_HEADER: &str = "ic-certificateexpression";
const MAX_PAYMENT_BODY_BYTES: usize = 64 * 1024;

#[ic_cdk::query(hidden = true)]
fn http_request(request: HttpRequest<'static>) -> HttpResponse {
    let (body_cbor, reply_cbor) = content_types(request.headers());
    let rt = match request.get_path() {
        Ok(path) => match (request.method().as_str(), path.as_str()) {
            ("HEAD", _) => Ok(Vec::new()),
            ("GET", "/") => encode(&store::state::info(), reply_cbor),
            ("GET", "/supported") => encode(&store::state::supported(), reply_cbor),
            ("POST", "/verify") => encode(&verify(request.body(), body_cbor), reply_cbor),
            ("POST", "/settle") if request.body().len() > MAX_PAYMENT_BODY_BYTES => {
                Err(HttpError::new(413, "payment request body is too large"))
            }
            // Run every settlement check on this free query path, so a payment
            // that cannot succeed never reaches the paid update call.
            ("POST", "/settle") => match check_payment(request.body(), body_cbor, now_ms()) {
                Ok(_) => return upgrade(),
                Err(rejection) => encode(&settle_response(Err(rejection)), reply_cbor),
            },
            (method, path) => Err(HttpError::new(
                404,
                format!("method {method}, path: {path}"),
            )),
        },
        Err(err) => Err(HttpError::new(400, err.to_string())),
    };
    respond(rt, reply_cbor, certified_headers(request.url()))
}

#[ic_cdk::update(hidden = true)]
async fn http_request_update(request: HttpUpdateRequest<'static>) -> HttpResponse {
    let (body_cbor, reply_cbor) = content_types(request.headers());
    let rt = match request.get_path() {
        Ok(path) => match (request.method().as_str(), path.as_str()) {
            ("POST", "/settle") => encode(
                &settle_response(settle(request.body(), body_cbor).await),
                reply_cbor,
            ),
            (method, path) => Err(HttpError::new(
                404,
                format!("method {method}, path: {path}"),
            )),
        },
        Err(err) => Err(HttpError::new(400, err.to_string())),
    };
    respond(rt, reply_cbor, vec![nosniff()])
}

fn upgrade() -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: vec![],
        body: b"Upgrade".to_vec().into(),
        upgrade: Some(true),
    }
}

fn respond(
    rt: Result<Vec<u8>, HttpError>,
    reply_cbor: bool,
    mut headers: Vec<HeaderField>,
) -> HttpResponse {
    let (status_code, content_type, body) = match rt {
        Ok(body) => (200, if reply_cbor { CBOR } else { JSON }, body),
        Err(err) => (err.status_code, "text/plain", err.message.into_bytes()),
    };
    headers.push(("content-type".to_string(), content_type.to_string()));
    headers.push(("content-length".to_string(), body.len().to_string()));
    HttpResponse {
        status_code,
        headers,
        body: body.into(),
        upgrade: None,
    }
}

fn nosniff() -> HeaderField {
    ("x-content-type-options".to_string(), "nosniff".to_string())
}

/// The headers of a query response. Every path is certified to skip response
/// verification, which still takes a witness for the requested path.
fn certified_headers(url: &str) -> Vec<HeaderField> {
    let witness = store::state::http_tree_with(|tree| {
        tree.witness(&store::state::DEFAULT_CERT_ENTRY, url)
            .expect("get witness failed")
    });
    let certificate = ic_cdk::api::data_certificate().expect("no data certificate available");

    vec![
        nosniff(),
        (
            IC_CERTIFICATE_EXPRESSION_HEADER.to_string(),
            store::state::DEFAULT_CEL_EXPR.clone(),
        ),
        (
            IC_CERTIFICATE_HEADER.to_string(),
            format!(
                "certificate=:{}:, tree=:{}:, expr_path=:{}:, version=2",
                BASE64.encode(certificate),
                BASE64.encode(cbor_into_vec(&witness).expect("failed to serialize witness")),
                BASE64.encode(
                    cbor_into_vec(&store::state::DEFAULT_EXPR_PATH.to_expr_path())
                        .expect("failed to serialize expr path")
                )
            ),
        ),
    ]
}

fn encode<T: Serialize>(value: &T, cbor: bool) -> Result<Vec<u8>, HttpError> {
    let res = if cbor {
        cbor_into_vec(value)
    } else {
        serde_json::to_vec(value).map_err(|err| err.to_string())
    };
    res.map_err(|err| HttpError::new(500, format!("failed to serialize response, error: {err}")))
}

/// Whether the request body and the response are CBOR rather than JSON. The
/// body is read by its `content-type`; the response is CBOR when the body is,
/// or when `accept` asks for it.
fn content_types(headers: &[HeaderField]) -> (bool, bool) {
    let lists_cbor = |header: &str| {
        headers
            .iter()
            .any(|(name, value)| name.eq_ignore_ascii_case(header) && value.contains(CBOR))
    };
    let body_cbor = lists_cbor("content-type");
    (body_cbor, body_cbor || lists_cbor("accept"))
}

fn now_ms() -> u64 {
    ic_cdk::api::time() / 1_000_000
}

fn decode_payment(body: &[u8], body_cbor: bool) -> Result<X402Request, X402Error> {
    if body.len() > MAX_PAYMENT_BODY_BYTES {
        return Err(X402Error::InvalidPayload(
            "Payment request body is too large".to_string(),
        ));
    }

    let req: X402Request = if body_cbor {
        from_slice(body)
            .map_err(|err| X402Error::InvalidPayload(format!("failed to decode cbor: {err}")))?
    } else {
        serde_json::from_slice(body)
            .map_err(|err| X402Error::InvalidPayload(format!("failed to decode json: {err}")))?
    };

    req.validate()?;
    Ok(req)
}

/// A payment that passed every check and can be settled.
struct CheckedPayment {
    payload: PaymentPayload,
    payer: Principal,
    fees: store::AssetFees,
}

/// Decodes a payment and checks it up to, and including, the payer's nonce.
fn check_payment(body: &[u8], body_cbor: bool, now_ms: u64) -> Result<CheckedPayment, Rejection> {
    let payload = decode_payment(body, body_cbor)
        .map_err(|err| (err, None))?
        .payment_payload;

    // Payer-independent checks first: verifying the signature can cost several
    // canister-signature verifications and also runs on the free query
    // preflight, so junk requests must not reach it.
    let fees = store::state::verify_requirements(&payload, now_ms).map_err(|err| (err, None))?;
    let payer = payload
        .payload
        .verify_signature(now_ms, Some(ic_cdk::api::canister_self()))
        .map_err(|err| (X402Error::InvalidPayloadSignature(err), None))?;
    store::state::verify_payer(payer, payload.payload.authorization.nonce)
        .map_err(|err| (err, Some(payer)))?;

    // Funds are not checked here: `icrc2_allowance` would be an inter-canister
    // call, which turns every verification into a paid update call.
    Ok(CheckedPayment {
        payload,
        payer,
        fees,
    })
}

fn verify(body: &[u8], body_cbor: bool) -> VerifyResponse {
    match check_payment(body, body_cbor, now_ms()) {
        Ok(payment) => VerifyResponse::valid(payment.payer.to_string()),
        Err((err, payer)) => VerifyResponse::invalid(payer.map(|p| p.to_string()), err),
    }
}

/// A payment the ledger has moved, as reported back to the resource server.
struct Settlement {
    payer: Principal,
    transaction: String,
    /// What the payer was charged, i.e. the authorized value including the
    /// facilitator's payment fee.
    amount: TokenAmount,
}

fn settle_response(result: Result<Settlement, Rejection>) -> SettleResponse {
    let network = NETWORK_ICP.to_string();
    match result {
        Ok(settlement) => SettleResponse::settled(
            network,
            settlement.payer.to_string(),
            settlement.transaction,
            settlement.amount,
        ),
        Err((err, payer)) => SettleResponse::failed(network, payer.map(|p| p.to_string()), err),
    }
}

async fn settle(body: &[u8], body_cbor: bool) -> Result<Settlement, Rejection> {
    let now_ms = now_ms();
    let CheckedPayment {
        payload,
        payer,
        fees,
    } = check_payment(body, body_cbor, now_ms)?;
    let rejected = move |err: X402Error| (err, Some(payer));

    let accepted = payload.accepted;
    let authorization = payload.payload.authorization;
    let created_at_time = payment_created_at_time(authorization.expires_at).map_err(rejected)?;

    // `check_payment` read the nonce and nothing has awaited since, so taking
    // the payer's lock now lets that nonce move funds at most once.
    let _guard = store::PayerGuard::acquire(payer).map_err(rejected)?;
    let idx = transfer_token_from(
        accepted.asset,
        payer,
        accepted.pay_to,
        // `verify_requirements` ensures the value exceeds the payment fee.
        authorization.value.0 - fees.payment_fee,
        Some(authorization.nonce.into()),
        Some(created_at_time),
    )
    .await
    .map_err(|err| {
        rejected(X402Error::SettleError(format!(
            "Failed to transfer payment: {err}"
        )))
    })?;
    // Format the inner BigUint directly; Candid Nat's Display groups digits.
    let transaction = idx.0.to_string();

    store::state::record_payment(store::PaymentLog {
        scheme: accepted.scheme,
        asset: accepted.asset,
        from: payer,
        to: accepted.pay_to,
        value: authorization.value.0,
        fee: fees.payment_fee,
        expires_at: authorization.expires_at,
        nonce: authorization.nonce,
        timestamp: now_ms,
        transaction: transaction.clone(),
    })
    .map_err(rejected)?;

    // Nothing awaits after this spawn, so the fee transfer starts only once the
    // update has replied. It still runs in this call context: the resource
    // server waits for one ledger call instead of two, and no extra message
    // is charged.
    if let Some(fee) = fees
        .payment_fee
        .checked_sub(fees.transfer_fee)
        .filter(|fee| *fee > 0)
    {
        ic_cdk::futures::spawn(collect_fee(
            accepted.asset,
            payer,
            fee,
            authorization.nonce,
            created_at_time,
        ));
    }

    Ok(Settlement {
        payer,
        transaction,
        amount: authorization.value,
    })
}

/// Pulls the facilitator's fee for a settled payment. A failure only costs the
/// facilitator its fee, so it is logged rather than reported.
async fn collect_fee(
    asset: Principal,
    payer: Principal,
    fee: u128,
    nonce: u64,
    created_at_time: u64,
) {
    match transfer_token_from(
        asset,
        payer,
        ic_cdk::api::canister_self(),
        fee,
        Some(nonce.into()),
        Some(created_at_time),
    )
    .await
    {
        Ok(_) => store::state::add_collected_fee(asset, fee),
        Err(err) => {
            ic_cdk::println!("failed to collect payment fee from {payer} for nonce {nonce}: {err}")
        }
    }
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
    fn a_rejected_settlement_keeps_the_settle_response_shape() {
        let body = encode(
            &settle_response(Err((
                X402Error::InvalidPayload("bad request".to_string()),
                None,
            ))),
            false,
        )
        .unwrap();
        let response: SettleResponse = serde_json::from_slice(&body).unwrap();

        assert!(!response.success);
        assert_eq!(response.network, NETWORK_ICP);
        assert!(response.transaction.is_empty());
        assert!(response.amount.is_none());
        assert_eq!(
            response.error_reason.as_deref(),
            Some("invalid_payload: bad request")
        );
    }

    #[test]
    fn the_body_is_read_by_its_content_type() {
        let headers = |pairs: &[(&str, &str)]| {
            pairs
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect::<Vec<_>>()
        };

        // A JSON body may ask for a CBOR response.
        assert_eq!(
            content_types(&headers(&[("Content-Type", JSON), ("Accept", CBOR)])),
            (false, true)
        );
        assert_eq!(content_types(&headers(&[("accept", CBOR)])), (false, true));
        // A CBOR body gets a CBOR response, as it always has.
        assert_eq!(
            content_types(&headers(&[("content-type", CBOR)])),
            (true, true)
        );
        assert_eq!(content_types(&headers(&[])), (false, false));
    }
}
