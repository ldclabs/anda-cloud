# `anda_x402_canister`

A fully on-chain x402 payment facilitator on the Internet Computer, part of [Anda Cloud](https://github.com/ldclabs/anda-cloud).

## Overview

`anda_x402_canister` is an ICP smart contract that functions as a payment facilitator based on the [x402 Protocol](https://www.x402.org). It handles payment verification and settlement for services within the Anda Cloud ecosystem, supporting multiple ICRC-2 compatible tokens.

## Features

- Implements the x402 payment protocol for decentralized payments.
- Supports multiple ICRC-2 compatible tokens as payment assets.
- Provides both Candid and HTTP APIs.
- Manages payment states, fees, and transaction logs on-chain.
- Supports both JSON and CBOR content types for the HTTP API.
- Fully deployed as a smart contract on the decentralized ICP blockchain, governed by ICPanda DAO.

## Demo

Try it online: https://ogkpr-lyaaa-aaaap-an5fq-cai.icp0.io/supported

## Quick Start

### Local Deployment

Deploy the canister:
```bash
# dfx canister create --specified-id ogkpr-lyaaa-aaaap-an5fq-cai anda_x402_canister
dfx deploy anda_x402_canister --argument "(opt variant {Init =
  record {
    name = \"Anda X402 Facilitator\";
    governance_canister = opt principal \"dwv6s-6aaaa-aaaaq-aacta-cai\";
  }
})"
```

### Candid API

The canister exposes a comprehensive Candid API. Key endpoints include:

```did
// Get canister information and state
info : () -> (Result_2) query;

// Get the next valid nonce for the caller to make a payment
next_nonce : () -> (Result_4) query;

// Get payment logs for the caller
my_payment_logs : (nat32, opt nat64) -> (Result_3) query;

// --- Administration ---

// Add a supported payment kind (version and scheme)
admin_add_supported_payment : (X402Version, Scheme) -> (Result);

// Remove a supported payment kind
admin_remove_supported_payment : (X402Version, Scheme) -> (Result);

// Add or update a supported ICRC-1 asset
admin_update_supported_asset : (principal, nat) -> (Result);

// Remove a supported asset
admin_remove_supported_asset : (principal) -> (Result);

// Collect fees from the canister
admin_collect_fees : (principal, principal, nat) -> (Result_1);
```

Full Candid API definition: [anda_x402_canister.did](https://github.com/ldclabs/anda-cloud/tree/main/rs/anda_x402_canister/anda_x402_canister.did)

### HTTP API

The canister supports HTTP requests for payment operations. Please see [x402 Protocol Specification](https://github.com/coinbase/x402/blob/main/specs/x402-specification.md) for details.

#### Facilitator Endpoints

- `GET /`: Get canister info, including supported assets and payment kinds.
- `GET /supported`: Get a list of supported payment kinds.
- `POST /verify`: Verify a payment authorization without settling.
- `POST /settle`: Settle a payment by transferring funds.

#### Content Types

The HTTP API supports both JSON and CBOR formats. The content type is determined by the `Accept` and `Content-Type` headers:

- For JSON: `application/json`
- For CBOR: `application/cbor`

## Data Types

### ICP Payload

The core data structure representing the payment payload for ICP chain:

```rust
pub struct IcpPayload {
    /// ICP based signature over the authorization data
    pub signature: ByteBufB64,
    /// Authorization parameters for the payment
    pub authorization: IcpPayloadAuthorization,
}

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
```

### State

The core data structure representing the canister's state:

```rust
pub struct State {
    pub name: String,
    pub governance_canister: Option<Principal>,
    pub supported_payments: BTreeSet<SupportedPaymentKind>,
    pub supported_assets: BTreeMap<Principal, AssetInfo>,
    pub total_collected_fees: BTreeMap<Principal, u128>,
    pub total_withdrawn_fees: BTreeMap<Principal, u128>,
}
```

### PaymentLogInfo

Contains information about a settled payment:

```rust
pub struct PaymentLogInfo {
    pub id: u64,
    pub to: Principal,
    pub fee: String,
    pub asset: Principal,
    pub value: String,
    pub scheme: Scheme,
    pub from: Principal,
    pub nonce: u64,
    pub timestamp: u64,
    pub expires_at: u64,
}
```

### SupportedPaymentKind

Defines a supported payment method:

```rust
pub struct SupportedPaymentKind {
    pub x402_version: X402Version,
    pub scheme: Scheme,
    pub network: String,
}
```

## License
Copyright © 2024-2025 [LDC Labs](https://github.com/ldclabs).

`ldclabs/anda-cloud` is licensed under the MIT License. See [LICENSE](../../LICENSE) for the full license text.
