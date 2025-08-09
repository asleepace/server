import { watchEvents } from './events.js'

export function bootstrapSessionPage() {
    const path = window.location.pathname
    const match = path.match(/\/s\/(.+)$/)
    const sessionKey = match ? match[1] : ''

    // Update session title and code snippets
    const sessionTitle = document.getElementById('session-title')
    if (sessionTitle && sessionKey) {
        sessionTitle.innerHTML = `Stream ID: #<a style="color: #ff79c6;" href="/s/${sessionKey}">${sessionKey}</a>`
    }

    const base = `${location.protocol}//${location.host}`
    const curl = `curl -X POST ${base}/s/${sessionKey} \\\n  -H 'Content-Type: text/plain' \\\n  --data-binary 'hello world'`
    const js = `await fetch('${base}/s/${sessionKey}', {\n  method: 'POST',\n  headers: { 'Content-Type': 'text/plain' },\n  body: 'hello world'\n})`
    const ts = `await fetch('${base}/s/${sessionKey}', {\n  method: 'POST',\n  headers: { 'Content-Type': 'application/json' },\n  body: JSON.stringify({ msg: 'hello' })\n})`
    const node = `import axios from 'axios'\nawait axios.post('${base}/s/${sessionKey}', 'hello world', { headers: { 'Content-Type': 'text/plain' } })`
    const py = `import requests\nrequests.post('${base}/s/${sessionKey}', data='hello world', headers={'Content-Type':'text/plain'})`

    const setText = (id, text) => { const el = document.getElementById(id); if (el) el.textContent = text }
    setText('curl-snippet', curl)
    setText('js-snippet', js)
    setText('ts-snippet', ts)
    setText('node-snippet', node)
    setText('py-snippet', py)

    // Sidebar handlers (moved from inline scripts)
    document.addEventListener('cd:toggle-sidebar', () => {
        const main = document.getElementById('layout')
        const aside = document.getElementById('sidebar')
        const open = aside && aside.style.display !== 'none'
        if (!aside || !main) return
        if (open) {
            aside.style.display = 'none'
            main.style.setProperty('--sidebar', '0px')
            document.documentElement.style.setProperty('--sidebar', '0px')
            try { localStorage.setItem('sidebar_open', '0') } catch { }
        } else {
            aside.style.display = 'block'
            const w = localStorage.getItem('sidebar_width') || (aside.style.width || '420px')
            aside.style.width = w
            main.style.setProperty('--sidebar', w)
            document.documentElement.style.setProperty('--sidebar', w)
            try { localStorage.setItem('sidebar_open', '1') } catch { }
        }
    })

        ; (function restoreSidebar() {
            const main = document.getElementById('layout')
            const aside = document.getElementById('sidebar')
            if (!aside || !main) return
            try {
                const w = localStorage.getItem('sidebar_width')
                const open = localStorage.getItem('sidebar_open')
                const theme = localStorage.getItem('sidebar_theme')
                if (w) { aside.style.width = w; main.style.setProperty('--sidebar', w); document.documentElement.style.setProperty('--sidebar', w) }
                if (open === '0') { aside.style.display = 'none'; main.style.setProperty('--sidebar', '0px'); document.documentElement.style.setProperty('--sidebar', '0px') }
                if (theme) { aside.setAttribute('data-theme', theme) }
            } catch { }
        })()

        ; (function persistWidth() {
            const main = document.getElementById('layout')
            const aside = document.getElementById('sidebar')
            if (!aside || !main || !('ResizeObserver' in window)) return
            let raf = 0
            const observer = new ResizeObserver(() => {
                cancelAnimationFrame(raf)
                raf = requestAnimationFrame(() => {
                    const w = getComputedStyle(aside).width
                    main.style.setProperty('--sidebar', w)
                    document.documentElement.style.setProperty('--sidebar', w)
                    try { localStorage.setItem('sidebar_width', w) } catch { }
                })
            })
            observer.observe(aside)
        })()

        ; (function sidebarTheme() {
            const aside = document.getElementById('sidebar')
            const drag = document.getElementById('drag-hint')
            if (!aside || !drag) return
            function toggle() {
                const cur = aside.getAttribute('data-theme') || 'light'
                const nx = cur === 'light' ? 'dark' : 'light'
                aside.setAttribute('data-theme', nx)
                try { localStorage.setItem('sidebar_theme', nx) } catch { }
            }
            drag.addEventListener('dblclick', toggle)

            // Left-edge drag to resize sidebar
            let resizing = false
            let startX = 0
            let startWidth = 0
            const minWidth = 280
            const maxWidth = Math.round(window.innerWidth * 0.65)

            const onMouseMove = (e) => {
                if (!resizing) return
                // For a right sidebar with a left-edge handle, moving LEFT increases width
                const dx = startX - e.clientX
                const next = Math.min(Math.max(startWidth + dx, minWidth), maxWidth)
                aside.style.width = `${next}px`
                const main = document.getElementById('layout')
                if (main) main.style.setProperty('--sidebar', `${next}px`)
                document.documentElement.style.setProperty('--sidebar', `${next}px`)
            }
            const onMouseUp = () => {
                if (!resizing) return
                resizing = false
                try { localStorage.setItem('sidebar_width', getComputedStyle(aside).width) } catch { }
                window.removeEventListener('mousemove', onMouseMove)
                window.removeEventListener('mouseup', onMouseUp)
                document.body.style.userSelect = ''
                document.body.style.cursor = ''
            }
            drag.addEventListener('mousedown', (e) => {
                // Only start drag if near the left edge of the screen (already positioned at left:0)
                resizing = true
                startX = e.clientX
                startWidth = parseInt(getComputedStyle(aside).width, 10) || minWidth
                window.addEventListener('mousemove', onMouseMove)
                window.addEventListener('mouseup', onMouseUp)
                document.body.style.userSelect = 'none'
                document.body.style.cursor = 'col-resize'
                e.preventDefault()
            })
        })()

    // Start watching events
    const eventSourceUrl = sessionKey ? `/events?s=${sessionKey}` : '/events'
    watchEvents({
        targetElement: 'event-stream',
        eventSource: eventSourceUrl,
        onErrorDisconnect: false,
        hotReload: true,
        hotReloadEventName: 'hot-reload',
        clientCmdHandler: (payload) => {
            // Default: append to stream; handled in events.js already
            // Optionally parse messages beginning with ':' and show hints
            if (payload.startsWith('info:')) {
                const el = document.createElement('p')
                el.style.color = 'var(--green-accent)'
                el.textContent = payload.slice(5).trim()
                const container = document.getElementById('event-stream')
                if (container) container.appendChild(el)
            }
        }
    })

    // CLI DSL
    const input = document.getElementById('cli-input')
    if (input) {
        input.addEventListener('keydown', async (e) => {
            if (e.key !== 'Enter') return
            const value = input.value.trim()
            input.value = ''
            if (!value) return

            const lc = value.toLowerCase()
            if (lc === ':clear') {
                const stream = document.getElementById('event-stream')
                if (stream) stream.innerHTML = ''
                return
            }
            if (lc.startsWith(':theme')) {
                const arg = lc.split(/\s+/)[1]
                const root = document.documentElement
                const newTheme = arg === 'light' ? 'light' : 'dark'
                root.setAttribute('data-theme', newTheme)
                try { localStorage.setItem('theme', newTheme) } catch { }
                return
            }
            if (lc === ':toggle') {
                document.dispatchEvent(new CustomEvent('cd:toggle-sidebar', { bubbles: true, composed: true }))
                return
            }
            if (lc === ':reload') {
                try { await fetch('/__reload', { method: 'POST' }) } catch { }
                return
            }
            // Default: send to /__client
            try { await fetch('/__client', { method: 'POST', headers: { 'Content-Type': 'text/plain' }, body: value }) } catch { }
        })
    }
}