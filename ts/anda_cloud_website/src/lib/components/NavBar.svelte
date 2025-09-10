<script lang="ts">
  import { onMount } from 'svelte'

  const navItems = [
    { href: '#project-kip', label: 'KIP' },
    { href: '#project-anda', label: 'Anda' },
    { href: '#project-andadb', label: 'Anda DB' },
    { href: '#project-andacloud', label: 'Anda Cloud' },
    { href: 'https://x.com/ICPandaDAO', label: 'Follow Usˆ', external: true }
  ]

  // Props / state
  let mobileMenuOpen = $state(false)
  // Dynamic light/dark detection:
  let lightMode = $state(false)

  function sampleBackground() {
    if (typeof window === 'undefined') return
    const x = window.innerWidth / 2
    // sample a few pixels below nav (nav height ~64px)
    const y = 64 + 4
    let el = document.elementFromPoint(x, y) as HTMLElement | null
    let detected: boolean | null = null
    const MAX_DEPTH = 6
    let depth = 0
    while (el && depth < MAX_DEPTH) {
      if (el.classList?.contains('section-light')) {
        detected = true
        break
      }
      if (el.classList?.contains('section-dark')) {
        detected = false
        break
      }
      el = el.parentElement
      depth++
    }
    lightMode = !!detected
  }

  let samplingRaf: number | null = null
  function scheduleSample() {
    if (samplingRaf != null) return
    samplingRaf = requestAnimationFrame(() => {
      samplingRaf = null
      sampleBackground()
    })
  }

  // Wrapper classes
  const navShellCss = $derived(
    lightMode
      ? 'bg-white/80 supports-[backdrop-filter]:bg-white/65 border-black/10 text-black'
      : 'bg-[rgba(11,13,15,0.72)]/90 border-white/5 text-white'
  )
  const navItemCss = $derived(
    lightMode
      ? 'text-black/60 hover:bg-black/20 hover:text-black'
      : 'text-white/60 hover:bg-white/20 hover:text-white'
  )

  // --- Scroll direction logic: scroll down -> hide, scroll up -> show ---
  let hidden = $state(false)
  let lastY = 0 // last scroll position
  let dir: 'up' | 'down' = 'up'
  let acc = 0 // accumulated delta in current direction
  const SWITCH_DELTA = 18 // px needed before toggling visibility
  const TOP_STICKY = 12 // always show within this distance from top

  function handleScroll() {
    const y = window.scrollY
    if (mobileMenuOpen) {
      hidden = false
      lastY = y
      return
    }
    const dy = y - lastY
    if (Math.abs(dy) < 2) return // ignore micro jitter

    // Always show near absolute top
    if (y <= TOP_STICKY) {
      hidden = false
      dir = 'up'
      acc = 0
      lastY = y
      lightMode = false
      return
    }

    if (dy > 0) {
      // scrolling down
      if (dir === 'up') {
        dir = 'down'
        acc = 0
      }
      acc += dy
      if (acc > SWITCH_DELTA) hidden = true
    } else {
      // scrolling up
      if (dir === 'down') {
        dir = 'up'
        acc = 0
      }
      acc += dy // dy negative
      if (-acc > SWITCH_DELTA) hidden = false
    }
    lastY = y
    sampleBackground()
  }

  onMount(() => {
    lastY = window.scrollY
    window.addEventListener('scroll', handleScroll, { passive: true })
    window.addEventListener('resize', scheduleSample, { passive: true })
    sampleBackground()

    return () => {
      window.removeEventListener('scroll', handleScroll)
      window.removeEventListener('resize', scheduleSample)
    }
  })
</script>

<nav
  class={`fixed top-0 right-0 left-0 z-50 backdrop-blur-xl transition-[background-color,transform,opacity] duration-300 will-change-transform ${navShellCss} ${hidden ? '-translate-y-full opacity-0' : 'translate-y-0 opacity-100'}`}
>
  <div class="mx-auto flex h-16 max-w-7xl items-center gap-6 px-4">
    <a href="/" class="group flex items-center gap-2">
      <img
        src="/_assets/images/anda.svg"
        alt="Anda Logo"
        class="h-10 rounded bg-emerald-500 px-2 py-1"
      />
    </a>
    <div class="ml-auto hidden items-center gap-1 md:flex">
      {#each navItems as item}
        {#if !item.external}
          <a
            href={item.href}
            class="relative rounded-md px-3 py-2 text-sm font-medium ${navItemCss}"
          >
            {item.label}
          </a>
        {:else}
          <a
            href={item.href}
            target="_blank"
            rel="noopener"
            class="relative rounded-md px-3 py-2 text-sm font-medium ${navItemCss}"
            >{item.label}</a
          >
        {/if}
      {/each}
    </div>
    <button
      class="ml-auto rounded p-2 hover:bg-white/10 md:hidden"
      onclick={() => (mobileMenuOpen = !mobileMenuOpen)}
      aria-label="Menu"
    >
      <div class="mb-1 h-0.5 w-6 bg-white"></div>
      <div class="mb-1 h-0.5 w-6 bg-white"></div>
      <div class="h-0.5 w-6 bg-white"></div>
    </button>
  </div>
  {#if mobileMenuOpen}
    <div
      class={`border-t md:hidden ${lightMode ? 'border-black/10 bg-white/85' : 'border-white/5 bg-[var(--color-bg-alt)]'}`}
    >
      <div class="flex flex-col gap-2 px-4 py-4">
        {#each navItems as item}
          <a
            href={item.href}
            class={`rounded px-3 py-2 text-sm transition ${lightMode ? 'text-black/70 hover:bg-black/5 hover:text-black' : 'hover:bg-white/5'}`}
            onclick={() => (mobileMenuOpen = false)}>{item.label}</a
          >
        {/each}
      </div>
    </div>
  {/if}
</nav>

<style>
  nav {
    --tw-backdrop-blur: blur(12px);
  }
</style>
