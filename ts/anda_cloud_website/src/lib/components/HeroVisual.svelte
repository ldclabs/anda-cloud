<script lang="ts">
  import { onMount } from 'svelte'
  let canvas: HTMLCanvasElement
  let ctx: CanvasRenderingContext2D | null = null
  let width = 0,
    height = 0,
    dpr = 1,
    raf = 0
  const mql =
    typeof window !== 'undefined'
      ? window.matchMedia('(prefers-reduced-motion: reduce)')
      : null

  // Configuration
  const ROTATION_SPEED = 0.00035 // globe slow spin
  const NODE_COUNT_DESKTOP = 42
  const NODE_COUNT_MOBILE = 21
  const TRANSMISSION_RATE = 0.012 // probability per frame to start a transmission
  const MAX_TRANSMISSIONS = 18
  const ARC_LIFT = 0.22 // arc height as fraction of sphere radius
  const FADE_NEAR_BACK = -0.35 // z threshold start fading

  interface Node {
    lat: number
    lon: number
    blink: number
    nextBlink: number
    hue: number
  }
  interface Transmission {
    a: number
    b: number
    t: number
    speed: number
  }
  let nodes: Node[] = []
  let transmissions: Transmission[] = []
  let rotation = 0

  function randLat() {
    // bias toward mid-latitudes for aesthetics
    return (Math.random() * 140 - 70) * (Math.random() * 0.6 + 0.4) // -70..70
  }
  function randLon() {
    return Math.random() * 360 - 180
  }

  function initNodes(count: number) {
    nodes = []
    for (let i = 0; i < count; i++) {
      // Two blue bands for subtle variation: cyan(195-205) & azure(212-225)
      const band =
        Math.random() < 0.55
          ? 195 + Math.random() * 10
          : 212 + Math.random() * 13
      nodes.push({
        lat: randLat(),
        lon: randLon(),
        blink: Math.random(),
        nextBlink: performance.now() + 500 + Math.random() * 2000,
        hue: band
      })
    }
  }

  function resize() {
    if (!canvas) return
    const rect = canvas.getBoundingClientRect()
    width = rect.width
    height = rect.height
    dpr = window.devicePixelRatio || 1
    canvas.width = Math.round(width * dpr)
    canvas.height = Math.round(height * dpr)
    ctx = canvas.getContext('2d')
    if (ctx) {
      ctx.setTransform(1, 0, 0, 1, 0, 0)
      ctx.scale(dpr, dpr)
    }
  }

  function project(lat: number, lon: number, R: number) {
    // Convert degrees to radians
    const phi = (lat * Math.PI) / 180 // latitude
    const theta = (lon * Math.PI) / 180 + rotation // longitude + rotation
    // Unit sphere (cartesian)
    const ux = Math.cos(phi) * Math.cos(theta)
    const uy = Math.sin(phi)
    const uz = Math.cos(phi) * Math.sin(theta)
    // Project directly to screen on sphere surface (no radial shrink) for true "on surface" feel
    const x = ux * R
    const y = uy * R * 0.9 // slight flatten for oblate aesthetic
    const z = uz * R
    // Brightness / size factor (0.3..1.0)
    const depth = (z / R + 1) / 2 // 0..1 front
    const bright = 0.3 + depth * 0.7
    return { x: width / 2 + x, y: height / 2 + y, z, bright }
  }

  function startTransmission() {
    if (transmissions.length >= MAX_TRANSMISSIONS) return
    const a = Math.floor(Math.random() * nodes.length)
    let b = Math.floor(Math.random() * nodes.length)
    if (a === b) b = (b + 1) % nodes.length
    // Avoid very close pairs (no visual interest)
    const al = nodes[a]
    const bl = nodes[b]
    const dLat = Math.abs(al.lat - bl.lat)
    const dLon = Math.abs(al.lon - bl.lon)
    if (dLat + dLon < 25) return // re-roll implicitly by skipping
    transmissions.push({ a, b, t: 0, speed: 0.0015 + Math.random() * 0.0015 })
  }

  function drawGlobeBase(R: number) {
    const g = ctx!
    const cx = width / 2,
      cy = height / 2
    // Outer glow
    const glow = g.createRadialGradient(cx, cy, R * 0.15, cx, cy, R * 1.1)
    glow.addColorStop(0, 'rgba(40,120,255,0.25)')
    glow.addColorStop(1, 'rgba(40,120,255,0)')
    g.fillStyle = glow
    g.beginPath()
    g.arc(cx, cy, R * 1.05, 0, Math.PI * 2)
    g.fill()
    // Sphere body
    const body = g.createRadialGradient(
      cx - R * 0.25,
      cy - R * 0.27,
      R * 0.18,
      cx,
      cy,
      R
    )
    body.addColorStop(0, '#071221')
    body.addColorStop(0.55, '#06101c')
    body.addColorStop(1, '#03070c')
    g.fillStyle = body
    g.beginPath()
    g.arc(cx, cy, R, 0, Math.PI * 2)
    g.fill()
    // Terminator (light rim)
    const rim = g.createRadialGradient(
      cx + R * 0.35,
      cy - R * 0.3,
      R * 0.25,
      cx,
      cy,
      R * 1.05
    )
    rim.addColorStop(0, 'rgba(80,180,255,0.18)')
    rim.addColorStop(1, 'rgba(80,180,255,0)')
    g.fillStyle = rim
    g.beginPath()
    g.arc(cx, cy, R * 1.02, 0, Math.PI * 2)
    g.fill()
  }

  function drawNodes(R: number, now: number) {
    const g = ctx!
    for (const n of nodes) {
      // Blink schedule
      if (now >= n.nextBlink) {
        n.blink = 1 // peak
        n.nextBlink = now + 800 + Math.random() * 2600
      } else {
        n.blink *= 0.94 // decay
      }
      const P = project(n.lat, n.lon, R)
      if (P.z < R * FADE_NEAR_BACK) continue // hide far-back to reduce clutter
      const alpha = (P.z / R + 1) / 2 // 0..1 depth factor
      const r = 2.2 + 2.4 * n.blink * (0.55 + alpha * 0.45) * P.bright
      const core = g.createRadialGradient(P.x, P.y, 0, P.x, P.y, r * 3.2)
      const h = n.hue
      core.addColorStop(0, `hsla(${h},95%,70%,${0.7 * n.blink + 0.2})`)
      core.addColorStop(0.55, `hsla(${h},95%,60%,${0.18 * n.blink})`)
      core.addColorStop(1, 'hsla(200,95%,60%,0)')
      g.fillStyle = core
      g.beginPath()
      g.arc(P.x, P.y, r * 2.2, 0, Math.PI * 2)
      g.fill()
      g.fillStyle = `hsla(${h},100%,${65 + 15 * n.blink}%,${0.55 + 0.45 * n.blink})`
      g.beginPath()
      g.arc(P.x, P.y, r * 0.55, 0, Math.PI * 2)
      g.fill()
    }
  }

  function interpolateGreatCircle(a: Node, b: Node, t: number) {
    // Convert to radians
    const φ1 = (a.lat * Math.PI) / 180,
      λ1 = (a.lon * Math.PI) / 180 + rotation
    const φ2 = (b.lat * Math.PI) / 180,
      λ2 = (b.lon * Math.PI) / 180 + rotation
    const sinφ1 = Math.sin(φ1),
      cosφ1 = Math.cos(φ1)
    const sinφ2 = Math.sin(φ2),
      cosφ2 = Math.cos(φ2)
    const Δ =
      2 *
      Math.asin(
        Math.sqrt(
          Math.sin((φ2 - φ1) / 2) ** 2 +
            cosφ1 * cosφ2 * Math.sin((λ2 - λ1) / 2) ** 2
        )
      )
    if (Δ === 0)
      return { x: cosφ1 * Math.cos(λ1), y: sinφ1, z: cosφ1 * Math.sin(λ1) }
    const A = Math.sin((1 - t) * Δ) / Math.sin(Δ)
    const B = Math.sin(t * Δ) / Math.sin(Δ)
    const x = A * cosφ1 * Math.cos(λ1) + B * cosφ2 * Math.cos(λ2)
    const y = A * sinφ1 + B * sinφ2
    const z = A * cosφ1 * Math.sin(λ1) + B * cosφ2 * Math.sin(λ2)
    return { x, y, z }
  }

  function drawTransmissions(R: number, now: number) {
    const g = ctx!
    // Advance transmissions
    for (let i = transmissions.length - 1; i >= 0; i--) {
      const tr = transmissions[i]
      tr.t += tr.speed * (1 + (Math.sin(now / 7000) + 1) * 0.15)
      if (tr.t >= 1) {
        transmissions.splice(i, 1)
        continue
      }
    }
    // Possibly add new
    if (Math.random() < TRANSMISSION_RATE) startTransmission()

    for (const tr of transmissions) {
      const a = nodes[tr.a]
      const b = nodes[tr.b]
      const steps = 42
      const path: Array<{ x: number; y: number; z: number }> = []
      for (let i = 0; i <= steps; i++) {
        const tt = i / steps
        const p3 = interpolateGreatCircle(a, b, tt)
        // lift arc (normalize then add height along radial direction)
        const lift = Math.sin(Math.PI * tt) * ARC_LIFT
        const mag = Math.sqrt(p3.x * p3.x + p3.y * p3.y + p3.z * p3.z)
        p3.x = (p3.x * (1 + lift)) / mag
        p3.y = (p3.y * (1 + lift)) / mag
        p3.z = (p3.z * (1 + lift)) / mag
        path.push(p3)
      }
      // Draw faint trail
      g.lineWidth = 1.2
      g.beginPath()
      let firstVisible = true
      for (let i = 0; i < path.length; i++) {
        const p = path[i]
        const z = p.z * R
        if (z < R * FADE_NEAR_BACK) {
          continue
        }
        const depth = (z / R + 1) / 2
        const sx = width / 2 + p.x * R
        const sy = height / 2 + p.y * R * 0.9
        if (firstVisible) {
          g.moveTo(sx, sy)
          firstVisible = false
        } else g.lineTo(sx, sy)
      }
      const hue = 200 + Math.sin(a.lat + a.lon + rotation * 400) * 12 // oscillate within blue range
      g.strokeStyle = `hsla(${hue},85%,60%,0.22)`
      g.stroke()

      // Moving packet (head) along arc
      const headT = Math.min(tr.t, 0.999)
      const headIndex = Math.floor(headT * (path.length - 1))
      const p = path[headIndex]
      const z = p.z * R
      if (z < R * FADE_NEAR_BACK) continue
      const depth = (z / R + 1) / 2
      const sx = width / 2 + p.x * R
      const sy = height / 2 + p.y * R * 0.9
      const r = 3.0 * (0.65 + depth * 0.35)
      const grad = g.createRadialGradient(sx, sy, 0, sx, sy, r * 3)
      grad.addColorStop(0, 'hsla(195,100%,72%,0.95)')
      grad.addColorStop(0.45, 'hsla(195,95%,60%,0.35)')
      grad.addColorStop(1, 'hsla(195,95%,55%,0)')
      g.fillStyle = grad
      g.beginPath()
      g.arc(sx, sy, r * 2.2, 0, Math.PI * 2)
      g.fill()
      g.fillStyle = '#fff'
      g.beginPath()
      g.arc(sx, sy, r * 0.65, 0, Math.PI * 2)
      g.fill()
    }
  }

  function frame(now: number) {
    if (!ctx) return
    ctx.clearRect(0, 0, width, height)
    const R = Math.min(width, height) * 0.42
    rotation += ROTATION_SPEED * (1 + Math.sin(now / 10000) * 0.15)
    drawGlobeBase(R)
    drawTransmissions(R, now)
    drawNodes(R, now)
    raf = requestAnimationFrame(frame)
  }

  onMount(() => {
    if (mql && mql.matches) return // reduced motion: keep static fallback (no nodes)
    resize()
    initNodes(window.innerWidth < 820 ? NODE_COUNT_MOBILE : NODE_COUNT_DESKTOP)
    const ro = new ResizeObserver(() => {
      resize()
    })
    ro.observe(canvas)
    raf = requestAnimationFrame(frame)
    return () => {
      cancelAnimationFrame(raf)
      ro.disconnect()
    }
  })
</script>

<div class="relative size-[380px] select-none">
  <canvas bind:this={canvas} class="absolute inset-0 h-full w-full"></canvas>
  <div
    class="pointer-events-none absolute inset-0 rounded-[32px] border border-white/5 [mask-image:radial-gradient(circle_at_50%_50%,black,transparent_78%)]"
  ></div>
  <div
    class="pointer-events-none absolute inset-0 rounded-[32px] bg-[radial-gradient(circle_at_50%_55%,rgba(80,170,255,0.14),transparent_70%)]"
  ></div>
</div>

<style>
  canvas {
    filter: drop-shadow(0 0 18px rgba(90, 180, 255, 0.18));
  }
</style>
