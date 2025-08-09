// ESM entry point for session page
import './components/index.js'
import './components/navbar.js'
import './components/snippet.js'
import './components/tabs.js'
import { bootstrapSessionPage } from './scripts/session.js'

// Basic diagnostics toggle: mute console.log in production unless overridden
(() => {
    try {
        const isLocalhost = /^(localhost|127\.0\.0\.1|\[::1\])$/.test(location.hostname)
        const override = localStorage.getItem('diagnostics')
        const diagnosticsOn = override === '1' || (override === null && isLocalhost)
        document.documentElement.setAttribute('data-env', diagnosticsOn ? 'dev' : 'prod')
        if (!diagnosticsOn) {
            // Keep warn/error; silence log/info/debug
            console.log = () => { }
            console.info = () => { }
            console.debug = () => { }
        }
    } catch { /* noop */ }
})()

document.addEventListener('DOMContentLoaded', () => {
    bootstrapSessionPage()
})