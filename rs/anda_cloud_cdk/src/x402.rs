//! Type definitions for the x402 protocol.
//!
//! https://github.com/coinbase/x402/blob/main/specs/x402-specification.md

use candid::{CandidType, Principal};
use ic_auth_types::ByteBufB64;
use ic_auth_types::canonical_cbor_into_vec;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{self, Debug, Display, Formatter},
    str::FromStr,
};
use url::Url;

use crate::{SignedEnvelope, sha3_256};

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum X402Error {
    /// Client does not have enough tokens to complete the payment
    #[error("InsufficientFunds: {0}")]
    InsufficientFunds(String),

    /// Payment authorization is not yet valid (before validAfter timestamp)
    #[error("InvalidPayloadAuthorizationValidAsset: {0}")]
    InvalidPayloadAuthorizationValidAsset(String),

    /// Payment amount is not matching the required amount
    #[error("InvalidPayloadAuthorizationValue: {0}")]
    InvalidPayloadAuthorizationValue(String),

    /// Payment authorization signature is invalid or improperly signed
    #[error("InvalidPayloadAuthorizationSignature: {0}")]
    InvalidPayloadAuthorizationSignature(String),

    /// Recipient address does not match payment requirements
    #[error("InvalidPayloadRecipientMismatch: {0}")]
    InvalidPayloadRecipientMismatch(String),

    /// Specified blockchain network is not supported
    #[error("InvalidNetwork: {0}")]
    InvalidNetwork(String),

    /// Payment payload is malformed or contains invalid data
    #[error("InvalidPayload: {0}")]
    InvalidPayload(String),

    /// Payment requirements object is invalid or malformed
    #[error("InvalidPaymentRequirements: {0}")]
    InvalidPaymentRequirements(String),

    /// Specified payment scheme is not supported
    #[error("InvalidScheme: {0}")]
    InvalidScheme(String),

    /// Payment scheme is not supported by the facilitator
    #[error("UnsupportedScheme: {0}")]
    UnsupportedScheme(String),

    /// Protocol version is not supported
    #[error("InvalidX402Version: {0}")]
    InvalidX402Version(u8),

    /// Blockchain transaction failed or was rejected
    #[error("InvalidTransactionState: {0}")]
    InvalidTransactionState(String),

    /// Unexpected error occurred during payment verification
    #[error("VerifyError: {0}")]
    VerifyError(String),

    /// Unexpected error occurred during payment settlement
    #[error("SettleError: {0}")]
    SettleError(String),
}

/// PaymentRequirementsResponse Schema
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequirementsResponse {
    /// Protocol version identifier
    pub x402_version: X402Version,
    /// Human-readable error message explaining why payment is required
    pub error: String,
    /// Array of payment requirement objects defining acceptable payment methods
    pub accepts: Vec<PaymentRequirements>,
}

/// Protocol version identifier. Currently only version 1 is supported.
#[derive(CandidType, Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum X402Version {
    /// Version `1`.
    V1,
}

impl Display for X402Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            X402Version::V1 => write!(f, "1"),
        }
    }
}

impl TryFrom<u8> for X402Version {
    type Error = X402Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(X402Version::V1),
            _ => Err(X402Error::InvalidX402Version(value)),
        }
    }
}

impl From<X402Version> for u8 {
    fn from(version: X402Version) -> Self {
        match version {
            X402Version::V1 => 1,
        }
    }
}

impl Serialize for X402Version {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            X402Version::V1 => serializer.serialize_u8(1),
        }
    }
}

impl<'de> Deserialize<'de> for X402Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let num = u8::deserialize(deserializer)?;
        X402Version::try_from(num).map_err(serde::de::Error::custom)
    }
}

/// Payment requirements set by the payment-gated endpoint for an acceptable payment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequirements {
    /// Payment scheme identifier (e.g., "exact")
    pub scheme: Scheme,
    /// Blockchain network identifier (e.g., "icp-druyg-tyaaa-aaaaq-aactq-cai")
    pub network: IcpNetwork,
    /// Required payment amount in atomic token units
    pub max_amount_required: TokenAmount,
    /// Token ledger canister address
    pub asset: Principal,
    /// Recipient wallet address for the payment
    pub pay_to: Principal,
    /// URL of the protected resource
    pub resource: Url,
    /// Human-readable description of the resource
    pub description: String,
    /// MIME type of the expected response
    pub mime_type: String,
    /// JSON schema describing the response format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
    /// Maximum time allowed for payment completion in seconds
    pub max_timeout_seconds: u64,
    /// Scheme-specific additional information.
    pub extra: Option<serde_json::Value>,
}

