//! Type definitions for the x402 protocol, version 2.
//!
//! <https://github.com/x402-foundation/x402/blob/main/specs/x402-specification-v2.md>
//!
//! The `icp` mechanism (scheme `exact` on network `icp:1`) authorizes payments
//! with an ICRC-2 allowance that the facilitator pulls with
//! `icrc2_transfer_from`, so its `assetTransferMethod` is
//! [`ASSET_TRANSFER_METHOD_ICRC2`] and it supports the
//! [`PaymentFlow::Authorization`] (default) and [`PaymentFlow::Upfront`] flows.

use candid::{CandidType, Principal};
use ic_auth_types::ByteBufB64;
use ic_auth_types::deterministic_cbor_into_vec;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::{
    fmt::{self, Debug, Display, Formatter},
    str::FromStr,
};

pub use serde_json::{Map, Value};

use crate::{SignedEnvelope, sha3_256};

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum X402Error {
    /// Client does not have enough tokens to complete the payment
    #[error("insufficient_funds: {0}")]
    InsufficientFunds(String),

    /// Payment amount is not matching the required amount
    #[error("invalid_payload_authorization_value: {0}")]
    InvalidPayloadAuthorizationValue(String),

    /// Payment authorization has expired
    #[error("invalid_payload_authorization_valid_before: {0}")]
    InvalidPayloadAuthorizationValidBefore(String),

    /// Payment authorization signature is invalid or improperly signed
    #[error("invalid_payload_signature: {0}")]
    InvalidPayloadSignature(String),

    /// Recipient address does not match payment requirements
    #[error("invalid_payload_recipient_mismatch: {0}")]
    InvalidPayloadRecipientMismatch(String),

    /// Specified blockchain network is not supported
    #[error("invalid_network: {0}")]
    InvalidNetwork(String),

    /// Payment payload is malformed or contains invalid data
    #[error("invalid_payload: {0}")]
    InvalidPayload(String),

    /// Payment requirements object is invalid or malformed
    #[error("invalid_payment_requirements: {0}")]
    InvalidPaymentRequirements(String),

    /// Specified payment scheme is not supported
    #[error("invalid_scheme: {0}")]
    InvalidScheme(String),

    /// Payment scheme is not supported by the facilitator
    #[error("unsupported_scheme: {0}")]
    UnsupportedScheme(String),

    /// Protocol version is not supported
    #[error("invalid_x402_version: {0}")]
    InvalidX402Version(u8),

    /// Blockchain transaction failed or was rejected
    #[error("invalid_transaction_state: {0}")]
    InvalidTransactionState(String),

    /// Unexpected error occurred during payment verification
    #[error("unexpected_verify_error: {0}")]
    VerifyError(String),

    /// Unexpected error occurred during payment settlement
    #[error("unexpected_settle_error: {0}")]
    SettleError(String),

    /// The settlement transaction was committed but its confirmation could not
    /// be established. This code is **non-terminal**: the caller must reconcile
    /// on chain before deciding whether to retry, so a [`SettleResponse`]
    /// carrying it MUST also carry a non-empty `transaction` and `network`.
    #[error("settlement_pending: {0}")]
    SettlementPending(String),
}

/// The protocol version implemented by this crate.
pub const X402_VERSION: u8 = 2;

/// CAIP-2 identifier of the Internet Computer mainnet.
pub const NETWORK_ICP: &str = "icp:1";

/// Reserved `PaymentRequirements::extra` key naming the asset transfer method
/// (x402 v2, section 6.1).
pub const EXTRA_ASSET_TRANSFER_METHOD: &str = "assetTransferMethod";

/// Reserved `PaymentRequirements::extra` key naming the payment flow
/// (x402 v2, section 6.1).
pub const EXTRA_PAYMENT_FLOW: &str = "paymentFlow";

/// The only asset transfer method of the `icp` mechanism: the payer grants the
/// facilitator an ICRC-2 allowance which the facilitator pulls with
/// `icrc2_transfer_from`. It is also the mechanism default, so
/// `extra.assetTransferMethod` may be omitted.
pub const ASSET_TRANSFER_METHOD_ICRC2: &str = "icrc2";

