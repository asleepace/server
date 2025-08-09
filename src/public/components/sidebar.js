import { $ } from './index.js'

export default function Sidebar({ state, onMounted, onMethod }) {
  const initialTheme = (state['data-theme'] || state.theme || 'dark').toLowerCase()
  const initialWidth = state.width || '420px'
  const minAttr = state.min || '280'
  const maxAttr = state.max || '65vw'

  const html = `
    <style>
      :host {
        display: flex;
        flex-direction: column;
        position: relative;
        overflow: hidden;
        box-sizing: border-box;
        width: ${initialWidth};
        min-width: ${/v?w$/.test(minAttr) ? minAttr : `${minAttr}px`};
        max-width: ${maxAttr};
        background: var(--surface);
        color: var(--text-color);
        border-left: 1px solid var(--border);
        z-index: 100;
        contain: content;
        flex: 0 0 auto; /* fixed-width in flex row */
        align-self: stretch;
      }
      :host([hidden]) { display: none; }

      /* Theme variants (host attribute) */
      :host([data-theme="dark"]) { background: #15171a; color: var(--text-color); }
      :host([data-theme="light"]) { background: var(--surface); }

      .handle {
        position: absolute;
        left: 0;
        top: 0;
        bottom: 0;
        width: 10px;
        cursor: col-resize;
        background: transparent;
        opacity: 1;
        z-index: 10;
        pointer-events: auto;
        touch-action: none;
      }

      :host(:hover) .handle { background: transparent; }

      .content { display:block; padding: 18px 32px 22px 32px; overflow:auto; max-height: 100%; }
      .content * { box-sizing: border-box; max-width: 100%; }
      .content > * + * { margin-top: 14px; }
      .content h2 { margin-bottom: 12px; }
      .content h3, .content h4 { margin-top: 16px; margin-bottom: 8px; }
      .content p + .segmented { margin-top: 12px; }

      /* Local typography for sidebar content */
      .content h2 {
        font-family: var(--font-headings);
        font-size: 1.3rem;
        margin: 0 0 10px 0;
        color: var(--text-color);
        letter-spacing: 0.2px;
      }
      .content h3, .content h4 {
        font-family: var(--font-ui);
        font-weight: 600;
        font-size: 0.9rem;
        color: var(--text-color);
        margin: 12px 0 8px 0;
      }
      .content p {
        font-size: 0.85rem;
        color: var(--muted-text);
        line-height: 1.5;
        margin: 6px 0;
      }
      .content small { font-size: 0.78rem; color: var(--muted-text); }
      .content code, .content pre { color: var(--code-text-color); }
    </style>
    <div class="handle" data-onpointerdown="@startResize" data-onmousedown="@startResize" data-ontouchstart="@startResize" data-ondblclick="@toggleTheme"></div>
    <div class="content"><slot></slot></div>
  `

  function applyWidth(pxOrCss) {
    const host = /** @type {HTMLElement} */ (this)
    host.style.width = pxOrCss
    const layout = host.closest('#layout')
    if (layout) layout.style.setProperty('--sidebar', pxOrCss)
    document.documentElement.style.setProperty('--sidebar', pxOrCss)
  }

  function clampWidth(nextPx, minPx, maxPx) {
    return Math.min(Math.max(nextPx, minPx), maxPx)
  }

  onMounted((host) => {
    // Guard against re-entry after initial mount
    if (host.__cdInitDone) return
    host.__cdInitDone = true

    // Restore width + theme
    const savedW = $.store.get('sidebar_width')
    const w = savedW || initialWidth
    applyWidth.call(host, w)
    const savedTheme = $.store.get('sidebar_theme') || initialTheme
    if (host.getAttribute('data-theme') !== savedTheme) {
      host.setAttribute('data-theme', savedTheme)
    }

    // Respect open state (from page logic)
    const open = $.store.get('sidebar_open')
    if (open === '0') host.style.display = 'none'
  })

  onMethod('startResize', function (e) {
    try {
      if (e.cancelable) e.preventDefault()
      e.stopPropagation()
      const host = /** @type {HTMLElement} */ (this)
      const type = e.type || ''
      const isPointer = type.startsWith('pointer')
      const isTouch = type.startsWith('touch')
      const isMouse = type === 'mousedown'

      if (isPointer && typeof e.pointerId === 'number' && e.target && typeof e.target.setPointerCapture === 'function') {
        try { e.target.setPointerCapture(e.pointerId) } catch { }
      }

      const getX = (ev) => (ev.touches && ev.touches[0] ? ev.touches[0].clientX : ev.clientX)
      const startX = getX(e)
      const startWidth = parseInt(getComputedStyle(host).width, 10)
      const minPx = /v?w$/.test(minAttr) ? 280 : parseInt(minAttr, 10)
      const maxPx = /v?w$/.test(maxAttr) ? Math.round(window.innerWidth * 0.65) : parseInt(maxAttr, 10)

      const cleanups = []
      const add = (target, evName, handler, opts) => { target.addEventListener(evName, handler, opts); cleanups.push(() => target.removeEventListener(evName, handler, opts)) }

      const onMove = (ev) => {
        if (ev.cancelable) ev.preventDefault()
        const dx = startX - getX(ev)
        const next = clampWidth(startWidth + dx, minPx, maxPx)
        applyWidth.call(host, `${next}px`)
      }
      const onUp = () => {
        cleanups.forEach(fn => fn())
        $.store.set('sidebar_width', getComputedStyle(host).width)
        document.body.style.userSelect = ''
        document.body.style.cursor = ''
      }

      if (isPointer) {
        add(window, 'pointermove', onMove)
        add(window, 'pointerup', onUp)
        add(window, 'pointercancel', onUp)
      } else if (isTouch) {
        add(window, 'touchmove', onMove, { passive: false })
        add(window, 'touchend', onUp)
        add(window, 'touchcancel', onUp)
      } else if (isMouse) {
        add(window, 'mousemove', onMove)
        add(window, 'mouseup', onUp)
      }

      document.body.style.userSelect = 'none'
      document.body.style.cursor = 'col-resize'
    } catch (err) {
      console.warn('[cd-sidebar] resize error:', err)
    }
  })

  onMethod('toggleTheme', function () {
    const cur = this.getAttribute('data-theme') || 'light'
    const nx = cur === 'light' ? 'dark' : 'light'
    this.setAttribute('data-theme', nx)
    $.store.set('sidebar_theme', nx)
  })

  return html
}


