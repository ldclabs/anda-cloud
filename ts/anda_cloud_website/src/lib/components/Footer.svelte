<script lang="ts">
  import ArrowRightUpLine from '$lib/icons/arrow-right-up-line.svelte'

  const year = new Date().getFullYear()
</script>

<footer id="about" class="border-t border-white/10 py-16 text-sm">
  <div class="mx-auto grid max-w-6xl gap-10 px-4 md:grid-cols-4">
    <div class="space-y-4 md:col-span-2">
      <img
        src="/_assets/images/anda.svg"
        alt="Anda Logo"
        class="h-14 rounded bg-cyan-500 px-4 py-2"
      />
      <p class="max-w-sm text-sm leading-relaxed text-white/80"
        >We are building the foundational infrastructure and applications that
        empower AI agents to thrive as first-class citizens in the Web3.</p
      >
    </div>
    <div>
      <h4 class="mb-3 font-semibold">Resources</h4>
      <ul class="space-y-2 text-[var(--color-text-dim)]">
        <li>
          <a
            class="flex items-center gap-1 hover:text-white"
            href="https://github.com/ldclabs/KIP"
            target="_blank"
          >
            <span>KIP Specification</span>
            <span class="*:size-4"><ArrowRightUpLine /></span>
          </a>
        </li>
        <li>
          <a
            class="flex items-center gap-1 hover:text-white"
            href="https://github.com/ldclabs/anda"
            target="_blank"
          >
            <span>Anda Framework</span>
            <span class="*:size-4"><ArrowRightUpLine /></span>
          </a>
        </li>
        <li>
          <a
            class="flex items-center gap-1 hover:text-white"
            href="https://github.com/ldclabs/anda-db"
            target="_blank"
          >
            <span>Anda DB</span>
            <span class="*:size-4"><ArrowRightUpLine /></span>
          </a>
        </li>
        <li>
          <a
            class="flex items-center gap-1 hover:text-white"
            href="https://github.com/ldclabs/anda-cloud"
            target="_blank"
          >
            <span>Anda Cloud</span>
            <span class="*:size-4"><ArrowRightUpLine /></span>
          </a>
        </li>
        <li>
          <a
            class="flex items-center gap-1 hover:text-white"
            href="https://github.com/ldclabs/anda-app"
            target="_blank"
          >
            <span>Anda App</span>
            <span class="*:size-4"><ArrowRightUpLine /></span>
          </a>
        </li>
      </ul>
    </div>
    <div>
      <h4 class="mb-3 font-semibold">About</h4>
      <ul class="space-y-2 text-[var(--color-text-dim)]">
        <li
          ><a
            class="flex items-center gap-1 hover:text-white"
            href="https://panda.fans"
            target="_blank"
          >
            <span>Panda.fans</span>
            <span class="*:size-4"><ArrowRightUpLine /></span>
          </a>
        </li>
        <li
          ><a
            class="flex items-center gap-1 hover:text-white"
            href="https://dmsg.net/PANDA"
            target="_blank"
          >
            <span>dMsg.net</span>
            <span class="*:size-4"><ArrowRightUpLine /></span>
          </a>
        </li>
        <li
          ><a
            class="flex items-center gap-1 hover:text-white"
            href="https://github.com/ldclabs"
            target="_blank"
          >
            <span>Github</span>
            <span class="*:size-4"><ArrowRightUpLine /></span>
          </a>
        </li>
        <li
          ><a
            class="flex items-center gap-1 hover:text-white"
            href="https://x.com/ICPandaDAO"
            target="_blank"
          >
            <span>X (Twitter)</span>
            <span class="*:size-4"><ArrowRightUpLine /></span>
          </a>
        </li>
      </ul>
    </div>
  </div>
  <div class="mt-14 text-center text-xs text-[var(--color-text-dim)]"
    >© {year}
    <a
      href="https://dashboard.internetcomputer.org/sns/d7wvo-iiaaa-aaaaq-aacsq-cai"
      target="_blank"
      class="text-emerald-500 underline underline-offset-2">ICPanda DAO</a
    >. All rights reserved.</div
  >
</footer>