/// When settlement happens relative to the resource server executing the paid
/// request (x402 v2, section 6.1).
///
/// The value travels in the reserved `extra.paymentFlow` key of
/// [`PaymentRequirements`]. It is a property of the *ordering* a resource
/// server applies, not of the facilitator's own work: verifying and settling an
/// ICP payment is identical under [`PaymentFlow::Authorization`] and
/// [`PaymentFlow::Upfront`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PaymentFlow {
    /// verify → resource → settle → respond. The mechanism default.
    #[default]
    Authorization,
    /// settle → resource → respond, for resources that need finality first.
    Upfront,
    /// settle → resource → settle → respond. Not supported by the `icp`
    /// mechanism, which has no two-step settlement.
    Escrow,
}

impl PaymentFlow {
    /// Whether the `icp` mechanism can serve this flow.
    ///
    /// `escrow` needs a deposit settle followed by a final-charge settle, which
    /// the single-shot ICRC-2 pull cannot express.
    pub fn is_supported_by_icp(&self) -> bool {
        matches!(self, PaymentFlow::Authorization | PaymentFlow::Upfront)
    }
}

impl Display for PaymentFlow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            PaymentFlow::Authorization => "authorization",
            PaymentFlow::Upfront => "upfront",
            PaymentFlow::Escrow => "escrow",
        };
        f.write_str(s)
    }
}

impl FromStr for PaymentFlow {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "authorization" => Ok(PaymentFlow::Authorization),
            "upfront" => Ok(PaymentFlow::Upfront),
            "escrow" => Ok(PaymentFlow::Escrow),
            other => Err(format!("unknown payment flow: {other}")),
        }
    }
}

/// PaymentRequired Schema
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequired {
    /// Protocol version identifier
    pub x402_version: u8,
    /// Human-readable error message explaining why payment is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// ResourceInfo object describing the protected resource
    pub resource: ResourceInfo,
    /// Array of payment requirement objects defining acceptable payment methods
    pub accepts: Vec<PaymentRequirements>,
    /// Protocol extensions data, keyed by extension identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl PaymentRequired {
    /// Checks the parts of the object this crate constrains: the protocol
    /// version, the resource metadata, and every offered requirement.
    pub fn validate(&self) -> Result<(), X402Error> {
        if self.x402_version != X402_VERSION {
            return Err(X402Error::InvalidX402Version(self.x402_version));
        }
        self.resource.validate()?;
        for accepted in &self.accepts {
            accepted.validate()?;
        }
        Ok(())
    }
}

/// Payment requirements set by the payment-gated endpoint for an acceptable payment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequirements {
    /// Payment scheme identifier (e.g., "exact")
    pub scheme: String,
    /// Blockchain network identifier in CAIP-2 format (e.g., "icp:1")
    pub network: String,
    /// Required payment amount in atomic token units
    pub amount: TokenAmount,
    /// Token ledger canister address
    pub asset: Principal,
    /// Recipient wallet address for the payment
    pub pay_to: Principal,
    /// Maximum time allowed for payment completion in seconds
    pub max_timeout_seconds: u64,
    /// Additional information. The `assetTransferMethod` and `paymentFlow`
    /// keys are reserved by the protocol (see [`PaymentRequirements::asset_transfer_method`]
    /// and [`PaymentRequirements::payment_flow`]); the remaining keys are
    /// scheme-specific.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Map<String, Value>>,
}

impl PaymentRequirements {
    /// The reserved `extra.assetTransferMethod` key, defaulting to
    /// [`ASSET_TRANSFER_METHOD_ICRC2`] when it is omitted.
    pub fn asset_transfer_method(&self) -> Result<&str, X402Error> {
        match self.extra_reserved(EXTRA_ASSET_TRANSFER_METHOD)? {
            Some(method) => Ok(method),
            None => Ok(ASSET_TRANSFER_METHOD_ICRC2),
        }
    }

    /// The reserved `extra.paymentFlow` key, defaulting to
    /// [`PaymentFlow::Authorization`] when it is omitted.
    pub fn payment_flow(&self) -> Result<PaymentFlow, X402Error> {
        match self.extra_reserved(EXTRA_PAYMENT_FLOW)? {
            Some(flow) => PaymentFlow::from_str(flow).map_err(|err| {
                X402Error::InvalidPaymentRequirements(format!("extra.{EXTRA_PAYMENT_FLOW}: {err}"))
            }),
            None => Ok(PaymentFlow::default()),
        }
    }

