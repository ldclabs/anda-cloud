// (c) 2023-present, LDC Labs. All rights reserved.
// See the file LICENSE for licensing terms.

import { deterministicEncode } from '@ldclabs/ic-auth'
import { sha3_256 } from '@noble/hashes/sha3'
import { assert, describe, it } from 'vitest'
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
