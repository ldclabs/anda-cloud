# `anda_x402_canister`

A fully on-chain x402 payment facilitator on the Internet Computer, part of [Anda Cloud](https://github.com/ldclabs/anda-cloud).

## Overview

`anda_x402_canister` is an ICP smart contract that functions as a payment facilitator based on the [x402 Protocol](https://www.x402.org), version 2. It handles payment verification and settlement for services within the Anda Cloud ecosystem, supporting multiple ICRC-2 compatible tokens.

## Features

- Implements the x402 payment protocol for decentralized payments.
- Supports multiple ICRC-2 compatible tokens as payment assets.
- Provides both Candid and HTTP APIs.
- Manages payment states, fees, and transaction logs on-chain.
- Supports both JSON and CBOR content types for the HTTP API.
- Fully deployed as a smart contract on the decentralized ICP blockchain, governed by ICPanda DAO.

## Online Anda x402 Facilitator Endpoints

Facilitator Info:
https://ogkpr-lyaaa-aaaap-an5fq-cai.icp0.io

Supported Payment Kinds:
https://ogkpr-lyaaa-aaaap-an5fq-cai.icp0.io/supported

Verify:
```
POST https://ogkpr-lyaaa-aaaap-an5fq-cai.icp0.io/verify
```

Settle:
```
POST https://ogkpr-lyaaa-aaaap-an5fq-cai.icp0.io/settle
```

## Quick Start

### Client Typescript SDK

```bash
npm install @ldclabs/anda_x402
```

npmjs: https://www.npmjs.com/package/@ldclabs/anda_x402

source: https://github.com/ldclabs/anda-cloud/tree/main/ts/anda_x402

### Integration Example

https://github.com/ldclabs/anda-cloud/tree/main/examples/ts/anda_x402/app.ts

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

Add supported asset:
```bash
# Add the PANDA token as a supported asset, payment fee is 0.1 PANDA
dfx canister call anda_x402_canister admin_update_supported_asset '(principal "druyg-tyaaa-aaaaq-aactq-cai", 10_000_000)'
# Check the canister info
dfx canister call anda_x402_canister info '()'
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

The canister supports HTTP requests for payment operations. Please see the
[x402 Protocol Specification v2](https://github.com/x402-foundation/x402/blob/main/specs/x402-specification-v2.md)
for details.

#### Facilitator Endpoints

- `GET /`: Get canister info, including supported assets and payment kinds.
- `GET /supported`: Get a list of supported payment kinds.
- `POST /verify`: Verify a payment authorization without settling.
- `POST /settle`: Settle a payment by transferring funds.

`GET /supported` also publishes the facilitator's signer under the `icp:*`
CAIP-2 pattern. That signer is this canister: it is the ICRC-2 spender that
pulls every payment, so it is the principal payers must approve.

```json
{
  "kinds": [{ "x402Version": 2, "scheme": "exact", "network": "icp:1" }],
  "extensions": [],
  "signers": { "icp:*": ["ogkpr-lyaaa-aaaap-an5fq-cai"] }
}
```

#### Mechanism: `exact` on `icp:1`

Payment requirements carry two protocol-reserved keys in `extra`
(specification section 6.1). Both may be omitted, in which case the mechanism
defaults below apply; any other value is rejected with
`invalid_payment_requirements`.

| Key                   | Supported values                    | Default           |
| --------------------- | ----------------------------------- | ----------------- |
| `assetTransferMethod` | `icrc2`                             | `icrc2`           |
| `paymentFlow`         | `authorization`, `upfront`          | `authorization`   |

`icrc2` means the payer approves this canister as an ICRC-2 spender and the
facilitator pulls the funds with `icrc2_transfer_from` at settlement.

`escrow` is not supported: a single ICRC-2 pull cannot express a deposit
settle followed by a final-charge settle. `authorization` and `upfront` differ
only in the order the *resource server* calls `/verify` and `/settle`; the
facilitator's own work is identical, and clients should prefer `authorization`
so funds move only after the resource has delivered.

#### Content Types

The HTTP API supports both JSON and CBOR formats. The content type is determined by the `Accept` and `Content-Type` headers:

- For JSON: `application/json`
- For CBOR: `application/cbor`

CBOR is recommended for settlement requests because its smaller ingress body
reduces bandwidth-related cycles and decoding work.

#### Settlement Guarantees

- Payment authorizations must expire between 1 minute and 1 hour from the
  canister's current time.
- Payment request bodies are limited to 64 KiB; oversized `/settle` requests
  are rejected by the free query path before they are upgraded.
- Structurally invalid, badly signed, stale-nonce, and unsupported settlement
  requests are also rejected during that query preflight, avoiding a paid
  update for requests that cannot succeed.
- Retrying the most recently settled authorization is idempotent and returns
  the original ledger transaction ID.
- ICRC-2 transfers use a deterministic `created_at_time`, so an ambiguous
  ledger response can be retried without charging the payer twice.

When upgrading from the legacy log layout, payer state is migrated lazily and
old logs remain readable. After accepting payments with this layout, do not
roll back to a pre-migration Wasm because it cannot read the new payer state.

## Data Types

### ICP Payment Payload

The core data structure representing the payment payload for ICP chain:

```ts
export interface IcpPayload {
  /// ICP based signature over the authorization data in base64 format
  signature: string
  /// Authorization parameters for the payment
  authorization: IcpPayloadAuthorization
}

export interface IcpPayloadAuthorization {
  /// Recipient's wallet address
  to: string
  /// Payment amount in atomic units.
  /// For `exact` scheme, this is the exact amount to be transferred.
  /// For `upto` scheme, this is the maximum amount that can be transferred.
  value: string
  /// Unix timestamp when authorization expires (in milliseconds)
  expiresAt: number
  /// A self-incrementing number and should be used to prevent replay attacks.
  nonce: number
}
```

### Settlement Response

```json
{
  "success": true,
  "transaction": "30",
  "network": "icp:1",
  "payer": "jjn6g-sh75l-r3cxb-wxrkl-frqld-6p6qq-d4ato-wske5-op7s5-n566f-bqe",
  "amount": "100000000"
}
```

`transaction` is the ICRC ledger transaction index of the transfer, and
`amount` is what the payer was charged in atomic units, i.e. the authorized
value including the facilitator's payment fee. A failed settlement reports
`success: false` with an `errorReason` and an empty `transaction`.

### Resource Metadata

`ResourceInfo` accompanies both `PaymentRequired` and `PaymentPayload`. Beyond
`url`, `description` and `mimeType`, v2 adds discovery metadata, which the
facilitator bounds so a resource server cannot smuggle unbounded or
non-printable text into a listing:

- `serviceName`: printable ASCII, at most 32 characters.
- `tags`: at most 5 entries, each constrained like `serviceName`.
- `iconUrl`: absolute `https`/`http` URL, at most 2048 characters.

## License
Copyright © 2024-2025 [LDC Labs](https://github.com/ldclabs).

`ldclabs/anda-cloud` is licensed under the MIT License. See [LICENSE](../../LICENSE) for the full license text.
