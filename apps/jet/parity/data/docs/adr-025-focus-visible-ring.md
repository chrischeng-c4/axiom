# ADR-025: `:focus-visible` ring paint driven by proxy element state

| Field | Value |
|-------|-------|
| Issue | #2156 |
| Parent epic | #2135 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Drive canvas-side focus-ring paint from the hidden proxy element's `:focus-visible` state. After every `focus` event on the proxy, jet runs `proxyEl.matches(':focus-visible')` and stores the resulting boolean on the canvas widget's render state. The next paint includes or omits the ring accordingly. Outline color/width/style/offset are read from the proxy's computed style. The keyboard-vs-mouse modality heuristic is delegated entirely to the user agent. |

## Context

`:focus-visible` is the CSS pseudo-class that distinguishes "focused
because the user is navigating with the keyboard or assistive tech"
from "focused because the user clicked with a pointing device." It is
the load-bearing primitive behind MUI's `Mui-focusVisible` rule, behind
WAI-ARIA Authoring Practices' focus-ring guidance, behind Chromium's
own form-control focus ring, and behind every contemporary design
system's accessibility story. Get it wrong in either direction and the
product is broken:

- **False negative** — user tabs into a button, no ring appears: the
  app is unusable by keyboard-only users, a WCAG 2.1 SC 2.4.7 (Focus
  Visible) failure, and an immediate blocker for any government /
  enterprise procurement that audits for accessibility.
- **False positive** — user clicks a button with a mouse and a ring
  lingers: visual noise, inconsistent with the rest of the page (where
  DOM buttons stay clean on mouse click), and surfaces as a hundred
  pixel-diff regressions against MUI's reference renderer.

The heuristic is intentionally fuzzy at the spec level: the WHATWG
HTML standard defines `:focus-visible` in terms of an internal "focus
visible state" flag that user agents set or clear based on the input
modality of the most recent user interaction, with per-element
overrides (text inputs and listboxes show the ring even on mouse
click, because their interaction model is keyboard-centric once
focused). Each engine implements the flag slightly differently for
edge cases — Chrome and Firefox disagree about whether a programmatic
`.focus()` call after a Tab keypress but before the next pointer
event should still match, for example — and the WPT suite contains
several known-fail tests that document these divergences.

