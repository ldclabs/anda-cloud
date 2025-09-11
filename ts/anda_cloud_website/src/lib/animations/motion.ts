export interface FadeInOptions {
  delay?: number
  duration?: number
  y?: number
  threshold?: number
  once?: boolean
}
export function fadeIn(node: HTMLElement, opts: FadeInOptions = {}) {
  const {
    delay = 0,
    duration = 520,
    y = 24,
    threshold = 0.25,
    once = true
  } = opts
  let hasShown = false
  node.style.opacity = '0'
  node.style.transform = `translate3d(0, ${y}px, 0)`
  node.style.transition = `opacity ${duration}ms cubic-bezier(.4,.15,.2,1) ${delay}ms, transform ${duration}ms cubic-bezier(.4,.15,.2,1) ${delay}ms`
  const obs = new IntersectionObserver(
    (entries) => {
      entries.forEach((e) => {
        if (e.isIntersecting) {
          if (once && hasShown) return
          hasShown = true
          requestAnimationFrame(() => {
            node.style.opacity = '1'
            node.style.transform = 'translate3d(0,0,0)'
          })
          if (once) obs.unobserve(node)
        } else if (!once) {
          node.style.opacity = '0'
          node.style.transform = `translate3d(0, ${y}px, 0)`
        }
      })
    },
    { threshold }
  )
  obs.observe(node)
  return {
    destroy() {
      obs.disconnect()
    }
  }
}

interface ParallaxOptions {
  speed?: number // parallax speed factor
  clamp?: boolean // avoid applying while远离视口（防止初始跳动）
  initialDelayFrames?: number // 延迟若干 rAF 以等待布局/字体稳定
  maxOffset?: number // 绝对值最大位移（可选）
  startWithinPx?: number // 当元素顶部距离视口底部 <= 该值时即开始 (允许下一屏预先参与)
  disableBelow?: number // 视口宽度低于该值时禁用（移动端避免重叠）
}
export function parallax(node: HTMLElement, opts: ParallaxOptions = {}) {
  const {
    speed = 0.12,
    clamp = true,
    initialDelayFrames = 2,
    maxOffset,
    startWithinPx = 200,
    disableBelow = 768
  } = opts

  let frame: number | null = null
  let baseY = 0
  let delayLeft = Math.max(0, initialDelayFrames)
  let firstApply = true
  let disabledByWidth = false

  function computeBase() {
    baseY = node.getBoundingClientRect().top + window.scrollY
  }
  computeBase()

  function apply() {
    frame = null
    if (disabledByWidth) return
    if (delayLeft > 0) {
      delayLeft--
      computeBase() // 期间再刷新一次基准
      frame = requestAnimationFrame(apply)
      return
    }
    const viewportH =
      window.innerHeight || document.documentElement.clientHeight
    const scrollY = window.scrollY
    // 根据当前滚动计算偏移 (允许下屏预先出现负值以形成提前视差)
    let offset = (scrollY - baseY) * speed
    // clamp：若距离大于阈值则不计算，减少过远元素开销
    if (clamp) {
      const distanceBelowFold = baseY - (scrollY + viewportH)
      if (distanceBelowFold > startWithinPx) {
        // 仍未到视区，下次再试
        firstApply = false
        return
      }
    }
    if (typeof maxOffset === 'number') {
      if (offset > maxOffset) offset = maxOffset
      else if (offset < -maxOffset) offset = -maxOffset
    }
    node.style.transform = `translate3d(0, ${offset.toFixed(2)}px, 0)`
    firstApply = false
  }

  function schedule() {
    if (frame == null) frame = requestAnimationFrame(apply)
  }

  function onScroll() {
    schedule()
  }
  function onResize() {
    computeBase()
    // 判断是否需要禁用
    if (disableBelow && window.innerWidth < disableBelow) {
      if (!disabledByWidth) {
        disabledByWidth = true
        // 清除 transform 以还原正常文档流
        node.style.transform = ''
      }
      return
    } else if (
      disabledByWidth &&
      (!disableBelow || window.innerWidth >= disableBelow)
    ) {
      // 恢复
      disabledByWidth = false
      firstApply = true
    }
    schedule()
  }
  window.addEventListener('scroll', onScroll, { passive: true })
  window.addEventListener('resize', onResize)

  // 初始宽度判定
  if (disableBelow && window.innerWidth < disableBelow) {
    disabledByWidth = true
    node.style.transform = ''
  }

  // 初始两帧稳定
  schedule()

  return {
    destroy() {
      if (frame) cancelAnimationFrame(frame)
      window.removeEventListener('scroll', onScroll)
      window.removeEventListener('resize', onResize)
    }
  }
}

export function staggerChildren(node: HTMLElement, gap = 90) {
  const children = Array.from(node.children) as HTMLElement[]
  children.forEach((c, i) =>
    c.style.setProperty('--stagger-index', i.toString())
  )
  const obs = new IntersectionObserver(
    (entries) => {
      entries.forEach((e) => {
        if (e.isIntersecting) {
          children.forEach((c, i) => {
            c.style.animation = `fadeRise .6s cubic-bezier(.4,.15,.2,1) ${i * gap}ms forwards`
          })
          obs.disconnect()
        }
      })
    },
    { threshold: 0.25 }
  )
  obs.observe(node)
  return {
    destroy() {
      obs.disconnect()
    }
  }
}
