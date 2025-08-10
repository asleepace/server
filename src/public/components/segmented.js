export default function AppSegmented({ attrs, on, store }) {
  const items = (attrs?.items || '').split(',').map(s => s.trim()).filter(Boolean)
  const valueKey = attrs?.['bind:value'] || attrs?.bindValue
  const valueAttr = attrs?.value
  const getVal = () => valueKey ? store.get(valueKey, valueAttr || items[0]) : (valueAttr || items[0])
  const val = getVal()

  on('click', 'button[data-k]', (e) => {
    const key = e.target?.closest('button')?.getAttribute('data-k')
    if (!key) return
    if (valueKey) store.set(valueKey, key)
    else e.currentTarget?.closest('app-segmented')?.setAttribute('value', key)
  })

  const buttons = items.map((t, i) => {
    const active = t === val
    const cls = `flex-1 px-3 py-2 text-[var(--text-color)] ${active ? 'bg-[var(--background-2)]' : 'hover:bg-white/5'} ${i !== 0 ? 'border-l border-[var(--border)]' : ''}`
    return `<button data-k="${t}" aria-pressed="${active}" class="${cls}">${active ? `❴ ${t} ❵` : t}</button>`
  }).join('')

  return `
    <div role="tablist" class="inline-flex w-full border border-[var(--border)] rounded-lg overflow-hidden bg-[var(--surface)]">
      ${buttons}
    </div>
  `
}


