
export default function CdTxt({ attrs }) {
  return `
    <style>
      :host { display: block; }
      :host([inline]) { display: inline; }

      /* Base */
      ::slotted(*) {
        margin: 0;
        color: var(--text-color);
        font-family: var(--font-ui);
        line-height: 1.5;
      }

      /* Variants */
      :host([variant="h1"]) ::slotted(*) { font-family: var(--font-headings); font-size: 2.2rem; font-weight: 800; line-height: 1.2; }
      :host([variant="h2"]) ::slotted(*) { font-family: var(--font-headings); font-size: 1.8rem; font-weight: 700; line-height: 1.25; }
      :host([variant="h3"]) ::slotted(*) { font-family: var(--font-headings); font-size: 1.3rem; font-weight: 700; }
      :host([variant="h4"]) ::slotted(*) { font-family: var(--font-headings); font-size: 1.0rem; font-weight: 600; }
      :host([variant="p"])  ::slotted(*) { font-size: 0.90rem; }
      :host([variant="small"]) ::slotted(*) { font-size: 0.85rem; }

      /* Size overrides */
      :host([size="xs"]) ::slotted(*) { font-size: 8px; }
      :host([size="sm"]) ::slotted(*) { font-size: 12px; }
      :host([size="md"]) ::slotted(*) { font-size: 16px; }
      :host([size="lg"]) ::slotted(*) { font-size: 18px; }
      :host([size="xl"]) ::slotted(*) { font-size: 22px; }

      /* Weight */
      :host([weight="400"]) ::slotted(*) { font-weight: 400; }
      :host([weight="500"]) ::slotted(*) { font-weight: 500; }
      :host([weight="600"]) ::slotted(*) { font-weight: 600; }
      :host([weight="700"]) ::slotted(*) { font-weight: 700; }
      :host([weight="800"]) ::slotted(*) { font-weight: 800; }

      /* Color */
      :host([muted]) ::slotted(*) { color: var(--muted-text); }
      :host([tint])  ::slotted(*) { color: var(--tint-color); }
      :host([green]) ::slotted(*) { color: var(--green-accent); }

      /* Mono / Code */
      :host([mono]) ::slotted(*) { font-family: var(--font-mono); }
      :host([code]) ::slotted(*) { font-family: var(--font-mono); background: var(--code-back-color); color: var(--code-text-color); padding: 0 0.25rem; border-radius: 4px; }

      /* Alignment */
      :host([align="center"]) ::slotted(*) { text-align: center; }
      :host([align="right"])  ::slotted(*) { text-align: right; }
      :host([align="left"])   ::slotted(*) { text-align: left; }

      /* Transform helpers */
      :host([uppercase]) ::slotted(*) { text-transform: uppercase; }
      :host([lowercase]) ::slotted(*) { text-transform: lowercase; }
      :host([capitalize]) ::slotted(*) { text-transform: capitalize; }

      /* Truncate */
      :host([truncate]) ::slotted(*) { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

      /* Utility classes forwarded to slotted nodes */
      ::slotted(.muted) { color: var(--muted-text); }
      ::slotted(.mono)  { font-family: var(--font-mono); }
      ::slotted(.bold)  { font-weight: 700; }
      ::slotted(.semibold) { font-weight: 600; }
      ::slotted(.small) { font-size: 0.85rem; }
      ::slotted(.kbd)   { font-family: var(--font-mono); background: var(--surface); border: 1px solid var(--border); padding: 0 0.35rem; border-radius: 4px; }
    </style>
    ${attrs?.text ? `<span>${attrs.text}</span>` : `<span><slot></slot></span>`}
  `
}