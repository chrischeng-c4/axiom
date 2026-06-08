# ADR-006: Single glass-pane input router

| Field | Value |
|-------|-------|
| Issue | #2164 |
| Parent epic | #2137 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Root canvas as sole event attachment; WASM hit-tester resolves semantic target; jet synthesizes DOM-compatible event sequence (pointer + legacy mouse) into per-target listener registry; supports setPointerCapture |

## Context

Jet renders its entire UI tree into a single `<canvas>` element. Unlike the
DOM, a canvas surface has **no per-node event dispatch** — the browser sees
one opaque rectangle and delivers exactly one pointer event per gesture,
addressed to the canvas itself. Every widget jet draws (buttons, list rows,
scroll thumbs, text runs) is invisible to the browser's hit-testing and
event-routing machinery.

Parity with DOM behaviour therefore requires jet to **re-implement, inside
the canvas, the slice of the DOM event model that web apps actually rely
on**. That slice includes:

- per-target `addEventListener` semantics
- the canonical pointer/mouse/click ordering (e.g. `pointerdown` →
  `pointerup` → `click` only when targets match and the pointer did not
  move past the drag threshold)
- bubbling up an ancestor chain
- `pointer-events: none` skipping
- `setPointerCapture` / `releasePointerCapture` redirection
- hover transitions (`pointerover` / `pointerenter` / `pointerout` /
  `pointerleave`) when the resolved target changes

This ADR is the **foundational primitive** that the rest of the pointer-
parity sub-issues build on. Without a single, deterministic event router,
every downstream feature has to reinvent its own ad-hoc dispatch and the
parity matrix collapses into incoherence. The dependents are:

- **#2165** — actual hit-tester implementation (the function this router
  calls; out of scope here, but the router's contract pins its signature)
- **#2166** — WPT (Web Platform Test) subset; verifies our synthesized
  sequences against the spec's reference traces
- **#2167** — nested scroll containers (consume `wheel` / `pointermove`
  with a target-aware redirect that piggybacks on this router)
- **#2168** — `touch-action` parity (pinch/zoom passive-listener policy
  applied at the glass-pane attachment site)
- **#2169** — drag-and-drop (HTML5 DnD events synthesized on top of the
  pointer stream this router emits)
- **#2170** — hover and cursor (mouse cursor changes driven by the
  `pointerover` transitions this router emits)

The parent tracking epic for all of the above is **#2137**.

The decision recorded here is intentionally narrow: it specifies **what
the router is**, **where it attaches**, **what events it observes**,
**what sequence it emits**, and **what test surface it exposes**. It does
not specify *how* the hit-tester walks the semantic tree, nor *which*
WPT cases we adopt — those are sibling ADRs / issues.

## The glass-pane element

The "glass pane" is the **single DOM element jet attaches all input
listeners to**. There is exactly one per jet root.

Two configurations are supported:

1. **Direct mode (default).** The root `<canvas>` *is* the glass pane.
   All listeners attach to the canvas element itself. This is the
   common case: jet is the only thing painting at this location.

2. **Overlay mode.** When the canvas is z-stacked with other DOM (e.g.
   embedded inside a host page that paints siblings above or below the
   canvas), a transparent overlay `<div>` is mounted as a sibling at the
   same position and size as the canvas, with `pointer-events: auto` and
   `background: transparent`. Listeners attach to the overlay. The
   canvas paints normally beneath; the overlay catches input. Mode is
   selected at jet-root construction time and never changes during the
   root's lifetime.

In both modes the attachment point is **singular**: there is exactly one
element per event type per jet root. Multiple jet roots on a page each
own their own glass pane.

### Observed native events

The router subscribes to the following native events on the glass pane:

| Family | Events |
|--------|--------|
| Pointer | `pointerdown`, `pointerup`, `pointermove`, `pointercancel` |
| Mouse (legacy) | `mouseover`, `mouseenter`, `mouseleave`, `mouseout` |
| Click | `click`, `dblclick`, `contextmenu` |
| Wheel | `wheel` |
| Touch | `touchstart`, `touchmove`, `touchend`, `touchcancel` |

