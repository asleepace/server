/**
 * @name index.js (main)
 * @file /scripts/components/index.js
 * @description shared client side classes, utils, etc.
 */


/**
 * ASCII Symbol to HTML code map.
 * 
 * @see https://www.toptal.com/designers/htmlarrows/symbols/
 */
export const AsciiSymbols = {
  /** # ⚙ Settings/Gear icon */
  Settings: '&#9881;',
  /** # ⚠ Warning triangle */
  Warning: '&#9888;',
  /** # ✖ Heavy multiplication X */
  CrossHeavy: '&#10006;',
  /** # ✕ Multiplication X */
  Cross: '&#10005;',
  /** # ✔ Heavy check mark */
  CheckHeavy: '&#10004;',
  /** # ✓ Check mark */
  Check: '&#10003;',
  /** # ✗ Ballot X */
  BallotX: '&#10007;',
  /** # ✘ Heavy ballot X */
  BallotXHeavy: '&#10008;',
  /** # ❘ Light vertical bar */
  VerticalBarLight: '&#10072;',
  /** # ❙ Medium vertical bar */
  VerticalBarMedium: '&#10073;',
  /** # ❚ Heavy vertical bar */
  VerticalBarHeavy: '&#10074;',
  /** # ➕ Heavy plus (add) */
  Plus: '&#10133;',
  /** # ☰ Hamburger/menu */
  Menu: '&#9776;',
  /** # ➩ Right arrow with hook */
  ArrowRightHook: '&#10153;',
  /** # ❴ Medium left-pointing angle bracket ornament */
  BracketLeftMedium: '&#10100;',
  /** # ❵ Medium right-pointing angle bracket ornament */
  BracketRightMedium: '&#10101;',
}


const sharedStyles = (function () {
  const styles = document.createElement('style')
  styles.textContent = `
        :host {
           /* Shared design tokens available to all components */
           --color-text: rgba(255, 255, 255, 0.86);
           --color-tint: #7efc7a;
           --color-bg: #15171a;
           --color-surface: #15171a;
           --color-border: #3a3d42;
           --color-shadow: rgba(0,0,0,0.2);
           --text-size: 14px;
           --text-font: monospace;
        }

        /* Minimal utilities only; no component-specific styling here */
        .flex { display:flex; }
        .flex-col { display:flex; flex-direction:column; }
        .flex-row { display:flex; flex-direction:row; }
        .w-full { width:100%; }
        .h-full { height:100%; }
        .m-0 { margin:0; }
        .p-0 { padding:0; }
        .p-1 { padding:4px; }
        .p-2 { padding:8px; }
    `
  return styles
})()

/**
 * ## BaseElement 
 * The base element which all other custom components should extend.
 * Provides basic logic for registering custom elements, rendering,
 * getting state, setting state, and shared styling.
 */
export class BaseElement extends HTMLElement {
  constructor() {
    super()
    this.attachShadow({ mode: 'open' })
    // Insert shared styles on construction; subsequent renders must preserve them
    this.shadowRoot.appendChild(sharedStyles.cloneNode(true))
    this.isMounted = false
  }

  onMounted(callbackFn) {
    if (this.isMounted) return
    this.isMounted = true
    callbackFn.call(this)
  }

  render() {
    return `<slot></slot>`
  }

  // Render while preserving shared styles
  renderToShadow() {
    const template = document.createElement('template')
    try {
      template.innerHTML = this.render() || ''
    } catch (e) {
      console.error('Error rendering component:', e)
      template.innerHTML = '<div>Error rendering component</div>'
    }
    // Always place shared styles first
    this.shadowRoot.replaceChildren(sharedStyles.cloneNode(true), template.content)
  }

  connectedCallback() {
    if (!this.isConnected) return
    this.renderToShadow()
    // onMounted is coordinated by $.define subclass after link step
  }

  attributeChangedCallback() {
    if (this.shadowRoot) this.renderToShadow()
  }

  // Ensure direct attribute updates trigger a re-render even without observedAttributes
  setAttribute(name, value) {
    super.setAttribute(name, value)
    if (this.shadowRoot) this.renderToShadow()
  }

  get state() {
    return this.getAttributeNames().reduce((state, attrName) => {
      state[attrName] = this.getAttribute(attrName)
      return state
    }, {})
  }

