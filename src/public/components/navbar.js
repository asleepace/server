class CDNav extends HTMLElement {
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
      nav { width:100%; border-bottom:1px solid var(--border); background: var(--surface); height: var(--nav-height); display:flex; align-items:center; justify-content:space-between; padding:0 12px; box-shadow: 0 2px 6px var(--shadow); box-sizing:border-box; }
      .left { display:flex; align-items:center; gap:8px; }
      .brand { font-family: var(--font-ui); font-weight:800; color: var(--text-color); text-decoration:none; }
      .right { display:flex; align-items:center; gap:6px; }
      .group { display:inline-flex; border:1px solid var(--border); border-radius:8px; overflow:hidden; background: var(--surface); }
      .group > * { border:none; border-right:1px solid var(--border); background:transparent; padding:6px 10px; color: var(--green-accent); font-family: var(--font-mono); text-decoration:none; }
      .group > *:last-child { border-right:none; }
      .group > *:hover { background: var(--background-2); }
      .ascii { font-family: var(--font-mono); color: var(--tint-color); }
    `;
        this.shadowRoot.innerHTML = `
      <style>${style}</style>
      <nav>
        <div class="left">
          <span class="ascii" style="color: --tint-color;">::</span>
          <a class="brand" href="/">${title}</a>
        </div>
        <div class="right">
          <div class="group">
            <a href="/info">info</a>
            <a href="/log">logs</a>
            ${showSidebarToggle ? '<button id="sidebar" title="toggle sidebar">sidebar</button>' : ''}
            <button id="theme" title="toggle theme">theme</button>
          </div>
        </div>
      </nav>
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
        // init from storage
        try {
            let saved = localStorage.getItem('theme');
            if (!saved) { saved = 'dark'; localStorage.setItem('theme', saved); }
            document.documentElement.setAttribute('data-theme', saved);
        } catch { }
    }
}

customElements.define('cd-nav', CDNav);


