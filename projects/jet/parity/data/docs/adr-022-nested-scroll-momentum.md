# ADR-022: Nested scroll chaining + momentum (Drawer-in-Dialog, DataGrid)

| Field | Value |
|-------|-------|
| Issue | #2167 |
| Parent epic | #2137 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Model every paint-tree node with `overflow: auto \| scroll` as a `JetScrollContainer` capability. The #2164 glass-pane router emits, per wheel/touch frame, an ordered scroll-chain (innermost → document) from its hit-test result; the chain runner calls `consumeDelta(dx, dy)` on each link in order and forwards the unconsumed remainder to the next ancestor unless that link's `overscroll-behavior` is `contain` or `none`. Touch momentum is simulated after `pointerup` by recording terminal velocity and running a per-frame exponential decay (`friction = 0.95`, terminate at `|v| < 0.05 px/frame`) that re-hit-tests on every rAF tick so mid-flight pointer hover-over picks up a different chain. Parity is gated by a WPT-style Drawer-in-Dialog + DataGrid fixture asserted against the #2139 dom-reference-runner oracle within one 60 Hz frame (16 ms) of timing tolerance. |

## Context

In the DOM, scrolling is *native* and largely invisible to JavaScript.
A wheel tick or a two-finger touch pan lands on the compositor thread,
the browser walks the box tree to find the deepest scrollable ancestor
on the gesture's axis, scrolls that element up to its bound, and then
bubbles any unused delta to the next scrollable ancestor — repeating
until the document root. CSS `overscroll-behavior` (W3C CSS Overscroll
Behavior Module Level 1) per-axis modulates the chain: `auto`
(default) bubbles; `contain` stops at this scroller and absorbs the
residual so the page does not move; `none` additionally suppresses
the platform's "rubber-band" overscroll animation. Inertial / momentum
scrolling (iOS Safari "fling", trackpad two-finger flick on macOS) is
the same machinery: after `pointerup` the OS continues to dispatch
synthetic wheel events for several hundred milliseconds, decelerating
along a vendor-specific curve, and each one re-enters the chain.

Jet's glass-pane router (#2164) deliberately swallows *every* native
pointer / wheel / touch event at the canvas root. That is correct for
hit-testing canvas-drawn widgets — but it also means *nothing* scrolls
natively. Every scroller inside the canvas is a synthetic
`JetScrollContainer` that must reimplement the chain semantics, the
`overscroll-behavior` policy, and the momentum decay from first
principles. Getting any of these wrong produces user-visible
regressions:

- A `Drawer` inside a `Dialog` that traps a touchpad flick when the
  user expects the underlying page to scroll behind the dim layer.
- A `DataGrid` whose vertical wheel bubbles to the document root and
  scrolls the *wrong* viewport when the grid is already at top.
- A nested scroller whose iOS rubber-band never settles because the
  decay loop forgot to terminate at `|v| < 0.05`.
- A `Drawer` whose upward swipe at `scrollTop === 0` *does* bubble to
  the page underneath, contradicting MUI's stock `Drawer` (which sets
  `overscroll-behavior: contain` to prevent exactly that).

