<script lang="ts">
  const year = new Date().getFullYear()
</script>

<footer class="border-t border-white/10 py-16 text-sm">
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
      <h4 class="mb-3 font-semibold">Ecosystem</h4>
      <ul class="space-y-2 text-[var(--color-text-dim)]">
        <li>
          <a
            class="hover:text-white"
            href="https://github.com/ldclabs/KIP"
            target="_blank">KIP</a
          >
        </li>
        <li>
          <a
            class="hover:text-white"
            href="https://github.com/ldclabs/anda"
            target="_blank">Anda</a
          >
        </li>
        <li>
          <a
            class="hover:text-white"
            href="https://github.com/ldclabs/anda-db"
            target="_blank">Anda DB</a
          >
        </li>
        <li>
          <a
            class="hover:text-white"
            href="https://github.com/ldclabs/anda-cloud"
            target="_blank">Anda Cloud</a
          >
        </li>
        <li>
          <a
            class="hover:text-white"
            href="https://github.com/ldclabs/anda-app"
            target="_blank">Anda App</a
          >
        </li>
      </ul>
    </div>
    <div>
      <h4 class="mb-3 font-semibold">Community</h4>
      <ul class="space-y-2 text-[var(--color-text-dim)]">
        <li
          ><a
            class="hover:text-white"
            href="https://github.com/ldclabs"
            target="_blank">Github</a
          ></li
        >
        <li
          ><a
            class="hover:text-white"
            href="https://x.com/ICPandaDAO"
            target="_blank">X (Twitter)</a
          ></li
        >
        <li
          ><a class="hover:text-white" href="https://dmsg.net/PANDA">dMsg.net</a
          ></li
        >
      </ul>
    </div>
  </div>
  <div class="mt-14 text-center text-xs text-[var(--color-text-dim)]"
    >© {year} ICPanda DAO. All rights reserved.</div
  >
</footer>