  set state(partialState) {
    Object.entries(partialState).forEach(([attrName, attrValue]) => {
      this.setAttribute(attrName, attrValue)
    })
    // Ensure UI updates even if attributes are not observed
    if (this.shadowRoot) this.renderToShadow()
  }
}

/**
 * # $
 * Shared client-side logic for interfacing with the DOM, selecting elements, handling
 * exceptions, accessing web apis, etc. Also provides short-hand notation for common
 * operations like get the document, creating elements, etc.
 */
export class $ {

  /**
   * Create a new element on the dom.
   * @param {string} tagName - html element to create.
   * @param {*?} object (optional) - object with properties
   * @returns {HTMLElement}
   */
  static create(tagName, object = {}) {
    const element = document.createElement(tagName)
    Object.entries(object).forEach(([key, value]) => {
      element[key] = value
    })
    return element
  }

  /**
   * Sets a global attribute on the document.documentElement.
   * @param {string} key 
   * @param {*} value 
   */
  static setGlobal(key, value) {
    document.documentElement.setAttribute(key, value)
  }

  /**
   * Shared memory which can be used like a scratch pad.
   */
  static memory = new Map()
  static {
    // Initialize theme from storage on page load
    const defaultTheme = $.store.get('theme') ?? 'dark'
    document.documentElement.setAttribute('data-theme', defaultTheme)

  }

  /**
   * Simple helper for querying elements on the dom, the callback will only trigger
   * if the element is found. Will return the result of callbackFn(element).
   * @param {string} selector 
   * @param {function} callbackFn
   * @returns {*}
   */
  static select(selector, callbackFn = (elem) => elem) {
    const result = document.querySelector(selector)
    if (!result) {
      console.warn(`@select failed finding "${selector}"`)
      return null
    }
    return callbackFn(result)
  }

  /**
   * Coerce the exception to an error instance.
   * @param {*} e - any exception
   * @returns {Error}
   */
  static err(e) {
    return e instanceof Error ? e : new Error(String(e))
  }

  /**
   * Try to run the given fn with the specified args.
   * @param {*} fn 
   * @param  {...any} args 
   * @returns {[value, null] | [null, Error]}
   */
  static try(fn, ...args) {
    try {
      return [fn(...args), null]
    } catch (e) {
      console.warn(e)
      return [null, this.err(e)]
    }
  }

  /**
   * Create a custom event with an optional detail
   * @param {string} name 
   * @param {*} detail 
   * @returns {CustomEvent}
   */
  static event(name, detail) {
    return new CustomEvent(name, { bubbles: true, composed: true, detail })
  }

  /**
   * A simple wrapper around localStorage which catches errors and handles
   * default values and type casting.
   */
  static get store() {
    return {
      /**
       * @param {string} storageKey 
       * @param {string | { toString(): string }} value
       * @returns {void}
       */
      set(storageKey, value) {
        $.try(() => localStorage.setItem(storageKey, String(value)))
      },
      /**
       * @param {string} storageKey 
       * @param {string?} fallbackValue 
       * @returns {string?}
       */
      get(storageKey, fallbackValue) {
        const [val, err] = $.try(() => localStorage.getItem(storageKey))
        return val ?? fallbackValue
      }
    }
  }

  /**
   * Attach a property to an element returned by a selector.
   * @param {string} selector
   * @param {string} propertyName
   * @param {*} value
   * @returns {void}
   */
  static attach(selector, propertyName, value) {
    $.select(selector, (elem) => {
      if (propertyName in elem) {
        elem[propertyName] = value
      } else {
        console.warn(`@attach missing property "${propertyName}" on`, elem)
      }
    })
  }

  /**
   * Attach a click handler to a selector.
   * @param {string} selector 
   * @param {Function} fn 
   * @returns {void}
   */
  static onclick(selector, fn) {
    const element = $.select(selector)
    if (!element) return
    element.onclick = fn.bind(element)
  }

