class CDTabs extends HTMLElement {
    constructor() {
        super();
        this.attachShadow({ mode: 'open' });
    }

    connectedCallback() {
        this.render();
    }

    render() {
        const style = `
      :host { display:block; width:100%; }
      .segmented { display:flex; width:100%; border:1px solid var(--border); border-radius:8px; overflow:hidden; background: var(--surface); }
      .segmented button { flex:1; border:none; padding:6px 10px; background:transparent; color: var(--text-color); cursor:pointer; font-family: var(--font-ui); }
      .segmented button[aria-pressed="true"] { background: var(--background-2); }
      .panel { margin-top:8px; }
    `;
        const tabs = (this.getAttribute('tabs') || 'one,two').split(',');
        const active = this.getAttribute('active') || tabs[0];

        this.shadowRoot.innerHTML = `
      <style>${style}</style>
      <div class="segmented">
        ${tabs.map(t => `<button data-k="${t}" aria-pressed="${t === active}">${t}</button>`).join('')}
      </div>
      <div class="panel"><slot name="${active}"></slot></div>
    `;
        this.shadowRoot.querySelectorAll('button').forEach(btn => {
            btn.addEventListener('click', () => this.switchTo(btn.dataset.k));
        });
    }

    switchTo(key) {
        this.setAttribute('active', key);
        this.render();
    }
}

customElements.define('cd-tabs', CDTabs);