    /// Checks that the reserved `extra` keys carry values the `icp` mechanism
    /// can serve. Everything else about a requirement — the scheme, the asset,
    /// the amount — is the facilitator's to accept or refuse.
    pub fn validate(&self) -> Result<(), X402Error> {
        let method = self.asset_transfer_method()?;
        if method != ASSET_TRANSFER_METHOD_ICRC2 {
            return Err(X402Error::InvalidPaymentRequirements(format!(
                "unsupported extra.{EXTRA_ASSET_TRANSFER_METHOD}: {method}, expected: {ASSET_TRANSFER_METHOD_ICRC2}"
            )));
        }

        let flow = self.payment_flow()?;
        if !flow.is_supported_by_icp() {
            return Err(X402Error::InvalidPaymentRequirements(format!(
                "unsupported extra.{EXTRA_PAYMENT_FLOW}: {flow}"
            )));
        }
        Ok(())
    }

    /// Reads a reserved `extra` key, which the protocol defines as a string.
    /// An explicit JSON `null` reads the same as an absent key.
    fn extra_reserved(&self, key: &str) -> Result<Option<&str>, X402Error> {
        match self.extra.as_ref().and_then(|extra| extra.get(key)) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(value)) => Ok(Some(value)),
            Some(value) => Err(X402Error::InvalidPaymentRequirements(format!(
                "extra.{key} must be a string, got: {value}"
            ))),
        }
    }
}

/// Metadata describing the payment-gated resource.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceInfo {
    /// the protected resource, e.g., URL of the resource endpoint
    pub url: String,
    /// Human-readable description of the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type of the expected response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Human-readable name of the service hosting the resource.
    /// Printable ASCII, at most [`MAX_LABEL_CHARS`] characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    /// Topical tags for the service, used for discovery filtering.
    /// At most [`MAX_TAGS`] entries, each constrained like `service_name`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Absolute `https`/`http` URL to an icon representing the service.
    /// At most [`MAX_ICON_URL_CHARS`] characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
}

/// Maximum length of `serviceName` and of each entry of `tags`.
pub const MAX_LABEL_CHARS: usize = 32;
/// Maximum number of `tags` entries.
pub const MAX_TAGS: usize = 5;
/// Maximum length of `iconUrl`.
pub const MAX_ICON_URL_CHARS: usize = 2048;

impl ResourceInfo {
    /// Enforces the size and character limits the specification puts on the
    /// discovery metadata, so a resource server cannot smuggle unbounded or
    /// non-printable text into a Bazaar listing.
    pub fn validate(&self) -> Result<(), X402Error> {
        if let Some(service_name) = &self.service_name {
            validate_label("resource.serviceName", service_name)?;
        }

        if let Some(tags) = &self.tags {
            if tags.len() > MAX_TAGS {
                return Err(X402Error::InvalidPaymentRequirements(format!(
                    "resource.tags has {} entries, at most {MAX_TAGS} are allowed",
                    tags.len()
                )));
            }
            for tag in tags {
                validate_label("resource.tags entry", tag)?;
            }
        }

        if let Some(icon_url) = &self.icon_url {
            if icon_url.chars().count() > MAX_ICON_URL_CHARS {
                return Err(X402Error::InvalidPaymentRequirements(format!(
                    "resource.iconUrl is longer than {MAX_ICON_URL_CHARS} characters"
                )));
            }
            if !icon_url.starts_with("https://") && !icon_url.starts_with("http://") {
                return Err(X402Error::InvalidPaymentRequirements(
                    "resource.iconUrl must be an absolute https or http URL".to_string(),
                ));
            }
        }

        Ok(())
    }
}

fn validate_label(field: &str, value: &str) -> Result<(), X402Error> {
    if value.is_empty() {
        return Err(X402Error::InvalidPaymentRequirements(format!(
            "{field} is empty"
        )));
    }
    if value.chars().count() > MAX_LABEL_CHARS {
        return Err(X402Error::InvalidPaymentRequirements(format!(
            "{field} is longer than {MAX_LABEL_CHARS} characters"
        )));
    }
    if !value.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
        return Err(X402Error::InvalidPaymentRequirements(format!(
            "{field} contains non-printable-ASCII characters"
        )));
    }
    Ok(())
}

/// Protocol extensions data, keyed by extension identifier.
///
/// Servers advertise the extensions they support in [`PaymentRequired`] and
/// clients echo them back in [`PaymentPayload`]. A client must return at least
/// the info it received; it may add to it but must not drop or overwrite it.
pub type Extensions = BTreeMap<String, Extension>;

