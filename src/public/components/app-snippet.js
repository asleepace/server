export default function AppSnippet({ attrs, on, store }) {
  const host = /** @type {HTMLElement} */ (this)
  const lang = (attrs?.lang || 'text').toLowerCase()
  const header = attrs?.header || ''
  const bindKey = attrs?.['data-bind-text']
  const directText = attrs?.text || ''
  const cls = 'rounded-md border border-[var(--code-bord-color)] overflow-hidden bg-[var(--code-back-color)] text-[var(--code-text-color)]'

  const getText = () => (bindKey ? (store.get(bindKey, directText) || '') : directText)
  const content = highlight(getText(), lang)

  // copy action
  on('click', 'button[data-action="copy"]', async (e) => {
    const raw = getText()
    const clean = sanitizeForCopy(raw)
    try { await navigator.clipboard.writeText(clean) } catch {
      try {
        const ta = document.createElement('textarea'); ta.value = clean; ta.style.position = 'fixed'; ta.style.opacity = '0'; document.body.appendChild(ta); ta.select(); document.execCommand('copy'); document.body.removeChild(ta)
      } catch { }
    }
  })

  // subscribe once for bind updates
  if (bindKey && !host.__appSnippetSub) {
    host.__appSnippetSub = store.subscribe(bindKey, (val) => {
      const code = host.querySelector('code[data-role="code"]')
      if (code) code.innerHTML = highlight(String(val ?? ''), lang)
    })
  }

  return `
    <div class="${cls}">
      <style>
        .tok-str { color: #f1fa8c; }
        .tok-com { color: #6272a4; }
        .tok-key { color: #ff79c6; }
        .tok-num { color: #bd93f9; }
        .tok-flag { color: #8be9fd; }
        .tok-var { color: #50fa7b; }
        .tok-cmd { color: #8be9fd; }
      </style>
      <header class="flex items-center justify-between px-2 py-1.5 text-[12px] font-mono border-b border-[#333] text-[#bbb]">
        <span>${header || lang.toUpperCase()}</span>
        <button type="button" class="px-2 py-0.5 text-[#ccc] border border-[#555] rounded hover:bg-[#222]" data-action="copy">COPY</button>
      </header>
      <pre class="m-0 p-3 overflow-auto"><code data-role="code" class="text-[12px]">${content}</code></pre>
    </div>
  `
}

// Simple tokenizer-based highlighter (copied from cd-snippet)
function escapeBasic(s) { return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;') }

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


