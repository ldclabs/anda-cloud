<script lang="ts">
  import { fadeIn, parallax } from '$lib/animations/motion'
  import { neuralGrid } from '$lib/animations/neuralGrid'

  const codeSnippet = `impl MemoryManagement {
  pub async fn connect(db: Arc<AndaDB>, nexus: Arc<CognitiveNexus>) -> Result<Self, BoxError> {
    let schema = Conversation::schema()?;
    let conversations = db
      .open_or_create_collection(
        schema,
        CollectionConfig {
          name: "conversations".to_string(),
          description: "conversations collection".to_string(),
        },
        async |collection| {
          // set tokenizer
          collection.set_tokenizer(jieba_tokenizer());
          // create BTree indexes if not exists
          collection.create_btree_index_nx(&["user"]).await?;
          collection.create_btree_index_nx(&["thread"]).await?;
          collection.create_btree_index_nx(&["period"]).await?;
          collection
            .create_bm25_index_nx(&["messages", "resources", "artifacts"])
            .await?;

          Ok::<(), DBError>(())
        },
      )
      .await?;

    // ... other collections ...
  }
}`
</script>

<section
  id="project-andadb"
  use:neuralGrid
  class="section-light relative flex min-h-screen items-center overflow-hidden py-24 will-change-transform"
>
  <div
    use:parallax
    class="mx-auto grid w-full max-w-6xl items-center gap-20 px-4 lg:grid-cols-2"
  >
    <div use:fadeIn={{ y: 40, duration: 600 }}>
      <h2 class="mb-6 text-4xl font-semibold tracking-tight md:text-5xl"
        >Anda DB: The Polyglot Brain for Your AI.</h2
      >
      <p class="mb-8 text-lg leading-relaxed"
        >Anda DB is the first database that natively speaks every language of AI
        memory: <b>structured (BTree), semantic (HNSW), and symbolic (KIP)</b>.
        A single, high-performance Rust engine to power every thought.</p
      >
      <ul class="mb-10 space-y-3 text-sm">
        <li class="flex gap-2"
          ><span class="text-emerald-500">•</span><span
            ><b>BTree Indexing:</b> For when precision is non-negotiable. Get lightning-fast,
            deterministic lookups on structured attributes and agent metadata.</span
          ></li
        >
        <li class="flex gap-2"
          ><span class="text-emerald-500">•</span><span
            ><b>BM25 Full-Text Search:</b> Understand intent, not just keywords.
            Delivers superior relevance scoring with language-aware tokenization
            for natural language queries.</span
          ></li
        >
        <li class="flex gap-2"
          ><span class="text-emerald-500">•</span><span
            ><b>HNSW Vector Search:</b> Think in concepts, not just text. Discover
            deeply related information through high-recall approximate similarity
            search on embeddings.</span
          ></li
        >
        <li class="flex gap-2"
          ><span class="text-emerald-500">•</span><span
            ><b>KIP-Native Symbolic Graph:</b> The heart of the engine, not a bolt-on.
            Provides atomic UPSERT semantics for true knowledge evolution, making
            memory metabolic.</span
          ></li
        >
        <li class="flex gap-2"
          ><span class="text-emerald-500">•</span><span
            ><b>Encrypted & Incremental Storage:</b> A brain that's both secure and
            fast. Protects memory with at-rest encryption while maintaining peak
            performance with non-blocking index updates.</span
          ></li
        >
      </ul>
      <div class="flex flex-wrap gap-3">
        <a
          href="https://github.com/ldclabs/anda-db"
          target="_blank"
          class="inline-flex h-12 items-center rounded-full bg-emerald-500 px-8 font-medium text-black shadow hover:brightness-95"
          >Source Code →</a
        >
        <a
          href="#project-andacloud"
          class="inline-flex h-12 items-center rounded-full border border-black/10 px-8 font-medium hover:bg-black/5"
          >Cloud Layer</a
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