  /**
   * Register custom elements on the window the specified name and render function,
   * these inherit shared styles, render logic, etc.
   * @param {string} elemName - name of custom element (e.g. "app-navbar" => <app-navbar />)
   * @param {function} renderFn - function which is called with current state and returns innerHTML
   * @returns {void}
   */
  static define(elemName, renderFn) {
    if (typeof window === 'undefined') return
    if (window.customElements.get(elemName)) return
    console.log(`@components registering: "${elemName}"`)

    window.customElements.define(elemName, class extends BaseElement {
      constructor() {
        super()
        this._renderFn = renderFn.bind(this)
        this._mountedCallbacks = []
        this._methods = {}
        this._delegated = new Map() // eventType -> [ { selector, handler } ]
      }

      onMounted(mountedCallback) {
        if (typeof mountedCallback !== 'function') return
        if (this.__mountedGuard) return
        // Defer until link step binds methods and initial render is complete
        this._mountedCallbacks.push(mountedCallback)
      }

      // Public alias for method map for ergonomic direct assignment in components
      get methods() { return this._methods }
      set methods(obj) { this._methods = obj || {} }

      // Register one or more methods
      onMethod(nameOrMap, maybeFn) {
        if (!nameOrMap) return
        // onMethod('name', fn)
        if (typeof nameOrMap === 'string' && typeof maybeFn === 'function') {
          this._methods[nameOrMap] = maybeFn.bind(this)
          return
        }
        // onMethod({ name: fn, ... })
        if (nameOrMap && typeof nameOrMap === 'object' && !Array.isArray(nameOrMap)) {
          Object.entries(nameOrMap).forEach(([key, fn]) => {
            if (typeof fn === 'function') this._methods[key] = fn.bind(this)
          })
          return
        }
        // onMethod((host) => ({ name(){...} })) or onMethod((host) => void)
        if (typeof nameOrMap === 'function') {
          try {
            const result = nameOrMap.call(this, this)
            if (result && typeof result === 'object') {
              Object.entries(result).forEach(([key, fn]) => {
                if (typeof fn === 'function') this._methods[key] = fn.bind(this)
              })
            }
          } catch (e) { console.warn('[components] onMethod callback error:', e) }
        }
      }

      // Alias: method('name', fn)
      method(name, fn) { this.onMethod(name, fn) }

      // Delegated events: bind once per event type on shadowRoot
      onEvent(eventType, cssSelector, handler) {
        if (!eventType || !cssSelector || typeof handler !== 'function') return
        const type = String(eventType)
        if (!this._delegated.has(type)) this._delegated.set(type, [])
        const arr = this._delegated.get(type)
        arr.push({ selector: cssSelector, handler })
        // Ensure listener installed once
        if (!this.__delegatedInstalled) this.__delegatedInstalled = new Set()
        if (!this.__delegatedInstalled.has(type)) {
          this.__delegatedInstalled.add(type)
          this.shadowRoot.addEventListener(type, (ev) => {
            const list = this._delegated.get(type) || []
            const target = ev.target
            for (const { selector, handler } of list) {
              if (target && target.closest && target.closest(selector)) {
                try { handler.call(this, ev) } catch (e) { console.warn(`[components] delegated ${type} handler error:`, e) }
              }
            }
          })
        }
      }

      // Dispatch a CustomEvent from host or provided target
      dispatch(name, detail, target) {
        try {
          const evt = $.event(String(name), detail)
            ; (target || this).dispatchEvent(evt)
        } catch (e) { console.warn('[components] dispatch error:', e) }
      }

      render() {
        try {
          const html = this._renderFn({
            state: this.state,
            onMounted: this.onMounted.bind(this),
            onMethod: this.onMethod.bind(this),
            method: this.method.bind(this),
            onEvent: this.onEvent.bind(this),
            dispatch: this.dispatch.bind(this)
          })
          if (typeof html === 'string') return this.#linkInlineHandlers(html || "")
          // Enforce string-only returns
          console.warn(`[components] ${elemName} should return an HTML string; got`, typeof html)
          return ""
        } catch (e) {
          console.error(`Error rendering ${elemName}:`, e)
          return `<div>Error rendering component</div>`
        }
      }

      renderToShadow() {
        // Call base renderer to update shadow DOM, then bind handlers for this render
        super.renderToShadow()
        try {
          const root = this.shadowRoot
          if (!root) return
          const all = root.querySelectorAll('*')
          all.forEach((el) => {
            Array.from(el.attributes).forEach(attr => {
              const name = attr.name.toLowerCase()
              const val = attr.value
              const isOn = name.startsWith('on')
              const isDataOn = name.startsWith('data-on')
              if (!isOn && !isDataOn) return
              const eventName = isOn ? name.slice(2) : name.slice(8)
              if (!val) return
              // Supported forms:
              // 1) @handlerName
              // 2) @method:handlerName or method:handlerName
              // 3) @event:eventName or event:eventName
              // 4) @event:@attrName to read from element attribute (e.g., @id)
              const raw = String(val).trim()
              const lower = raw.replace(/^@/, '')
              let bound = false
              // method:NAME
              const m = lower.match(/^method:([a-zA-Z_$][\w$]*)$/)
              if (m) {
                const fn = this._methods && this._methods[m[1]]
                if (typeof fn === 'function') {
                  el.removeAttribute(attr.name)
                  el.addEventListener(eventName, fn.bind(this))
                  bound = true
                }
              }
              // event:NAME or event:@attr
              if (!bound) {
                const evm = lower.match(/^event:(.+)$/)
                if (evm) {
                  const spec = evm[1]
                  el.removeAttribute(attr.name)
                  el.addEventListener(eventName, (e) => {
                    let eventToDispatch = spec
                    if (spec.startsWith('@')) {
                      const key = spec.slice(1)
                      // Prefer host value, fallback to element attribute
                      const hostVal = this.getAttribute(key) ?? (this[key] != null ? String(this[key]) : '')
                      eventToDispatch = hostVal || el.getAttribute(key) || ''
                    }
                    if (eventToDispatch) this.dispatch(eventToDispatch, { originalEvent: e, target: el })
                  })
                  bound = true
                }
              }
              // bare handler name (from @handler or handler())
              if (!bound) {
                const bare = raw.replace(/^@/, '').replace(/\(\)$/, '')
                const fn = this._methods && this._methods[bare]
                if (typeof fn === 'function') {
                  el.removeAttribute(attr.name)
                  el.addEventListener(eventName, fn.bind(this))
                  bound = true
                }
              }
              if (!bound) {
                // Leave attribute intact for visibility but warn once
                console.warn(`[components] unbound handler for ${name}="${val}" on`, el)
              }
            })
          })
        } catch (err) {
          console.warn('[components] link step (render-time) failed:', err)
        }
      }

      // Replace @onclick="method()" with a safe data-onclick and leave original for clarity
      #linkInlineHandlers(html) {
        if (!html || typeof html.replace !== 'function') return html
        // Support @onclick="handler()" and @onclick="@handler" forms
        // Convert to data-onclick="@handler" so connectedCallback can bind
        return html
          // on*="@handler"
          .replace(/\s(on[a-z]+)="@([a-zA-Z_$][\w$]*)"/g, ' data-$1="@$2"')
          // @on*="handler()" or @on*="@handler" → data-on*
          .replace(/\s@on([a-z]+)="([a-zA-Z_$][\w$]*)\(\)"/g, ' data-on$1="@$2"')
          .replace(/\s@on([a-z]+)="@([a-zA-Z_$][\w$]*)"/g, ' data-on$1="@$2"')
          // Generic @event="..." → data-onevent (e.g., @click="...")
          .replace(/\s@([a-z]+)="([^"]+)"/g, ' data-on$1="$2"')
        // Allow method:NAME and event:SPEC to pass through unchanged; we only convert attribute name to data-on*
      }

      connectedCallback() {
        super.connectedCallback()
        // Run onMounted callbacks once (render already happened in base constructor path)
        if (!this.__mountedGuard) {
          this.__mountedGuard = true
          const cbs = this._mountedCallbacks.splice(0)
          cbs.forEach(callback => {
            try { callback.call(this, this) } catch (e) { console.error(`Error in onMounted callback for ${elemName}:`, e) }
          })
        }
      }
    })
  }

  static find(selector) {
    return new $(selector)
  }

  // instance
  element = null
  constructor(selector) {
    this.element = $.select(selector)
  }
}

