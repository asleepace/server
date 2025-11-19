// Hooks editor: manage client-provided JS snippets that transform incoming messages
// Uses the function-first component pattern with the centralized linking step

export default function CdHooks({ state, onMounted, onMethod }) {
  const style = `
    :host { display:block; }
    .toolbar { display:flex; gap:8px; margin:6px 0 10px 0; }
    .btn { border:1px solid var(--border); background:transparent; color: var(--text-color); padding:4px 8px; border-radius:6px; font-family: var(--font-ui); }
    .btn:hover { background: var(--background-2); }
    .list { display:flex; flex-direction:column; gap:10px; }
    .card { border:1px solid var(--border); background: var(--surface); border-radius:8px; padding:8px; display:flex; flex-direction:column; gap:6px; }
    .card header { display:flex; align-items:center; justify-content:space-between; gap:8px; }
    .card header .meta { display:flex; align-items:center; gap:8px; }
    .name { border:1px solid var(--border); background:transparent; color: var(--text-color); border-radius:6px; padding:4px 6px; min-width:160px; font-family: var(--font-ui); }
    .code { width:100%; min-height:110px; resize:vertical; border:1px solid var(--border); border-radius:6px; background: var(--code-back-color); color: var(--code-text-color); font-family: var(--font-mono, Menlo, monospace); font-size:12px; padding:8px; }
    .row { display:flex; align-items:center; gap:6px; }
    .muted { color: var(--muted-text); font-size: 0.85rem; }
  `

  function renderList(hooks) {
    return hooks.map((h, i) => `
      <div class="card" data-id="${h.id}">
        <header>
          <div class="meta">
            <input class="name" value="${h.name || 'Hook'}" placeholder="name" />
            <label class="row"><input type="checkbox" class="toggle" ${h.enabled ? 'checked' : ''}/> enabled</label>
          </div>
          <div class="row">
            <button class="btn" onclick="@moveUp">↑</button>
            <button class="btn" onclick="@moveDown">↓</button>
            <button class="btn" onclick="@delete">Delete</button>
          </div>
        </header>
        <textarea class="code" spellcheck="false" placeholder="function transform(ctx, text) {\n  return text\n}">${h.code || 'function transform(ctx, text) {\n  return text\n}'}</textarea>
        <div class="row">
          <button class="btn" onclick="@save">Save</button>
          <span class="muted">Runs sequentially on incoming messages. Args: (ctx, text)</span>
        </div>
      </div>
    `).join('')
  }

  // Local helpers
  const load = () => { try { return JSON.parse(localStorage.getItem('cd_hooks') || '[]') } catch { return [] } }
  const saveAll = (hooks) => { try { localStorage.setItem('cd_hooks', JSON.stringify(hooks)) } catch { }; try { window.CD_HOOKS && window.CD_HOOKS.reload && window.CD_HOOKS.reload() } catch { } }
  const uid = () => Math.random().toString(36).slice(2, 9)

  // Register component methods via onMethod; render returns a string only
  onMethod({
    add() {
      const hooks = load()
      hooks.push({ id: uid(), name: 'Hook', code: 'function transform(ctx, text) {\n  return text\n}', enabled: true })
      saveAll(hooks)
      const list = this.shadowRoot.querySelector('.list')
      if (list) list.innerHTML = renderList(hooks)
    },
    save(e) {
      const card = e.currentTarget.closest('.card')
      if (!card) return
      const hooks = load()
      const id = card.dataset.id
      const name = card.querySelector('.name').value
      const code = card.querySelector('.code').value
      const enabled = card.querySelector('.toggle').checked
      const idx = hooks.findIndex(h => h.id === id)
      if (idx >= 0) hooks[idx] = { id, name, code, enabled }
      saveAll(hooks)
    },
    delete(e) {
      const card = e.currentTarget.closest('.card')
      if (!card) return
      const hooks = load().filter(h => h.id !== card.dataset.id)
      saveAll(hooks)
      const list = this.shadowRoot.querySelector('.list')
      if (list) list.innerHTML = renderList(hooks)
    },
    moveUp(e) {
      const card = e.currentTarget.closest('.card')
      if (!card) return
      const hooks = load()
      const i = hooks.findIndex(h => h.id === card.dataset.id)
      const j = Math.max(0, i - 1)
      if (i !== j) { const [h] = hooks.splice(i, 1); hooks.splice(j, 0, h) }
      saveAll(hooks)
      const list = this.shadowRoot.querySelector('.list')
      if (list) list.innerHTML = renderList(hooks)
    },
    moveDown(e) {
      const card = e.currentTarget.closest('.card')
      if (!card) return
      const hooks = load()
      const i = hooks.findIndex(h => h.id === card.dataset.id)
      const j = Math.min(hooks.length - 1, i + 1)
      if (i !== j) { const [h] = hooks.splice(i, 1); hooks.splice(j, 0, h) }
      saveAll(hooks)
      const list = this.shadowRoot.querySelector('.list')
      if (list) list.innerHTML = renderList(hooks)
    }
  })

  onMounted((host) => {
    const list = host.shadowRoot.querySelector('.list')
    if (list) list.innerHTML = renderList(load())
  })

  return `
      <style>${style}</style>
      <div class="toolbar">
        <button class="btn" onclick="@add">➕ Add hook</button>
      </div>
      <div class="list"></div>
    `
}


