// Minimal runtime loader for .wc single-file web components
// MVP features:
// - @component tag: HTMLElement
// - @prop name: type = default (reflect?)  [parsed but not enforced]
// - @style ... @end  → inlined <style>
// - @template ... @end → HTML with {{prop}} bindings and on:click="method"
// - @script block with @method name: () => { ... }
// Compiles to light-DOM component via App.define(tag, renderFn)


async function fetchText(url) {
  const res = await fetch(url)
  if (!res.ok) throw new Error(`wc: failed to fetch ${url}: ${res.status}`)
  return await res.text()
}

function parseWC(src) {
  const out = { component: null, props: [], style: '', template: '', methods: {} }
  // @component MyTag: HTMLElement
  const compMatch = src.match(/@component\s+([\w-]+)\s*:\s*([\w]+)/)
  if (compMatch) out.component = { tag: compMatch[1], base: compMatch[2] }
  // @prop (modifiers)? name: Type (= default)?  ; modifiers optional, comma/space separated
  const propRe = /^(?:\t| )*@prop\s*(?:\(([^)]*)\))?\s*([a-zA-Z_][\w-]*)\s*:\s*([A-Za-z_][\w]*)(?:\s*=\s*([^\n]+))?/gm
  let pm
  while ((pm = propRe.exec(src))) {
    out.props.push({ mods: (pm[1] || '').trim(), name: pm[2], type: pm[3], def: (pm[4] || '').trim() })
  }
  // @style ... @end
  const styleMatch = src.match(/@style[\s\S]*?\n([\s\S]*?)\n@end/)
  if (styleMatch) out.style = styleMatch[1]
  // @template ... @end
  const tplMatch = src.match(/@template(?:\s*\(html\))?[\s\S]*?\n([\s\S]*?)\n@end/)
  if (tplMatch) out.template = tplMatch[1]
  // @method name: () => { ... }
  const methodRe = /@method\s+([a-zA-Z_][\w]*)\s*:\s*\(\)\s*=>\s*\{([\s\S]*?)\}/g
  let mm
  while ((mm = methodRe.exec(src))) {
    out.methods[mm[1]] = mm[2]
  }
  if (!out.component) throw new Error('wc: missing @component declaration')
  return out
}

function compileToModule(parsed) {
  const { component, props, style, template, methods } = parsed
  // Build prop accessors with (attr) modifier and type coercion
  const toJsType = (t) => {
    const k = String(t || '').toLowerCase()
    if (k === 'number') return 'number'
    if (k === 'boolean') return 'boolean'
    if (k === 'string') return 'string'
    return 'any'
  }
  const attrGetters = (props || []).filter(p => /\battr\b/.test(p.mods || '')).map(p => {
    const jsName = p.name.replace(/-([a-z])/g, (_, a) => a.toUpperCase())
    const attrName = `data-attr-${p.name.replace(/[A-Z]/g, m => `-${m.toLowerCase()}`)}`
    const ctor = (p.type || 'String')
    const parseExpr = ctor.toLowerCase() === 'number' ? `Number(value)` : ctor.toLowerCase() === 'boolean' ? `value === 'true' || value === ''` : `String(value)`
    const jsType = toJsType(ctor)
    return `/** @returns {${jsType}} */\nObject.defineProperty(this, '${jsName}', { get: () => { const value = this.getAttribute('${attrName}'); return (${parseExpr}); } })`
  }).join('\n')
  // Convert {{prop}} to JS interpolation using attrs
  const htmlEscaped = template
    .replace(/`/g, '\\`')
    .replace(/\{\{\s*([a-zA-Z_][\w-]*)\s*\}\}/g, '${(attrs["$1"] ?? "")}')
    // on:click="save" → data-on-click="save"
    .replace(/on:([a-zA-Z]+)="([a-zA-Z_][\w]*)"/g, 'data-on-$1="$2"')

  const methodNames = Object.keys(methods)
  const methodsInit = methodNames.map(n => `this["${n}"] = function(event){ ${methods[n]} }`).join('\n')
  const eventsInit = `
    // delegated listeners compiled from on:*
    on('click','[data-on-click]', (e) => {
      const name = e.target && e.target.closest && e.target.closest('[data-on-click]')?.getAttribute('data-on-click')
      if (!name || typeof this[name] !== 'function') return
      try { this[name](e) } catch(err) { console.warn('handler error', name, err) }
    })`

  const typeName = component.tag.replace(/(^|-)([a-z])/g, (_, __, c) => c.toUpperCase()) + 'Props'
  const typedef = `/**\n * @typedef {Object} ${typeName}\n${(props || []).map(p => ` * @property {${toJsType(p.type)}} ${p.name}`).join('\n')}\n */`

  const module = `
    import { App } from '/components/index.js'
    // @ts-check
    ${typedef}
    App.define('${component.tag}', function({ attrs, on, store }){
      if (!this.__wcMethodsInit) { ${methodsInit}; this.__wcMethodsInit = true }
      if (!this.__wcPropsInit) { ${attrGetters}; this.__wcPropsInit = true }
      ${eventsInit}
      return \`<style>${style.replace(/`/g, '\\`')}</style>${htmlEscaped}\`
    })
  `
  return module
}

async function defineWC(url) {
  const src = await fetchText(url)
  const parsed = parseWC(src)
  const code = compileToModule(parsed)
  const blob = new Blob([code], { type: 'application/javascript' })
  const modUrl = URL.createObjectURL(blob)
  try {
    await import(modUrl)
  } finally {
    URL.revokeObjectURL(modUrl)
  }
}

// Public API
export const WCRuntime = { defineWC }

// Auto-load any <link rel="wc" href="..."> declarations
export function bootstrapWCAuto() {
  const links = Array.from(document.querySelectorAll('link[rel="wc"][href]'))
  links.forEach(link => {
    defineWC(link.getAttribute('href')).catch(err => console.warn('[wc]', err))
  })
}

// Optional automatic bootstrap
if (typeof window !== 'undefined') {
  window.WCRuntime = WCRuntime
}


