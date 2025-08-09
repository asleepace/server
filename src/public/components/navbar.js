import { $ } from './index.js'

$.define('cd-nav', function ({ state, onMounted }) {
    onMounted(() => {
        const themeBtn = this.shadowRoot.querySelector('#theme')
        const sidebarBtn = this.shadowRoot.querySelector('#sidebar')
        const homeBtn = this.shadowRoot.querySelector('#home')
        const newBtn = this.shadowRoot.querySelector('#new')
        const cursor = this.shadowRoot.querySelector('#cursor')

        if (themeBtn) {
            themeBtn.setAttribute('aria-label', 'Toggle theme')
            themeBtn.onclick = () => {
                const root = document.documentElement
                const dark = root.getAttribute('data-theme') === 'dark'
                const newTheme = dark ? 'light' : 'dark'
                root.setAttribute('data-theme', newTheme)
                $.store.set('theme', newTheme)
            }
        }

        if (sidebarBtn) {
            sidebarBtn.setAttribute('aria-label', 'Toggle sidebar')
            sidebarBtn.onclick = () => this.dispatchEvent($.event('cd:toggle-sidebar'))
        }

        if (homeBtn) {
            homeBtn.onclick = () => { window.location.href = '/' }
        }

        if (newBtn) {
            newBtn.onclick = () => { window.location.href = '/session/new' }
        }

        if (cursor) {
            cursor.setAttribute('aria-hidden', 'true')
        }
    })

    return `
        <style>
            :host { display:flex; width:100%; height:var(--nav-height); position:sticky; top:0; z-index:10; }
            nav { width:100%; border-bottom:1px solid var(--border); background: var(--surface); height:var(--nav-height); display:flex; align-items:center; justify-content:space-between; padding:0 12px; box-shadow:0 2px 6px var(--shadow); box-sizing:border-box; }
            .nav-title { font-weight:800; letter-spacing:0.2px; color:var(--text-color); display:flex; align-items:center; gap:4px; font-family: var(--font-headings); }
            .nav-bttns { display:flex; align-items:center; gap: 0; }
            button { border:none; border-right:1px solid var(--border); background:transparent; padding:6px 10px; color: var(--green-accent); font-family: var(--font-mono); cursor:pointer; }
            button:hover { background: rgba(126, 252, 122, 0.1); }
            button:last-child { border-right:none; }
            #cursor { color: var(--green-accent); opacity:1; animation: blink 1s steps(1) infinite; }
            @keyframes blink { 50% { opacity: 0; } }
        </style>
        <nav>
            <div class="nav-title">
                <div>ConsoleDump</div>
                <div id="cursor">&#10074;</div>
            </div>
            <div class="nav-bttns">
                <button id="home">Home</button>
                <button id="new">New Session</button>
                <button id="settings">Settings</button>
                <button id="theme">Theme</button>
                <button id="sidebar">Sidebar</button>
            </div>
        </nav>
    `
})