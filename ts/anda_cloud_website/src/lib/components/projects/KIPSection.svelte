<script lang="ts">
  import { fadeIn, parallax } from '$lib/animations/motion'
  import { neuralGrid } from '$lib/animations/neuralGrid'
  const kqlExample = `// Find all non-NSAID drugs that treat 'Headache', have a risk level below 4
FIND(
  ?drug.name,
  ?drug.attributes.risk_level
)
WHERE {
  ?drug {type: "Drug"}
  ?headache {name: "Headache"}

  (?drug, "treats", ?headache)

  NOT {
    (?drug, "is_class_of", {name: "NSAID"})
  }

  FILTER(?drug.attributes.risk_level < 4)
}
ORDER BY ?drug.attributes.risk_level ASC
LIMIT 20
`
  const kmlExample = `// Description: Defines the novel nootropic drug "Cognizine" and its effects.
UPSERT {
  // Define the main drug concept: Cognizine
  CONCEPT ?cognizine {
    { type: "Drug", name: "Cognizine" }
    SET ATTRIBUTES {
      molecular_formula: "C12H15N5O3",
      dosage_form: { "type": "tablet", "strength": "500mg" },
      risk_level: 2,
      description: "A novel nootropic drug designed to enhance cognitive functions."
    }
    SET PROPOSITIONS {
      // Link to an existing concept (Nootropic)
      ("is_class_of", { type: "DrugClass", name: "Nootropic" })

      // Link to an existing concept (Brain Fog)
      ("treats", { type: "Symptom", name: "Brain Fog" })
    }
  }
}
WITH METADATA {
  // Global metadata for all facts in this capsule
  source: "KnowledgeCapsule:Nootropics_v1.0",
  author: "LDC Labs Research Team",
  confidence: 0.95,
  status: "reviewed"
}`

  const metaExample = `DESCRIBE CONCEPT TYPE "Drug"`
  let tab: 'kql' | 'kml' | 'meta' = $state('kql')
</script>

<section
  id="project-kip"
  use:neuralGrid={{ palette: 'dark' }}
  class="section-dark relative flex min-h-screen items-center overflow-hidden py-24 will-change-transform"
>
  <div
    use:parallax
    class="mx-auto grid w-full max-w-6xl items-center gap-20 px-4 lg:grid-cols-2"
  >
    <div use:fadeIn={{ y: 40, duration: 600 }}>
      <h2 class="mb-6 text-4xl font-semibold tracking-tight md:text-5xl"
        >KIP: The Language for AI Memory.</h2
      >
      <p class="mb-8 text-lg leading-relaxed"
        >An open, universal protocol that empowers AI agents to <b
          >Remember (KQL), Learn (KML), and Reflect (META)</b
        >. It transforms their internal world from a fleeting monologue into a
        persistent, structured dialogue.</p
      >
      <ul class="mb-10 space-y-3 text-sm">
        <li class="flex gap-2"
          ><span class="text-emerald-500">•</span><span
            ><b>Graph-Native Model:</b> Think in connections, not just documents.
            Unlocks complex, multi-hop reasoning impossible for standard RAG.</span
          ></li
        >
        <li class="flex gap-2"
          ><span class="text-emerald-500">•</span><span
            ><b>LLM-Friendly Syntax:</b> Less hallucination, more precision. Drastically
            simplifies the prompt engineering for reliable memory interaction.</span
          ></li
        >
        <li class="flex gap-2"
          ><span class="text-emerald-500">•</span><span
            ><b>Auditable by Design:</b> From intention to action, every memory operation
            is a verifiable log entry. Trust becomes programmable.</span
          ></li
        >
        <li class="flex gap-2"
          ><span class="text-emerald-500">•</span><span
            ><b>Metabolic Knowledge Loop:</b> UPSERT isn't just a command; it's a
            heartbeat. Enables continuous learning, correction, and growth.</span
          ></li
        >
      </ul>
      <div class="flex flex-wrap gap-3">
        <a
          href="https://github.com/ldclabs/KIP"
          target="_blank"
          class="inline-flex h-12 items-center rounded-full bg-emerald-500 px-8 font-medium text-black shadow hover:brightness-95"
          >KIP Spec →</a
        >
        <a
          href="#project-andadb"
          class="inline-flex h-12 items-center rounded-full border border-black/10 bg-white/10 px-8 font-medium backdrop-blur hover:bg-white/20"
          >Run It In Anda DB</a
        >
      </div>
    </div>
    <div class="relative" use:fadeIn={{ y: 40, duration: 700, delay: 120 }}>
      <div class="mb-3 flex gap-2">
        <button
          onclick={() => (tab = 'kql')}
          class="rounded border border-white/10 px-3 py-1.5 text-xs font-medium transition hover:border-white/30 {tab ===
          'kql'
            ? 'bg-white/10 text-white'
            : 'text-[var(--color-text-dim)]'}">KQL</button
        >
        <button
          onclick={() => (tab = 'kml')}
          class="rounded border border-white/10 px-3 py-1.5 text-xs font-medium transition hover:border-white/30 {tab ===
          'kml'
            ? 'bg-white/10 text-white'
            : 'text-[var(--color-text-dim)]'}">KML</button
        >
        <button
          onclick={() => (tab = 'meta')}
          class="rounded border border-white/10 px-3 py-1.5 text-xs font-medium transition hover:border-white/30 {tab ===
          'meta'
            ? 'bg-white/10 text-white'
            : 'text-[var(--color-text-dim)]'}">META</button
        >
      </div>
      <div class="grid rounded-xl border border-white/10 bg-white/90 shadow">
        <div
          class="border-b border-white/5 px-4 py-2 text-xs tracking-wider text-black/50 uppercase"
          >{tab === 'kql'
            ? 'Knowledge Query Language'
            : tab === 'kml'
              ? 'Knowledge Manipulation Language'
              : 'Introspection / META'}</div
        >
        <pre
          class="max-h-[420px] overflow-auto p-2 [font-family:ui-monospace,monospace] text-xs leading-relaxed text-black"
          ><code
            >{tab === 'kql'
              ? kqlExample
              : tab === 'kml'
                ? kmlExample
                : metaExample}</code
          ></pre
        >
      </div>
    </div>
  </div>
</section>