Pointer events are the primary stream; mouse / touch events are observed
for legacy-compat synthesis and to detect the `touch-action` pinch/zoom
path that the browser handles natively (see #2168).

### Listener registration policy

- `pointer*`, `mouse*`, `click`, `dblclick`, `contextmenu`: registered
  with `{ passive: false, capture: false }` so the router may call
  `preventDefault()` when an in-canvas widget wishes to suppress
  default browser behaviour (e.g. text selection, context menu).
- `wheel`: split policy. Default `{ passive: true }` for smooth scroll;
  upgraded to `{ passive: false }` only when at least one widget under
  the current pointer has registered a wheel listener that **may**
  `preventDefault`. This avoids the well-known scroll-jank penalty
  while still allowing custom zoom widgets to consume wheel deltas.
- `touchstart` / `touchmove`: `{ passive: true }` globally, with the
  glass-pane element carrying `touch-action: none` (or finer-grained
  per-region values) per #2168. This delegates pinch / pan gesture
  recognition to the browser when the widget declines to handle it,
  rather than burning a non-passive listener.
- `pointercancel`, `touchcancel`: `{ passive: true }`. Always allowed
  to fire — they are the browser informing us that the gesture is gone.

## Event → hit-test pipeline

For each native event the router executes a fixed pipeline:

1. **Capture.** The native listener on the glass pane receives the
   `Event` object.
2. **Coordinate transform.** Compute canvas-space coordinates:

       const rect = glassPane.getBoundingClientRect();
       const dpr  = window.devicePixelRatio;
       const cx   = (event.clientX - rect.left) * dpr;
       const cy   = (event.clientY - rect.top)  * dpr;

   For touch events, the transform is applied per `Touch` in the
   `changedTouches` list.

3. **Hit-test in WASM.** Call:

       wasm_hit_test(cx, cy) -> Option<SemanticTarget>

   where `SemanticTarget` is the widget-tree node currently painted at
   that pixel, or `None` for empty canvas. The hit-tester is defined
   in #2165; this router treats it as a pure function.

4. **Ancestor walk.** From the hit target, walk up the semantic parent
   chain to synthesize the DOM-compatible bubble sequence. Each
   ancestor that has registered a listener for the current event type
   appears in the dispatch list, in capture-then-bubble order.

5. **Dispatch.** For each entry in the dispatch list, invoke the
   widget's listener with a synthesized event object whose `target`,
   `currentTarget`, `clientX/Y`, `pointerId`, `button`, `buttons`,
   `pressure`, `pointerType`, `isPrimary`, `ctrlKey`, etc., are
   populated from the native event. The synthesized event supports
   `stopPropagation()`, `stopImmediatePropagation()`, and
   `preventDefault()` — the last of which forwards to the native
   event so the browser sees a consistent decision.

The dispatcher lives entirely inside WASM; the JS glass-pane handler is
a thin trampoline that performs the coordinate transform and forwards
into WASM.

## DOM-compatible event sequence

To be a credible parity layer, jet must emit events in the **exact
order** a real DOM would emit them. The following sequences are
mandatory.

### Pointer movement — target unchanged

    pointermove → mousemove

(Targets unchanged ⇒ no `over` / `enter` / `out` / `leave` events.)

### Pointer movement — target changed

When the resolved target changes from `prev` to `curr`:

    pointerout    (on prev)
    pointerleave  (on prev and ancestors no longer in curr's chain)
    pointerover   (on curr)
    pointerenter  (on curr and new ancestors)
    pointermove   (on curr)
    mouseout      (on prev)
    mouseleave    (on prev and ancestors no longer in curr's chain)
    mouseover     (on curr)
    mouseenter    (on curr and new ancestors)
    mousemove     (on curr)

`enter` / `leave` **do not bubble**, per the spec; the router enforces
this by issuing them once per ancestor without a bubble walk. `over` /
`out` do bubble.

### Press

    pointerdown → mousedown

Issued at the resolved target. If the widget calls `preventDefault()`
on the `pointerdown`, the corresponding `mousedown` is suppressed (DOM
compat).

### Release

    pointerup → mouseup

Then, **iff** all of the following hold, also dispatch `click`:

- `up.target === down.target` (same semantic target), **and**
- the pointer travelled less than the drag threshold (default 5 CSS
  pixels) between `down` and `up`, **and**
- the button is the primary button (`button === 0` for mouse,
  `isPrimary` for touch).

A second `click` within the platform double-click interval (default
500 ms) on the same target also dispatches `dblclick`.

### Right-click

`contextmenu` is dispatched on the resolved target *after* the
`pointerdown` → `pointerup` pair for `button === 2`. Widgets may
`preventDefault()` to suppress the browser's native menu.

### Cancel

    pointercancel

Dispatched to the captured target (if any) or to the resolved target
at the cancel coordinates otherwise. Releases any active pointer
capture. No `click` follows a cancelled press.

### Legacy mouse shim

The five-event legacy sequence

    mouseover → mouseenter → mousedown → mouseup → click

is emitted as a **trailing shim** after the pointer-family events for
each gesture. This is what older libraries (e.g. jQuery plugins, D3v3)
expect, and is what we observe on real browsers when a primary
pointer event fires. The shim is gated by `pointerType === "mouse"`
(touch-driven gestures don't synthesize legacy mouse events until the
gesture ends; this matches Chromium behaviour).

## pointer-events: none semantics

CSS `pointer-events: none` is not visible to a canvas. The router
implements the same semantics on the **semantic tree** instead:

- Each semantic node carries a `pointer_events: PointerEventsMode`
  field with variants `Auto` (default) and `None` (and in future
  `VisiblePainted` etc. — currently elided).
- The hit-tester, when walking the painted-pixel stack at `(cx, cy)`,
  **skips** any node whose mode is `None` and continues to the node
  beneath. The first node with `Auto` is returned as the hit target.
- For ancestor-walk purposes, `pointer_events: None` ancestors are
  *also* skipped: an event dispatched to a descendant does not bubble
  through a `None` ancestor's listeners.
- The mode is set by the widget author at construction time and may
  be toggled at runtime; the change takes effect on the next event.

This gives jet widgets the same "invisible to pointer" knob that DOM
authors get from CSS, without requiring the canvas itself to know
anything about CSS.

## Capture and release

`setPointerCapture(pointerId)` redirects all subsequent events for
that pointer to the captured widget, **regardless of where on the
canvas the pointer moves**. This is the standard primitive for
implementing drag, slider thumbs, and modal interaction within a
single gesture.

The router maintains a `PointerCaptureMap`:

    PointerCaptureMap : Map<PointerId, SemanticTargetId>

On synthesized `pointerdown`, a widget may call
`event.target.setPointerCapture(event.pointerId)`. The router records
the binding and, until release, the dispatch step bypasses the
hit-tester for that pointer and dispatches directly to the captured
target's listeners (still synthesizing the full ancestor bubble
sequence rooted at the captured target).

Capture is released on any of:

- explicit `releasePointerCapture(pointerId)` by the widget
- `pointerup` for that pointer (matches spec — implicit release)
- `pointercancel` for that pointer
- the captured widget being removed from the semantic tree

When capture is active, `pointerover` / `pointerout` events continue
to fire on the **actual** target under the pointer (per spec), but
`pointermove` / `pointerup` / `pointerdown` go to the captured target.

## Test surface

Every synthesized event is mirrored into a **circular event buffer**
inside WASM (default capacity 4096 entries, drop-oldest on overflow).
The buffer is exported via:

    wasm_event_buffer_drain() -> Vec<SynthesizedEventRecord>
    wasm_event_buffer_clear()
    wasm_event_buffer_set_capacity(n: usize)

Each `SynthesizedEventRecord` carries:

- monotonic sequence number
- event type (`pointerdown`, `click`, ...)
- target id (semantic-tree node id)
- canvas-space `(x, y)`
- pointer id, button, buttons, pointerType
- whether `preventDefault` / `stopPropagation` was called by any
  listener during dispatch
- the native event's `timeStamp`

This buffer is the **direct producer** for the
`pointer-hitmap.json` test channel introduced in #2139: the test
runner drains the buffer between assertions and serializes the
records into the snapshot file that WPT comparisons run against.

Because the buffer captures the *synthesized* stream — i.e. exactly
what widgets observe — tests assert on the contract jet promises to
its widget authors, not on the raw native event noise.

## Out of scope

This ADR specifies the **router**. The following are explicitly
deferred to sibling issues and will receive their own ADRs:

- **Hit-tester implementation** — the body of `wasm_hit_test`,
  including the painted-pixel stack data structure, semantic-vs-
  visual layering rules, and z-index resolution. → **#2165**
- **WPT subset adoption** — which Web Platform Tests we run, how we
  shim their harness, and the conformance gate. → **#2166**
- **Nested scroll containers** — wheel / pointermove redirection
  into in-canvas scroll regions, scroll chaining, and overscroll
  behaviour. → **#2167**
- **`touch-action` parity** — per-region pinch/zoom policy, the
  passive-listener split, and gesture-recogniser delegation. → **#2168**
- **Drag-and-drop** — HTML5 DnD event synthesis (`dragstart`,
  `dragover`, `drop`, etc.) on top of this router's pointer stream,
  including the data-transfer object. → **#2169**
- **Hover and cursor** — `cursor:` style resolution and OS cursor
  updates driven by the `pointerover` transitions emitted here. → **#2170**

## Follow-ups

1. **Event-coalescing policy.** `getCoalescedEvents()` parity for
   high-frequency `pointermove` streams. Currently jet dispatches
   every native move; batching may be needed at 240 Hz+.
2. **Composed-path semantics.** `event.composedPath()` for shadow-DOM-
   like nesting once we introduce widget-internal sub-trees.
3. **Trusted-event flag.** Synthesized events currently mark `isTrusted
   = false`; investigate whether parity tests expect `true`.
4. **Keyboard router.** The pointer router is the model for a sibling
   `KeyboardInputRouter` (focus traversal, key event synthesis,
   `keydown` → `keypress` → `keyup` sequence). File as its own epic
   once #2137 closes.
5. **Accessibility tree.** An a11y tree should ride on the same
   semantic tree the hit-tester walks; ARIA event semantics may
   require extending `SynthesizedEventRecord` with accessibility
   metadata.
6. **Pointer-lock.** `requestPointerLock()` redirects raw pointer
   deltas without a position. Pairs with the capture map but uses
   different math; design once a real consumer appears.
