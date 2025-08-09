export default function CdTabs({ state, onMounted }) {
    const style = `
      :host { display:block; width:100%; }
      .segmented { display:flex; width:100%; border:1px solid var(--border); border-radius:8px; overflow:hidden; background: var(--surface); }
      .segmented button { flex:1; border:none; padding:6px 10px; background:transparent; color: var(--text-color); cursor:pointer; font-family: var(--font-ui); }
      .segmented button[aria-pressed="true"] { background: var(--background-2); }
      .segmented button:focus-visible { outline: 2px dashed var(--tint-color); outline-offset: 2px; }
      .panel { margin-top:8px; }
    `
    const tabs = (state.tabs || 'instructions,settings,hooks').split(',')
    const active = state.active || tabs[0]

    onMounted((host) => {
        const btns = Array.from(host.shadowRoot.querySelectorAll('button'))
        btns.forEach(btn => {
            btn.addEventListener('click', () => {
                host.setAttribute('active', btn.dataset.k)
            })
        })
        const segmented = host.shadowRoot.querySelector('.segmented')
        if (segmented) {
            segmented.setAttribute('role', 'tablist')
            btns.forEach((b) => {
                b.setAttribute('role', 'tab')
                b.setAttribute('aria-selected', String(b.getAttribute('aria-pressed') === 'true'))
                b.tabIndex = b.getAttribute('aria-pressed') === 'true' ? 0 : -1
            })
            segmented.addEventListener('keydown', (e) => {
                const current = btns.findIndex(b => b.getAttribute('aria-pressed') === 'true')
                const wrap = (i) => (i + btns.length) % btns.length
                let next = current
                if (e.key === 'ArrowRight') next = wrap(current + 1)
                else if (e.key === 'ArrowLeft') next = wrap(current - 1)
                else if (e.key === 'Home') next = 0
                else if (e.key === 'End') next = btns.length - 1
                else return
                e.preventDefault()
                btns[next].click()
                btns[next].focus()
            })
        }
    })

    return `
      <style>${style}</style>
      <div class="segmented">
        ${tabs.map(t => `<button data-k="${t}" aria-pressed="${t === active}">${t === active ? `❴ ${t} ❵` : t}</button>`).join('')}
      </div>
      <div class="panel"><slot name="${active}"></slot></div>
    `
}