/// One entry of an [`Extensions`] map.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extension {
    /// Extension-specific data provided by the server
    pub info: Map<String, Value>,
    /// JSON Schema defining the expected structure of `info`
    pub schema: Map<String, Value>,
}

/// Describes a signed request to transfer a specific amount of funds on-chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentPayload {
    /// Protocol version identifier
    pub x402_version: u8,
    /// ResourceInfo object describing the resource being accessed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<ResourceInfo>,
    /// PaymentRequirements object indicating the payment method chosen
    pub accepted: PaymentRequirements,
    /// Scheme-specific payment data
    pub payload: IcpPayload,
    /// Protocol extensions data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Scheme-specific payment payload for ICP payments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IcpPayload {
    /// The signature of the `authorization` signed by Internet Identity
    pub signature: ByteBufB64,
    /// Parameters required for payment.
    pub authorization: IcpPayloadAuthorization,
}

impl IcpPayload {
    /// Verifies the signature of the payment authorization.
    pub fn verify_signature(
        &self,
        now_ms: u64,
        expect_target: Option<Principal>,
    ) -> Result<Principal, String> {
        let digest = self.authorization.digest();
        let envelope = SignedEnvelope::from_bytes(&self.signature)?;
        envelope.verify(now_ms, expect_target, Some(&digest))?;
        Ok(envelope.sender())
    }
}

/// Authorization parameters for an ICP payment payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IcpPayloadAuthorization {
    /// Recipient's wallet address
    pub to: Principal,
    /// Payment amount in atomic units.
    /// For `exact` scheme, this is the exact amount to be transferred.
    /// For `upto` scheme, this is the maximum amount that can be transferred.
    pub value: TokenAmount,
    /// Expiration time of the authorization in milliseconds since epoch
    pub expires_at: u64,
    /// A self-incrementing number and should be used to prevent replay attacks.
    pub nonce: u64,
}

impl IcpPayloadAuthorization {
    pub fn digest(&self) -> [u8; 32] {
        IcpPayloadAuthorizationRaw::from(self).digest()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IcpPayloadAuthorizationRaw {
    pub to: String,
    pub value: String,
    pub expires_at: u64,
    pub nonce: u64,
}

impl From<&IcpPayloadAuthorization> for IcpPayloadAuthorizationRaw {
    fn from(auth: &IcpPayloadAuthorization) -> Self {
        IcpPayloadAuthorizationRaw {
            to: auth.to.to_string(),
            value: auth.value.to_string(),
            expires_at: auth.expires_at,
            nonce: auth.nonce,
        }
    }
}

impl IcpPayloadAuthorizationRaw {
    pub fn digest(&self) -> [u8; 32] {
        let data = deterministic_cbor_into_vec(&self)
            .expect("failed to serialize IcpPayloadAuthorization");
        sha3_256(&data)
    }
}

/// Wrapper for a payment payload and requirements sent by the client to a facilitator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct X402Request {
    /// Protocol version identifier. Clients that predate this field are read as
    /// [`X402_VERSION`], which [`X402Request::validate`] then requires the
    /// payload to agree with.
    #[serde(default = "default_x402_version")]
    pub x402_version: u8,
    pub payment_payload: PaymentPayload,
    pub payment_requirements: PaymentRequirements,
}

fn default_x402_version() -> u8 {
    X402_VERSION
}

impl X402Request {
    /// Checks the request against everything the protocol constrains before a
    /// facilitator looks at its own configuration: the two copies of the
    /// version, of the requirements, and of the payment details must agree, and
    /// the reserved `extra` keys and resource metadata must be well-formed.
    pub fn validate(&self) -> Result<(), X402Error> {
        if self.x402_version != self.payment_payload.x402_version {
            return Err(X402Error::InvalidX402Version(self.x402_version));
        }

        self.payment_requirements.validate()?;
        if let Some(resource) = &self.payment_payload.resource {
            resource.validate()?;
        }

        if self.payment_payload.accepted != self.payment_requirements {
            return Err(X402Error::InvalidPaymentRequirements(
                "Payment payload's accepted requirements do not match the provided payment requirements.".to_string(),
            ));
        }

        if self.payment_payload.payload.authorization.to != self.payment_requirements.pay_to {
            return Err(X402Error::InvalidPayloadRecipientMismatch(format!(
                "{}, expected: {}",
                self.payment_payload.payload.authorization.to, self.payment_requirements.pay_to,
            )));
        }

        if self.payment_payload.payload.authorization.value.0 != self.payment_requirements.amount.0
        {
            return Err(X402Error::InvalidPayloadAuthorizationValue(format!(
                "{}, expected: {}",
                self.payment_payload.payload.authorization.value, self.payment_requirements.amount
            )));
        }

        Ok(())
    }
}

/// Result returned by a facilitator after verifying a [`PaymentPayload`] against the provided [`PaymentRequirements`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyResponse {
    /// Indicates whether the payment authorization is valid
    pub is_valid: bool,
    /// Address of the payer's wallet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer: Option<String>,
    /// Reason for invalidity (omitted if valid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_reason: Option<String>,
    /// Protocol extensions data, keyed by extension identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
    /// Scheme-specific additional data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Map<String, Value>>,
}

