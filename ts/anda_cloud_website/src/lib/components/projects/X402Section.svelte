<script lang="ts">
  import { fadeIn, parallax } from '$lib/animations/motion'
  import { neuralGrid } from '$lib/animations/neuralGrid'
  import ArrowDownLine from '$lib/icons/arrow-down-line.svelte'
  import ArrowRightLine from '$lib/icons/arrow-right-line.svelte'

  const serverExample = `// Monetize any resource with unparalleled ease.
app.use(
  paymentMiddleware(
    "your-icp-wallet-principal",
    {
      "/premium/*": {
        price: {
          amount: "10000000",
          asset: {
            address: "druyg-tyaaa-aaaaq-aactq-cai",
            decimals: 8,
          },
        },
        network: "icp",
      },
    },
    {
      url: "https://ogkpr-lyaaa-aaaap-an5fq-cai.icp0.io",
    },
  )
);`

  const settlementExample = `// 1. Server calls the on-chain facilitator
const response = await fetch(
  "https://ogkpr-lyaaa-aaaap-an5fq-cai.icp0.io/settle",
  {
    method: "POST",
    body: {
      "paymentPayload": {
        /* PaymentPayload */
      },
      "paymentRequirements": {
        /* PaymentRequirements */
      }
    }
  }
);

// 2. Payment is instantly final once the promise resolves.
// No polling, no waiting for block confirmations.
const { success, payer, transaction } = await response.json();

// 3. Serve the content to the user.
servePremiumContent();
`
  let tab: 'server' | 'settlement' = $state('server')
</script>

<section
  id="project-x402"
  use:neuralGrid
  class="section-light relative flex min-h-screen items-center overflow-hidden py-24 will-change-transform"
>
  <div
    use:parallax
    class="mx-auto grid w-full max-w-6xl items-center gap-20 px-4 lg:grid-cols-2"
  >
    <div use:fadeIn={{ y: 40, duration: 600 }}>
      <h2 class="mb-6 text-4xl font-semibold tracking-tight md:text-5xl"
        >x402 Payments, Evolved. Instant, On-Chain, and Multi-Asset.</h2
      >
      <p class="mb-8 text-lg leading-relaxed"
        >Leverage the power of the <a
            href="https://www.x402.org"
            target="_blank"
          ><b class="hover:text-emerald-500">x402 protocol</b></a> with Anda's fully on-chain
        facilitator on the Internet Computer. Experience true real-time payment
        settlement, unparalleled security, and the freedom to accept assets from
        any blockchain.</p
      >
      <ul class="mb-10 space-y-3 text-sm">
        <li class="flex gap-2"
          ><span class="text-emerald-500">•</span><span
            ><b>True Real-Time Settlement:</b> ICP's 1-3 second transaction finality
            means payments are confirmed the moment the <code>/settle</code> request
            completes. No polling, no waiting.</span
          ></li
        >
        <li class="flex gap-2"
          ><span class="text-emerald-500">•</span><span
            ><b>Unstoppable & Trustless:</b> Our entire facilitator logic runs on
            the blockchain as a verifiable smart contract, eliminating centralized
            points of failure.</span
          ></li
        >
        <li class="flex gap-2"
          ><span class="text-emerald-500">•</span><span
            ><b>Accept Any Token:</b> Powered by our <a
            href="https://1bridge.app"
            target="_blank"
          ><b class="hover:text-emerald-500">One Bridge</b></a> technology,
            will accept payments in tokens from Ethereum, BNB chain, Solana and other major
            blockchains seamlessly.</span
          ></li
        >
      </ul>
      <div class="flex flex-wrap gap-3">
        <a
          href="https://github.com/ldclabs/anda-cloud/tree/main/rs/anda_x402_canister"
          target="_blank"
          class="flex h-12 items-center gap-1 rounded-full bg-emerald-500 px-8 font-medium text-black shadow hover:brightness-95"
          ><span>Source Code</span><span><ArrowRightLine /></span></a
        >
        <a
          href="#get-started"
          class="flex h-12 items-center gap-1 rounded-full border border-black/10 px-8 font-medium hover:bg-black/5"
          ><span>Get Started</span><span><ArrowDownLine /></span></a
        >
      </div>
    </div>
    <div class="relative" use:fadeIn={{ y: 40, duration: 700, delay: 120 }}>
      <div class="mb-3 flex gap-2">
        <button
          onclick={() => (tab = 'server')}
          class="rounded border border-black/10 px-3 py-1.5 text-xs font-medium transition hover:border-black/20 {tab ===
          'server'
            ? 'bg-black/5 text-black'
            : 'text-black/50'}">Server Integration</button
        >
        <button
          onclick={() => (tab = 'settlement')}
          class="rounded border border-black/10 px-3 py-1.5 text-xs font-medium transition hover:border-black/20 {tab ===
          'settlement'
            ? 'bg-black/5 text-black'
            : 'text-black/50'}">Instant Settlement</button
        >
      </div>
      <div class="grid rounded-xl border border-black/5 bg-black/90 shadow">
        <div
          class="border-b border-white/10 px-4 py-2 text-xs tracking-wider text-white/50 uppercase"
          >{tab === 'server'
            ? '1 Line of Code to Monetize'
            : 'Instant Finality on ICP'}</div
        >
        <pre
          class="max-h-[420px] overflow-auto p-2 font-[ui-monospace,monospace] text-xs leading-relaxed text-white"
          ><code
            >{tab === 'server' ? serverExample : settlementExample}</code
          ></pre
        >
      </div>
    </div>
  </div>
</section>
