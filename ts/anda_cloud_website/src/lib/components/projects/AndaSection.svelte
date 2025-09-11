<script lang="ts">
  import { fadeIn, parallax } from '$lib/animations/motion'
  import { neuralGrid } from '$lib/animations/neuralGrid'
  import ArrowDownLine from '$lib/icons/arrow-down-line.svelte'
  import ArrowRightLine from '$lib/icons/arrow-right-line.svelte'

  const codeSnippet = `anda/
├── anda_core/        # Core library containing base types and interfaces
├── anda_engine/      # Engine implementation for agent runtime and management
├── anda_engine_server/ # A http server to serve multiple Anda engines
├── anda_web3_client/ # The Rust SDK for Web3 integration in non-TEE environments
├── anda_cli/         # The command line interface for Anda engine server
├── agents/           # Various Agent implementations
│ ├── anda_assistant/ # An AI Assistant powered by the Knowledge Interaction Protocol (KIP).
│ └── .../            # More agents in future releases
├── tools/            # Various Tool libraries
│ ├── anda_icp/       # Anda agent tools offers integration with the Internet Computer (ICP).
│ └── .../            # More tools in future releases
└── examples/         # AI agents examples
`
</script>

<section
  id="project-anda"
  use:neuralGrid={{ palette: 'dark' }}
  class="section-light relative flex min-h-screen items-center overflow-hidden py-24 will-change-transform"
>
  <div
    use:parallax
    class="mx-auto grid w-full max-w-6xl items-center gap-20 px-4 lg:grid-cols-2"
  >
    <div use:fadeIn={{ y: 40, duration: 600 }}>
      <h2 class="mb-6 text-4xl font-semibold tracking-tight md:text-5xl">
        Anda: Build More Than Agents. Build Ecosystems.</h2
      >
      <p class="mb-8 text-lg leading-relaxed"
        >Anda provides the open-source Rust runtime and cryptographic primitives
        to create complex, multi-agent systems that can securely collaborate,
        transact, and build shared intelligence.</p
      >
      <ul class="mb-10 space-y-3 text-sm">
        <li class="flex gap-2"
          ><span class="text-emerald-500">•</span><span
            ><b>Dynamic Skill Composition:</b> Build agents that learn and adapt
            by composing complex capabilities, not just calling static tools.</span
          ></li
        >
        <li class="flex gap-2"
          ><span class="text-emerald-500">•</span><span
            ><b>Effortless Multi-Agent Orchestration:</b> Deploy sophisticated swarm
            and market dynamics with built-in patterns, not complex custom code.</span
          ></li
        >
        <li class="flex gap-2"
          ><span class="text-emerald-500">•</span><span
            ><b>Cryptographic Identity:</b> Forge trustworthy agents with non-spoofable,
            on-chain identities rooted in ICP principals.</span
          ></li
        >
        <li class="flex gap-2"
          ><span class="text-emerald-500">•</span><span
            ><b>Privacy by Design:</b> Architected for TEEs, enabling confidential
            agent state and operations in a zero-trust world.</span
          ></li
        >
      </ul>
      <div class="flex flex-wrap gap-3">
        <a
          href="https://github.com/ldclabs/anda"
          target="_blank"
          class="flex h-12 items-center gap-1 rounded-full bg-emerald-500 px-8 font-medium text-black shadow hover:brightness-95"
          ><span>Source Code</span><span><ArrowRightLine /></span></a
        >
        <a
          href="#project-kip"
          class="flex h-12 items-center gap-1 rounded-full border border-black/10 px-8 font-medium hover:bg-black/5"
          ><span>Memory Layer</span><span><ArrowDownLine /></span></a
        >
      </div>
    </div>
    <div class="relative" use:fadeIn={{ y: 40, duration: 700, delay: 120 }}>
      <div
        class="grid rounded-xl border border-white/10 bg-black p-2 text-xs shadow"
      >
        <pre
          class="overflow-auto rounded-lg p-2 [font-family:ui-monospace,monospace] leading-relaxed text-white"
          ><code>{codeSnippet}</code></pre
        >
      </div>
    </div>
  </div>
</section>