impl VerifyResponse {
    /// Constructs a successful verification response with the given `payer` address.
    pub fn valid(payer: String) -> Self {
        VerifyResponse {
            is_valid: true,
            payer: Some(payer),
            ..Default::default()
        }
    }

    /// Constructs a failed verification response with the given `payer` address and error `reason`.
    pub fn invalid(payer: Option<String>, reason: X402Error) -> Self {
        VerifyResponse {
            is_valid: false,
            payer,
            invalid_reason: Some(reason.to_string()),
            ..Default::default()
        }
    }
}

/// Returned from a facilitator after attempting to settle a payment on-chain.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettleResponse {
    /// Indicates whether the payment settlement was successful
    pub success: bool,
    /// Error reason if settlement failed (omitted if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_reason: Option<String>,
    /// Ledger transaction ID. Empty when no transaction was committed, which
    /// [`X402Error::SettlementPending`] is the one error not allowed to be.
    pub transaction: String,
    /// Blockchain network identifier in CAIP-2 format
    pub network: String,
    /// Address of the payer's wallet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer: Option<String>,
    /// The actual amount settled in atomic units (omitted if not applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<TokenAmount>,
    /// Protocol extensions data, keyed by extension identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl SettleResponse {
    /// Constructs a response for a payment that settled as `transaction`,
    /// charging the payer `amount` atomic units.
    pub fn settled(
        network: String,
        payer: String,
        transaction: String,
        amount: TokenAmount,
    ) -> Self {
        SettleResponse {
            success: true,
            transaction,
            network,
            payer: Some(payer),
            amount: Some(amount),
            ..Default::default()
        }
    }

    /// Constructs a response for a payment that did not settle and committed
    /// no transaction.
    ///
    /// Use [`SettleResponse::pending`] instead when a transaction *was*
    /// committed but could not be confirmed.
    pub fn failed(network: String, payer: Option<String>, reason: X402Error) -> Self {
        SettleResponse {
            success: false,
            error_reason: Some(reason.to_string()),
            transaction: String::new(),
            network,
            payer,
            ..Default::default()
        }
    }

    /// Constructs a non-terminal response for a payment whose `transaction` was
    /// committed but whose confirmation could not be established. The caller
    /// must reconcile `transaction` against the ledger before retrying, which
    /// is why the protocol requires it to be non-empty here.
    pub fn pending(
        network: String,
        payer: Option<String>,
        transaction: String,
        reason: String,
    ) -> Self {
        SettleResponse {
            success: false,
            error_reason: Some(X402Error::SettlementPending(reason).to_string()),
            transaction,
            network,
            payer,
            ..Default::default()
        }
    }
}

/// A precise on-chain token amount in base units (e.g., USDC with 6 decimals).
/// Represented as a stringified `u128` in JSON to prevent precision loss.
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct TokenAmount(pub u128);

impl<'de> Deserialize<'de> for TokenAmount {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let string = String::deserialize(deserializer)?;
        let value = u128::from_str(&string).map_err(serde::de::Error::custom)?;
        Ok(TokenAmount(value))
    }
}