/// Describes a signed request to transfer a specific amount of funds on-chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentPayload {
    /// Protocol version identifier
    pub x402_version: X402Version,
    /// Payment scheme identifier (e.g., "exact")
    pub scheme: Scheme,
    /// Blockchain network identifier
    pub network: IcpNetwork,
    /// Payment data object
    pub payload: IcpPayload,
}

/// Scheme-specific payment payload for ICP payments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IcpPayload {
    /// ICP based signature over the authorization data
    pub signature: ByteBufB64,
    /// Authorization parameters for the payment
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
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IcpPayloadAuthorization {
    /// Payment scheme identifier
    pub scheme: Scheme,
    /// token ledger canister address
    pub asset: Principal,
    /// Recipient's wallet address
    pub to: Principal,
    /// Payment amount in atomic units.
    /// For `exact` scheme, this is the exact amount to be transferred.
    /// For `upto` scheme, this is the maximum amount that can be transferred.
    pub value: TokenAmount,
    /// Unix timestamp when authorization expires (in milliseconds)
    pub expires_at: u64,
    /// A self-incrementing number and should be used to prevent replay attacks.
    pub nonce: u64,
}

impl IcpPayloadAuthorization {
    pub fn digest(&self) -> [u8; 32] {
        let data =
            canonical_cbor_into_vec(&self).expect("failed to serialize IcpPayloadAuthorization");
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
        if self.payment_payload.scheme != self.payment_requirements.scheme {
            return Err(X402Error::UnsupportedScheme(
                self.payment_payload.scheme.to_string(),
            ));
        }
        if self.payment_payload.network != self.payment_requirements.network {
            return Err(X402Error::InvalidNetwork(format!(
                "{}, expected: {}",
                self.payment_payload.network, self.payment_requirements.network
            )));
        }

        if self.payment_payload.payload.authorization.scheme != self.payment_payload.scheme {
            return Err(X402Error::InvalidPayload(format!(
                "mismatched scheme in payload authorization: {}, expected: {}",
                self.payment_payload
                    .payload
                    .authorization
                    .scheme,
                self.payment_payload.scheme,
            )));
        }

        if self.payment_payload.payload.authorization.asset != self.payment_requirements.asset {
            return Err(X402Error::InvalidPayloadAuthorizationValidAsset(format!(
                "mismatched asset in payload authorization: {}, expected: {}",
                self.payment_payload.payload.authorization.asset,
                self.payment_requirements.asset,
            )));
        }

        if self.payment_payload.payload.authorization.to != self.payment_requirements.pay_to {
            return Err(X402Error::InvalidPayloadRecipientMismatch(format!(
                "{}, expected: {}",
                self.payment_payload.payload.authorization.to,
                self.payment_requirements.pay_to,
            )));
        }

        if self.payment_payload.payload.authorization.value.0
            != self.payment_requirements.max_amount_required.0
        {
            return Err(X402Error::InvalidPayloadAuthorizationValue(format!(
                "{}, expected: {}",
                self.payment_payload.payload.authorization.value,
                self.payment_requirements.max_amount_required
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
    pub payer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_reason: Option<String>,
}

impl VerifyResponse {
    /// Constructs a successful verification response with the given `payer` address.
    pub fn valid(payer: String) -> Self {
        VerifyResponse {
            is_valid: true,
            payer,
            invalid_reason: None,
        }
    }

    /// Constructs a failed verification response with the given `payer` address and error `reason`.
    pub fn invalid(payer: String, reason: X402Error) -> Self {
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
//   "payer": "0x857b06519E91e3A54538791bDbb0E22373e36b66",
//   "transaction": "",
//   "network": "base-sepolia"
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
    pub network: IcpNetwork,
    /// Address of the payer's wallet
    pub payer: String,
}

/// The payment schemes supported by the x402 protocol. Payment schemes define how payments are formed, validated, and settled on specific payment networks. Schemes are independent of the underlying transport mechanism.
#[derive(Debug, CandidType, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Scheme {
    /// `exact` transfers a specific amount (ex: pay $1 to read an article)
    Exact,
    /// `upto` transfers up to an amount, based on the resources consumed during a request (ex: generating tokens from an LLM)
    Upto,
}

impl Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Scheme::Exact => "exact",
            Scheme::Upto => "upto",
        };
        write!(f, "{s}")
    }
}

impl FromStr for Scheme {
    type Err = X402Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "exact" => Ok(Scheme::Exact),
            "upto" => Ok(Scheme::Upto),
            _ => Err(X402Error::InvalidScheme(s.to_string())),
        }
    }
}

