// Shared SSE helpers (ESM)
export function parseEvent(src) {
  const text = (src?.data ?? '').trim().split(',').join(' ')
  const name = src?.event || 'message'
  return `${name}: ${text}`
}

export function createRowElement({ tagName = 'p', text = '', style } = {}) {
  const element = document.createElement(tagName)
  element.textContent = text
  if (style) element.style = style
  return element
}

export function isNearBottom(container, threshold = 50) {
  return (
    container.scrollHeight - container.scrollTop - container.clientHeight < threshold
  )
}

export function watchEvents(
  config = {
    eventSource: '/events',
    targetElement: 'event-stream',
    onErrorDisconnect: false,
    maxLines: 5000,
    hotReload: false,
    hotReloadEventName: 'hot-reload',
    clientCmdHandler: undefined,
  },
) {
  const eventSource = new EventSource(config.eventSource)
  const container = config.targetElement ? document.getElementById(config.targetElement) : null

  // Maintain a soft cap on displayed rows
  function enforceCap() {
    if (!container) return
    const max = config.maxLines ?? 5000
    while (container.childNodes.length > max) container.removeChild(container.firstChild)
  }

  function insertChildAndScroll(elem) {
    if (!container) return
    const atBottom = isNearBottom(container)
    container.appendChild(elem)
    enforceCap()
    if (atBottom) elem.scrollIntoView({ behavior: 'smooth' })
  }

  eventSource.onopen = (event) => {
    const elem = createRowElement({ text: 'connected!', style: 'color: green' })
    insertChildAndScroll(elem)
  }

  eventSource.addEventListener('base64', (event) => {
    try {
      const decoded = atob(event.data)
      const elem = createRowElement({ text: decoded })
      insertChildAndScroll(elem)
    } catch (e) {
      const elem = createRowElement({ text: '[base64 decode error]' })
      insertChildAndScroll(elem)
    }
  })

  eventSource.onmessage = (event) => {
    const data = parseEvent(event)
    const elem = createRowElement({ text: data })
    insertChildAndScroll(elem)
  }

  // Hot reload via custom SSE event
  if (config.hotReload) {
    eventSource.addEventListener(config.hotReloadEventName || 'hot-reload', () => {
      try {
        // Debounce reloads in case of burst events
        if (watchEvents.__reloading) return
        watchEvents.__reloading = true
        setTimeout(() => (watchEvents.__reloading = false), 1000)
        // Try to update stylesheets without full reload first
        const links = Array.from(document.querySelectorAll('link[rel="stylesheet"]'))
        if (links.length) {
          links.forEach((link) => {
            const href = new URL(link.href, location.href)
            href.searchParams.set('hr', String(Date.now()))
            link.href = href.toString()
          })
        }
        // Full reload as fallback
        location.reload()
      } catch {
        location.reload()
      }
    })
  }

  // Client command channel: event: client-cmd, data: string
  eventSource.addEventListener('client-cmd', (evt) => {
    const payload = (evt && evt.data) ? String(evt.data) : ''
    if (typeof config.clientCmdHandler === 'function') {
      try { config.clientCmdHandler(payload) } catch { /* noop */ }
      return
    }
    // default behavior: log to stream if available
    insertChildAndScroll(createRowElement({ text: `[client] ${payload}`, style: 'color:#7efc7a' }))
  })

  eventSource.onerror = () => {
    if (config.onErrorDisconnect) eventSource.close()
    const warn = createRowElement({ text: 'error: disconnected', style: 'color: red' })
    insertChildAndScroll(warn)
  }

  // return an unsubscribe function
  return () => eventSource.close()
}
