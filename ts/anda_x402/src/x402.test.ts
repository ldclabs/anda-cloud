// (c) 2023-present, LDC Labs. All rights reserved.
// See the file LICENSE for licensing terms.

import { deterministicEncode } from '@ldclabs/ic-auth'
import { sha3_256 } from '@noble/hashes/sha3'
import { assert, describe, it } from 'vitest'
import { formatAmount } from './types.js'
import type { IcpPayloadAuthorization } from './types.js'

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
    new Intl.Locale(globalThis?.navigator?.language || 'en')
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