/**
 * Light-DOM component system (App*) for simple, attribute-driven primitives
 */
export class AppStore {
  constructor(initial = {}) {
    this.data = { ...initial }
    this.listeners = new Map() // key -> Set<fn>
  }
  get(key, fallback = undefined) { return this.data[key] ?? fallback }
  set(key, value) {
    this.data[key] = value
    const subs = this.listeners.get(key)
    if (subs) subs.forEach(fn => { try { fn(value, key) } catch { } })
    document.dispatchEvent(new CustomEvent('app:state', { detail: { key, value } }))
  }
  subscribe(key, fn) {
    if (!this.listeners.has(key)) this.listeners.set(key, new Set())
    this.listeners.get(key).add(fn)
    return () => this.listeners.get(key)?.delete(fn)
  }
}

export class App {
  static get store() {
    if (!window.__APP_STORE) window.__APP_STORE = new AppStore()
    return window.__APP_STORE
  }

  // Bind data-* attributes to App.store
  static bind(root, store = App.store) {
    const q = (sel) => Array.from(root.querySelectorAll(sel))

    // text bindings: data-bind-text="key"
    q('[data-bind-text]').forEach(el => {
      const key = el.getAttribute('data-bind-text')
      const apply = (val) => { el.textContent = val ?? '' }
      apply(store.get(key, el.textContent))
      store.subscribe(key, apply)
    })

    // html bindings: data-bind-html
    q('[data-bind-html]').forEach(el => {
      const key = el.getAttribute('data-bind-html')
      const apply = (val) => { el.innerHTML = val ?? '' }
      apply(store.get(key, el.innerHTML))
      store.subscribe(key, apply)
    })

    // value bindings: data-bind-value
    q('[data-bind-value]').forEach(el => {
      const key = el.getAttribute('data-bind-value')
      const apply = (val) => {
        if ('value' in el) el.value = val ?? ''
        else el.setAttribute('value', val ?? '')
      }
      const initial = 'value' in el ? el.value : el.getAttribute('value')
      apply(store.get(key, initial))
      store.subscribe(key, apply)
    })

    // visibility binding: data-bind-eq="key:value"
    q('[data-bind-eq]').forEach(el => {
      const spec = el.getAttribute('data-bind-eq') || ''
      const [key, expected] = spec.split(':')
      if (!key) return
      const apply = (val) => {
        const show = String(val) === String(expected)
        el.classList.toggle('hidden', !show)
      }
      apply(store.get(key))
      store.subscribe(key, apply)
    })
  }

