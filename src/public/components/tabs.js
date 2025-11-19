export default function CdTabs({ state, onMounted }) {
  const style = `
      :host { display:flex; flex-direction: column; gap: 8px; }
      .panel { padding-top:12px; display:flex; flex-direction: column; gap: 16px; }
      .panel ::slotted(*) { display: flex; flex-direction: column; padding-bottom: 8px; margin: 0; }
      .panel ::slotted(cd-snippet),
      .panel ::slotted(h3),
      .panel ::slotted(h4),
      .panel ::slotted(p) { padding-top: 16px; padding-bottom: 8px; }
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
      <div class="inline-flex w-full border border-[var(--border)] rounded-lg overflow-hidden bg-[var(--surface)]" role="tablist">
        ${tabs.map((t, i) => `
          <button role="tab" tabindex="${t === active ? 0 : -1}" aria-selected="${t === active}" data-k="${t}" aria-pressed="${t === active}"
            class="flex-1 px-3 py-2 font-[var(--font-ui)] text-[var(--text-color)] hover:bg-[var(--background-2)] ${t === active ? 'bg-[var(--background-2)]' : ''} ${i !== 0 ? 'border-l border-[var(--border)]' : ''}">
            ${t === active ? `❴ ${t} ❵` : t}
          </button>
        `).join('')}
      </div>
      <div class="panel">
        <slot name="${active}"></slot>
      </div>
    `
}
