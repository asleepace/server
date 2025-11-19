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
        </style>
        <nav class="w-full h-[var(--nav-height)] flex items-center justify-between px-3 border-b border-[var(--border)] bg-[var(--surface)] shadow-[0_2px_6px_var(--shadow)] box-border">
            <div class="font-extrabold tracking-[0.2px] text-[var(--text-color)] flex items-center gap-2 font-[var(--font-headings)]">ConsoleDump</div>
            <div class="flex items-center gap-3">
                <div class="inline-flex border border-[var(--border)] rounded-lg overflow-hidden bg-[var(--background-2)]">
                    <button id="new" class="px-3 py-1.5 text-[var(--text-color)] font-[var(--font-ui)] hover:bg-white/5 flex items-center gap-1">
                        <span aria-hidden="true">${AsciiSymbols.Plus}</span>
                        <span>New session</span>
                    </button>
                </div>
                <div class="inline-flex border border-[var(--border)] rounded-lg overflow-hidden bg-[var(--background-2)]" aria-label="tools">
                    <button id="settings" title="Global Settings" class="px-3 py-1.5 hover:bg-white/5 text-[16px] font-extrabold">${AsciiSymbols.Settings}</button>
                    <button id="editor" title="Code Editor Mode" class="px-3 py-1.5 hover:bg-white/5 text-[16px] font-extrabold">${AsciiSymbols.BracketLeftMedium}${AsciiSymbols.BracketRightMedium}</button>
                    <button id="sidebar" title="Toggle Tabs" class="px-3 py-1.5 hover:bg-white/5 text-[16px] font-extrabold">${AsciiSymbols.Menu}</button>
                </div>
            </div>
        </nav>
    `
}