impl Serialize for Scheme {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Scheme::Exact => serializer.serialize_str("exact"),
            Scheme::Upto => serializer.serialize_str("upto"),
        }
    }
}

impl<'de> Deserialize<'de> for Scheme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Scheme::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IcpNetwork(pub Principal);

impl Display for IcpNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "icp-{}", self.0)
    }
}

impl FromStr for IcpNetwork {
    type Err = X402Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(stripped) = s.strip_prefix("icp-") {
            let principal = Principal::from_str(stripped)
                .map_err(|e| X402Error::InvalidNetwork(format!("invalid principal: {}", e)))?;
            Ok(IcpNetwork(principal))
        } else {
            Err(X402Error::InvalidNetwork(
                "network must start with 'icp-'".to_string(),
            ))
        }
    }
}

impl Serialize for IcpNetwork {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for IcpNetwork {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        IcpNetwork::from_str(&s).map_err(serde::de::Error::custom)
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

#[derive(Clone, CandidType, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct SupportedPaymentKind {
    pub x402_version: X402Version,
    pub scheme: Scheme,
    pub network: String,
}

#[derive(Clone, CandidType, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportedPaymentKindsResponse {
    pub kinds: Vec<SupportedPaymentKind>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;
    use std::str::FromStr;

    #[test]
    fn test_x402_version_serialization() {
        let v1 = X402Version::V1;
        let serialized = serde_json::to_string(&v1).unwrap();
        assert_eq!(serialized, "1");
        let deserialized: X402Version = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(deserialized, X402Version::V1));
    }

    #[test]
    fn test_x402_version_try_from() {
        assert!(matches!(X402Version::try_from(1), Ok(X402Version::V1)));
        assert!(matches!(
            X402Version::try_from(0),
            Err(X402Error::InvalidX402Version(0))
        ));
    }

    #[test]
    fn test_scheme_serialization() {
        let exact = Scheme::Exact;
        let serialized = serde_json::to_string(&exact).unwrap();
        assert_eq!(serialized, "\"exact\"");
        let deserialized: Scheme = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, Scheme::Exact);

        let upto = Scheme::Upto;
        let serialized = serde_json::to_string(&upto).unwrap();
        assert_eq!(serialized, "\"upto\"");
        let deserialized: Scheme = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, Scheme::Upto);
    }

    #[test]
    fn test_scheme_from_str() {
        assert_eq!(Scheme::from_str("exact").unwrap(), Scheme::Exact);
        assert_eq!(Scheme::from_str("upto").unwrap(), Scheme::Upto);
        assert!(matches!(
            Scheme::from_str("invalid"),
            Err(X402Error::InvalidScheme(_))
        ));
    }

    #[test]
    fn test_icp_network_serialization() {
        let principal = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
        let network = IcpNetwork(principal);
        let serialized = serde_json::to_string(&network).unwrap();
        assert_eq!(serialized, "\"icp-ryjl3-tyaaa-aaaaa-aaaba-cai\"");
        let deserialized: IcpNetwork = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, network);
    }

    #[test]
    fn test_icp_network_from_str() {
        let valid_str = "icp-ryjl3-tyaaa-aaaaa-aaaba-cai";
        let network = IcpNetwork::from_str(valid_str).unwrap();
        assert_eq!(
            network.0,
            Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap()
        );

        let invalid_prefix = "ryjl3-tyaaa-aaaaa-aaaba-cai";
        assert!(matches!(
            IcpNetwork::from_str(invalid_prefix),
            Err(X402Error::InvalidNetwork(_))
        ));

        let invalid_principal = "icp-invalid";
        assert!(matches!(
            IcpNetwork::from_str(invalid_principal),
            Err(X402Error::InvalidNetwork(_))
        ));
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
            scheme: Scheme::Exact,
            network: IcpNetwork(principal),
            max_amount_required: TokenAmount(1000),
            asset: principal,
            pay_to: principal,
            resource: Url::from_str("https://example.com").unwrap(),
            description: "Test resource".to_string(),
            mime_type: "application/json".to_string(),
            output_schema: None,
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
        assert_eq!(valid_response.payer, payer);
        assert!(valid_response.invalid_reason.is_none());

        let reason = X402Error::InsufficientFunds(100.to_string());
        let invalid_response = VerifyResponse::invalid(payer.clone(), reason.clone());
        assert!(!invalid_response.is_valid);
        assert_eq!(invalid_response.payer, payer);
        assert_eq!(invalid_response.invalid_reason, Some(reason.to_string()));
    }
}
