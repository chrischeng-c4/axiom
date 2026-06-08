# ADR-028: Hover events + cursor style parity

| Field | Value |
|-------|-------|
| Issue | #2170 |
| Parent epic | #2137 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Per-frame, after the #2164 glass-pane router resolves the topmost hit-test widget at the current pointer position, jet (a) writes `canvas.style.cursor = widget.cursor` on the host `<canvas>` element so the browser asks the OS to swap the cursor sprite at OS level — visible *before* any JS paint runs — and (b) fires synthetic `pointerover` / `pointerenter` (on the new top-hover widget and the enter-set of its proxy ancestor chain) plus `pointerout` / `pointerleave` (on the old top-hover widget and the leave-set) on the per-widget proxy DOM nodes from #2158, so `:hover`-driven CSS on the proxy fires and `aria-describedby` tooltip behavior matches the DOM reference oracle. Per-widget cursor is sourced from a `cursor` column added to the #2158 role-mapping table (button/link → `pointer`, textfield → `text`, resizer → `nwse-resize`, drag-handle → `grab` / `grabbing` during press, disabled → `not-allowed`); React `style={{ cursor: … }}` is propagated through the same proxy `style` reading pipeline. The host canvas's CSS update is rAF-coalesced. Touch input replicates Chromium/Safari "phantom hover": on tap, jet fires `pointerover` with `pointerType: "touch"` and immediately fires `pointerout` on the next touch landing elsewhere. Parity is gated by a WPT-style fixture with overlapping hover/cursor cells (Button next to TextField) that asserts pointer events fire on the correct widget in the correct order **and** `getComputedStyle(canvas).cursor` equals the DOM reference oracle's cursor for the same pointer coordinate. |

## Context

Two of the quietest signals in any web UI come not from JavaScript
but from the browser-level interaction between the OS pointer and
the document tree: the **cursor sprite** (`cursor: pointer` over a
button, `cursor: text` over an input, `cursor: not-allowed` over a
disabled control) and the **:hover pseudo-class** (the selector that
turns a `Button`'s background from grey to blue as the pointer
enters its bounding box). Both are evaluated by the browser per
animation frame against the element currently under the OS pointer.
Both reach the user *before* any application JavaScript has a chance
to run: the cursor swap is an OS-level operation negotiated between
the browser's compositor and the window server; the `:hover` style
recalculation is a style-resolver pass that happens inside the same
paint that incorporates the pointer's new position.

