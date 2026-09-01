# `@ldclabs/anda_x402`
[![Test](https://github.com/ldclabs/anda-cloud/actions/workflows/test.yml/badge.svg)](https://github.com/ldclabs/anda-cloud/actions/workflows/test.yml)
[![NPM version](http://img.shields.io/npm/v/@ldclabs/anda_x402.svg)](https://www.npmjs.com/package/@ldclabs/anda_x402)

`@ldclabs/anda_x402` is the Typescript version of the client SDK for Anda x402 facilitator.

It implements version 2 of the
[x402 Protocol Specification](https://github.com/x402-foundation/x402/blob/main/specs/x402-specification-v2.md)
for the `exact` scheme on the `icp:1` network.

## Usage

```ts
import { X402Canister, X402_VERSION } from '@ldclabs/anda_x402'

const x402 = new X402Canister('ogkpr-lyaaa-aaaap-an5fq-cai', identity)

// What the facilitator implements, including the `icp:*` signer the payer
// must approve as their ICRC-2 spender.
const supported = await x402.getSupported()

// `paymentRequired` is the server's `PaymentRequired` object, carried by the
// base64-encoded `PAYMENT-REQUIRED` response header over HTTP.
const request = await x402.buildX402RequestFrom(paymentRequired, assetId)
```

`buildX402RequestFrom` picks the `accepts[]` entry to pay:

- Entries for another network or asset, or whose reserved `extra` keys this
  SDK does not recognize, are skipped — another entry may still be payable.
- When a resource offers both, `authorization` wins over `upfront`, so the
  funds move only after the resource has delivered.
- Extensions advertised in `PaymentRequired` are echoed back in the
  `PaymentPayload`, as the protocol requires.

The mechanism supports `assetTransferMethod: "icrc2"` and the
`authorization` (default) and `upfront` payment flows. `escrow` is not
supported: a single ICRC-2 pull cannot express a deposit settle followed by a
final-charge settle.

## License

Copyright © 2024-2025 [LDC Labs](https://github.com/ldclabs).

Licensed under the MIT License. See [LICENSE](../../LICENSE-MIT) for details.