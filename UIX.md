# UI/UX Spec

Vision: Minimal, retro-modern console UI with a right-side inspector. Hacker vibe via monospace, clean borders, subtle shadows, tasteful ASCII.

## Layout

- Global top bar: `<cd-nav>` sticky at top; width equals message pane width.
- Two-pane app layout:
  - Left pane: message stream + command line input at bottom
  - Right pane: togglable inspector with tabs (instructions, settings, hooks)
- Responsive: left pane flexes; right pane has explicit width with min/max and can be toggled/drag-resized.

## Behavior

- SSE live stream appends lines to message pane with soft cap to prevent DOM bloat.
- Hot-reload support via `event: hot-reload`.
- Client commands via `event: client-cmd` (and via POST `/__client`).
- Command line DSL (client-side):
  - `:clear` — clears messages
  - `:theme <light|dark>` — toggles theme
  - `:toggle` — toggles sidebar
  - `:reload` — triggers hot reload (POST `/__reload`)
  - any other input → POST `/__client` with the text payload

## Style

- Monospace for stream/CLI; system UI for nav.
- Tokens from `index.css` (light/dark); subtle borders; rounded panel corners.
- Panels: event stream and sidebar as distinct surfaces.

## Accessibility

- Stream container: `role="log"` and `aria-live="polite"`.
- Buttons include aria-labels; cursor blink aria-hidden.

## TODO

- [x] Nav + content width syncing
- [x] Robust sidebar resizing and persistence
- [x] Command line input at bottom of left pane
- [x] Client command handling for DSL
- [x] Clean styles and responsive flex layout
- [ ] Future: virtualized stream rendering for huge logs
- [ ] Future: keyboard support for tabs (Left/Right/Home/End)