For a real DOM page neither requires a single line of author code.
`<button>Click me</button>` ships with `cursor: pointer` from the
user-agent stylesheet, MUI's `Button` adds `:hover { background-color:
rgba(25,118,210,0.04) }`, and the user sees both immediately on
first pointer movement across the element. No `mousemove` listener,
no rAF tick, no work on the JS event loop.

Jet has neither for free. The host `<canvas>` is one rectangular
element. It carries exactly one CSS `cursor` value at a time and
exactly one `:hover` state at a time (over the whole canvas, not over
the widget under the pointer). Without explicit projection — by jet,
on every frame — every interactive widget in a canvas-rendered
surface looks dead: the cursor stays the default arrow over what is
clearly a button, hover styling never fires, and the affordance gap
between "this looks like the web" and "this is a half-functional
canvas demo" is widened by exactly one of the loudest signals the
platform provides. ADR-026 (#2168) called the cursor and `:hover`
mismatch "the cheapest-to-spot regression in the entire pointer
channel"; this ADR is the one that closes it.

The mechanism is constrained by what the browser actually exposes.
We cannot reach into the OS and swap the cursor sprite ourselves —
no Web API permits it. We can only write a value into
`canvas.style.cursor` and rely on the browser's per-frame style
resolution to ask the window server for the matching sprite. The
same is true of `:hover`: there is no synthetic-event API that says
"please re-run selector matching as if the cursor were over this
sub-region of the canvas". We can, however, evaluate the hover
delta ourselves and fire `pointerover` / `pointerout` / `pointerenter`
/ `pointerleave` on the **proxy DOM nodes** introduced in #2158, so
that `:hover` rules attached to those proxies *do* fire — and any
CSS-driven tooltip / styling on the proxy follows the proxy's
`:hover` state in the DOM, not the canvas's.

Two upstream issues set the surface this ADR projects onto:

- **#2164 (glass-pane router).** On every `pointermove`, the router
  resolves a hit-test ancestor chain from the topmost widget at the
  cursor position up to the canvas root. ADR-028 reads the topmost
  hit-test widget for cursor selection and the full enter/leave-set
  for hover event synthesis.
- **#2158 (per-widget proxy emitter).** Each interactive jet widget
  already has a proxy DOM node in a position-absolute overlay layer
  used for ARIA, focus, and link semantics. ADR-028 adds a `cursor`
  column to the emitter's role-mapping table and uses the existing
  proxy node as the target for synthetic hover events.

## Decision

Hover and cursor parity is implemented as a per-frame pass that
runs after #2164's hit-test resolution and writes through two
projections.

### 1. Cursor projection (OS sprite swap)

Each jet widget carries a `cursor` value resolved by the following
precedence:

1. Consumer override from the proxy `style.cursor` (React
   `style={{ cursor: 'crosshair' }}` reaches us through the same
   #2158 proxy-style reader that already carries `touchAction` etc.).
2. Disabled-state mapping (`not-allowed`) if the widget's
   `aria-disabled === 'true'`.
3. Role-mapping table column added in this ADR (see Implementation).
4. Inherited cursor from the canvas root (default `auto`).

Once per `rAF` frame, after the router has produced the topmost
hit-test widget, jet writes:

```js
const target = router.currentHitTarget;
const cursor = resolveCursor(target);          // precedence above
if (cursor !== canvas.style.cursor) {
  canvas.style.cursor = cursor;                // OS swaps sprite
}
```

The browser's style resolver picks up the change in the same paint
and asks the window server to swap the OS cursor sprite. The
user sees the swap on the *next* compositor frame, with no JS work
required for the visual change itself. We do not implement
JS-level cursor rendering (no canvas-drawn cursor sprite) — we
defer entirely to the OS, which matches every DOM-rendered page's
behaviour and avoids the latency of double-buffering the cursor
inside the canvas.

Custom URL cursors (`cursor: url(/foo.png), pointer`) pass through
unchanged from the proxy `style.cursor` to the canvas `style.cursor`.
We do *not* own URL fetching, CORS, or the cursor-hotspot pixel
offset — those are the browser's contract with the OS. This is out
of scope for ADR-028 but expressly **not blocked**: any consumer
that writes a valid URL-cursor value into a widget's proxy style
will see the browser fetch and apply it normally.

### 2. Hover event synthesis (`:hover` CSS + tooltip + aria-describedby)

The router's hit-test resolution emits an `enterSet` and a
`leaveSet` per frame (introduced for #2164's pointer routing). When
the topmost hit-test widget changes between frames:

```
oldTop = previousFrame.hitTarget
newTop = currentFrame.hitTarget
leaveChain = ancestors(oldTop) \ ancestors(newTop)   // proxy nodes
enterChain = ancestors(newTop) \ ancestors(oldTop)   // proxy nodes
```

Jet then dispatches, on the *proxy DOM nodes* corresponding to
those widgets:

- `pointerout` (bubbles) on `oldTop`'s proxy.
- `pointerleave` (does not bubble) on every node in `leaveChain`.
- `pointerover` (bubbles) on `newTop`'s proxy.
- `pointerenter` (does not bubble) on every node in `enterChain`.

These are synthetic `PointerEvent` instances with the current
`clientX` / `clientY` / `pointerType` populated. Because the events
fire on real DOM proxy nodes, any CSS rule keyed on `:hover` for
those proxies fires through the browser's normal selector machinery
— we do *not* re-implement selector matching. Any `aria-describedby`
tooltip behaviour that the consumer wires up (e.g. MUI `Tooltip`'s
`onPointerEnter` handler) fires by the same mechanism, for free.

The choice to fire on the **proxy** rather than the canvas is
load-bearing: the canvas has exactly one bounding box and one
`:hover` state for the entire surface; the proxy nodes are
positioned per widget and carry per-widget `:hover` state. This
also means a developer inspecting the DOM in DevTools sees the
proxy node go `:hover` in the elements panel as they move over the
canvas widget — the affordance is fully observable through normal
browser tooling.

### 3. Per-widget cursor mapping (extends #2158)

The role-mapping table from #2158 gains a `cursor` column:

| Role | `cursor` |
|------|----------|
| `button` | `pointer` |
| `link` | `pointer` |
| `menuitem` | `pointer` |
| `tab` | `pointer` |
| `textbox` / `searchbox` | `text` |
| `slider` thumb | `grab` (`grabbing` during press) |
| `resizer` (column / panel) | `nwse-resize` / `ew-resize` / `ns-resize` (per axis) |
| `drag-handle` | `grab` / `grabbing` |
| anything with `aria-disabled="true"` | `not-allowed` |
| default | `auto` |

The proxy-style override (precedence 1 above) wins over any
role-default value, so `<Button style={{ cursor: 'crosshair' }} />`
projects `crosshair`, not `pointer`.

### 4. Touch input — phantom hover replication

On a touch-input device the OS has no cursor sprite, so the cursor
projection is a no-op (the browser's `canvas.style.cursor` write is
honoured but invisible). The `:hover` projection, however, must
replicate the Chromium / Safari "phantom hover" behaviour: after a
`tap` (touchend without movement), the browser briefly applies
`:hover` to the tapped element until the *next* touch lands
elsewhere, then immediately removes it.

Jet replicates this on the proxy nodes:

- On `tap` (touchend), fire `pointerover` + `pointerenter` with
  `pointerType: "touch"` on the tapped widget's proxy.
- On the next touch landing elsewhere (or after a 250 ms idle
  window — matching Chromium's `kStuckHoverTimeoutMs`), fire
  `pointerout` + `pointerleave` on the previously-tapped proxy.

This matches what `:hover` does on real mobile DOM and avoids the
"stuck hover" trap where a touched `Button` keeps its `:hover` blue
forever because no `pointermove` ever clears it.

### 5. Cross-frame coalescing

Both projections are coalesced into a single rAF pass:

```
frame:
  hit = router.resolveTopHit(currentPointer)
  if hit != previousHit:
    fireHoverEvents(previousHit, hit)         // proxy DOM
    previousHit = hit
  cursor = resolveCursor(hit)
  if cursor != canvas.style.cursor:
    canvas.style.cursor = cursor              // OS sprite
