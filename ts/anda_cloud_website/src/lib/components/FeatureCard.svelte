<script lang="ts">
  import { fadeIn } from '$lib/animations/motion'
  interface Props {
    icon?: string
    title: string
    body: string
    bullets?: string[]
    link: string
    id?: string
  }

  let {
    icon = '💡',
    title,
    body,
    bullets = [],
    link,
    id = ''
  }: Props = $props()
</script>

<div
  {id}
  use:fadeIn={{ y: 28, duration: 600 }}
  class="group relative overflow-hidden rounded-xl border border-emerald-500/20 bg-gradient-to-br from-white/[0.06] to-white/[0.02] p-6 transition will-change-transform hover:border-emerald-500/50"
>
  <div
    class="pointer-events-none absolute inset-0 opacity-0 transition group-hover:opacity-100"
    style="background:radial-gradient(circle at 30% 20%, rgba(61,220,132,0.15), transparent 60%);"
  ></div>
  <div class="relative">
    <div class="animate-float mb-4 text-3xl select-none">{icon}</div>
    <h3 class="mb-2 flex items-center gap-2 text-xl font-semibold"
      >{title}
      <span
        class="rounded-full bg-violet-500/20 px-2 py-1 text-[10px] font-medium tracking-wider text-violet-500"
        >MIT</span
      >
    </h3>
    <p class="mb-4 text-sm leading-relaxed text-white/80">{body}</p>
    {#if bullets.length}
      <ul class="mb-5 space-y-1 text-xs">
        {#each bullets as b}
          <li class="flex items-start gap-2"
            ><span class="mt-0.5 text-emerald-500">•</span><span>{b}</span></li
          >
        {/each}
      </ul>
    {/if}
    <a
      href={link}
      target="_blank"
      rel="noopener"
      class="inline-flex items-center gap-1 text-sm font-medium hover:underline"
      >View on GitHub →</a
    >
  </div>
</div>
