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
    '❘': '&#10072',
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
           --color-border: #3a3d42;
           --text-size: 14px;
           --text-font: monospace;
        }

        nav { 
            width:100%; 
            border-bottom:1px solid var(--border); 
            background: var(--surface); 
            height: var(--nav-height); 
            display:flex; 
            align-items:center; 
            justify-content:space-between; 
            padding:0 12px; 
            box-shadow: 0 2px 6px var(--shadow); 
            box-sizing:border-box; 
        }

        .nav-title {
            font-weight: bold;
        }

        .nav-bttns {
            display: flex;
            flex-direction: row;
            justify-items: center;
            align-items-center;
        }

        button {
            border:none; 
            border-right:1px solid var(--border); 
            background:transparent; 
            padding:6px 10px; color: 
            var(--green-accent); 
            font-family: var(--font-mono); 
            text-decoration:none; 
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
    constructor() {
        super()
        this.shadowRoot({ mode: 'open' })
        this.shadowRoot.appendChild(sharedStyles)
    }
    render() {
        return `<slot />`
    }
    connectedCallback() {
        this.render()
    }
    attributeChangedCallback() {
        this.render()
    }
    get state() {
        return this.getAttributeNames().reduce((state, attrName) => {
            state[attrName] = this.getAttribute(attrName)
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
            return renderFn.call(this, this.state)
        }
    }
    console.log(`[@components] registering: "${name}"`)
    window.customElements.define(name, Elem)
}

/**
 * Simple helper for querying elements on the dom, the callback will only trigger
 * if the element is found. Will return the result of callbackFn(element).
 * @param {*} elem 
 * @param {*} callbackFn 
 * @returns 
 */
function $select(selector, callbackFn) {
    const result = document.querySelector(selector)
    if (!result) return console.warn(`[$select] failed to find: "${elem}"`)
    return callbackFn(elem)
}

function $onclick(selector, onClickFn) {
    $select(selector, item => {
        item.onClickFn = onClickFn
    })
}

function $try(fn) {
    try { return [fn(), null] }
    catch (e) { console.warn(e); return [null, e instanceof Error ? e : new Error(String(e))] }
}

function $event(name, detail) {
    return new CustomEvent(name, { bubbles: true, composed: true, detail })
}

const $storage = {
    set(sotrageKey, value) {
        return $try(() => localStorage.setItem(sotrageKey, String(value)))[1]
    },
    get(storageKey, fallbackValue) {
        return $try(() => localStorage.getItem(storageKey))[0] ?? fallbackValue
    }
}

$define('cd-navbar', (state) => {
    console.log('[@cd-navbar] rendering:', state)

    $onclick('#theme', () => {
        const root = document.documentElement;
        const dark = root.getAttribute('data-theme') === 'dark';
        root.setAttribute('data-theme', dark ? 'light' : 'dark');
        try { localStorage.setItem('theme', dark ? 'light' : 'dark'); } catch { }
    })

    $onclick('#sidebar', () => {
        this.dispatchEvent($event('cd:toggle-sidebar'))
    })

    $select('#cursor', (cursor) => {
        return setInterval(() => {
            cursor.style.opacity = cursor.style.opacity === '0' ? '1' : '0';
        }, 900);
    })

    // init with theme from storage

    const defualtTheme = $storage.get('theme') ?? 'dark'
    document.documentElement.setAttribute('data-theme', defualtTheme)

    return `
  <nav>
    <div class="nav-title">
      <
    </div>
    <div class="nav-bttns">
        <button>Home</button>
        <button>New Session</button>
        <button>Settings</button>
        <button>Toggle</button>
    </div>
  </nav>
`})