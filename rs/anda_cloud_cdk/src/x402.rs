//! Type definitions for the x402 protocol.
//!
//! https://github.com/coinbase/x402/blob/main/specs/x402-specification-v2.md

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
    /// Protocol extensions data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Payment requirements set by the payment-gated endpoint for an acceptable payment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequirements {
    /// Payment scheme identifier (e.g., "exact")
    pub scheme: String,
    /// Blockchain network identifier (e.g., "icp:mainnet")
    pub network: String,
    /// Required payment amount in atomic token units
    pub amount: TokenAmount,
    /// Token ledger canister address
    pub asset: Principal,
    /// Recipient wallet address for the payment
    pub pay_to: Principal,
    /// Maximum time allowed for payment completion in seconds
    pub max_timeout_seconds: u64,
    /// Scheme-specific additional information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Map<String, Value>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
}

/// Describes additional extension data for x402 payment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
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
    pub payment_payload: PaymentPayload,
    pub payment_requirements: PaymentRequirements,
}

impl X402Request {
    /// Constructs a new `X402Request`.
    pub fn validate(&self) -> Result<(), X402Error> {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyResponse {
    pub is_valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_reason: Option<String>,
}

impl VerifyResponse {
    /// Constructs a successful verification response with the given `payer` address.
    pub fn valid(payer: String) -> Self {
        VerifyResponse {
            is_valid: true,
            payer: Some(payer),
            invalid_reason: None,
        }
    }

    /// Constructs a failed verification response with the given `payer` address and error `reason`.
    pub fn invalid(payer: Option<String>, reason: X402Error) -> Self {
        VerifyResponse {
            is_valid: false,
            payer,
            invalid_reason: Some(reason.to_string()),
        }
    }
}

// {
//   "success": false,
//   "errorReason": "insufficient_funds",
//   "transaction": "",
//   "network": "icp:mainnet"
// }

/// Returned from a facilitator after attempting to settle a payment on-chain.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettleResponse {
    /// Indicates whether the payment settlement was successful
    pub success: bool,
    /// Error reason if settlement failed (omitted if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_reason: Option<String>,
    /// Blockchain transaction hash (empty string if settlement failed)
    pub transaction: String,
    /// Blockchain network identifier
    pub network: String,
    /// Address of the payer's wallet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer: Option<String>,
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
    pub extensions: Vec<Extensions>,
    /// Map of CAIP-2 patterns (e.g., `eip155:*`) to public signer addresses
    pub signers: BTreeMap<String, Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

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
            network: "icp:mainnet".to_string(),
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
