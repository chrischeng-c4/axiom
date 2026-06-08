# Keyboard Focus Observation Channel

Channel epic: **#2135** — `epic(jet): parity — keyboard focus observation channel`
Umbrella: **#2133** — `epic(jet): perceptual parity harness — jet ↔ modern FE stacks on DOM`
Sub-issues: **#2152 – #2157**
Sibling channels: pixel (#2134), a11y (#2136 — shares the `<jet-semantics>` shadow subtree), pointer (#2137), IME (#2138).

## Goal

`jet` paints its entire visible UI to a single `<canvas id="jet-root">` via WebGPU; there is no DOM for buttons, text fields, dialogs, menus, etc.

The keyboard, however, is owned by the host browser. Tab / Shift-Tab navigation, `:focus-visible` heuristics, `element.focus()`, autofocus, and screen-reader caret tracking all assume real focusable DOM elements.

The **keyboard focus observation channel** proves that, from the perspective of a keyboard user — and any tool that drives the browser through the standard focus model (keyboard, AT, password manager, browser extensions, WPT, Playwright, Cypress, automation harnesses) — jet is **observably indistinguishable** from a React + MUI / Angular + Material / Vue + Vuetify app rendered to real DOM.

Concretely:

- Pressing Tab in a jet app moves focus through widgets in the same order as the equivalent React DOM tree.
- Opening a jet `<Dialog>` traps focus exactly like MUI's `FocusTrap` does.
- An ARIA composite widget (toolbar, listbox, grid, menu) responds to roving-tabindex arrow-key conventions.
- Calling `someJetWidget.focus()` from external JS yields the same focus / blur events, scroll-into-view behaviour, and `:focus-visible` ring as the DOM equivalent.

Failing any of these makes jet unusable for keyboard-only users, breaks every automated test framework that drives apps via the DOM focus API, and silently breaks every screen reader — because AT walks focus, not pixels.

## Architecture

jet bridges keyboard focus between the WebGPU canvas and the browser's native focus engine via a **hidden semantic shadow subtree**, modelled on Flutter Web's `flt-semantics-host` (see `flutter/engine/lib/web_ui/lib/src/engine/semantics/`). For every focusable jet widget the renderer mounts a proxy element inside `<jet-semantics>`:

```
<body>
  <canvas id="jet-root">           <!-- all visible UI, painted via WebGPU -->
  <div id="jet-ime-overlay">       <!-- IME bridge (channel #2138) -->
  <jet-semantics>                  <!-- shadow subtree (focus + a11y, #2135 + #2136) -->
    <button tabindex="0"
            role="button"
            aria-label="Save"
            style="position:absolute; left:124px; top:88px;
                   width:96px; height:32px;
                   opacity:0; pointer-events:auto;">
    </button>
    <div tabindex="0" role="textbox" ...></div>
    ...
  </jet-semantics>
</body>
```

Each proxy is a real, focusable DOM element with:

1. A real `tabindex` (`0` for "in tab order", `-1` for "programmatic only", omitted for "not focusable"). The browser computes tab order from this — jet does **not** intercept `keydown` Tab.
2. A real ARIA `role` and label (shared with the a11y channel, #2136).
3. `position: absolute` over the widget's hit rect in canvas coordinates, kept in sync as the widget moves / resizes / scrolls. This is what makes the proxy a valid hit target for AT and for mouse-routed focus.
4. `opacity: 0` (not `display: none`, not `visibility: hidden` — both remove the element from the focus tree).

The browser handles **everything keyboard-focus-related natively** via these proxies:

- Tab / Shift-Tab sequential navigation: native sequential focus navigation algorithm on the proxy elements.
- `:focus-visible`: native heuristic (keyboard vs pointer focus, sticky on TextInput) computed on the proxy.
- `element.focus()` / `.blur()` / `document.activeElement`: native, on the proxy.
- `autofocus` attribute: native, on the proxy.
- Focus trap: when a jet Dialog opens, jet uses the standard "inert siblings + sentinel-wrap" pattern on the proxy subtree, identical to MUI's `FocusTrap`.

jet's runtime then **observes** focus state on each proxy (via `focus` / `blur` / `focusin` / `focusout` events plus `:focus-visible` matching on every render tick) and forwards the result into the WASM widget tree, which (a) updates internal focus state, (b) fires React-equivalent `onFocus` / `onBlur` callbacks, and (c) repaints the WebGPU layer to show the focus ring exactly where the proxy is.

This split — browser owns focus state, jet observes and paints — is the same split Flutter Web uses (`SemanticsObject` / `Focusable` / `EngineSemanticsOwner`) and is the only design that can be observably equivalent to DOM apps without re-implementing the focus algorithm.

**Why "observe, don't intercept" is the only viable design.** Any approach where jet intercepts Tab at the `keydown` level and runs its own traversal algorithm fails in at least four ways:

1. Assistive technology (VoiceOver, NVDA, JAWS, Orca) walks focus, not pixels — without real `document.activeElement` updates AT cannot follow the user.
2. Browser-native shortcuts (Ctrl+Tab, F6, Ctrl+Home, find-on-page) bypass `keydown` handlers and only know about real focusable elements.
3. Password managers, autofill, and extension content-scripts look for real `<input>` / `<button>` in the DOM.
4. WPT / Playwright / Cypress / Testing Library all drive apps through native focus APIs and would see jet as a no-op blob.

Mirroring focusable widgets into a real, hidden DOM subtree is the single design that satisfies all four.

**Subtree lifecycle.** The `<jet-semantics>` subtree is rebuilt incrementally each frame the widget tree changes (not on every paint). Proxies survive across re-renders as long as the underlying widget identity is stable — this keeps `document.activeElement` from blurring spuriously when React reconciliation reorders an unrelated sibling. Proxies removed in a given frame (widget unmount, modal close) have their `tabindex` cleared and `inert` set one frame before deletion to give the browser a chance to retarget focus deterministically rather than dropping it to `<body>`.

**Proxy element type selection.** The mapping from jet widget to proxy element type is intentionally close to MUI's underlying DOM. `<Button>` → `<button>`, `<TextField>` → `<input type="text">` (or `<textarea>` for multi-line), `<Checkbox>` → `<input type="checkbox">`, `<Link>` → `<a href>`, `<Tab>` → `<button role="tab">`, generic focusable region → `<div tabindex="0">`. Using the real element type maximises compatibility with password managers, autofill, and AT heuristics; using a `<div role>` everywhere would be lighter but breaks `Form` submission, `:invalid`, `:autofill`, autocomplete attributes, and a long tail of UA features.

## Sub-issues

| # | Priority | Subject | Depends on |
|---|---|---|---|
| #2152 | P1 | Hidden DOM focus proxy — `<jet-semantics>` shadow subtree contract (one focusable proxy element per focusable jet widget; real `tabindex`/`role`/`position:absolute`) | Foundation #2139 (harness runner), #2144 (manifest); paired with a11y #2136 (shared subtree) |
| #2153 | P1 | Tab order decision — semantic-tree vs paint order; recommended: semantic-tree to match React DOM order; WPT `sequential-focus-navigation*` subset as conformance gate | #2152, foundation #2142 (WPT vendor) |
| #2154 | P1 | Focus trap for Dialog / Menu / Drawer — port MUI `FocusTrap.test.tsx` fixtures (Tab cycles within modal, Shift-Tab in reverse, Esc restores prior focus, initial-focus-target on open) | #2152, #2153 |
| #2155 | P2 | Roving tabindex for composite widgets — toolbar / listbox / grid / menu; ARIA APG fixtures (`w3c/aria-practices`); only one element has `tabindex=0`, arrow keys move within | #2152, #2153 |
| #2156 | P1 | `:focus-visible` ring paint — query `proxyEl.matches(':focus-visible')` per render tick; browser owns the heuristic (mouse focus vs keyboard focus); jet observes and paints | #2152, pixel channel #2134 (paint-diff infra) |
| #2157 | P1 | Programmatic focus API — `element.focus()` bridging: move proxy focus, update paint, fire focus/blur on WASM side, honour `preventScroll`; gate against WPT `the-autofocus-attribute/` | #2152, #2153 |

## Technical approach

- **One widget = one proxy.** Mount, reposition, and unmount proxies from the React-on-WASM reconciler exactly as widgets enter / move / leave the tree. Recycle proxies on identity-stable widgets to keep `document.activeElement` continuity across re-renders.
- **Proxy positioning.** Compute the proxy's `transform: translate(x, y)` and `width` / `height` from the widget's hit rect in canvas coordinates, updated in the same frame as the WebGPU paint. This lets AT and mouse-routed focus land on the visually-correct element. Flutter Web does the same: `SemanticsObject` mirrors the render object's transform into the DOM element's matrix.
- **tabindex management.** Three states: `0` (in document tab order), `-1` (focusable programmatically only — used for the off-element member of a roving-tabindex composite), absent (not focusable). Widgets like `<TextField disabled>` or `<Button disabled>` get the attribute removed entirely, matching the DOM behaviour. No custom `keydown` Tab handler — the browser's native sequential focus navigation must own this.
- **Focus event forwarding.** Listen to `focus` / `blur` / `focusin` / `focusout` on `<jet-semantics>` (delegation) and translate the proxy's identity (`data-jet-id` attribute → widget handle) into a focus-state update on the WASM side, which then fires React-equivalent `onFocus` / `onBlur`. Order: DOM event fires first, then WASM-side callbacks, matching the order a React DOM app would observe.
- **`:focus-visible` is not re-implemented.** jet calls `proxyEl.matches(':focus-visible')` once per render tick for the currently-focused proxy; the browser's heuristic (which differs across UA — Firefox vs Chromium vs WebKit — by design per the spec) is the authoritative source. The focus ring is then painted by the WebGPU layer over the widget's hit rect. This is the same trick Flutter Web uses for keyboard-vs-pointer focus indication.
- **Focus trap = inert + sentinels.** When jet's Dialog/Menu/Drawer is open, the renderer sets `inert` on every proxy outside the trap region and wraps the trap region with two zero-size sentinel proxies; when a sentinel receives focus (Tab past the end / Shift-Tab past the start), it programmatically moves focus back to the first / last real proxy inside the trap. This is the exact MUI `FocusTrap` algorithm; the parity fixtures are a direct port of `mui-base/src/FocusTrap/FocusTrap.test.tsx`.
- **Roving tabindex.** For ARIA composite widgets (toolbar, listbox, grid, menu), only one proxy in the group has `tabindex=0` at a time; jet handles arrow keys at the widget level, moves the `tabindex=0` flag, and calls `.focus()` on the new active proxy. Tab from outside lands on the active member; Tab from inside leaves the group. ARIA APG provides the canonical test matrix.
- **`element.focus()` bridging.** When external JS calls `someJetWidget.focus()` (where `someJetWidget` is the proxy element exposed via jet's React ref forwarding), the four observable effects must all happen: (a) proxy is `document.activeElement`, (b) the next paint shows the focus ring, (c) WASM-side `onFocus` fires synchronously, (d) the proxy is scrolled into view unless `preventScroll: true`. `preventScroll` and `focusVisible` (Firefox-only) must be passed through unchanged. WPT `wpt/html/interaction/focus/the-autofocus-attribute/` is the conformance gate.
- **`autofocus` attribute.** Set the HTML `autofocus` attribute on the proxy of any widget whose React/Vue/Angular equivalent would carry it. The browser's "autofocus the first one in document order at insertion time" rule applies natively. Note that within a `<Dialog>`, the autofocus interaction with focus-trap restoration is well-defined by the spec and matched by MUI; jet inherits that behaviour from the proxy subtree mechanically.
- **`blur()` and focus loss.** `proxy.blur()` clears focus to `<body>`; jet observes this and fires WASM-side `onBlur`. Window-blur (Cmd-Tab away from the tab) leaves `document.activeElement` unchanged per spec; jet's paint keeps the focus ring drawn (matching DOM behaviour) and restores keyboard input when the window re-focuses.
- **Cross-iframe focus.** jet apps embedded in a parent page must respect `document.hasFocus()` and focus delegation across the iframe boundary. The proxy subtree lives inside jet's iframe; standard iframe focus rules apply unchanged. No additional work — this is a free consequence of using real DOM proxies.
- **Scroll-into-view interaction.** When a proxy is `focus()`ed and `preventScroll` is not set, the browser scrolls the proxy into view. Because the proxy is positioned over the widget's hit rect, this scrolls the *page* (host scroller) to bring the widget on-screen — exactly what a DOM app does. For widgets inside a jet-internal scrollable region (a virtual list, a `Tabs` overflow strip), the widget's own scroll model must observe the focus event and scroll its internal viewport before the next paint, otherwise the browser-driven page scroll will not reveal the widget.

## Dependencies

**Foundation (this channel cannot run without these):**

- **#2139** — parity harness runner: hosts the three reference stacks (React+MUI / Angular+Material / Vue+Vuetify) side-by-side with jet and drives synchronized scenarios.
- **#2142** — WPT vendor: pinned WPT subset under `vendor/wpt/html/interaction/focus/**` plus a harness that runs the same tests against jet and a reference DOM stack and diffs the resulting focus-event traces.
- **#2144** — parity manifest: per-widget mapping of jet widget ↔ reference component ↔ ARIA role ↔ focus expectations, so the focus channel knows which proxies are expected at which positions.

**Cross-channel coupling:**

- **a11y #2136** — *shares the `<jet-semantics>` shadow subtree*. Every focus proxy is also an a11y proxy. The two channels must agree on the subtree's lifecycle (mount/unmount, attribute set, ordering). #2152 (proxy contract) is jointly owned with #2136.
- **IME #2138** — input proxies share the focus lifecycle: focusing a jet `<TextField>` proxy must transfer focus to the IME overlay's `contenteditable`, and back on blur, without observable flicker in `document.activeElement`.
- **pointer #2137** — clicking a widget moves focus to its proxy (matching browser behaviour where mousedown on a `<button>` focuses it on Chromium and Safari but not Firefox — the proxy gets the UA-native behaviour for free).
- **pixel #2134** — focus-ring paint is a pixel-channel observation; pixel diff fixtures must include the focus-ring states (no ring, `:focus-visible` ring) for every focusable widget.

**External references / fixtures:**

- W3C ARIA APG composite-widget pattern fixtures (toolbar, listbox, grid, menu).
- WPT focus-management subset (sequential focus navigation, autofocus, focus trap interactions with `inert`).
- MUI `mui-base/src/FocusTrap/FocusTrap.test.tsx` test corpus, ported.

## Success criteria

- **Tab order matches across stacks.** For each of ~60 representative MUI components (Button, IconButton, TextField, Select, Checkbox, Radio, Switch, Slider, DatePicker, Autocomplete, Dialog, Menu, Drawer, Tabs, Stepper, Accordion, DataGrid, …), the sequential focus order of jet equals the sequential focus order of React + MUI under the same scenario. Driver: synthesized Tab / Shift-Tab key events; observable: ordered list of `document.activeElement.dataset.parityId`.
- **FocusTrap parity.** The full MUI `FocusTrap.test.tsx` suite passes against jet's `<Dialog>` / `<Menu>` / `<Drawer>` (Tab cycles, Shift-Tab reverse-cycles, Esc restores prior focus, initial focus target honoured, `disableAutoFocus` / `disableRestoreFocus` / `disableEnforceFocus` semantics).
- **Roving tabindex parity.** The ARIA APG examples for toolbar / listbox / grid / menu pass: exactly one member has `tabindex=0`, Tab enters/exits the group atomically, arrow keys move the active member, Home / End jump to ends, typeahead works where the pattern specifies it.
- **`:focus-visible` parity.** For each focusable widget: keyboard focus paints a ring; mouse focus does **not** paint a ring (Chromium/WebKit) — but **does** paint a ring on TextField regardless of input modality (`:focus-visible` always matches on text-input). Per-UA differences are accepted (the spec allows them); jet inherits whichever the host browser computes.
- **`element.focus()` parity.** WPT `html/interaction/focus/the-autofocus-attribute/`, `sequential-focus-navigation*`, and `focus-event-targets*` pass against jet at the same rate as against the React DOM reference stack (deltas ≤ documented waivers).
- **No regression in keyboard-only navigation of the demo app.** Manual / scripted test: complete the demo app's primary user flow using only Tab, Shift-Tab, arrow keys, Enter, Esc, and Space — no mouse — with focus always visible.
- **Focus-event trace equivalence.** For each parity scenario, the ordered sequence of `(target.parityId, eventType)` recorded across all `focus` / `blur` / `focusin` / `focusout` listeners on `document` is byte-identical between jet and React+MUI (allowing documented per-UA variance for `focusin`/`focusout` ordering — Webkit fires `focusin` before `focus` on the target; jet inherits this from the proxy).
- **No "ghost" focus on `<body>`.** Whenever the user has not Esc'd away from a focusable region, `document.activeElement !== document.body`. If a widget unmounts while focused, focus moves to a deterministic neighbour (next sibling, or previous sibling, or parent's focus target) — matching the React DOM behaviour rather than dropping to `<body>`. The deterministic neighbour algorithm is specified in #2152 and tested as part of the unmount-while-focused fixture set.
- **Coverage gate.** ≥ 95% of MUI's 60 representative components carry a green focus-channel fixture; documented waivers explain each red. ≥ 80% of the WPT focus subset passes; documented waivers explain the rest.

## Out of scope / waivers

- **`:focus-within` styling of canvas children of a canvas widget.** Impossible — once we are inside the canvas, there are no DOM children for the host CSS engine to style. The `<jet-semantics>` subtree is flat from CSS's perspective; jet does not expose nested proxies for sub-parts of a single widget.
- **CSS-driven focus styles inside jet widgets.** Apps that depend on `:focus { outline: 2px solid blue }` selectors targeting in-widget elements get no styling — jet paints the ring itself. Documented as a known divergence; affects only authoring style, not observable behaviour.
- **Per-UA `:focus-visible` heuristic differences.** jet inherits whatever Chromium / WebKit / Firefox each compute for `:focus-visible` on the proxy. We do not normalise; cross-UA divergence is an upstream behaviour, not a jet bug.
- **Sub-widget focus traversal.** A jet widget is a single proxy; if a widget has internal interactive parts (e.g. DataGrid cells), the widget itself implements the roving-tabindex pattern over its internal model and exposes a single proxy with `tabindex=0`. We do not mount a proxy per cell.
- **Spatial / 2-D arrow-key focus** (`tvOS`-style "navigate spatially"). Not part of any DOM stack's default behaviour; if needed, ship as an opt-in widget-level concern, not a channel-wide gate.
- **Browser extensions that walk the DOM looking for `<input>` elements.** Password managers, autofill, etc., recognise jet `<TextField>` proxies only if the proxy is a real `<input>`. #2152 leaves the choice between `<input>` (compat) and `<div role="textbox">` (lighter) explicit; the recommendation is `<input type="text">` to maximise compat, even though it slightly increases the proxy footprint.

## Worked example — focus-event trace

A canonical scenario: a form with `[FirstName] [LastName] [Submit]`. User presses Tab three times starting from `<body>`. Expected event trace on jet (proxy IDs `pid-first`, `pid-last`, `pid-submit`):

```
keydown      Tab        target=body
focusin      pid-first  bubbles=true
focus        pid-first  bubbles=false
(WASM)       onFocus    widget=FirstName
keyup        Tab        target=pid-first
keydown      Tab        target=pid-first
focusout     pid-first  relatedTarget=pid-last
blur         pid-first  relatedTarget=pid-last
(WASM)       onBlur     widget=FirstName
focusin      pid-last   relatedTarget=pid-first
focus        pid-last   relatedTarget=pid-first
(WASM)       onFocus    widget=LastName
keyup        Tab        target=pid-last
keydown      Tab        target=pid-last
focusout     pid-last   relatedTarget=pid-submit
blur         pid-last   relatedTarget=pid-submit
(WASM)       onBlur     widget=LastName
focusin      pid-submit relatedTarget=pid-last
focus        pid-submit relatedTarget=pid-last
(WASM)       onFocus    widget=Submit
keyup        Tab        target=pid-submit
```

The same scenario against React + MUI must produce a byte-identical trace (modulo the parity-id mapping). The harness logs both, diffs, and reports any discrepancy as a parity failure on this channel.

## Worked example — Dialog focus trap

A `<Button>` opens a `<Dialog>` containing `[CancelBtn] [OkBtn]`. The trap must:

1. On open, save the previously-focused proxy (`pid-opener`) and focus the dialog's initial-focus target (`pid-cancel`).
2. Mark every proxy outside the dialog `inert`, so Tab cannot reach them.
3. Insert two zero-size sentinel proxies before / after the dialog's proxy subtree.
4. On Tab from `pid-ok`, the trailing sentinel receives focus → trap handler programmatically moves focus back to `pid-cancel`.
5. On Shift-Tab from `pid-cancel`, the leading sentinel receives focus → trap handler moves focus to `pid-ok`.
6. On Esc (or any other dialog-close path), `inert` is cleared, sentinels are removed, and focus is restored to `pid-opener`.

These six steps are the exact MUI `FocusTrap` algorithm. The parity fixture asserts the same six-step trace under jet and under React + MUI, with the same `focus` / `blur` / `focusin` / `focusout` interleaving.

## Implementation phasing

Suggested merge order (each step is independently mergeable; each adds a new gate to the parity harness):

1. **#2152 (proxy contract)** — paired with #2136. Establishes the `<jet-semantics>` subtree, the per-widget proxy element-type mapping, and the proxy-positioning loop. Unblocks every other focus sub-issue.
2. **#2153 (tab order)** — wires WPT `sequential-focus-navigation*` into the harness and lands the semantic-tree-order decision in writing.
3. **#2157 (programmatic focus)** — `element.focus()` bridging; gated against WPT `the-autofocus-attribute/`.
4. **#2156 (`:focus-visible` paint)** — depends on the pixel channel's paint-diff infra.
5. **#2154 (focus trap)** — ports MUI's `FocusTrap.test.tsx` corpus; lands the inert + sentinel pattern.
6. **#2155 (roving tabindex)** — ARIA APG composite-widget fixtures; lowest priority because it only affects a small set of widgets (toolbar, listbox, grid, menu) and degrades gracefully (each member behaves like a normal focusable until the channel ships).

## Prior art and references

- Flutter Web semantics engine — `flutter/engine/lib/web_ui/lib/src/engine/semantics/` — the canonical implementation of "canvas app exposes a hidden DOM subtree as the focus + a11y bridge". Key files: `semantics.dart` (`EngineSemanticsOwner`, `SemanticsObject`), `focusable.dart`. <https://github.com/flutter/engine/tree/main/lib/web_ui/lib/src/engine/semantics>
- Flutter SDK focus API — `FocusableActionDetector`, `Focus`, `FocusNode`, `FocusTraversalPolicy`. <https://api.flutter.dev/flutter/widgets/Focus-class.html>, <https://api.flutter.dev/flutter/widgets/FocusableActionDetector-class.html>
- WPT focus tests — sequential focus navigation, autofocus, focus-event targets. <https://github.com/web-platform-tests/wpt/tree/master/html/interaction/focus>
- W3C ARIA Authoring Practices Guide — composite widget patterns and roving tabindex. <https://www.w3.org/WAI/ARIA/apg/patterns/>; example fixtures live in <https://github.com/w3c/aria-practices/tree/main/content/patterns>.
- HTML Living Standard — sequential focus navigation algorithm and `tabindex` semantics. <https://html.spec.whatwg.org/multipage/interaction.html#sequential-focus-navigation-and-the-tabindex-attribute>
- CSS Selectors L4 — `:focus-visible` definition and UA heuristic. <https://www.w3.org/TR/selectors-4/#the-focus-visible-pseudo>
- MUI `FocusTrap` source + tests — `mui/material-ui/packages/mui-base/src/FocusTrap/FocusTrap.tsx` and `FocusTrap.test.tsx`. <https://github.com/mui/material-ui/tree/master/packages/mui-base/src/FocusTrap>
- WAI-ARIA `inert` and the focus-trap pattern. <https://html.spec.whatwg.org/multipage/interaction.html#the-inert-attribute>, <https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/#focus_trapping>
- React focus event ordering — synthetic event model on top of native focus/blur/focusin/focusout. <https://react.dev/reference/react-dom/components/common#focusevent>
