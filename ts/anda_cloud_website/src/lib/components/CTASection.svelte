<script lang="ts">
  import { fadeIn, parallax } from '$lib/animations/motion'
  import { neuralGrid } from '$lib/animations/neuralGrid'
  import SectionHeading from '$lib/components/SectionHeading.svelte'
</script>

<section
  id="get-started"
  use:neuralGrid={{ palette: 'blue' }}
  class="section-light relative min-h-screen py-24"
>
  <div use:parallax={{ speed: 0.1 }} class="">
    <div
      use:fadeIn={{ y: 40, duration: 600 }}
      class="w-fullmax-w-6xl relative mx-auto px-4"
    >
      <SectionHeading
        eyebrow="Meet Anda"
        title="Meet Anda AI"
        subtitle="Anda is your personal symbiotic agent—an extension of your own mind."
        align="center"
      />
      <div class="mx-auto max-w-4xl text-center">
        <img
          src="/_assets/images/anda-ai-app.webp"
          alt="Anda AI App"
          class="m-auto mb-8"
        />
        <ul class="m-auto mb-8 space-y-2 text-left text-sm">
          <li class=""
            >It begins by learning <b>locally</b>, right on your device,
            building a private memory that evolves as you do. Then, it
            <b>acts</b> as your trusted proxy across the Anda network, discovering,
            collaborating, and transacting on your behalf.</li
          >
          <li class=""
            >Ultimately, Anda becomes your secure digital self in the emerging
            agentic web.</li
          >
          <li class="font-bold"
            >It’s time to stop just reading about the future. It’s time to own a
            piece of it.</li
          >
        </ul>
        <div
          class="flex flex-col items-center justify-center gap-4 sm:flex-row"
        >
          <a
            href="https://github.com/ldclabs/anda-app/releases/latest"
            target="_blank"
            rel="noopener"
            class="group relative inline-flex h-12 items-center justify-center rounded-full bg-emerald-500 px-8 text-lg font-medium text-black shadow-[0_4px_24px_-4px_rgba(61,220,132,0.55)] transition hover:brightness-95 focus:ring-2 focus:ring-emerald-500/60 focus:outline-none"
          >
            <span class="mr-2">⬇</span> Download Anda AI
          </a>
          <a
            href="https://x.com/ICPandaDAO"
            target="_blank"
            rel="noopener"
            class="inline-flex h-12 items-center rounded-full border border-black/10 px-8 font-medium hover:bg-black/5"
            >Follow Us on X →</a
          >
        </div>
      </div>
    </div>
  </div>
</section>
