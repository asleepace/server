export default function CdSnippet({ state, onMounted }) {
  const lang = (state.lang || 'text').toUpperCase()
  const style = `
      :host { display:block; margin: 0 0 12px 0; }
      .wrap { border:1px solid var(--code-bord-color); background: var(--code-back-color); color: var(--code-text-color); border-radius:4px; }
      header { display:flex; align-items:center; justify-content:space-between; padding:6px 8px; font-family: Menlo, monospace; font-size:12px; border-bottom:1px solid #333; color:#bbb; }
      button { background: transparent; color:#ccc; border:1px solid #555; border-radius:3px; padding:2px 6px; cursor:pointer; font-family: Menlo, monospace; font-size:11px; }
      button:hover { background:#222; }
      pre { margin:0; padding:10px 12px; overflow:auto; overflow-x:auto; }
      code { white-space: pre; word-break: normal; font-size: 12px; }
      /* Dracula-ish token colors */
      .tok-str { color: #f1fa8c; }
      .tok-com { color: #6272a4; }
      .tok-key { color: #ff79c6; }
      .tok-num { color: #bd93f9; }
      .tok-flag { color: #8be9fd; }
      .tok-var { color: #50fa7b; }
      .tok-cmd { color: #8be9fd; }
    `;
  onMounted((host) => {
    const codeEl = host.shadowRoot.getElementById('code')
    const copyBtn = host.shadowRoot.getElementById('copy')
    const getLang = () => (host.getAttribute('lang') || 'text').toLowerCase()
    const update = () => {
      const codeText = (host.textContent || '').replace(/\n$/, '')
      codeEl.innerHTML = highlight(codeText, getLang())
    }
    // Initial render; defer one microtask to allow external text setters first
    queueMicrotask(update)
    // Watch for content changes to keep snippet in sync with host textContent
    const mo = new MutationObserver(() => update())
    mo.observe(host, { characterData: true, childList: true, subtree: true })
    host.__snippetObserver = mo
    copyBtn.onclick = async () => {
      const clean = sanitizeForCopy((host.textContent || '').replace(/\n$/, ''))
      try { await navigator.clipboard.writeText(clean) } catch {
        const ta = document.createElement('textarea'); ta.value = clean; ta.style.position = 'fixed'; ta.style.opacity = '0'; document.body.appendChild(ta); ta.select(); try { document.execCommand('copy') } catch { }; document.body.removeChild(ta)
      }
      const old = copyBtn.textContent; copyBtn.textContent = 'COPIED'; setTimeout(() => copyBtn.textContent = old, 900)
    }
  })

  return `
      <style>${style}</style>
      <div class="wrap">
        <header>
          <span>&rArr; ${lang}</span>
          <button id="copy" title="Copy">COPY</button>
        </header>
        <pre><code id="code"></code></pre>
      </div>
    `
}

// --- helpers (scoped to module) ---
const escapeBasic = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
const sanitizeForCopy = (src) => src
  .replace(/\r\n?|\u2028|\u2029/g, '\n')
  .replace(/[\u201C\u201D]/g, '"')
  .replace(/[\u2018\u2019]/g, "'")
  .replace(/[\u2013\u2014]/g, '-')
  .replace(/\u00A0/g, ' ')
  .replace(/[\u200B\u200C\u200D\uFEFF]/g, '')

function highlight(src, lang) {
  const rulesByLang = {
    bash: [
      [/((?:^|\s)#.*$)/gm, 'tok-com'],
      [/(^|\s)([a-zA-Z][\w-]*)(?=\s|$)/gm, 'tok-cmd'],
      [/(-{1,2}[\w-]+)/g, 'tok-flag'],
      [/(\$[A-Za-z_][A-Za-z0-9_]*)/g, 'tok-var'],
      [/(['"][^'"\n]*['"])/g, 'tok-str']
    ],
    js: [
      [/\/\*[\s\S]*?\*\//g, 'tok-com'],
      [/(^|[^:])\/\/.*$/gm, 'tok-com'],
      [/(['"`][^'"`\\]*(?:\\.[^'"`\\]*)*['"`])/g, 'tok-str'],
      [/(\b\d+(?:\.\d+)?\b)/g, 'tok-num'],
      [/(\b(?:const|let|var|function|return|async|await|import|from|export|new|try|catch|finally|if|else|for|while|switch|case|default|yield|class|extends)\b)/g, 'tok-key']
    ],
    py: [
      [/(?:'''[\s\S]*?'''|"""[\s\S]*?"""|(?:'[^'\\]*(?:\\.[^'\\]*)*'|"[^"\\]*(?:\\.[^"\\]*)*"))/g, 'tok-str'],
      [/(#.*$)/gm, 'tok-com'],
      [/(\b\d+(?:\.\d+)?\b)/g, 'tok-num'],
      [/(\b(?:def|return|async|await|import|from|as|class|pass|lambda|yield|for|while|if|elif|else|try|except|finally|with|True|False|None)\b)/g, 'tok-key']
    ]
  }
  const START_BASE = 0xE000
  const tokens = []
  let working = src
  const rules = rulesByLang[lang] || []
  for (const [pattern, cls] of rules) {
    working = working.replace(pattern, (m) => {
      const id = tokens.push({ text: m, cls }) - 1
      // Single-char placeholder in private-use range avoids collisions with token regexes
      return String.fromCharCode(START_BASE + id)
    })
  }
  let escaped = escapeBasic(working)
  escaped = escaped.replace(/[\uE000-\uE3FF]/g, (ch) => {
    const id = ch.charCodeAt(0) - START_BASE
    const t = tokens[id]
    return t ? `<span class="${t.cls}">${escapeBasic(t.text)}</span>` : ch
  })
  return escaped
}
