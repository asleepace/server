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
            }

            onMounted(mountedCallback) {
                if (typeof mountedCallback !== 'function') return
                if (this.__mountedGuard) return
                // Defer until link step binds methods and initial render is complete
                this._mountedCallbacks.push(mountedCallback)
            }

            render() {
                try {
                    const result = this._renderFn({
                        state: this.state,
                        onMounted: this.onMounted.bind(this)
                    })
                    if (typeof result === 'string') return result || ""
                    if (result && typeof result === 'object') {
                        const { html = "", methods = {} } = result
                        this._methods = methods || {}
                        return html
                    }
                    return ""
                } catch (e) {
                    console.error(`Error rendering ${elemName}:`, e)
                    return `<div>Error rendering component</div>`
                }
            }

            connectedCallback() {
                super.connectedCallback()
                // Execute link step, then run onMounted callbacks once per element
                queueMicrotask(() => {
                    // Link step: bind @method handlers declared as inline on*="@method"
                    try {
                        const root = this.shadowRoot
                        if (root) {
                            const all = root.querySelectorAll('*')
                            all.forEach((el) => {
                                Array.from(el.attributes).forEach(attr => {
                                    const name = attr.name.toLowerCase()
                                    const val = attr.value
                                    // Support both on* and data-on* to avoid inline event attribute parsing issues
                                    const isOn = name.startsWith('on')
                                    const isDataOn = name.startsWith('data-on')
                                    if (!isOn && !isDataOn) return
                                    const eventName = isOn ? name.slice(2) : name.slice(8)
                                    if (!val || !val.startsWith('@')) return
                                    const handlerName = val.slice(1)
                                    const fn = this._methods && this._methods[handlerName]
                                    if (typeof fn === 'function') {
                                        el.removeAttribute(attr.name)
                                        el.addEventListener(eventName, fn.bind(this))
                                    } else {
                                        console.warn(`[components] missing handler @${handlerName} for`, name, el)
                                    }
                                })
                            })
                        }
                    } catch (err) {
                        console.warn('[components] link step failed:', err)
                    }
                    if (!this.__mountedGuard) {
                        this.__mountedGuard = true
                        const cbs = this._mountedCallbacks.splice(0)
                        cbs.forEach(callback => {
                            try { callback.call(this, this) } catch (e) { console.error(`Error in onMounted callback for ${elemName}:`, e) }
                        })
                    }
                })
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

