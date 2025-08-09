import { $, AsciiSymbols } from './index.js'

export default function CdNav({ state, onMounted }) {
    onMounted(() => {
        const newBtn = this.shadowRoot.querySelector('#new')
        const settingsBtn = this.shadowRoot.querySelector('#settings')
        const editorBtn = this.shadowRoot.querySelector('#editor')
        const sidebarBtn = this.shadowRoot.querySelector('#sidebar')

        if (newBtn) newBtn.onclick = () => { window.location.href = '/session/new' }
        if (settingsBtn) settingsBtn.onclick = () => this.dispatchEvent($.event('cd:open-settings'))
        if (editorBtn) editorBtn.onclick = () => this.dispatchEvent($.event('cd:toggle-editor'))
        if (sidebarBtn) sidebarBtn.onclick = () => this.dispatchEvent($.event('cd:toggle-sidebar'))
    })

    return `
        <style>
            :host { display:flex; width:100%; height:var(--nav-height); position:sticky; top:0; z-index:10; }
            nav { width:100%; border-bottom:1px solid var(--border); background: var(--surface); height:var(--nav-height); display:flex; align-items:center; justify-content:space-between; padding:0 12px; box-shadow:0 2px 6px var(--shadow); box-sizing:border-box; }
            .nav-title { font-weight:800; letter-spacing:0.2px; color:var(--text-color); display:flex; align-items:center; gap:8px; font-family: var(--font-headings); }
            .nav-bttns { display:flex; align-items:center; gap: 12px; }
            .group { display:inline-flex; border:1px solid var(--border); border-radius:8px; overflow:hidden; background: var(--background-2); }
            .group > button { border:none; background:transparent; padding:6px 12px; color: var(--text-color); font-family: var(--font-ui); cursor:pointer; }
            .group > button + button { border-left:1px solid var(--border); }
            .group > button:hover { background: rgba(255,255,255,0.06); }
            .group.icons > button { font-size: 16px; font-weight: 800; line-height: 1; }
            .new { color: var(--text-color); }
            .new::before { content: '${AsciiSymbols.Plus}'; margin-right: 6px; }
        </style>
        <nav>
            <div class="nav-title">ConsoleDump</div>
            <div class="nav-bttns">
                <div class="group">
                    <button id="new" class="new">New session</button>
                </div>
                <div class="group icons" aria-label="tools">
                    <button id="settings" title="Global Settings">${AsciiSymbols.Settings}</button>
                    <button id="editor" title="Code Editor Mode">${AsciiSymbols.BracketLeftMedium}${AsciiSymbols.BracketRightMedium}</button>
                    <button id="sidebar" title="Toggle Tabs">${AsciiSymbols.Menu}</button>
                </div>
            </div>
        </nav>
    `
}