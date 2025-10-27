import { X402Canister, PaymentRequirementsResponse } from '@ldclabs/anda_x402'
import { Ed25519KeyIdentity } from '@ldclabs/ic-auth'
import assert from 'node:assert'

// npx tsx app.ts
async function main() {
  const canisterId = 'ogkpr-lyaaa-aaaap-an5fq-cai' // Replace with your canister ID
  const assetId = 'druyg-tyaaa-aaaaq-aactq-cai' // PANDA token canister ID
  const identity = Ed25519KeyIdentity.fromSecretKey(new Uint8Array(32).fill(8))
  assert.equal(
    identity.getPrincipal().toText(),
    'jjn6g-sh75l-r3cxb-wxrkl-frqld-6p6qq-d4ato-wske5-op7s5-n566f-bqe'
  )

  const x402 = new X402Canister(canisterId, identity, 'http://localhost:4943')

  const info = await x402.getInfo()
  console.log('Info:', info)

  const req: PaymentRequirementsResponse = {
    x402Version: 1,
    error: 'some error',
    accepts: [
      {
        scheme: 'exact',
        network: x402.network,
        maxAmountRequired: '100000000', // 1 PANDA
        asset: assetId,
        payTo: 'rrkah-fqaaa-aaaaa-aaaaq-cai',
        resource: 'https://github.com/ldclabs',
        description: 'Payment for some resource',
        maxTimeoutSeconds: 300
      }
    ]
  }

  // Client: approve allowance
  await x402.ensureAllowance(assetId, BigInt(100000000 + 10000)) // 1 PANDA + fee

  // Client: build x402 request
  const x402Request = await x402.buildX402Request(req, assetId)
  console.log('\n\nX402 Request:', x402Request)

  // Resource server: verify x402 request
  const verifyRes = await fetch(
    `http://127.0.0.1:4943/verify?canisterId=${canisterId}`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(x402Request)
    }
  )
  const verifyJson = await verifyRes.json()
  console.log('\n\nVerify Response:', verifyJson)
  // Verify Response: {
  //   isValid: true,
  //   payer: 'jjn6g-sh75l-r3cxb-wxrkl-frqld-6p6qq-d4ato-wske5-op7s5-n566f-bqe'
  // }

  // Resource server: settle x402 request
  const settleRes = await fetch(
    `http://127.0.0.1:4943/settle?canisterId=${canisterId}`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(x402Request)
    }
  )
  const settleJson = await settleRes.json()
  console.log('\n\nSettle Response:', settleJson)
  // Settle Response: {
  //   success: true,
  //   transaction: 'druyg-tyaaa-aaaaq-aactq-cai:23',
  //   network: 'icp-ogkpr-lyaaa-aaaap-an5fq-cai',
  //   payer: 'jjn6g-sh75l-r3cxb-wxrkl-frqld-6p6qq-d4ato-wske5-op7s5-n566f-bqe'
  // }
}

main().catch((error) => {
  console.error('Error in main:', error)
})
