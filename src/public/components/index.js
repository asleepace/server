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
           --color-text: rgba(255, 255, 255, 0.8);
           --color-tint: #7efc7a;
           --color-bg: #15171a;
           --color-surface: #15171a;
           --color-border: #3a3d42;
           --color-shadow: rgba(0,0,0,0.2);
           --text-size: 14px;
           --text-font: monospace;
           --size-navbar: 50px;

           display: flex;
           flex-direction: row;
           width: 100%;
           height: var(--size-navbar);
           background-color: var(--color-bg);
           position: absolute;
           left: 0;
           right: 0;
           top: 0;
           z-index: 50;
        }

        nav { 
            width: 100%; 
            border-bottom: 1px solid var(--color-border); 
            background: var(--color-surface); 
            height: var(--size-navbar); 
            display: flex; 
            align-items: center; 
            justify-content: space-between; 
            padding: 0 12px; 
            box-shadow: 0 2px 6px var(--color-shadow); 
            box-sizing: border-box; 
        }

        .nav-title {
            font-weight: bold;
            color: var(--color-text);
            display: flex;
            align-items: center;
            gap: 4px;
        }

        .nav-bttns {
            display: flex;
            flex-direction: row;
            justify-content: center;
            align-items: center;
        }

        button {
            border: none; 
            border-right: 1px solid var(--color-border); 
            background: transparent; 
            padding: 6px 10px; 
            color: var(--color-tint); 
            font-family: var(--text-font); 
            text-decoration: none; 
            cursor: pointer;
        }

        button:hover {
            background: rgba(126, 252, 122, 0.1);
        }

        button:last-child {
            border-right: none;
        }

        #cursor {
            color: var(--color-tint);
            opacity: 1;
            transition: opacity 0.1s;
        }

        .flex {
           display: flex;
           flex: 1;
        }
        
        .flex-col {
            display: flex;
            flex-direction: column;
        }

        .flex-row {
            display: flex;
            flex-direction: row;
        }

        .w-full { width: 100%; }
        .h-full { height: 100%; }
        .min-w-full { min-width: 100%; }
        .min-h-full { min-height: 100%; }

        .px-0 {
            padding-left: 0px;
            padding-right: 0px;
        }
        .px-1 {
            padding-left: 4px;
            padding-right: 4px;
        }
        .px-2 {
            padding-left: 8px;
            padding-right: 8px;
        }

        .py-0 {
            padding-top: 0px;
            padding-bottom: 0px;
        }
        .py-1 {
            padding-top: 4px;
            padding-bottom: 4px;
        }
        .py-2 {
            padding-top: 8px;
            padding-bottom: 8px;
        }

        .p-0 {
            padding: 0px;
        }

        .p-1 {
            padding: 4px;
        }

        .p-2 {
            padding: 8px;
        }

        .m-0 {
            margin: 0px;
        }
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
        console.log('@component base constructor!')
        this.attachShadow({ mode: 'open' })
        // Insert shared styles on construction; subsequent renders must preserve them
        this.shadowRoot.appendChild(sharedStyles.cloneNode(true))
        this.isMounted = false
    }

    onMounted(callbackFn) {
        console.log('@component mounted (base)', this)
        if (this.isMounted) return
        this.isMounted = true
        callbackFn.call(this)
    }

    render() {
        console.log('@component base render')
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
        // Call onMounted after rendering is complete
        queueMicrotask(() => this.onMounted(() => { }))
    }

    attributeChangedCallback() {
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
            }

            onMounted(mountedCallback) {
                if (this.isMounted) {
                    // Already mounted, call immediately
                    mountedCallback.call(this, this)
                } else {
                    // Store callback for later
                    this._mountedCallbacks.push(mountedCallback)
                }
            }

            render() {
                try {
                    const result = this._renderFn({
                        state: this.state,
                        onMounted: this.onMounted.bind(this)
                    })
                    return result || ""
                } catch (e) {
                    console.error(`Error rendering ${elemName}:`, e)
                    return `<div>Error rendering component</div>`
                }
            }

            connectedCallback() {
                super.connectedCallback()

                // Execute all stored mounted callbacks
                setTimeout(() => {
                    if (!this.isMounted) {
                        this.isMounted = true
                        this._mountedCallbacks.forEach(callback => {
                            try {
                                callback.call(this, this)
                            } catch (e) {
                                console.error(`Error in onMounted callback for ${elemName}:`, e)
                            }
                        })
                        this._mountedCallbacks = []
                    }
                }, 0)
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
