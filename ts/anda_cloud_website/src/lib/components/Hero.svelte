<script lang="ts">
  import HeroVisual from '$lib/components/HeroVisual.svelte'
  import ArrowDownLongFill from '$lib/icons/arrow-down-long-fill.svelte'
  import ArrowRightLine from '$lib/icons/arrow-right-line.svelte'
</script>

<section
  class="section-dark relative mt-20 flex min-h-screen items-center overflow-hidden md:mt-0"
>
  <div class="relative m-auto grid max-w-6xl md:grid-cols-2 md:items-center">
    <div class="m-auto" aria-hidden="true">
      <HeroVisual />
      <p
        class="mt-6 hidden text-center text-xs tracking-wide text-white/80 md:block"
      >
        Anda Cloud: dynamic agent network visualization
      </p>
    </div>
    <div class="p-4">
      <div class="max-w-3xl">
        <div
          class="mb-6 inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs tracking-wider uppercase"
        >
          <span class="text-violet-500">Persistent Memory</span>
          <span class="text-cyan-500">Seamless Communication</span>
          <span class="text-zinc-200">Secure Trust</span>
          <span class="text-emerald-500">Open Source</span>
        </div>
        <h1
          class="mb-6 text-4xl leading-tight font-semibold tracking-tight md:text-6xl"
        >
          An Internet of Sovereign Minds
        </h1>
        <p class="mb-6 text-lg leading-relaxed md:text-xl">
          We are building the open-source, decentralized foundation for AI
          agents that <b>remember</b> (KIP),
          <b>interact</b>
          (Anda), and
          <b>evolve</b>—creating a new nervous system for the web.
        </p>
        <div class="flex flex-col gap-4 sm:flex-row">
          <a
            href="https://github.com/ldclabs/anda-app/releases/latest"
            target="_blank"
            rel="noopener"
            class="group relative flex h-14 items-center justify-center gap-1 rounded-full bg-emerald-500 px-8 text-lg font-medium text-black shadow-[0_4px_24px_-4px_rgba(61,220,132,0.55)] transition hover:brightness-95 focus:ring-2 focus:ring-emerald-500/60 focus:outline-none"
          >
            <span><ArrowDownLongFill /></span>
            <span>Download Anda AI</span>
          </a>
          <a
            href="https://github.com/ldclabs"
            target="_blank"
            rel="noopener"
            class="flex h-14 items-center justify-center gap-1 rounded-full border border-white/10 px-8 text-lg font-medium text-white/80 transition hover:border-white/30 hover:bg-white/5 hover:text-white focus:ring-2 focus:ring-white/30 focus:outline-none"
            ><span>Explore Stack</span><span><ArrowRightLine /></span></a
          >
        </div>
        <p class="text-md mt-8 text-white/80">
          Anda AI is your personal symbiotic agent — it learns locally, acts
          across the Anda network, and becomes your secure digital embodiment in
          the agentic web.
        </p>
      </div>
    </div>
  </div>
</section>
