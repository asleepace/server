
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
    styles.innerText = `
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
    static name = 'cd-elem'
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



function defineCustomElement(config = {
    name: 'cd-custom-element',
    styles: undefined,
    render(state) {
        return `<slot />`
    }
}) {
    const ElemClass = class extends BaseElement {
        static name = config.name
        constructor() {
            super()
            if (config.styles) {
                const styles = document.createElement('style')
                styles.textContent = config.styles
                this.appendChild(styles)
            }
        }
        render() {
            this.innerHTML = config.render(this.state)
        }
    }

    if (!window.customElements.get(config.name)) {
        console.log('[components] registering:', config.name)
        window.customElements.define(config.name, ElemClass)
    }
    return ElemClass
}


defineCustomElement({
    name: 'approw',
    render() {
        return `<div class="flex-row"><slot /></div>`
    }
})

defineCustomElement({
    name: 'appcol',
    render() {
        return `<div class="flex-col"><slot /></div>`
    }
})


defineCustomElement({
    name: 'AppNavigation',
    state: `
        row button {
           padding-horizontal: 8px;
        }
    `,
    render(state) {
        return `
            <nav>
              <row><slot /></row>
              <row>
                <button>Home</button>
                <button>New Session</button>
                <button>Settings</button>
                <button>Toggle</button>
              </row>
            </nav>
        `
    }
})


/**
 * ## CDNavButton
 * Custom navigation buttons web component.
 */
class CDNavButton extends BaseElement {
    static name = 'cd-button'
    constructor() {
        super()
    }
    render() {
        this.innerHTML = `<button><slot /></button>`
    }
}


class CDNav extends BaseElement {
    constructor() {
        super();
        this.attachShadow({ mode: "open" });
    }

    connectedCallback() { this.render(); }

    render() {
        const title = this.getAttribute("title") || "ConsoleDump";
        const showSidebarToggle = this.hasAttribute('sidebar-toggle');
        const style = `
      :host { display:block; position:sticky; top:0; z-index:1000; width:100vw; }
      .left { display:flex; align-items:center; gap:8px; }
      .brand { font-family: var(--font-ui); font-weight:800; color: var(--text-color); text-decoration:none; }
      .right { display:flex; align-items:center; gap:6px; }
      .group { display:inline-flex; border:1px solid var(--border); border-radius:8px; overflow:hidden; background: var(--surface); }
      .group > * { border:none; border-right:1px solid var(--border); background:transparent; padding:6px 10px; color: var(--green-accent); font-family: var(--font-mono); text-decoration:none; }
      .group > *:last-child { border-right:none; }
      .group > *:hover { background: var(--background-2); }
      .ascii { font-family: var(--font-mono); color: var(--tint-color); }
      .navbtns > * {
         padding-horizontal: 8px;
      }
    `;
        this.shadowRoot.innerHTML = `
      <style>${style}</style>
      <appnaviation />
    `;
        const btn = this.shadowRoot.getElementById('theme');
        btn.onclick = () => {
            const root = document.documentElement;
            const dark = root.getAttribute('data-theme') === 'dark';
            root.setAttribute('data-theme', dark ? 'light' : 'dark');
            try { localStorage.setItem('theme', dark ? 'light' : 'dark'); } catch { }
        };
        const sb = this.shadowRoot.getElementById('sidebar');
        if (sb) {
            sb.onclick = () => this.dispatchEvent(new CustomEvent('cd:toggle-sidebar', { bubbles: true, composed: true }));
        }
        // blink cursor near brand
        const cursor = this.shadowRoot.getElementById('cursor');
        if (cursor) {
            setInterval(() => {
                cursor.style.opacity = cursor.style.opacity === '0' ? '1' : '0';
            }, 900);
        }
        // init from storage
        try {
            let saved = localStorage.getItem('theme');
            if (!saved) { saved = 'dark'; localStorage.setItem('theme', saved); }
            document.documentElement.setAttribute('data-theme', saved);
        } catch { }
    }
}

customElements.define('cd-nav', CDNav);