```

There is no per-`pointermove` work beyond cheap pointer-position
caching; the projection cost is paid once per frame regardless of
how many pointer events arrived in the interval. This matches the
budget set in ADR-026 (#2168) for the touch-action latch.

## Consequences

**Positive**

- The user sees the correct cursor sprite over every interactive
  widget within one compositor frame of the pointer entering the
  widget's bounding box — the same latency as a DOM page.
- `:hover` CSS rules attached to proxy nodes fire correctly,
  including nested hover (`Card:hover` + `Card > Button:hover`
  simultaneously true while the cursor is over the Button), because
  the enter-set walks the proxy ancestor chain.
- MUI `Tooltip`, `Button`, `Link` — anything that wires
  `onPointerEnter` / `onPointerLeave` or relies on `:hover` CSS —
  works inside a jet-rendered surface with no consumer code changes.
- The aria-describedby tooltip pathway works for free because the
  same `pointerover` event drives MUI Tooltip's open delay timer.
- DevTools elements panel shows the proxy node's `:hover` state
  flipping in real time, so debugging affordances match the DOM.

**Negative**

- Per-frame hit-test + style.cursor write adds two memory reads and
  at most one CSS property write per frame; negligible on modern
  hardware but non-zero. ADR-029 (perf budget) will measure it.
- Synthetic `pointerover` / `pointerout` events fire on the proxy,
  not on the canvas, so listeners attached directly to the canvas
  (rather than to widgets) will not see them. Documented as a known
  asymmetry — consumers wire to widget proxies, not the host canvas.
- Touch phantom-hover replication runs a 250 ms timer per tap,
  introducing a small piece of state the test harness must reset
  between cases.

**Neutral**

- The cursor and hover paths share a single rAF pass; if the host
  app frame-skips, both projections frame-skip together. This is
  consistent with how the DOM behaves under frame-budget pressure.

## Alternatives considered

1. **`document.elementFromPoint` on every `pointermove`.** Asking
   the browser to hit-test through the canvas back into the proxy
   layer beneath would let the browser drive `:hover` directly. We
   would lose, though: the proxy layer is `pointer-events: none` by
   construction (so the canvas receives pointer events), which
   defeats `elementFromPoint`. We would have to toggle
   `pointer-events` per frame, which is both expensive and creates
   a race against the next `pointermove`.
2. **Render the cursor inside the canvas.** Draw a custom cursor
   sprite at the pointer position each frame. Rejected: it
   double-buffers the cursor with the OS cursor (the OS cursor
   cannot be hidden cross-browser without `cursor: none`, and even
   then trails behind by one frame); it loses every OS-level cursor
   feature (high-DPI sprite scaling, accessibility cursor themes,
   inverted cursor over dark UI); and it breaks user expectations
   on Linux/Wayland where the cursor is a compositor concern.
3. **Drive `:hover` purely from the canvas.** Set `canvas:hover`
   styles and accept that the whole canvas is one hover target.
   Rejected: this is the status quo. The point of this ADR is to
   make per-widget hover work.
4. **Use the View Transitions API to animate cursor changes.**
   Rejected: View Transitions are document-level, not cursor-level;
   the cursor is an OS concern, not a DOM concern.

## Open questions

1. **Cursor over scrollbars.** If a jet-managed scroller exposes a
   thumb, should the cursor be `default` (matching native OS scrollbar
   behaviour) or `pointer` (matching some MUI variants)? Tentative
   answer: defer to the role-mapping table, which currently sets
   scrollbar to `default`; consumers can override per widget.
2. **Cursor across iframe boundaries.** If a jet surface is hosted
   inside an iframe and the OS pointer crosses the iframe edge,
   does the outer document's `mouseleave` clear our hit-test before
   the inner frame's `mouseenter` fires? Needs a fixture.
3. **Hover during pointer capture.** ADR-022's (#2167) capture
   semantics keep dispatch glued to the capturing widget regardless
   of pointer position. Should `:hover` follow the same rule
   (sticky on the capturing widget) or follow the geometric
   hit-test (which is what the issue's R6(a) actually asks for —
   confirmed in this ADR; we implement sticky-hover during capture).
4. **`:visited` link styling.** Issue R7 asks for `:visited` /
   `:link` integration with the visited-property allowlist. This
   ADR scopes the cursor + `:hover` channels; `:visited` is
   deferred to a follow-up ADR that owns the link channel's
   visited-history integration.

## References

- W3C Pointer Events Level 2 §6 — boundary events
  (`pointerover` / `pointerout` / `pointerenter` / `pointerleave`).
  <https://www.w3.org/TR/pointerevents2/#the-pointerover-event>
- CSS Basic UI Module Level 4 — `cursor` property.
  <https://www.w3.org/TR/css-ui-4/#cursor>
- CSS Selectors Level 4 — `:hover` pseudo-class.
  <https://www.w3.org/TR/selectors-4/#the-hover-pseudo>
- Chromium `kStuckHoverTimeoutMs` — touch phantom-hover duration.
  `third_party/blink/renderer/core/input/touch_event_manager.cc`
- WebKit `TouchPressureBasedHover.cpp` — Safari touch hover model.
- WPT `pointerevents/pointerevent_boundary_events_*` — boundary
  event ordering tests this fixture mirrors.
- ADR-006 / #2164 — glass-pane router (hit-test + enter/leave-set).
- ADR-009 / #2148 — proxy emitter pattern.
- ADR-022 / #2167 — pointer capture semantics.
- ADR-026 / #2168 — `touch-action` semantics (sibling pre-input
  CSS-driven channel).
- #2158 — per-widget proxy emitter role-mapping table (extended by
  this ADR with a `cursor` column).
- #2137 — pointer parity epic (parent).
