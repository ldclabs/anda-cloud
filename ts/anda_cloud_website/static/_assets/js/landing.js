document.addEventListener('DOMContentLoaded', function () {
  // 导航栏滚动效果
  const header = document.querySelector('.header')
  const heroSection = document.querySelector('.hero')

  window.addEventListener('scroll', function () {
    if (window.scrollY > 50) {
      header.style.padding = '10px 0'
      header.style.backdropFilter = 'blur(10px)'
      header.style.background = 'rgba(5, 10, 24, 0.8)'
    } else {
      header.style.padding = '20px 0'
      header.style.backdropFilter = ''
      header.style.background = ''
    }
  })

  // Anda 诗歌动画
  andaPoetry()

  // 创建神经网络节点动画
  // createNeuralNetwork()

  // 添加滚动动画
  const sections = document.querySelectorAll('section')
  const cards = document.querySelectorAll('.comparison-card, .stack-card, .audience-card')

  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        entry.target.classList.add('fade-in')
      }
    })
  }, { threshold: 0.1 })

  sections.forEach(section => {
    observer.observe(section)
  })

  cards.forEach(card => {
    observer.observe(card)
  })

  // 平滑滚动
  document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
      e.preventDefault()

      const targetId = this.getAttribute('href')
      const targetElement = document.querySelector(targetId)

      if (targetElement) {
        window.scrollTo({
          top: targetElement.offsetTop - 80,
          behavior: 'smooth'
        })
      }
    })
  })
})

// 添加打字机效果
class TypeWriter {
  constructor(element, text, prefixIdx, speed = 100) {
    this.element = element
    this.text = text
    this.speed = speed
    this.index = 0
    this.prefixIdx = prefixIdx
    this.isDeleting = false
    this.type()
  }

  type() {
    const currentText = this.isDeleting
      ? this.text.substring(0, this.index - 1)
      : this.text.substring(0, this.index + 1)

    this.element.innerHTML = currentText

    // 调整速度
    let typeSpeed = this.speed

    if (this.isDeleting) {
      typeSpeed /= 2
    }

    // 增加或减少索引
    if (!this.isDeleting) {
      this.index++
      if (this.index === this.text.length) {
        // 完成打字，等待一段时间后开始删除
        typeSpeed = 1000
        this.isDeleting = true
      }
    } else {
      this.index--
      if (this.index <= this.prefixIdx) {
        // 完成删除，等待一段时间后重新开始
        this.isDeleting = false
        typeSpeed = 500
      }
    }

    setTimeout(() => this.type(), typeSpeed)
  }
}

function andaPoetry() {
  const canvas = document.getElementById('anda-poetry')
  const ctx = canvas.getContext('2d')
  let width = canvas.width = window.innerWidth
  let height = canvas.height = window.innerHeight

  // 思维节点系统
  class KnowledgeNode {
    constructor() {
      this.pos = { x: Math.random() * width, y: Math.random() * height }
      this.vel = { x: (Math.random() - 0.5) * 0.4, y: (Math.random() - 0.5) * 0.4 }
      this.links = []
      this.history = []
    }

    update() {
      // 量子涨落
      this.vel.x += (Math.random() - 0.5) * 0.001
      this.vel.y += (Math.random() - 0.5) * 0.001

      this.pos.x += this.vel.x
      this.pos.y += this.vel.y

      this.history.push({ ...this.pos })
      if (this.history.length > 20) this.history.shift()

      return this.pos.x >= 0 && this.pos.x <= width &&
        this.pos.y >= 0 && this.pos.y <= height
    }

    draw() {
      // 思维轨迹
      ctx.beginPath()
      this.history.forEach((p, i) => {
        ctx.globalAlpha = i / 10
        ctx.strokeStyle = `hsl(${i * 8}, 70%, 60%)`
        ctx.lineWidth = 1
        if (i > 0) {
          ctx.moveTo(this.history[i - 1].x, this.history[i - 1].y)
          ctx.lineTo(p.x, p.y)
        }
      })
      ctx.stroke()

      // 核心光点
      const grad = ctx.createRadialGradient(
        this.pos.x, this.pos.y, 1,
        this.pos.x, this.pos.y, 10
      )
      grad.addColorStop(0, '#7af8ff')
      grad.addColorStop(1, 'transparent')
      ctx.fillStyle = grad
      ctx.beginPath()
      ctx.arc(this.pos.x, this.pos.y, 8, 0, Math.PI * 2)
      ctx.fill()
    }
  }

  function getNodesNumber() {
    return Math.floor((width * height) / 30000)
  }

  const nodes = Array.from({ length: getNodesNumber() }, () => new KnowledgeNode())

  // 主循环
  function animate() {
    ctx.fillStyle = 'rgba(22, 22, 66, 1)'
    ctx.fillRect(0, 0, canvas.width, canvas.height)

    // 移除超出边界的节点并生成新节点
    for (let i = nodes.length - 1; i >= 0; i--) {
      if (!nodes[i].update()) {
        nodes.splice(i, 1)
        nodes.push(new KnowledgeNode())
      }
    }

    nodes.forEach(node => {
      node.update()
      node.draw()

      // 建立知识连接
      nodes.forEach(other => {
        if (node !== other) {
          const dx = other.pos.x - node.pos.x
          const dy = other.pos.y - node.pos.y
          const dist = Math.sqrt(dx * dx + dy * dy)

          if (dist < 150) {
            ctx.beginPath()
            ctx.strokeStyle = `rgba(122, 248, 255, ${0.3 - dist / 500})`
            ctx.lineWidth = 2
            ctx.moveTo(node.pos.x, node.pos.y)
            ctx.lineTo(other.pos.x, other.pos.y)
            ctx.stroke()
          }
        }
      })
    })

    // 生成思维代码雨
    if (Math.random() < 0.01) {
      const rainCount = getRandomInt(3, 7)
      for (let i = 0; i < rainCount; i++) {
        ctx.fillStyle = 'rgba(100, 220, 255, 0.8)'
        ctx.font = '14px monospace'
        ctx.fillText(
          getRandomInt(4228250625, Number.MAX_SAFE_INTEGER).toString(16).slice(0, 8).toUpperCase(),
          Math.random() * width,
          Math.random() * height
        )
      }
    }

    requestAnimationFrame(animate)
  }

  // 响应式重置
  window.addEventListener('resize', () => {
    width = canvas.width = window.innerWidth
    height = canvas.height = window.innerHeight
  })

  // 添加意识涟漪
  window.addEventListener('mousedown', e => {
    nodes.forEach(node => {
      const dx = e.clientX - node.pos.x
      const dy = e.clientY - node.pos.y
      const dist = Math.sqrt(dx * dx + dy * dy)

      if (dist < 200) {
        node.vel.x += dx / dist * 0.3
        node.vel.y += dy / dist * 0.3
      }
    })
  })

  // 启动思维宇宙
  animate()

  function getRandomInt(min, max) {
    const minCeiled = Math.ceil(min)
    const maxFloored = Math.floor(max)
    return Math.floor(Math.random() * (maxFloored - minCeiled) + minCeiled)
  }
}

// 初始化打字机效果
window.addEventListener('load', function () {
  const tagline = document.querySelector('.footer-tagline p:first-child')
  if (tagline) {
    new TypeWriter(tagline, 'Anda - Where AI agents grow roots.', 4, 150)
  }
})