impl Serialize for TokenAmount {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl Display for TokenAmount {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u128> for TokenAmount {
    fn from(value: u128) -> Self {
        TokenAmount(value)
    }
}

impl From<u64> for TokenAmount {
    fn from(value: u64) -> Self {
        TokenAmount(value as u128)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SupportedKind {
    pub x402_version: u8,
    pub scheme: String,
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Map<String, Value>>,
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SupportedKindCan {
    pub x402_version: u8,
    pub scheme: String,
    pub network: String,
}

impl From<&SupportedKind> for SupportedKindCan {
    fn from(kind: &SupportedKind) -> Self {
        SupportedKindCan {
            x402_version: kind.x402_version,
            scheme: kind.scheme.clone(),
            network: kind.network.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportedResponse {
    /// Array of supported payment kind objects
    pub kinds: Vec<SupportedKind>,
    /// Array of extension identifiers the facilitator has implemented
    pub extensions: Vec<String>,
    /// Map of CAIP-2 patterns (e.g., `eip155:*`) to public signer addresses
    pub signers: BTreeMap<String, Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    fn payment_requirements() -> PaymentRequirements {
        let principal = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
        PaymentRequirements {
            scheme: "exact".to_string(),
            network: NETWORK_ICP.to_string(),
            amount: TokenAmount(1000),
            asset: principal,
            pay_to: principal,
            max_timeout_seconds: 300,
            extra: None,
        }
    }

    #[test]
    fn test_token_amount_serialization() {
        let amount = TokenAmount(123456789012345678901234567890);
        let serialized = serde_json::to_string(&amount).unwrap();
        assert_eq!(serialized, "\"123456789012345678901234567890\"");
        let deserialized: TokenAmount = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, amount);
    }

    #[test]
    fn test_payment_requirements_serialization() {
        let principal = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
        let req = PaymentRequirements {
            scheme: "exact".to_string(),
            network: "icp:1".to_string(),
            amount: TokenAmount(1000),
            asset: principal,
            pay_to: principal,
            max_timeout_seconds: 300,
            extra: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: PaymentRequirements = serde_json::from_str(&json).unwrap();
        assert_eq!(req, deserialized);
    }

    #[test]
    fn test_verify_response() {
        let payer = Principal::anonymous().to_text();
        let valid_response = VerifyResponse::valid(payer.clone());
        assert!(valid_response.is_valid);
        assert_eq!(valid_response.payer, Some(payer.clone()));
        assert!(valid_response.invalid_reason.is_none());

        let reason = X402Error::InsufficientFunds(100.to_string());
        let invalid_response = VerifyResponse::invalid(Some(payer.clone()), reason.clone());
        assert!(!invalid_response.is_valid);
        assert_eq!(invalid_response.payer, Some(payer));
        assert_eq!(invalid_response.invalid_reason, Some(reason.to_string()));
    }

    #[test]
    fn reserved_extra_keys_default_to_the_icp_mechanism() {
        let mut req = payment_requirements();
        assert_eq!(
            req.asset_transfer_method().unwrap(),
            ASSET_TRANSFER_METHOD_ICRC2
        );
        assert_eq!(req.payment_flow().unwrap(), PaymentFlow::Authorization);
        req.validate().unwrap();

        // An explicit JSON null reads the same as an absent key.
        req.extra = Some(Map::from_iter([
            (EXTRA_PAYMENT_FLOW.to_string(), Value::Null),
            (EXTRA_ASSET_TRANSFER_METHOD.to_string(), Value::Null),
        ]));
        assert_eq!(req.payment_flow().unwrap(), PaymentFlow::Authorization);
        req.validate().unwrap();

        req.extra = Some(Map::from_iter([(
            EXTRA_PAYMENT_FLOW.to_string(),
            Value::String("upfront".to_string()),
        )]));
        assert_eq!(req.payment_flow().unwrap(), PaymentFlow::Upfront);
        req.validate().unwrap();

        // `escrow` is a flow of the protocol, but not one this mechanism serves.
        req.extra = Some(Map::from_iter([(
            EXTRA_PAYMENT_FLOW.to_string(),
            Value::String("escrow".to_string()),
        )]));
        assert_eq!(req.payment_flow().unwrap(), PaymentFlow::Escrow);
        assert!(req.validate().is_err());

        req.extra = Some(Map::from_iter([(
            EXTRA_PAYMENT_FLOW.to_string(),
            Value::String("sometime-later".to_string()),
        )]));
        assert!(req.payment_flow().is_err());

        req.extra = Some(Map::from_iter([(
            EXTRA_PAYMENT_FLOW.to_string(),
            Value::Number(2.into()),
        )]));
        assert!(req.payment_flow().is_err());
    }

    #[test]
    fn resource_metadata_limits_are_enforced() {
        let mut resource = ResourceInfo {
            url: "https://api.example.com/premium-data".to_string(),
            service_name: Some("Example Market Data".to_string()),
            tags: Some(vec!["market-data".to_string(), "finance".to_string()]),
            icon_url: Some("https://api.example.com/icon.png".to_string()),
            ..Default::default()
        };
        resource.validate().unwrap();

        let mut too_long = resource.clone();
        too_long.service_name = Some("x".repeat(MAX_LABEL_CHARS + 1));
        assert!(too_long.validate().is_err());

        let mut non_ascii = resource.clone();
        non_ascii.service_name = Some("市场数据".to_string());
        assert!(non_ascii.validate().is_err());

        let mut too_many_tags = resource.clone();
        too_many_tags.tags = Some(vec!["tag".to_string(); MAX_TAGS + 1]);
        assert!(too_many_tags.validate().is_err());

        let mut relative_icon = resource.clone();
        relative_icon.icon_url = Some("/icon.png".to_string());
        assert!(relative_icon.validate().is_err());

        // The optional fields stay optional.
        resource.service_name = None;
        resource.tags = None;
        resource.icon_url = None;
        resource.validate().unwrap();
    }

    #[test]
    fn extensions_are_keyed_by_extension_identifier() {
        let json = r#"{
            "bazaar": {
                "info": { "listed": true },
                "schema": { "type": "object" }
            }
        }"#;
        let extensions: Extensions = serde_json::from_str(json).unwrap();
        assert_eq!(extensions.len(), 1);
        assert_eq!(
            extensions["bazaar"].info.get("listed"),
            Some(&Value::Bool(true))
        );
    }

    #[test]
    fn request_version_must_agree_with_its_payload() {
        let req = payment_requirements();
        let json = serde_json::json!({
            "paymentPayload": {
                "x402Version": X402_VERSION,
                "accepted": req,
                "payload": {
                    "signature": "",
                    "authorization": {
                        "to": req.pay_to.to_string(),
                        "value": req.amount.to_string(),
                        "expiresAt": 1_800_000_000_000u64,
                        "nonce": 1,
                    }
                }
            },
            "paymentRequirements": req,
        });

        // A client that predates the top-level field still validates.
        let request: X402Request = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(request.x402_version, X402_VERSION);
        request.validate().unwrap();

        let mut mismatched = json;
        mismatched["x402Version"] = serde_json::json!(1);
        let request: X402Request = serde_json::from_value(mismatched).unwrap();
        assert!(matches!(
            request.validate(),
            Err(X402Error::InvalidX402Version(1))
        ));
    }

    #[test]
    fn a_pending_settlement_reports_the_transaction_to_reconcile() {
        let res = SettleResponse::pending(
            NETWORK_ICP.to_string(),
            None,
            "42".to_string(),
            "ledger call timed out".to_string(),
        );
        assert!(!res.success);
        assert_eq!(res.transaction, "42");
        assert!(
            res.error_reason
                .as_deref()
                .is_some_and(|reason| reason.starts_with("settlement_pending:"))
        );

        let settled = SettleResponse::settled(
            NETWORK_ICP.to_string(),
            Principal::anonymous().to_text(),
            "42".to_string(),
            TokenAmount(1000),
        );
        let json = serde_json::to_value(&settled).unwrap();
        assert_eq!(json["amount"], serde_json::json!("1000"));
        assert!(json.get("errorReason").is_none());
    }

    #[test]
    fn test_icp_payload_authorization_digest() {
        let auth = IcpPayloadAuthorization {
            to: Principal::from_text(
                "77ibd-jp5kr-moeco-kgoar-rro5v-5tng4-krif5-5h2i6-osf2f-2sjtv-kqe",
            )
            .unwrap(),
            value: TokenAmount(100000000),
            expires_at: 1761536123382,
            nonce: 42,
        };

        let data = deterministic_cbor_into_vec(&auth)
            .expect("failed to serialize IcpPayloadAuthorization");
        println!("CBOR Data: {}", hex::encode(&data));
        // a462746f581dfd5458e209ca338118c5ddaf66d37151417bd3e91e748ba2ea499d5502656e6f6e6365182a6576616c756569313030303030303030696578706972657341741b0000019a23bc21f6

        let digest = auth.digest();
        let expected_hex = "269d40d6a23a75d9e4935d3010a8b8327115bb3dbadc7c311f43fec2445ae8f9"; // Placeholder
        assert_eq!(hex::encode(digest), expected_hex);
    }
}