  // Define a light-DOM component: renderFn returns a string
  static define(tagName, renderFn) {
    if (typeof window === 'undefined') return
    if (window.customElements.get(tagName)) return
    window.customElements.define(tagName, class extends HTMLElement {
      constructor() {
        super()
        this._renderFn = renderFn.bind(this)
        this.__delegated = new Map() // type -> [{selector,handler}]
        this.__onInstalled = new Set()
        this.__attrObserver = new MutationObserver(() => this.renderToLight())
      }

      connectedCallback() {
        this.renderToLight()
        this.__attrObserver.observe(this, { attributes: true })
      }

      disconnectedCallback() {
        this.__attrObserver.disconnect()
      }

      on(eventType, selector, handler) {
        if (!this.__delegated.has(eventType)) this.__delegated.set(eventType, [])
        this.__delegated.get(eventType).push({ selector, handler })
        if (!this.__onInstalled.has(eventType)) {
          this.__onInstalled.add(eventType)
          this.addEventListener(eventType, (ev) => {
            const arr = this.__delegated.get(eventType) || []
            const t = ev.target
            for (const { selector, handler } of arr) {
              if (t && t.closest && t.closest(selector)) {
                try { handler.call(this, ev) } catch (e) { console.warn(`[app] delegated ${eventType} handler error:`, e) }
              }
            }
          })
        }
      }

      renderToLight() {
        try {
          const html = this._renderFn({
            attrs: Object.fromEntries(this.getAttributeNames().map(n => [n, this.getAttribute(n)])),
            on: this.on.bind(this),
            store: App.store,
          })
          if (typeof html === 'string') this.innerHTML = html
          else console.warn(`[app] ${tagName} should return a string`)
        } catch (e) { console.error(`[app] render error in ${tagName}:`, e) }
        // bind after render
        App.bind(this)
      }
    })
  }
}

