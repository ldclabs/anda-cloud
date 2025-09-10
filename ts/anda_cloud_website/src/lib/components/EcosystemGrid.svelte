<script lang="ts">
  import { staggerChildren } from '$lib/animations/motion'
  import FeatureCard from './FeatureCard.svelte'

  const ecosystem = [
    {
      id: 'anda',
      icon: '🤖',
      title: 'Anda Framework',
      tagline: 'Agent-to-Agent Runtime',
      body: 'Composable Rust framework for autonomous agents with on-chain identity, cryptography, and secure execution.',
      highlights: ['ICP identity layer', 'Composable toolchain', 'TEE-ready'],
      link: 'https://github.com/ldclabs/anda'
    },
    {
      id: 'kip',
      icon: '🧠',
      title: 'KIP Protocol',
      tagline: 'A Knowledge Interaction Protocol',
      body: 'Declarative instructions enabling LLMs to query, evolve and introspect a persistent cognitive memory graph.',
      highlights: [
        'Unified cognitive memory',
        'Transparent + auditable',
        'LLM-friendly syntax'
      ],
      link: 'https://github.com/ldclabs/KIP'
    },
    {
      id: 'andadb',
      icon: '🗄️',
      title: 'Anda DB',
      tagline: 'Cognitive Nexus & Indexing',
      body: 'Purpose-built agent database: BTree, BM25, HNSW, hybrid retrieval + KIP-native knowledge graph.',
      highlights: [
        'Graph + vector synergy',
        'Multimodal data',
        'Pluggable + Encrypted storage'
      ],
      link: 'https://github.com/ldclabs/anda-db'
    },
    {
      id: 'andacloud',
      icon: '☁️',
      title: 'Anda Cloud',
      tagline: 'Decentralized Agent Services',
      body: 'On-chain registration, discovery & forthcoming payment + gateway services for a global agent economy.',
      highlights: ['Registry & discovery', 'Payment (WIP)', 'Gateway (WIP)'],
      link: 'https://github.com/ldclabs/anda-cloud'
    }
  ]
</script>

<div class="grid gap-6 md:grid-cols-2 xl:grid-cols-4" use:staggerChildren>
  {#each ecosystem as item}
    <FeatureCard
      id={item.id}
      icon={item.icon}
      title={item.title}
      body={item.body}
      bullets={item.highlights}
      link={item.link}
    />
  {/each}
</div>
