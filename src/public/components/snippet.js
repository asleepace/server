class CDSnippet extends HTMLElement {
    static get observedAttributes() { return ["lang"]; }

    constructor() {
        super();
        this.attachShadow({ mode: "open" });
    }

    connectedCallback() { this.render(); }
    attributeChangedCallback() { this.render(); }

    get language() { return this.getAttribute("lang") || "text"; }

    render() {
        const code = (this.textContent || "").trim();
        const lang = this.language.toUpperCase();
        const style = `
      :host { display:block; margin: 0 0 12px 0; }
      .wrap { border:1px solid var(--code-bord-color); background: var(--code-back-color); color: var(--code-text-color); border-radius:4px; }
      header { display:flex; align-items:center; justify-content:space-between; padding:6px 8px; font-family: Menlo, monospace; font-size:12px; border-bottom:1px solid #333; color:#bbb; }
      button { background: transparent; color:#ccc; border:1px solid #555; border-radius:3px; padding:2px 6px; cursor:pointer; font-family: Menlo, monospace; font-size:11px; }
      button:hover { background:#222; }
      pre { margin:0; padding:10px 12px; overflow:auto; }
      code { white-space: pre-wrap; word-break: break-word; }
    `;
        this.shadowRoot.innerHTML = `
      <style>${style}</style>
      <div class="wrap">
        <header>
          <span>&rArr; ${lang}</span>
          <button id="copy" title="Copy">COPY</button>
        </header>
        <pre><code id="code"></code></pre>
      </div>
    `;
        this.shadowRoot.getElementById("code").textContent = code;
        this.shadowRoot.getElementById("copy").onclick = async () => {
            try {
                await navigator.clipboard.writeText(code);
                const btn = this.shadowRoot.getElementById("copy");
                const old = btn.textContent;
                btn.textContent = "COPIED";
                setTimeout(() => (btn.textContent = old), 900);
            } catch { }
        };
    }
}

customElements.define("cd-snippet", CDSnippet);


