// Shared SSE helpers (ESM)
import { AsciiTitle3D } from '../components/ascii.js'
export function parseEvent(src) {
  const text = (src?.data ?? '').trim().split(',').join(' ')
  const name = src?.event || 'message'
  return `${name}: ${text}`
}

export function createRowElement({ tagName = 'p', text = '', style, html, className } = {}) {
  const element = document.createElement(tagName)
  if (className) element.className = className
  if (html != null) element.innerHTML = html
  else element.textContent = text
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

  // Load client-side hooks from localStorage (array of {id,name,code,enabled})
  function getEnabledHooks() {
    try {
      const hooks = JSON.parse(localStorage.getItem('cd_hooks') || '[]')
      // Seed a default prompt prehook if none exist
      if (!hooks || hooks.length === 0 || !hooks.some(h => h && /prompt/i.test(h.name))) {
        hooks.push({
          id: 'builtin_prompt',
          name: 'Prompt',
          enabled: true,
          code: `function transform(ctx, text){
  var ts = ctx && ctx.ts ? ctx.ts : new Date();
  var time = (ts instanceof Date ? ts : new Date(ts)).toLocaleTimeString();
  var sender = (ctx && ctx.sender) ? String(ctx.sender) : 'system';
  function esc(s){return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;')}
  return { html: '<span style="color:#8a8f98">'+time+'</span> <span style="color:#7efc7a">@'+sender+'</span> <span style="color:#9aa0a6">&raquo;&raquo;</span> <span>'+esc(text)+'</span>' };
}`
        })
        try { localStorage.setItem('cd_hooks', JSON.stringify(hooks)) } catch { }
      }
      return hooks.filter(h => h && h.enabled && typeof h.code === 'string')
    } catch { return [] }
  }

  function applyHooks(text, ctx) {
    const hooks = getEnabledHooks()
    let output = text
    for (const h of hooks) {
      try {
        // eslint-disable-next-line no-new-func
        const fn = new Function('ctx', 'text', `${h.code}; return (typeof transform==='function') ? transform(ctx, text) : text;`)
        const context = Object.assign({ name: h.name }, ctx || {})
        output = fn(context, output)
      } catch (e) {
        console.warn('[hook] error in hook', h.name, e)
      }
    }
    return output
  }

  // formatting helpers
  const escapeBasic = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  function formatPrompt(sender, content, ts = new Date()) {
    const time = ts.toLocaleTimeString()
    const senderLabel = String(sender || 'system')
    const safe = escapeBasic(content)
    return `<span style="color:#8a8f98">${time}</span> <span style="color:#7efc7a">@${senderLabel}</span> <span style="color:#9aa0a6">&raquo;&raquo;</span> <span>${safe}</span>`
  }
  function finalizeRenderText(raw, sender, ts) {
    if (raw && typeof raw === 'object' && typeof raw.html === 'string') return raw.html
    return formatPrompt(sender, String(raw), ts)
  }

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
    const href = (typeof location !== 'undefined') ? location.href : ''
    // ASCII banner as plain text line (no pre background)
    insertChildAndScroll(createRowElement({ tagName: 'p', text: AsciiTitle3D, className: 'ascii', style: 'white-space: pre; margin:0;' }))
    // Welcome lines
    const now = Date.now()
    const lines = [
      ':: ~~~ Welcome to ConsoleDump ~~~ ::',
      `:: Session ${href} ::`,
      ':: Pipe your data here from anywhere to debug in realtime.',
    ]
    lines.forEach((line) => insertChildAndScroll(createRowElement({ html: finalizeRenderText(applyHooks(line, { sender: 'system', ts: now }), 'system') })))
    insertChildAndScroll(createRowElement({ html: finalizeRenderText(applyHooks(`connected to ${href}`, { sender: 'system', ts: now }), 'system') }))
    insertChildAndScroll(createRowElement({ html: finalizeRenderText(applyHooks('type @help below to see commands...', { sender: 'system', ts: now + 1000 }), 'system') }))
    insertChildAndScroll(createRowElement({ html: finalizeRenderText(applyHooks('happy debugging!', { sender: 'system', ts: now + 2000 }), 'system') }))
  }

  eventSource.addEventListener('base64', (event) => {
    try {
      const decoded = atob(event.data)
      const result = applyHooks(decoded, { sender: 'server' })
      const elem = createRowElement({ html: finalizeRenderText(result, 'server') })
      insertChildAndScroll(elem)
    } catch (e) {
      const elem = createRowElement({ html: finalizeRenderText('[base64 decode error]', 'system') })
      insertChildAndScroll(elem)
    }
  })

  eventSource.onmessage = (event) => {
    const data = parseEvent(event)
    const result = applyHooks(data, { sender: 'server' })
    const elem = createRowElement({ html: finalizeRenderText(result, 'server') })
    insertChildAndScroll(elem)
  }

  // Hot reload via custom SSE event
  if (config.hotReload) {
    eventSource.addEventListener(config.hotReloadEventName || 'hot-reload', () => {
      try {
        // Debounce reloads in case of burst events
        const now = Date.now()
        const last = Number(localStorage.getItem('cd_hr_ts') || '0')
        if (now - last < 1500) return
        localStorage.setItem('cd_hr_ts', String(now))
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
    insertChildAndScroll(createRowElement({ html: finalizeRenderText(applyHooks(payload, { sender: 'client' }), 'client') }))
  })

  eventSource.onerror = () => {
    if (config.onErrorDisconnect) eventSource.close()
    const warn = createRowElement({ text: 'error: disconnected', style: 'color: red' })
    insertChildAndScroll(warn)
  }

  // return an unsubscribe function
  return () => eventSource.close()
}
