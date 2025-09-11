// neuralGrid: draws luminous pulses traveling along implicit grid lines
// Applied to light sections to replace radial ripples with line pulses.
// Respects prefers-reduced-motion.

interface NeuralGridOptions {
  palette?: 'auto' | 'blue' | 'dark'
}

type Pulse = {
  x: number
  y: number
  dir: 'h' | 'v'
  sign: 1 | -1
  dist: number
  hue: number
}

export function neuralGrid(node: HTMLElement, opts: NeuralGridOptions = {}) {
  const mql = window.matchMedia('(prefers-reduced-motion: reduce)')
  if (mql.matches) return {}

  const spacing = 40
  const SPEED = 6
  const THICKNESS = 2
  const PULSE_LEN = 118
  const MAX_PULSES = 80
  const AMBIENT_MS = 2600
  const palette = opts.palette ?? 'auto'
  const effectivePalette =
    palette === 'auto'
      ? node.classList.contains('section-dark')
        ? 'dark'
        : 'blue'
      : palette
  const POINTER_INTERVAL_MS = 1800
  const POINTER_MIN_DISTANCE = spacing * 0.7

  const canvas = document.createElement('canvas')
  Object.assign(canvas.style, {
    position: 'absolute',
    inset: '0',
    width: '100%',
    height: '100%',
    pointerEvents: 'none',
    zIndex: '0',
    mixBlendMode: 'plus-lighter'
  })
  node.appendChild(canvas)
  const ctx = canvas.getContext('2d')
  if (!ctx) {
    canvas.remove()
    return {}
  }

  let width = 0,
    height = 0,
    dpr = window.devicePixelRatio || 1
  function resize() {
    const rect = node.getBoundingClientRect()
    width = rect.width
    height = rect.height
    dpr = window.devicePixelRatio || 1
    canvas.width = Math.round(width * dpr)
    canvas.height = Math.round(height * dpr)
    ctx!.setTransform(1, 0, 0, 1, 0, 0)
    ctx!.scale(dpr, dpr)
  }
  resize()
  const ro = new ResizeObserver(resize)
  ro.observe(node)

  let pulses: Pulse[] = []
  let raf = 0
  let lastAmbient = performance.now()
  let running = false

  function spawnAt(x: number, y: number) {
    // snap to nearest grid intersection
    const gx = Math.round(x / spacing) * spacing
    const gy = Math.round(y / spacing) * spacing
    let baseHue: number
    if (effectivePalette === 'blue') {
      baseHue = 195 + Math.random() * 25 // blue/cyan band
    } else {
      // dark palette: mix green & purple accents
      // Choose band: 0-1 => green (140-155), purple (265-278)
      if (Math.random() < 0.55) {
        baseHue = 140 + Math.random() * 15 // green band
      } else {
        baseHue = 265 + Math.random() * 13 // purple band
      }
    }
    const dirs: Array<[Pulse['dir'], 1 | -1]> = [
      ['h', 1],
      ['h', -1],
      ['v', 1],
      ['v', -1]
    ]
    for (const [dir, sign] of dirs) {
      if (pulses.length >= MAX_PULSES) break
      pulses.push({
        x: gx,
        y: gy,
        dir,
        sign,
        dist: 0,
        hue: baseHue + (dir === 'h' ? 5 : -5) + (Math.random() * 10 - 5)
      })
    }
  }

  function ambientSpawn(now: number) {
    if (now - lastAmbient < AMBIENT_MS) return
    lastAmbient = now
    // Random intersection near center bias
    const biasX = width / 2 + (Math.random() - 0.5) * width * 0.6
    const biasY = height / 2 + (Math.random() - 0.5) * height * 0.6
    spawnAt(biasX, biasY)
  }

  function step(now: number) {
    if (!running) return
    ctx!.clearRect(0, 0, width, height)
    ambientSpawn(now)
    ctx!.lineCap = 'round'
    for (const p of pulses) {
      p.dist += SPEED
      const len = PULSE_LEN
      const start = p.dist - len
      if (p.dir === 'h') {
        const x1 = p.x + start * p.sign
        const x2 = p.x + p.dist * p.sign
        if ((p.sign === 1 && x1 > width + 50) || (p.sign === -1 && x1 < -50)) {
          p.dist = Infinity
          continue
        }
        drawSegment(x1, x2, p.y, p.y, p)
      } else {
        const y1 = p.y + start * p.sign
        const y2 = p.y + p.dist * p.sign
        if ((p.sign === 1 && y1 > height + 50) || (p.sign === -1 && y1 < -50)) {
          p.dist = Infinity
          continue
        }
        drawSegment(p.x, p.x, y1, y2, p)
      }
    }
    pulses = pulses.filter((p) => p.dist !== Infinity)
    raf = requestAnimationFrame(step)
  }

  function start() {
    if (running) return
    running = true
    lastAmbient = performance.now()
    raf = requestAnimationFrame(step)
  }

  function stop() {
    if (!running) return
    running = false
    cancelAnimationFrame(raf)
    raf = 0
    // Clear canvas to avoid lingering glow when scrolled out
    ctx!.clearRect(0, 0, width, height)
  }

  function drawSegment(
    x1: number,
    x2: number,
    y1: number,
    y2: number,
    p: Pulse
  ) {
    const dx = x2 - x1
    const dy = y2 - y1
    const segLen = Math.hypot(dx, dy)
    if (segLen <= 0) return
    const grd = ctx!.createLinearGradient(x1, y1, x2, y2)
    // Gradient: head brighter
    if (effectivePalette === 'blue') {
      grd.addColorStop(0, `hsla(${p.hue}, 95%, 68%, 0)`)
      grd.addColorStop(0.35, `hsla(${p.hue}, 95%, 70%, 0.25)`)
      grd.addColorStop(0.7, `hsla(${p.hue + 8}, 90%, 75%, 0.5)`)
      grd.addColorStop(1, `hsla(${p.hue + 16}, 95%, 80%, 0.9)`)
    } else {
      // dark palette: slightly dimmer tail, brighter head
      grd.addColorStop(0, `hsla(${p.hue}, 85%, 60%, 0)`)
      grd.addColorStop(0.35, `hsla(${p.hue}, 90%, 62%, 0.22)`)
      grd.addColorStop(0.7, `hsla(${p.hue + 10}, 95%, 68%, 0.55)`)
      grd.addColorStop(1, `hsla(${p.hue + 18}, 98%, 75%, 0.95)`)
    }
    ctx!.strokeStyle = grd
    ctx!.lineWidth = THICKNESS
    ctx!.beginPath()
    ctx!.moveTo(x1, y1)
    ctx!.lineTo(x2, y2)
    ctx!.stroke()
  }

  // Throttled pointer spawning: require time + distance thresholds and headroom in pulse budget
  let lastPointerTime = 0
  let lastPX = -9999
  let lastPY = -9999
  function pointer(e: PointerEvent) {
    if (!running) return
    if (pulses.length > MAX_PULSES * 0.9) return // avoid overload
    const now = performance.now()
    if (now - lastPointerTime < POINTER_INTERVAL_MS) return
    const rect = node.getBoundingClientRect()
    const x = e.clientX - rect.left
    const y = e.clientY - rect.top
    const dx = x - lastPX
    const dy = y - lastPY
    if (Math.hypot(dx, dy) < POINTER_MIN_DISTANCE) return
    lastPointerTime = now
    lastPX = x
    lastPY = y
    spawnAt(x, y)
  }
  node.addEventListener('pointermove', pointer)
  node.addEventListener('pointerdown', (e) => {
    if (!running) start() // first interaction triggers start if not yet visible e.g., programmatic scroll
    // pointerdown always spawns (immediate feedback)
    const rect = node.getBoundingClientRect()
    const x = e.clientX - rect.left
    const y = e.clientY - rect.top
    lastPointerTime = 0 // reset to allow immediate spawn
    lastPX = x
    lastPY = y
    spawnAt(x, y)
  })
  // Use IntersectionObserver to lazily start / stop animation based on visibility
  const io = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.target === node) {
          if (entry.isIntersecting && entry.intersectionRatio > 0.05) start()
          else stop()
        }
      }
    },
    { threshold: [0, 0.05, 0.1, 0.25] }
  )
  io.observe(node)

  // In case the element is already in view when mounted
  if (typeof window !== 'undefined') {
    const rect = node.getBoundingClientRect()
    const inView =
      rect.bottom > 0 &&
      rect.right > 0 &&
      rect.top <
        (window.innerHeight || document.documentElement.clientHeight) &&
      rect.left < (window.innerWidth || document.documentElement.clientWidth)
    if (inView) start()
  }

  return {
    destroy() {
      stop()
      node.removeEventListener('pointermove', pointer)
      node.removeEventListener('pointerdown', pointer)
      ro.disconnect()
      io.disconnect()
      canvas.remove()
    }
  }
}
