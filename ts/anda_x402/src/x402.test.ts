// (c) 2023-present, LDC Labs. All rights reserved.
// See the file LICENSE for licensing terms.

import { deterministicEncode } from '@ldclabs/ic-auth'
import { sha3_256 } from '@noble/hashes/sha3.js'
import { assert, describe, it } from 'vitest'
import {
  ASSET_TRANSFER_METHOD_ICRC2,
  NETWORK_ICP,
  assetTransferMethodOf,
  formatAmount,
  isSupportedRequirement,
  paymentFlowOf
} from './types.js'
import type { IcpPayloadAuthorization, PaymentRequirements } from './types.js'

describe('IcpPayload', () => {
  it('IcpPayloadAuthorization digest', async () => {
    const auth: IcpPayloadAuthorization = {
      to: '77ibd-jp5kr-moeco-kgoar-rro5v-5tng4-krif5-5h2i6-osf2f-2sjtv-kqe',
      value: '100000000',
      expiresAt: 1761536123382,
      nonce: 42
    }
    const digest = sha3_256(deterministicEncode(auth))
    assert.equal(
      Buffer.from(digest).toString('hex'),
      '269d40d6a23a75d9e4935d3010a8b8327115bb3dbadc7c311f43fec2445ae8f9'
    )
  })
})

describe('formatAmount', () => {
  // `formatAmount` renders in the ambient locale, so build the expectations
  // with that locale's decimal separator instead of hardcoding '.'.
  const decimalSep = new Intl.NumberFormat(
    new Intl.Locale(
      (globalThis as { navigator?: { language?: string } }).navigator
        ?.language || 'en'
    )
  )
    .formatToParts(1.1)
    .find((p) => p.type === 'decimal')!.value
  const expect = (s: string) => s.replace('.', decimalSep)

  it('formats without losing precision', async () => {
    assert.equal(formatAmount(0n, 8), expect('0'))
    assert.equal(formatAmount(100000000n, 8), expect('1'))
    assert.equal(formatAmount(150000000n, 8), expect('1.5'))
    assert.equal(formatAmount(99999999n, 8), expect('0.999999'))
    assert.equal(formatAmount(-150000000n, 8), expect('-1.5'))
    assert.equal(formatAmount(123n, 0), expect('123'))
    // more than 53 bits of significand: must not round up to '1'
    assert.equal(formatAmount(999999999999999999n, 18), expect('0.999999'))
    assert.equal(formatAmount(1000000000000000001n, 18), expect('1'))
    assert.equal(formatAmount(12345678n, 8, 2), expect('0.12'))
  })
})

describe('reserved extra keys', () => {
  const base: PaymentRequirements = {
    scheme: 'exact',
    network: NETWORK_ICP,
    amount: '1000',
    asset: 'druyg-tyaaa-aaaaq-aactq-cai',
    payTo: '77ibd-jp5kr-moeco-kgoar-rro5v-5tng4-krif5-5h2i6-osf2f-2sjtv-kqe',
    maxTimeoutSeconds: 300
  }
  const withExtra = (extra: Record<string, unknown>): PaymentRequirements => ({
    ...base,
    extra
  })

  it('defaults to the icp mechanism when the keys are absent', () => {
    assert.equal(paymentFlowOf(base), 'authorization')
    assert.equal(assetTransferMethodOf(base), ASSET_TRANSFER_METHOD_ICRC2)
    assert.isTrue(isSupportedRequirement(base))
  })

  it('reads the flows the icp mechanism can and cannot serve', () => {
    assert.equal(
      paymentFlowOf(withExtra({ paymentFlow: 'upfront' })),
      'upfront'
    )
    assert.isTrue(isSupportedRequirement(withExtra({ paymentFlow: 'upfront' })))

    // A protocol flow this mechanism has no two-step settlement for.
    assert.equal(paymentFlowOf(withExtra({ paymentFlow: 'escrow' })), 'escrow')
    assert.isFalse(isSupportedRequirement(withExtra({ paymentFlow: 'escrow' })))

    // A flow from a future revision must not be guessed at.
    assert.throws(() => paymentFlowOf(withExtra({ paymentFlow: 'someday' })))
    assert.isFalse(
      isSupportedRequirement(withExtra({ paymentFlow: 'someday' }))
    )
  })

  it('rejects asset transfer methods of other networks', () => {
    assert.equal(
      assetTransferMethodOf(withExtra({ assetTransferMethod: 'eip3009' })),
      'eip3009'
    )
    assert.isFalse(
      isSupportedRequirement(withExtra({ assetTransferMethod: 'eip3009' }))
    )
    assert.isTrue(
      isSupportedRequirement(withExtra({ assetTransferMethod: 'icrc2' }))
    )
    assert.throws(() =>
      assetTransferMethodOf(withExtra({ assetTransferMethod: 2 }))
    )
  })
})
