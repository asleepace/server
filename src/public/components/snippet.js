class CDSnippet extends HTMLElement {
  static get observedAttributes() { return ["lang"]; }

  constructor() {
    super();
    this.attachShadow({ mode: "open" });
    this._observer = null;
  }

  connectedCallback() {
    this.render();
    // Observe light DOM text changes so external code can set textContent
    if (!this._observer) {
      this._observer = new MutationObserver(() => this.render());
      this._observer.observe(this, { characterData: true, childList: true, subtree: true });
    }
  }

  disconnectedCallback() {
    if (this._observer) {
      this._observer.disconnect();
      this._observer = null;
    }
  }

  attributeChangedCallback() { this.render(); }

  get language() { return this.getAttribute("lang") || "text"; }

  render() {
    const code = (this.textContent || "").replace(/\n$/, "");
    const lang = this.language.toUpperCase();
    const style = `
      :host { display:block; margin: 0 0 12px 0; }
      .wrap { border:1px solid var(--code-bord-color); background: var(--code-back-color); color: var(--code-text-color); border-radius:4px; }
      header { display:flex; align-items:center; justify-content:space-between; padding:6px 8px; font-family: Menlo, monospace; font-size:12px; border-bottom:1px solid #333; color:#bbb; }
      button { background: transparent; color:#ccc; border:1px solid #555; border-radius:3px; padding:2px 6px; cursor:pointer; font-family: Menlo, monospace; font-size:11px; }
      button:hover { background:#222; }
      pre { margin:0; padding:10px 12px; overflow:auto; overflow-x:auto; }
      code { white-space: pre; word-break: normal; }
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
    const sanitizeForCopy = (src) => {
      return src
        // Normalize CRLF/CR to LF
        .replace(/\r\n?|\u2028|\u2029/g, "\n")
        // Replace curly quotes with straight
        .replace(/[\u201C\u201D]/g, '"')
        .replace(/[\u2018\u2019]/g, "'")
        // Replace en/em dashes with hyphen
        .replace(/[\u2013\u2014]/g, '-')
        // Convert nbsp to space
        .replace(/\u00A0/g, ' ')
        // Remove zero-width and BOM chars
        .replace(/[\u200B\u200C\u200D\uFEFF]/g, '');
    }
    const copyBtn = this.shadowRoot.getElementById("copy");
    copyBtn.onclick = async () => {
      const clean = sanitizeForCopy(code)
      try {
        await navigator.clipboard.writeText(clean);
      } catch {
        // Fallback copy
        const ta = document.createElement('textarea')
        ta.value = clean
        ta.style.position = 'fixed'
        ta.style.opacity = '0'
        document.body.appendChild(ta)
        ta.select()
        try { document.execCommand('copy') } catch { }
        document.body.removeChild(ta)
      }
      const old = copyBtn.textContent
      copyBtn.textContent = 'COPIED'
      setTimeout(() => (copyBtn.textContent = old), 900)
    };
  }
}

customElements.define("cd-snippet", CDSnippet);