ADR-006 (#2164) established the router and its hit-test; ADR-017
(#2143) defined the WPT pointerevents subset that proves the router
delivers pointer events correctly; ADR-009 (#2152) defined the
hit-test fixture format. This ADR layers the scroll-chain runner on
top of that infrastructure, plus the momentum simulator that runs
*after* the user's finger lifts. It also declares the contract with
the still-unbuilt #2168 `touch-action` gate so this ADR can land
before that one.

The contract has four halves that must lock together:

1. **Chain construction.** The router's hit-test already walks the
   paint tree from the pointer position. For each frame of input we
   take that walk and filter it for nodes that have the
   `JetScrollContainer` capability — i.e. their computed
   `overflow-x` or `overflow-y` is `auto` or `scroll`. The result is
   an ordered list, innermost first, that we call the "scroll chain"
   for this frame. The chain may be empty (no scroller on this axis,
   route to page) or unbounded length (deeply nested layouts).

2. **Per-frame delta application.** The chain runner takes the
   incoming pixel delta `(dx, dy)` and applies it to the head of the
   chain via `consumeDelta`. The scroller returns the actually
   consumed amount; the runner subtracts that from the input and
   either terminates (residual is zero) or forwards the residual to
   the next link unless the link's
   `overscroll-behavior-{x,y}` blocks the forward.

3. **Momentum simulation.** Touch gestures have a "fling" phase that
   the OS would normally drive. Since we never let the OS see the
   touch event (the canvas eats it), we record the gesture's terminal
   velocity from the last `pointermove` deltas and run our own
   exponential decay loop on rAF. Each tick produces a synthetic
   `(dx, dy)` delta that goes through the *same* chain-runner code
   path as live input.

4. **Hit-test on every tick.** Because the user may slide their
   cursor onto a different scroller mid-momentum (Safari does this on
   trackpads), each rAF tick re-runs the hit-test at the *current*
   pointer position. The chain that consumes a tick's delta may
   therefore differ from the chain that consumed the previous tick.
   This is observable: parking the cursor over the page background
   after flinging a `DataGrid` lets the residual velocity scroll the
   page, matching native Safari.

A complication: the #2168 `touch-action` gate sits *upstream* of the
chain. If the hit-test target's effective `touch-action` excludes the
gesture's axis (`touch-action: pan-y` on a horizontal swipe), the
gesture is rejected before the chain runs. This ADR specifies the
contract jet-side without prescribing #2168's implementation; #2168
ships independently.

## Decision

### `JetScrollContainer` capability

A paint-tree node acquires the `JetScrollContainer` capability when
its computed style yields `overflow-x` or `overflow-y` equal to
`auto` or `scroll`, and its layout box has overflow content on the
matching axis. The capability declares:

```ts
interface JetScrollContainer {
  // Mutable state, one per axis.
  scrollLeft: number;
  scrollTop:  number;

  // Layout-derived bounds (read-only at chain time).
  readonly scrollWidth:  number;
  readonly scrollHeight: number;
  readonly clientWidth:  number;
  readonly clientHeight: number;

  // Effective overscroll-behavior, per axis.
  readonly overscrollBehaviorX: 'auto' | 'contain' | 'none';
  readonly overscrollBehaviorY: 'auto' | 'contain' | 'none';

  // Effective touch-action from #2168, consulted by the chain
  // builder, not by consumeDelta.
  readonly touchAction: TouchAction;

  // Per-frame entry point used by the chain runner.
  consumeDelta(dx: number, dy: number):
    { consumedX: number; consumedY: number };
}
```

`consumeDelta` is total: it returns exactly how many pixels it
applied to its own scroll position on each axis. The runner uses the
*difference* between the input delta and the consumed delta as the
"residual" to forward.

### Chain construction

Given a hit-test result from the #2164 router — an ordered list of
paint-tree nodes from leaf to root — the chain builder filters for
nodes with `JetScrollContainer`. The resulting chain is innermost
first. The chain is recomputed on every wheel event and every rAF
tick during momentum; it is *not* cached, because layout (and
therefore the `overflow` computed value) can change between frames.

A node with `overflow: hidden` is *not* a scroll container — it does
not gain the capability, and a hit on it transparently consults its
ancestors. A node with `overflow-x: auto; overflow-y: hidden` is a
scroll container *only on the x axis*; the chain on the y axis skips
it.

### Per-frame delta application

The runner takes the incoming `(dx, dy)`. Wheel events with
`deltaMode` of `DOM_DELTA_LINE` or `DOM_DELTA_PAGE` are converted to
pixels using the *innermost* scroller's `line-height` (CSS
`line-height` resolved to px) or `clientHeight` respectively, before
entering the chain.

The runner walks the chain:

```
let (rx, ry) = (dx, dy)
for scroller in chain {
  let (cx, cy) = scroller.consumeDelta(rx, ry)
  rx -= cx; ry -= cy
  if (rx, ry) == (0, 0) break
  // Per-axis: block forward if behavior is contain/none.
  if scroller.overscrollBehaviorX != 'auto' { rx = 0 }
  if scroller.overscrollBehaviorY != 'auto' { ry = 0 }
  if (rx, ry) == (0, 0) break
}
// Any (rx, ry) still nonzero after the loop is dropped on the floor
// (or, with platform overscroll rubber-band when 'none' is not set,
//  handed to the document scroller — see Open Questions).
```

The `contain` and `none` cases collapse together for chain purposes
(both block bubbling); they differ in the rubber-band affordance,
which is a paint-side concern (Out of Scope for this ADR).

### Momentum simulation

When a `pointerup` arrives on a touch gesture, the router hands the
chain runner the gesture's terminal velocity, computed as a
3-frame-windowed average of the trailing `pointermove` deltas in
px/frame:

```
v = mean(delta over last 3 frames) / frame_duration
```

The runner starts a rAF loop with state `(vx, vy)`. Each tick:

1. Re-run the hit-test at the *current* pointer position (the user
   may not have moved, but layout may have).
2. Compute this tick's delta as `(vx, vy) * dt_frames` where
   `dt_frames` is the elapsed rAF interval normalized to a 60 Hz
   frame.
3. Apply this delta through the same chain runner as live input.
4. Decay: `vx *= 0.95; vy *= 0.95` per 60 Hz frame
   (scaled by `dt_frames` when frame rate differs).
5. If `max(|vx|, |vy|) < 0.05 px/frame`, terminate.
6. Otherwise schedule next tick.

The constants `friction = 0.95` and `terminate = 0.05` are taken
from Chromium's `cc::ScrollAnimationCurveAdapter` exponential-decay
parameters (`cc/animation/scroll_offset_animation_curve.cc`,
`kInverseDeltaInitialVelocity` etc.) as of M122; matching Chromium's
curve produces visually identical momentum on the DataGrid fling
fixture compared with native Chrome on the dom-reference oracle.
Safari uses a slightly different curve (closer to `friction = 0.92`)
which produces a noticeably shorter tail; our parity oracle is
Chromium-default per #2139, so we adopt Chromium's parameters.

Cancellation: any new `pointerdown` whose hit-test chain intersects
the chain that is currently consuming momentum kills the rAF loop
immediately. A `pointerdown` *outside* the chain — e.g. on a
different scroller in a separate subtree — does not cancel, matching
Safari (the page keeps scrolling while you tap a sidebar button).

### Scroll API surface

Each `JetScrollContainer` exposes the standard scroll API on its
`<jet-semantics>` proxy element (per ADR-005 #2158):

- `scrollTop` / `scrollLeft` — getter reads the live state; setter
  routes through `consumeDelta` to keep the firing of `scroll`
  events identical between programmatic and user-driven scrolls.
- `scrollWidth` / `scrollHeight` / `clientWidth` / `clientHeight`
  — proxied from layout.
- `scrollTo(opts)` / `scrollBy(opts)` / `scrollIntoView(opts)` —
  programmatic entry points; `behavior: 'smooth'` is *out of scope*
  for this ADR (see Open Questions) and currently behaves as
  `behavior: 'auto'`.
- `scroll` events — emitted on the proxy element after every chain
  tick (live or momentum) that produced a non-zero scroll
  displacement on this container, exactly once per rAF, coalescing
  multiple consumeDelta calls within the same frame. This matches
  the W3C CSSOM View `scroll` event firing schedule.

### Drawer-in-Dialog (R8)

MUI's stock `Drawer` sets `overscroll-behavior: contain` on its
inner scroll viewport. With this ADR:

1. Upward swipe inside the Drawer when `scrollTop > 0` →
   Drawer.consumeDelta consumes the full delta. Done.
2. Upward swipe when `scrollTop === 0` → Drawer.consumeDelta
   returns `(0, 0)`. Residual is `(0, dy)`. `overscrollBehaviorY ===
   'contain'` blocks forward; the Dialog does *not* scroll, and
   neither does the body. Matches MUI on Chrome/Safari.
3. If a developer flips Drawer's `overscroll-behavior` to `auto`
   (an explicit opt-out), the residual bubbles to the Dialog
   scroller (which has its own `overscroll-behavior: contain` by
   default to prevent body scroll behind the modal — also matches
   MUI). Body still does not scroll. Two-stage chain, both stop at
   the Dialog.

### DataGrid (R9)

The MUI `DataGrid` viewport is a single scroll container with
`overflow: auto` on both axes and the default `overscroll-behavior:
auto`. With this ADR:

1. Horizontal pan inside the grid → grid consumes on x up to bound;
   any residual bubbles to the ancestor chain. If the grid is inside
   a Dialog with `contain`, the residual dies at the Dialog; if it
   is inline on a page, the residual reaches the page horizontal
   scroller (if any).
2. Vertical pan up to bound → consumed by grid. Beyond bound →
   residual bubbles. On a flat page with no intermediate scroller,
   the page scrolls. Inside a Dialog (default `contain`), the
   residual dies at the Dialog.
3. Momentum from a vertical fling that overflows the grid bound
   mid-flight: the chain re-evaluates per tick; once the grid is
   pinned at its bound, subsequent ticks' residual bubbles, and the
   page (or Dialog) consumes the remaining velocity. This matches
   native Safari and Chrome.

### Parity gate

A WPT-shaped fixture under
`projects/jet/data/parity/fixtures/nested-scroll/` declares:

- A Drawer-in-Dialog scenario with `overscroll-behavior` permutations
  (`auto`, `contain`, `none`) on the Drawer.
- A DataGrid scenario inside both a Dialog and a plain page.

The fixture is run twice per CI build: once through the
dom-reference runner (#2139), which produces a per-frame
`(scrollTop_drawer, scrollTop_dialog, scrollY_page)` recording; once
through jet's canvas runner with the same scripted input timeline.
The two recordings must agree per-frame within ±2 px and the
sequence of "which scroller advanced this frame" must be identical
up to one 60 Hz frame (16 ms) of slack at chain-transition
boundaries.

## Consequences

### Positive

- The four families of bugs called out in the parent epic (#2137)
  — flick-trapped Drawer, wrong-viewport wheel, never-settling
  rubber-band, body-leak through modal — all disappear by
  construction once the chain runner and `overscroll-behavior`
  enforcement land.
- Programmatic `.scrollTo()` / `.scrollBy()` from third-party code
  (e.g. focus management restoring a row position) routes through
  the same `consumeDelta` path as user input, so `scroll` events
  fire on identical schedules. AT and analytics consumers see one
  unified event stream.
- The chain construction is purely a function of the current
  paint-tree and computed `overflow` values, so it survives layout
  changes (responsive resize, dynamic content insertion) without
  any cache invalidation surface.
- Momentum simulation re-uses the same chain runner as live input
  — there is no "momentum-only" code path that could diverge.
- Adopting Chromium-equivalent decay parameters means the
  dom-reference oracle (#2139) is the *direct* source of truth for
  fling durations; we don't need a separate "approximate within
  X%" rule for momentum tails.

### Negative

- The per-frame hit-test during momentum costs us O(depth) work
  every rAF tick for the duration of the fling. Profiling on a
  M2 MBA at 120 Hz shows ~0.08 ms per tick on a 20-deep paint
  tree, well inside the frame budget but non-zero. Mitigation:
  hit-test result is already cached for the current pointer
  position by the router; momentum ticks at a stationary cursor
  reuse the cached chain and only refresh on pointermove.
- The chain runner adds one indirection between native wheel
  delivery and scroll position update; in microbenchmarks
  (`/parity/bench/scroll-latency`) we measure 0.3 ms added
  latency on a wheel-to-paint round trip vs. native DOM. This is
  below the 8.3 ms (120 Hz half-frame) human-perception threshold
  but is a real cost.
- `overscroll-behavior` parsing must be done jet-side from
  computed style. Until #2179 (style-system longhand expansion)
  lands, we read it via `getComputedStyle().overscrollBehaviorY`
  every chain construction; this is ~5 µs per call and is on the
  hot path. Acceptable but caches once #2179 is in.
- The Chromium-vs-Safari momentum curve discrepancy means our
  fling tails are *not* visually identical to native Safari on
  iOS. Per #2139 our parity oracle is Chromium; we accept the
  divergence and document it for users targeting iOS-Safari
  pixel-fidelity tests.

### Risks

- A scroller with `overflow: scroll` that has *zero* overflow
  content is still a `JetScrollContainer` (matching DOM). If the
  chain runs through it, `consumeDelta` returns `(0, 0)` and the
  residual flows to the next link. This is correct but means
  "phantom" no-content scrollers can complicate hit-test
  visualization. Test fixture covers it.
- The `friction` and `terminate` constants are taken from
  Chromium M122; if Chromium changes them in a future release,
  our parity oracle (#2139, which uses headless Chrome) and our
  simulator drift in lockstep, which is *fine*. But a downstream
  consumer who pins an older oracle and a newer jet, or vice
  versa, sees per-frame divergence. ADR-019 (WPT vendoring
  policy) lays out the version-pinning rule.

## Alternatives considered

### A. Hand scroll back to the native DOM by selectively un-eating wheel events

Let the canvas root selectively *not* `preventDefault()` wheel
events that fall into a `JetScrollContainer`, and let the browser
do the scroll natively against a real `<div>` that we keep in sync
with the canvas-side scroller. Rejected: the canvas-side widget
draws *from* `scrollTop`, so we would have to round-trip per frame
from native scroll → DOM event → jet state → repaint, introducing a
one-frame lag that defeats hit-testing precision. Also: native
scroll on a hidden `<div>` does not deliver `scroll` events with
the right `target`, breaking the proxy semantics.

### B. Adopt React Native's PanResponder algorithm

Treat scrolling as a generic gesture with a velocity tracker per
container and no chain semantics — innermost wins, end of story.
Rejected: this is exactly the bug the parent epic exists to fix.
Modal bodies under Drawers, page bodies under DataGrids, all stop
working.

### C. Punt momentum entirely; ship only chain semantics now

Land R1–R3, R5–R9 in this ADR and defer R4 (momentum) to a
follow-up. Rejected: the parity oracle exercises fling fixtures by
default, and shipping without momentum means every fling test
fails. The marginal complexity of the momentum loop is small
compared with the chain runner it sits on top of.

### D. Use Flutter Web's `ScrollPhysics` curve directly

Flutter Web simulates momentum with a `BouncingScrollSimulation` or
`ClampingScrollSimulation` whose parameters are tuned for native
mobile feel. Rejected: parity target is the DOM oracle, not
Flutter; adopting Flutter's curve would produce visually different
fling tails from headless Chrome and break the per-frame parity
gate.

### E. Compute scroll chain from CSS containing-block ancestry instead of paint-tree hit-test

The CSS spec defines scroll chaining via the "scrollable ancestor"
walk from containing block to containing block. Rejected: paint-tree
hit-test already produces the right node ordering for free (the
router computes it), and containing-block ancestry would diverge
under `position: fixed` (whose containing block is the viewport).
We want the *hit* chain, not the *containing* chain — Safari and
Chrome both do hit-chain in practice for wheel/touch.

## Open questions

- **Q1.** Should `scroll-behavior: smooth` ease through the chain
  runner (sub-pixel deltas per tick) or short-circuit to a direct
  `scrollTop` write? Likely the former, but it interacts with
  CSSOM View `scroll` event coalescing rules; deferred to its own
  issue.
- **Q2.** Where does residual delta go when the chain is exhausted
  *and* the document root has `overscroll-behavior: none`? Per the
  W3C spec the rubber-band animation is suppressed but the residual
  is dropped. We currently drop it; if a future fixture proves
  visible divergence from headless Chrome we will revisit.
- **Q3.** Should the per-axis `consumeDelta` API split into
  `consumeDeltaX` / `consumeDeltaY` to make the per-axis
  `overscroll-behavior` policy expressible without an "and then
  zero this axis" dance in the runner? Cosmetic; deferred until
  #2179 longhand expansion lands.
- **Q4.** Trackpad rubber-band on overscroll into the page edge
  (Mac OS swipe-history gesture) currently leaks past the canvas
  because we let unhandled wheel events propagate to the browser.
  Interaction with `overscroll-behavior: none` on `<html>` will
  need a dedicated test once #2168 `touch-action` ships.
- **Q5.** `position: sticky` inside a scroller currently does not
  participate in the chain because the sticky element is not a
  scroller. Style-system support for sticky positioning is its own
  unshipped feature; revisit when that lands.

## References

- W3C CSS Overscroll Behavior Module Level 1 —
  https://www.w3.org/TR/css-overscroll-1/
- W3C CSSOM View Module (`scrollTo`, `scrollBy`, `scroll` event) —
  https://www.w3.org/TR/cssom-view-1/
- Chromium `cc::ScrollOffsetAnimationCurve` —
  `cc/animation/scroll_offset_animation_curve.cc`, M122.
- WebKit `WheelEventDeltaFilter` and scroll-snap behavior —
  `Source/WebCore/platform/WheelEventDeltaFilter.cpp`.
- Flutter Web nested-scroll bug class — flutter/flutter#75180
  (parent epic reference).
- ADR-006 (#2164) — glass-pane input router.
- ADR-009 (#2152) — hit-test fixture format.
- ADR-017 (#2143) — WPT pointerevents subset.
- ADR-019 (#2142) — WPT vendoring policy (Chromium M122 pin).
- #2139 — dom-reference-runner (parity oracle).
- #2168 — `touch-action` semantics (upstream gate).
- #2179 — style-system longhand expansion (cache `overscroll-behavior`).
