/**
 * ASCII Symbol to HTML code map.
 * 
 * @see https://www.toptal.com/designers/htmlarrows/symbols/
 */
const AsciiSymbols = {
    '⚙': '&#9881;',
    '⚠': '&#9888;',
    '✖': '&#10006;',
    '✕': '&#10005;',
    '✔': '&#10004;',
    '✓': '&#10003;',
    '✗': '&#10007;',
    '✘': '&#10008;',
    '❘': '&#10072;',
    '❙': '&#10073;',
    '❚': '&#10074;',
    '➩': '&#10153;',
    '❴': '&#10100;',
    '❵': '&#10101;',
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
class BaseElement extends HTMLElement {
    isMounted = false
    constructor() {
        super()
        this.attachShadow({ mode: 'open' })
        this.shadowRoot.appendChild(sharedStyles.cloneNode(true))
    }
    render() {
        return `<slot />`
    }
    connectedCallback() {
        if (!this.isConnected) return
        this.shadowRoot.innerHTML = this.render()
    }
    attributeChangedCallback() {
        this.shadowRoot.innerHTML = this.render()
    }

    onMounted(callbackFn) {
        if (this.isMounted) return
        callbackFn.call(this)
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
    }
}

/**
 * Register custom elements on the window the specified name and render function,
 * these inherit shared styles, render logic, etc.
 * @param {string} name - name of custom element (e.g. "app-navbar" => <app-navbar />)
 * @param {function} renderFn - function which is called with current state and returns innerHTML
 * @returns 
 */
function $define(name, renderFn) {
    if (typeof window === 'undefined') return
    if (window.customElements.get(name)) return

    const Elem = class extends BaseElement {
        constructor() {
            super()
        }
        render() {
            console.log(`[@component:${name}] rendering:`, this.state)
            const html = renderFn.call(this, this.state)

            // Set up event listeners after rendering
            setTimeout(() => {
                this.setupEventListeners()
            }, 0)

            return html
        }

        setupEventListeners() {
            // Handle navbar-specific event listeners
            if (name === 'cd-navbar') {
                this.setupNavbarListeners()
            }
        }

        setupNavbarListeners() {
            const themeBtn = this.shadowRoot.querySelector('#theme')
            const sidebarBtn = this.shadowRoot.querySelector('#sidebar')
            const cursor = this.shadowRoot.querySelector('#cursor')

            if (themeBtn) {
                themeBtn.onclick = () => {
                    const root = document.documentElement;
                    const dark = root.getAttribute('data-theme') === 'dark';
                    const newTheme = dark ? 'light' : 'dark'
                    root.setAttribute('data-theme', newTheme);
                    $storage.set('theme', newTheme)
                }
            }

            if (sidebarBtn) {
                sidebarBtn.onclick = () => {
                    this.dispatchEvent($event('cd:toggle-sidebar'))
                }
            }

            if (cursor) {
                setInterval(() => {
                    cursor.style.opacity = cursor.style.opacity === '0' ? '1' : '0';
                }, 900);
            }
        }
    }

    console.log(`[@components] registering: "${name}"`)
    window.customElements.define(name, Elem)
}

/**
 * Simple helper for querying elements on the dom, the callback will only trigger
 * if the element is found. Will return the result of callbackFn(element).
 * @param {string} selector 
 * @param {function} callbackFn 
 * @returns 
 */
function $select(selector, callbackFn) {
    const result = document.querySelector(selector)
    if (!result) {
        console.warn(`[$select] failed to find: "${selector}"`)
        return null
    }
    return callbackFn(result)
}

function $onclick(selector, callbackFn) {
    $select(selector, item => {
        item.onclick = callbackFn
    })
}

function $try(fn) {
    try {
        return [fn(), null]
    }
    catch (e) {
        console.warn(e);
        return [null, e instanceof Error ? e : new Error(String(e))]
    }
}

function $event(name, detail) {
    return new CustomEvent(name, { bubbles: true, composed: true, detail })
}

const $storage = {
    set(storageKey, value) {
        return $try(() => localStorage.setItem(storageKey, String(value)))[1]
    },
    get(storageKey, fallbackValue) {
        return $try(() => localStorage.getItem(storageKey))[0] ?? fallbackValue
    }
}

// Initialize theme from storage on page load
const defaultTheme = $storage.get('theme') ?? 'dark'
document.documentElement.setAttribute('data-theme', defaultTheme)

$define('cd-nav', function (state) {
    console.log('[@cd-navbar] rendering:', state)

    return `
        <nav>
            <div class="nav-title">
                <div>ConsoleDump</div>
                <div id="cursor"></div>
            </div>
            <div class="nav-bttns">
                <button>Home</button>
                <button>New Session</button>
                <button>Settings</button>
                <button id="theme">Theme</button>
                <button id="sidebar">Sidebar</button>
            </div>
        </nav>
    `
})