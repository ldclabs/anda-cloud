// (c) 2023-present, LDC Labs. All rights reserved.
// See the file LICENSE for licensing terms.

import { assert, describe, it } from 'vitest'
import { deterministicEncode } from '@ldclabs/ic-auth'
import { sha3_256 } from '@noble/hashes/sha3'
import type { IcpPayloadAuthorization } from './types.js'

describe('IcpPayload', () => {
  it('IcpPayloadAuthorization digest', async () => {
    const auth: IcpPayloadAuthorization = {
      scheme: 'exact',
      asset: 'druyg-tyaaa-aaaaq-aactq-cai',
      to: '77ibd-jp5kr-moeco-kgoar-rro5v-5tng4-krif5-5h2i6-osf2f-2sjtv-kqe',
      value: '100000000',
      expiresAt: 1761536123382,
      nonce: 42
    }
    const digest = sha3_256(deterministicEncode(auth))
    assert.equal(
      Buffer.from(digest).toString('hex'),
      '14a9c138b21790526a43aa8ca2bb1f0c3618eda2fe02347002cac8f11b255cfc'
    )
  })
})
