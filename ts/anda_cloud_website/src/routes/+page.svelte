<script lang="ts">
  import { parallax } from '$lib/animations/motion'
  import { neuralGrid } from '$lib/animations/neuralGrid'
  import CTASection from '$lib/components/CTASection.svelte'
  import EcosystemGrid from '$lib/components/EcosystemGrid.svelte'
  import Footer from '$lib/components/Footer.svelte'
  import Hero from '$lib/components/Hero.svelte'
  import NavBar from '$lib/components/NavBar.svelte'
  import AndaCloudSection from '$lib/components/projects/AndaCloudSection.svelte'
  import AndaDBSection from '$lib/components/projects/AndaDBSection.svelte'
  import AndaSection from '$lib/components/projects/AndaSection.svelte'
  import KIPSection from '$lib/components/projects/KIPSection.svelte'
  import SectionHeading from '$lib/components/SectionHeading.svelte'
  import X402Section from '$lib/components/projects/X402Section.svelte'
</script>

<NavBar />
<main class="relative">
  <Hero />
  <section
    id="ecosystem"
    use:neuralGrid={{ palette: 'blue' }}
    class="section-light relative min-h-screen py-24"
  >
    <div use:parallax class="mx-auto max-w-6xl px-4">
      <SectionHeading
        eyebrow="Anda Stack"
        title="Memory-Centric Agents. An Evolutionary Network"
        subtitle="Four interoperable pillars: protocol, runtime, memory, and decentralized services."
        align="center"
      />
      <EcosystemGrid />
    </div>
  </section>

  <section
    use:neuralGrid={{ palette: 'dark' }}
    class="section-dark relative min-h-screen py-24"
  >
    <div use:parallax class="mx-auto max-w-6xl px-4">
      <SectionHeading
        eyebrow="Why It Matters"
        title="Where Code Becomes Cognition"
        subtitle="We're not just building better tools. We're laying the foundation for a new kind of lifeform: autonomous, persistent, and collaborative."
        align="center"
      />
      <div
        class="mt-10 grid gap-10 text-sm leading-relaxed text-white/80 md:grid-cols-3"
      >
        <div
          class="group relative overflow-hidden rounded-xl border border-violet-500/20 bg-gradient-to-br from-white/[0.06] to-white/[0.02] p-6 transition will-change-transform hover:border-violet-500/50"
        >
          <h3 class="mb-3 text-lg font-semibold text-white"
            >From Data to Memory: The Living Brain</h3
          >
          <p
            ><b class="text-white"
              >KIP transforms AI's relationship with knowledge.</b
            >
            Instead of just retrieving from static vector dumps, agents use a symbolic
            graph to build a living, metabolic memory. They don't just recall facts;
            they integrate new experiences, refine their understanding, and evolve
            their worldview.
            <b class="text-white"
              >This is the difference between a library and a mind.</b
            ></p
          >
        </div>
        <div
          class="group relative overflow-hidden rounded-xl border border-violet-500/20 bg-gradient-to-br from-white/[0.06] to-white/[0.02] p-6 transition will-change-transform hover:border-violet-500/50"
        >
          <h3 class="mb-3 text-lg font-semibold text-white"
            >From Anonymity to Identity: The Verifiable Self</h3
          >
          <p
            ><b class="text-white"
              >Trust is not a feature; it's a prerequisite.</b
            >
            Anda anchors each agent to a cryptographic identity on-chain. This isn't
            just a name, it's a sovereign self—a foundation for accountability and
            ownership. Every thought process becomes an auditable KQL query, and
            every piece of knowledge carries its verifiable provenance.
            <b class="text-white"
              >Intelligence without integrity is just noise.</b
            ></p
          >
        </div>
        <div
          class="group relative overflow-hidden rounded-xl border border-violet-500/20 bg-gradient-to-br from-white/[0.06] to-white/[0.02] p-6 transition will-change-transform hover:border-violet-500/50"
        >
          <h3 class="mb-3 text-lg font-semibold text-white"
            >From Isolation to Emergence: The Networked Mind</h3
          >
          <p
            ><b class="text-white">Intelligence blossoms in connection.</b>
            The Anda framework allows sovereign agents to discover, communicate,
            and collaborate in a secure peer-to-peer fabric. They can form alliances,
            exchange value, and contribute to shared knowledge graphs, bootstrapping
            a collective intelligence that surpasses any single agent.
            <b class="text-white"
              >This is how we move from building smart tools to cultivating a
              smart ecosystem.</b
            ></p
          >
        </div>
      </div>
    </div>
  </section>
  <AndaSection />
  <KIPSection />
  <AndaDBSection />
  <AndaCloudSection />
  <X402Section />
  <CTASection />
</main>
<Footer />
