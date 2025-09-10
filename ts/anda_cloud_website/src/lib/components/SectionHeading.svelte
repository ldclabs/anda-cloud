<script lang="ts">
  interface Props {
    eyebrow?: string
    title: string
    subtitle?: string
    align?: 'left' | 'center'
  }

  let { eyebrow = '', title, subtitle = '', align = 'left' }: Props = $props()
</script>

<div
  class={`mb-12 ${align === 'center' ? 'mx-auto max-w-4xl text-center' : ''}`}
>
  {#if eyebrow}
    <div
      class="mb-3 text-xs font-semibold tracking-wider text-emerald-500 uppercase"
      >{eyebrow}</div
    >
  {/if}
  <h2 class="mb-4 text-2xl font-semibold tracking-tight md:text-4xl">{title}</h2
  >
  {#if subtitle}
    <p class="text-base leading-relaxed text-white/80 md:text-lg">{subtitle}</p>
  {/if}
</div>
