import {
  X402Canister,
  PaymentRequirementsResponse,
  formatAmount
} from '@ldclabs/anda_x402'
import { Ed25519KeyIdentity } from '@ldclabs/ic-auth'
import * as readline from 'node:readline/promises'
import { stdin as input, stdout as output } from 'node:process'
import { readFileSync, writeFileSync } from 'node:fs'

// Run with:
// npx tsx app.ts
async function main() {
  const rl = readline.createInterface({ input, output })

  // This is the Anda x402 Facilitator
  // https://ogkpr-lyaaa-aaaap-an5fq-cai.icp0.io
  const facilitatorCanisterId = 'ogkpr-lyaaa-aaaap-an5fq-cai'

  // Load or create identity
  let identity: Ed25519KeyIdentity
  try {
    const identityJson = readFileSync('icp_x402_identity.json', 'utf-8')
    identity = Ed25519KeyIdentity.fromJSON(identityJson)
  } catch (_e) {
    identity = Ed25519KeyIdentity.generate()
    writeFileSync('icp_x402_identity.json', JSON.stringify(identity.toJSON()))
    console.log(
      `Generated new identity and saved to icp_x402_identity.json.\nPrincipal: ${identity
        .getPrincipal()
        .toText()}\n`
    )
  }

  const x402 = new X402Canister(facilitatorCanisterId, identity)
  const info = await x402.getInfo()
  console.log(`Welcome to the ${info.name}!`)
  console.log(`Your wallet principal: ${identity.getPrincipal().toText()}`)

  const assetChoices = Object.entries(info.supportedAssets || {})
  if (assetChoices.length === 0) {
    console.log('No supported assets found.')
    rl.close()
    return
  }

  console.log('\nPlease select a token to pay with:')
  assetChoices.forEach(([assetId, assetInfo], index) => {
    console.log(`${index + 1}) ${assetInfo.symbol} (${assetId})`)
  })

  const choiceAnswer = await rl.question(
    `Enter your choice (1-${assetChoices.length}): `
  )
  const choiceIndex = parseInt(choiceAnswer, 10) - 1
  const [assetId, selectedAssetInfo] = assetChoices[choiceIndex] || {}
  if (!selectedAssetInfo) {
    console.log(`Invalid choice ${choiceAnswer}`)
    rl.close()
    return
  }

  const amount = 1n * 10n ** BigInt(selectedAssetInfo.decimals)
  // https://dmsg.net/PANDA wallet
  const payTo =
    '77ibd-jp5kr-moeco-kgoar-rro5v-5tng4-krif5-5h2i6-osf2f-2sjtv-kqe'
  const req: PaymentRequirementsResponse = {
    x402Version: 1,
    error: 'some error',
    accepts: [
      {
        scheme: 'exact',
        network: x402.network,
        maxAmountRequired: amount.toString(),
        asset: assetId,
        payTo,
        resource: 'https://github.com/ldclabs',
        description: 'Payment for some resource',
        maxTimeoutSeconds: 300
      }
    ]
  }

  // Client: build x402 request
  const x402Request = await x402.buildX402Request(req, assetId)
  console.log(
    `\nTipping ${formatAmount(amount, selectedAssetInfo.decimals)} ${selectedAssetInfo.symbol} to https://dmsg.net/PANDA wallet (${payTo})`
  )
  console.log('\nX402 Request:', JSON.stringify(x402Request, null, 2))

  const before = await x402.getBalanceOf(assetId, identity.getPrincipal())
  console.log(
    `\nYour balance: ${formatAmount(before, selectedAssetInfo.decimals)} ${selectedAssetInfo.symbol}`
  )
  // Client: approve allowance
  const amountToApprove = amount + BigInt(selectedAssetInfo.transferFee)
  const answer = await rl.question(
    `Approve allowance of ${formatAmount(amountToApprove, selectedAssetInfo.decimals)} ${selectedAssetInfo.symbol} (with ${formatAmount(selectedAssetInfo.transferFee, selectedAssetInfo.decimals)} fee)? (y/N) `
  )

  if (before < amountToApprove) {
    console.log(
      `Insufficient balance. You need at least ${formatAmount(
        amountToApprove,
        selectedAssetInfo.decimals
      )} ${selectedAssetInfo.symbol}.`
    )
    rl.close()
    return
  }

  if (answer.toLowerCase() !== 'y' && answer.toLowerCase() !== 'yes') {
    console.log('Aborted by user.')
    rl.close()
    return
  }

  await x402.ensureAllowance(assetId, amountToApprove)
  console.log('Allowance approved, starting payment...')

  // Resource server: verify x402 request (optional)
  // const verifyRes = await fetch(`${x402.endpoint}/verify`, {
  //   method: 'POST',
  //   headers: { 'Content-Type': 'application/json' },
  //   body: JSON.stringify(x402Request)
  // })
  // const verifyJson = await verifyRes.json()
  // console.log('\n\nVerify Response:', verifyJson)
  // Verify Response: {
  //   isValid: true,
  //   payer: 'jjn6g-sh75l-r3cxb-wxrkl-frqld-6p6qq-d4ato-wske5-op7s5-n566f-bqe'
  // }

  // Resource server: settle x402 request
  const settleRes = await fetch(`${x402.endpoint}/settle`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(x402Request)
  })
  const settleJson = await settleRes.json()
  console.log('\n\nSettle Response:', settleJson)
  // Settle Response: {
  //   success: true,
  //   transaction: '3:druyg-tyaaa-aaaaq-aactq-cai:30',
  //   network: 'icp-ogkpr-lyaaa-aaaap-an5fq-cai',
  //   payer: 'jjn6g-sh75l-r3cxb-wxrkl-frqld-6p6qq-d4ato-wske5-op7s5-n566f-bqe'
  // }

  await new Promise((resolve) => setTimeout(resolve, 3000)) // wait for fee transaction
  const after = await x402.getBalanceOf(assetId, identity.getPrincipal())
  console.log(
    `Your balance:`,
    formatAmount(before, selectedAssetInfo.decimals),
    '->',
    formatAmount(after, selectedAssetInfo.decimals),
    selectedAssetInfo.symbol
  )

  // Client: list my payment logs
  const logs = await x402.listMyPaymentLogs(2)
  console.log('\n\nYour Payment Logs:', logs)
  rl.close()
}

main().catch((error) => {
  console.error('Error in main:', error)
})
