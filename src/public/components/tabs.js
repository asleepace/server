export default function CdTabs({ state, onMounted }) {
  const style = `
      :host { display:flex; flex-direction: column; }
      .segmented { display:flex; width:100%; border:1px solid var(--border); border-radius:8px; overflow:hidden; background: var(--surface); }
      .segmented button { flex:1; border:none; padding:8px 12px; background:transparent; color: var(--text-color); cursor:pointer; font-family: var(--font-ui); }
      .segmented button + button { border-left:1px solid var(--border); }
      .segmented button[aria-pressed="true"] { background: var(--background-2); }
      .segmented button:focus-visible { outline: 2px dashed var(--tint-color); outline-offset: 2px; }
      .panel { margin-top:12px; display:block; }
      .panel ::slotted(*) { display:block; margin: 0 0 14px 0; }
      .panel ::slotted(cd-snippet) { margin: 8px 0 16px 0; }
      .panel ::slotted(h3),
      .panel ::slotted(h4) { margin: 10px 0 8px 0; }
      .panel ::slotted(p) { margin: 0 0 10px 0; }
    `
  const tabs = (state.tabs || 'instructions,settings,hooks').split(',')
  const active = state.active || tabs[0]

  onMounted((host) => {
    const root = host.shadowRoot
    // Delegated click handler survives re-renders because it's bound on shadowRoot
    root.addEventListener('click', (e) => {
      const btn = e.target && e.target.closest && e.target.closest('button')
      if (!btn || !root.contains(btn)) return
      const key = btn.dataset.k
      if (key) host.setAttribute('active', key)
    })
    // Keyboard navigation (query fresh each time to survive re-renders)
    root.addEventListener('keydown', (e) => {
      const container = root.querySelector('.segmented')
      if (!container) return
      const btns = Array.from(root.querySelectorAll('.segmented button'))
      if (!btns.length) return
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
  })

  return `
      <style>${style}</style>
      <div class="segmented" role="tablist">
        ${tabs.map(t => `<button role="tab" tabindex="${t === active ? 0 : -1}" aria-selected="${t === active}" data-k="${t}" aria-pressed="${t === active}">${t === active ? `❴ ${t} ❵` : t}</button>`).join('')}
      </div>
      <div class="panel"><slot name="${active}"></slot></div>
    `
}