Jet's renderer paints on a canvas. The browser has no opinion about
what the canvas pixels show; `:focus-visible` is a CSS pseudo-class
that styles DOM elements, not GPU surfaces. But every focusable jet
widget is shadowed by a `<jet-focus-proxy>` (ADR-004, #2152) — a real
DOM element parked inside `<jet-semantics>` whose entire job is to be
the substrate for browser semantics that don't otherwise reach the
canvas. The proxy is what gets DOM focus on Tab traversal (ADR-003,
#2153). The proxy is what holds the focus trap when one is active
(ADR-007, #2154). The proxy is what receives programmatic
`.focus(opts)` calls from React refs (ADR-020, #2157). And — the
subject of this ADR — the proxy is what the browser's
`:focus-visible` heuristic operates on.

The mechanical question is: how does the canvas-side renderer know
whether the proxy currently matches `:focus-visible`? Three rejected
approaches frame the chosen one:

1. **Reimplement the heuristic in jet.** Wrong on every axis.
   The heuristic is a moving target across browser versions, has
   per-element overrides that the spec hand-waves away, and any
   discrepancy with the user agent's actual decision produces an
   accessibility bug that QA will catch but root-cause attribution
   will take days. We would also have to ship and maintain a
   modality detector that is strictly worse than the browser's
   built-in one.
2. **Listen for `focus` events and infer keyboard vs mouse.** Same
   reimplementation problem, slightly less obvious failure mode. The
   browser does not expose the modality flag on the `FocusEvent`
   object; we would have to track `pointerdown` / `keydown` ordering
   ourselves, and we would still get per-element overrides wrong.
3. **Render a real DOM focus ring on the proxy and screenshot it
   into the canvas.** This was prototyped and rejected at the
   ADR-004 design stage: the proxy is `position: absolute;
   pointer-events: none; opacity: 0` precisely so the user does not
   see two focus rings (the proxy's and the canvas's). Making the
   proxy paint its own ring resurrects the double-ring problem the
   shadow subtree was designed to eliminate.

The chosen approach — query the proxy via `matches(':focus-visible')`
on every focus state transition and let the user agent be the oracle
— inverts the failure mode. By construction, jet's ring decision is
identical to whatever the user's browser would have decided for a DOM
button in the same position with the same interaction history. Bugs
in `:focus-visible` are bugs in the browser, and we get those fixes
for free when users update Chrome or Firefox; conformance is by
delegation rather than by reimplementation.

## Decision

Bind canvas-side focus-ring paint to `proxyEl.matches(':focus-visible')`,
sampled at every proxy focus state transition. Read the ring's visual
parameters (color, width, style, offset) from the proxy's computed
style so designers can restyle the ring via standard CSS.

### Signal pipeline

```ts
// Inside the canvas focus controller (one per jet root).
function onProxyFocus(ev: FocusEvent) {
  const proxy = ev.target as HTMLElement;
  const jetNodeId = proxy.dataset.jetNodeId!;
  const visible = proxy.matches(':focus-visible');
  renderState.focusVisible.set(jetNodeId, visible);
  scheduleRepaint(jetNodeId);
}

function onProxyBlur(ev: FocusEvent) {
  const proxy = ev.target as HTMLElement;
  const jetNodeId = proxy.dataset.jetNodeId!;
  renderState.focusVisible.set(jetNodeId, false);
  scheduleRepaint(jetNodeId);
}

proxyRoot.addEventListener('focus', onProxyFocus, { capture: true });
proxyRoot.addEventListener('blur',  onProxyBlur,  { capture: true });
```

Capture-phase listeners on the proxy root catch every proxy
focus/blur in a single delegated handler. `matches(':focus-visible')`
is a synchronous DOM query that returns whatever the user agent's
internal focus-visible flag currently says — exactly the same boolean
that would have driven a CSS `:focus-visible { … }` rule on the
proxy.

### Computed-style read for ring parameters

```ts
function readRingStyle(proxy: HTMLElement): RingStyle {
  const cs = getComputedStyle(proxy);
  return {
    color:  cs.outlineColor,
    width:  parseFloat(cs.outlineWidth),
    style:  cs.outlineStyle,
    offset: parseFloat(cs.outlineOffset),
  };
}
```

The proxy carries the `<jet-focus-proxy>` class; user CSS can target
`<jet-semantics> jet-focus-proxy:focus-visible` and override `outline`
properties exactly as on a native `<button>`. Jet reads them via
`getComputedStyle` after focus and applies them to the canvas paint.
Designers therefore restyle the focus ring without learning a jet-
specific API.

### Render-state schema

```ts
interface FocusVisibleRenderState {
  // jetNodeId -> currently-focus-visible flag
  focusVisible: Map<string, boolean>;
  // jetNodeId -> ring style at last focus transition
  ringStyle:    Map<string, RingStyle>;
}
```

`focusVisible` is consulted by the canvas paint loop; when truthy for
a node, the renderer draws an outline with the corresponding
`ringStyle` parameters. When falsy or absent, no ring is drawn.

### Heuristic edge cases — all deferred to the user agent

| Trigger | UA decides | Ring paint |
|---|---|---|
| `Tab` / `Shift+Tab` lands on proxy | matches | ON |
| Arrow-key navigation reaches proxy | matches | ON |
| Mouse click on canvas widget (proxy refocused via `pointerdown` synth) | does not match | OFF |
| Mouse click on a text-entry widget (jet `<TextField>`) | matches (per-element override) | ON |
| Mouse click on a listbox option | matches (per-element override) | ON |
| `proxy.focus()` after preceding keydown | matches | ON |
| `proxy.focus()` after preceding pointerdown | does not match | OFF |
| Assistive tech focus (`accessibilitySetFocus`) | matches | ON |
| Window regains focus on previously-focused proxy | UA-dependent (sticky) | follows UA |

Every row above is decided by the user agent, observed by jet via a
single `matches(':focus-visible')` call. The "per-element override"
rows work because the proxy for a text-entry widget is an
`<input type="text">`, and the proxy for a listbox option is a
`<div role="option" tabindex="0">` parented by a `<div role="listbox"
tabindex="0">`; the browser applies the same per-element rules it
would apply to any DOM form control.

### Mouse-click focus synthesis (interaction with ADR-004)

ADR-004 specifies that canvas-side `pointerdown` on a focusable widget
calls `proxy.focus({ preventScroll: true })` *inside* the
`pointerdown` handler, so the browser observes the focus transfer as
mouse-originated. This ADR depends on that contract: a synthesized
focus made inside a non-`pointerdown` callback would be misclassified
as programmatic-from-unknown-modality, and the UA's `:focus-visible`
flag would (correctly) match — producing a ring on mouse click, which
is wrong. ADR-004's "synthesize focus inside the same event tick as
the user gesture" rule is therefore load-bearing for this ADR.

### Debug overlay

Gated behind `?jet-debug=focus` (parsed at module init), jet draws a
1px semi-transparent outline around every proxy currently matching
`:focus-visible`. Implementation:

```css
[data-jet-debug-focus="1"] jet-focus-proxy:focus-visible {
  outline: 1px solid rgba(255, 0, 255, 0.5) !important;
  outline-offset: 0;
}
```

The overlay does not affect the canvas paint; it makes the proxy ring
visible in the DOM so parity work can compare "what the browser
thinks" against "what jet paints" in the same screenshot.

### Forced-override hook (dev-only)

```ts
// Exported only when NODE_ENV !== 'production'.
declare global {
  interface Window {
    __jetSetFocusVisible(
      jetNodeId: string,
      value: boolean | null,
    ): void;
  }
}
```

`__jetSetFocusVisible(id, true)` forces the canvas paint into ring-on
for that node regardless of proxy state. `false` forces ring-off.
`null` releases the override and restores the proxy-derived value.
The hook exists so the canvas renderer's own unit tests can paint
the ring without arranging a real focus event, and so screenshot
fixtures for the canvas team are decoupled from the focus pipeline.

### Test bar

The parity test for this ADR is a WPT-style fixture: a page with a
single `<button>` and a single jet `<Button>` side by side, both
restyled with the same `outline: 2px solid royalblue; outline-offset:
2px;`. The fixture cycles four interaction pathways:

1. Tab into the DOM button, Tab again into the jet button.
2. Click the DOM button, click the jet button.
3. Tab into both, then `document.activeElement.blur()` both.
4. Programmatically focus each via `.focus()` from a `setTimeout(0)`
   handler scheduled inside a `keydown` listener.

After each step the fixture captures a screenshot of the jet button
and asserts pixel equality with the DOM button's `:focus-visible`
state (true → ring present, false → ring absent), within the
ADR-011 (#2147) tier-A tolerance (≤2 pixels off, channel-wise
ΔE ≤ 1). Cross-browser variance is absorbed because both the DOM
button and the jet button observe the same UA's heuristic.

## Consequences

### Positive

- **Conformance by delegation.** Jet matches `:focus-visible` exactly
  because it asks the user agent the same question the user agent
  asks itself. No reimplementation surface, no drift across browser
  versions.
- **Per-element overrides come for free.** Text inputs, listbox
  options, and any other elements the UA special-cases inherit their
  treatment without jet shipping a per-widget table.
- **Cross-browser nuances inherited by construction.** Chrome and
  Firefox's known WPT divergences manifest identically in jet's
  paint, so the parity test bar passes on both engines without
  per-engine branches.
- **Standard CSS for restyling.** Designers target
  `<jet-semantics> jet-focus-proxy:focus-visible { outline: … }` and
  jet picks up the new look on next focus. No jet-specific theming
  API for this corner.
- **Single shared paint path.** Whether the focus came from Tab,
  click, programmatic call, or assistive tech, the canvas paint
  decision is one boolean lookup; no per-pathway branches in the
  renderer.

### Negative

- **`matches(':focus-visible')` cost per focus event.** Each call is
  a synchronous DOM style query. At human focus-change rates (≤ a
  few per second) this is negligible; for synthetic stress tests
  that fire thousands of focus events per second the call shows up
  in profiles. Mitigation: the query is only inside the focus/blur
  handlers, not the paint loop, so steady-state cost is zero.
- **Couples canvas paint to DOM CSS resolution.** A jet root that
  loses its `<jet-semantics>` subtree (e.g. mid-test teardown) loses
  the ability to query `:focus-visible`. The renderer treats a
  missing proxy as `focusVisible: false`, which matches the user
  observation (no focused element → no ring) but masks an internal
  invariant violation. Logged at warn level when caught.
- **Programmatic-focus modality is browser-policy-dependent.** Two
  different browsers may disagree about whether `.focus()` after an
  arbitrary key event matches `:focus-visible`. Jet inherits that
  disagreement; it surfaces as cross-browser pixel diffs in screenshot
  baselines but is correct in each cell of the parity matrix.

### Neutral

- The `__jetSetFocusVisible` dev hook lives in shipped code, gated
  by `NODE_ENV`. Dead-code-elimination in production builds removes
  it.

## Alternatives considered

- **In-jet modality detector.** Track `keydown` / `pointerdown`
  ordering in a global listener; classify focus events accordingly.
  Rejected: reimplements an underspecified browser internal, can never
  match per-element overrides, ships divergence-by-construction.
- **Render a faded ring on the proxy itself and screenshot it into the
  canvas.** Rejected at ADR-004 stage; resurrects the double-ring
  visual bug, costs a per-frame screenshot pipeline.
- **Skip `:focus-visible` and always paint the ring on `:focus`.**
  Rejected: mouse-click on a jet button would always leave a ring,
  diverging from every MUI baseline screenshot and failing the basic
  visual parity gate.
- **Use the user agent's CSS `:focus-visible { … }` rule on the proxy
  and read the computed `outline-width` (non-zero ⇒ visible).**
  Considered. Works for the boolean signal but ties the visibility
  query to the *current* ring style — if the user sets `outline: 0`
  on the proxy for whatever reason, the signal would always read
  false. `matches(':focus-visible')` separates "is the state on?"
  from "what style would it paint?", which is the right factoring.

## Open questions

- **Forced-colors-mode adaptation.** Windows High Contrast and similar
  modes change the focus indicator policy at the OS level. Out of
  scope for this ADR; tracked separately under the theming workstream.
- **`:focus-within` ancestor styling.** Containers that want to style
  themselves while a descendant has visible focus rely on
  `:focus-within`, which is independent of `:focus-visible`. The
  proxy lives inside the document tree, so `:focus-within` on
  ancestors of `<jet-semantics>` already works for DOM consumers.
  Whether canvas-side container widgets need an analogous signal is
  deferred.
- **Sticky `:focus-visible` after window blur/focus.** Both Chrome
  and Firefox have nuanced behaviour when the page loses and regains
  focus while a `:focus-visible` element is focused. Jet's
  delegation-based approach inherits whatever the UA does; a parity
  test fixture for this case is filed as a follow-up.

## References

- Issue #2156 (this ADR)
- Issue #2135 — parent epic, jet ⇄ MUI parity
- ADR-004 / #2152 — `<jet-focus-proxy>` shadow subtree (substrate)
- ADR-003 / #2153 — Tab traversal through proxies
- ADR-007 / #2154 — focus trap scope
- ADR-020 / #2157 — `element.focus(options)` bridging
- ADR-011 / #2147 — pixel tolerance ladder (tier-A bound)
- WHATWG HTML — `:focus-visible` and the focus visible state flag
- CSS Selectors Level 4 — `:focus-visible` pseudo-class
- WPT `css/selectors/focus-visible-*` — conformance fixtures
- `@mui/material` `useIsFocusVisible` — upstream parity reference
