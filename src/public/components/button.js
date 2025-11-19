export default function AppButton({ attrs, on, store }) {
  const label = attrs?.label || ''
  const icon = attrs?.icon || ''
  const variant = attrs?.variant || 'ghost' // primary | ghost | outline
  const size = attrs?.size || 'md' // sm|md|lg
  const action = attrs?.action || ''

  const sizeCls = size === 'sm' ? 'px-2 py-1 text-sm' : size === 'lg' ? 'px-4 py-2 text-base' : 'px-3 py-1.5 text-sm'
  const base = 'inline-flex items-center gap-1 rounded-md border transition-colors'
  const palette = variant === 'primary'
    ? 'bg-[var(--tint-color)] text-black border-[var(--tint-color)] hover:bg-[var(--tint-highlight)]'
    : variant === 'outline'
      ? 'bg-transparent text-[var(--text-color)] border-[var(--border)] hover:bg-white/5'
      : 'bg-[var(--background-2)] text-[var(--text-color)] border-[var(--border)] hover:bg-white/5'

  // one delegated listener for actions
  on('click', 'button', (e) => {
    const act = e.currentTarget?.getAttribute('data-action') || action
    if (!act) return
    // dispatch:evt or nav:/path or store:key=value
    if (act.startsWith('dispatch:')) {
      const name = act.slice('dispatch:'.length)
      e.currentTarget?.dispatchEvent(new CustomEvent(name, { bubbles: true, composed: true }))
    } else if (act.startsWith('nav:')) {
      const href = act.slice('nav:'.length)
      try { window.location.href = href } catch { }
    } else if (act.startsWith('store:')) {
      const [key, value] = act.slice('store:'.length).split('=')
      if (key) store.set(key, value)
    }
  })

  return `
    <button type="button" class="${base} ${palette} ${sizeCls}" data-action="${action}">
      ${icon ? `<span aria-hidden="true">${icon}</span>` : ''}
      ${label ? `<span>${label}</span>` : ''}
      <slot></slot>
    </button>
  `
}


