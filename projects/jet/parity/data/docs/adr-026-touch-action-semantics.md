# ADR-026: Touch-action semantics — browser-vs-JS gesture ownership

| Field | Value |
|-------|-------|
| Issue | #2168 |
| Parent epic | #2137 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Project per-jet-node `touch-action` declarations onto the host `<canvas>` element by combining a `touch-action: none` default on the canvas root with a router-driven gesture-handoff path. The #2164 glass-pane router computes the *effective* `touch-action` at `pointerdown` as the most-restrictive intersection along the hit-test ancestor chain, latches it for the lifetime of that pointerId (W3C "consulted at gesture start"), then either (a) synthesizes a single `pointercancel` and stops dispatching — releasing the gesture back to the browser's compositor for native scroll / pinch — when the gesture is permitted, or (b) calls `event.preventDefault()` on the underlying native event and continues routing synthesized pointer events when forbidden. The per-widget proxy emitter (#2158) is extended with a `touch-action` column: `manipulation` for `button` / `link` / `checkbox` (keeps the ~5 ms compositor latency budget while still firing `click`), `pan-x pan-y` for scrollers, `none` for custom-gesture widgets (Slider, drag-handle, signature pad). React/JSX consumers may override per-widget via `style={{ touchAction: '…' }}`. Parity is gated by a WPT-style fixture per `touch-action` value asserted against the #2139 dom-reference-runner. |

## Context

`touch-action` is the W3C-defined CSS property
([Pointer Events Level 2 §6](https://www.w3.org/TR/pointerevents2/#the-touch-action-css-property))
that lets a page declare, *before* any gesture begins, which touch
behaviors the user agent is permitted to handle natively. The
accepted value set is
`{auto, none, pan-x, pan-y, pan-left, pan-right, pan-up, pan-down,
pinch-zoom, manipulation}`, with multi-value forms such as
`pan-x pinch-zoom` parsed as the *union* of their components.
The contract is enforced at the compositor level: by the time
`pointerdown` fires the browser has already decided what gesture
class it will own, and JavaScript receives only what is left over.

For jet this is the *only* part of the pointer channel where the
browser executes contract semantics **before** anything we wrote
runs. Hit-testing, capture, routing, dispatch — every other layer
discussed in ADR-006 (#2164), ADR-009 (#2148), and ADR-022 (#2167)
is something jet *adds* on top of the canvas. `touch-action` is
the inverse: the browser consumes the gesture, or doesn't, *based
only on the CSS value present on the host `<canvas>` element when
the touch lands*. We cannot reimplement it. We can only project
per-jet-node semantics onto the single canvas element in a way the
browser will honor.

This matters for two distinct reasons.

The first is **correctness**. Jet's glass-pane router (#2164) is
JS-side. If the user starts a horizontal swipe on a canvas-drawn
carousel and the host canvas's `touch-action` is `auto` (the CSS
default), the browser interprets the swipe as a horizontal page
scroll and *never dispatches `pointermove`* — the carousel can
never respond, no matter what the router does. The router does
not see what the compositor consumed. Conversely, if the canvas's
`touch-action` is `none` and the user attempts a vertical pan to
scroll the page, the browser is *forbidden* from scrolling and the
page is dead under their finger until JS releases the gesture.
Both failure modes are real shipped-product bugs: the carousel
that "doesn't work on iOS Safari" and the canvas-app page that
"won't scroll on Android Chrome".

The second is **latency**. The user-perceived response time of
touch scrolling is set by where the scroll handler lives. When
the browser owns the gesture (`touch-action: auto` over a
scrollable region, or `touch-action: manipulation` for a tap),
the scroll runs on the compositor thread independently of the
main-thread event loop; measurable end-to-end latency is in the
3 – 7 ms band even on a busy main thread. When JS owns the gesture
(`touch-action: none`, our default for the canvas root), every
frame round-trips through `requestAnimationFrame`, jet's
synthetic dispatch, and the consumer's handler; under a typical
React commit load that band is 16 – 50 ms, and worse on long-task
spikes. The 10× cost on the wrong-default path is exactly why we
cannot universally set `touch-action: none` on every node "to be
safe": doing so would punish every tap and pan in the app to
guard against the minority of widgets that actually need
JS-owned gestures.

So jet ships *both* defaults at once. The canvas root keeps
`touch-action: none` so we can guarantee JS delivery to the
router — that is the baseline contract for canvas-drawn UI. The
per-widget semantics emitter (#2158, ADR-005) then advertises a
*less-restrictive* `touch-action` on each proxy node where the
widget can tolerate the browser owning some subset of gestures.
The router watches for these proxy-node values during hit-test
and uses the gesture-handoff path to *release* the gesture back
to the browser when the effective value permits it. The result
is JS-owned gestures only where they are required, with the
compositor-fast path everywhere else.

The Pointer Events WPT subset that ADR-017 (#2153) gates against
includes the touch-action conformance tests; this ADR is the
implementation specification those fixtures will exercise.

## Decision

### Per-node declaration and parsing

Every jet paint-tree node accepts a `touch-action` field. The
accepted set is the W3C value set verbatim:
`auto | none | pan-x | pan-y | pan-left | pan-right | pan-up |
pan-down | pinch-zoom | manipulation`. Multi-value forms
(`pan-x pinch-zoom`, `pan-y pinch-zoom`, etc.) are parsed as the
union of their components. The Pointer Events Level 2 grammar is
the source of truth — values jet does not understand are a
parse error, never a silent default. `manipulation` is treated
as the shorthand the spec defines (`pan-x pan-y pinch-zoom` plus
double-tap-zoom suppression).

### Effective `touch-action` is intersection at pointerdown

Per R2 and R7, the effective `touch-action` for a `pointerdown`
coordinate is the *most-restrictive intersection* of the
`touch-action` values of every node on the hit-test ancestor
chain (from the canvas root down to the leaf widget at that
coordinate). The intersection is computed once, at
`pointerdown`, and *latched* for the lifetime of that pointerId.
Later `pointermove` deltas do not re-evaluate — this matches the
W3C "touch-action only consulted at gesture start" rule and is
what makes the contract predictable for the user when a swipe
crosses a region boundary mid-gesture.

The intersection algebra is defined per axis:

- `auto` ∩ X = X (auto is identity).
- `none` ∩ X = `none` (none is absorbing).
- `pan-x` ∩ `pan-y` = `none` (no axis remains).
- `pan-x` ∩ `pan-left` = `pan-left` (sub-direction wins).
- `manipulation` ∩ `none` = `none` (manipulation is not stronger).
- `pinch-zoom` ∩ `pan-y` = `pan-y` (pinch and pan are independent;
  the pan-axis survives but pinch is dropped because pan-y alone
  does not include it).

The full table is enumerated in
`jet-touch-action-intersection.md` (named in the issue's Spec
Plan); a property test sweeps all 100 ordered pairs.

### Host canvas default is `none`

The host `<canvas>` element ships with CSS `touch-action: none`
(R3). This is what guarantees the router sees every touch event
— if we left the canvas at `auto`, the browser would consume
two-finger pans as native page scroll before the router ran,
and no per-node intersection would help. Consumers who embed a
canvas inside a scrollable page must opt into native scroll on
the canvas region via the React style override; that is by
design, not an oversight (see *Consumer override* below).

### The gesture-handoff path

The router branches on the latched effective `touch-action` at
the first `pointermove` whose axis the user's gesture has
established:

- **Effective value permits the gesture** (e.g. user pans
  vertically and effective value is `pan-y` or `auto`):
  the router synthesizes a *single* `pointercancel` for the
  active pointerId, removes the pointer from its capture map,
  and stops dispatching further events from that gesture (R4).
  Critically, the router does **not** call `preventDefault()`
  on the underlying native event in this branch — that is what
  allows the browser's compositor to take over from the next
  native frame and scroll / pinch the host canvas natively.
  The single `pointercancel` is what the consumer's widget
  handler observes; it can release any local state.

- **Effective value forbids the gesture** (e.g. user pans on
  `touch-action: none`): the router calls
  `event.preventDefault()` on the underlying native event and
  continues dispatching synthesized pointer events through the
  normal #2164 path (R5). The browser is now blocked from
  consuming the gesture; JS delivery is guaranteed for the
  remainder of the pointerId's life.

This is the only path that satisfies both halves of the
contract: native gestures stay on the compositor thread when
allowed, and JS gestures stay reliable when required.

### `manipulation` double-tap-zoom suppression

`manipulation` is the spec's "fast tap" value: pan and
pinch-zoom remain allowed but the platform's 300 ms double-tap-
to-zoom delay is suppressed. R6 implements this in the router
by tracking the last `pointerup` per pointer-type within a 300 ms
window and ≤ 10 px move bound; on the second tap inside that
window the router emits `click` synchronously and discards the
zoom intent. This is what gives the per-widget default of
`manipulation` on buttons / links / checkboxes its latency win:
the click fires without the browser's `~300 ms` deferral, while
the compositor still owns the underlying scroll because the
gesture itself never re-entered the JS-dispatch path.

### Per-widget proxy mapping (#2158 extension)

ADR-005 (#2143) defined the per-widget semantics emitter that
writes ARIA role / state onto each widget's proxy node. This
ADR adds a `touch-action` column to that table:

| Widget class | Proxy `touch-action` | Rationale |
|---|---|---|
| `button`, `link`, `checkbox`, `menuitem` | `manipulation` | Browser still owns scroll/pinch; suppresses 300 ms double-tap delay so `click` fires immediately. |
| `Panel` (scrollable), `DataGrid`, `ListView` | `pan-x pan-y` | Browser owns scrolling along both axes; non-pan gestures (pinch, long-press) reach JS. |
| `Slider`, `drag-handle`, `signature-pad`, `Carousel` | `none` | Custom gestures owned by JS end-to-end. |
| `image-viewer`, `zoomable-canvas` | `pinch-zoom` | Browser owns pinch; pans reach JS (canvas owns its own pan / drag). |
| Decorative / non-interactive nodes | inherit (no proxy emission) | Falls through to ancestor; usually canvas root's `none`. |

The defaults above are what the emitter ships; consumers
override per widget via the React style prop documented next.

### Consumer override

React / JSX consumers override the per-widget default by passing
`style={{ touchAction: 'none' }}` (or any other valid value) on
the widget element. Jet's React adapter forwards the value to
the proxy node's `style` attribute; the browser honors it
without further routing logic. The override applies only to the
specific widget — the intersection rule still takes effect on
hit-tests that include both the override target and an ancestor
with a more restrictive value (most-restrictive wins).

This is the documented escape hatch for cases the per-widget
table did not anticipate: e.g. a `button` inside a custom
gesture surface that needs `touch-action: none` to prevent its
own click from being swallowed by the parent's pan.

### Pinch-zoom on the canvas root

The browser's pinch-to-zoom is on by default for `<canvas>`.
For canvas apps that own pinch (an image viewer, a zoomable
diagram) the canvas host emits `touch-action: pinch-zoom` on
the relevant subtree — that lets the browser still scroll
on pans but blocks it from pinching, leaving pinch for JS.
For apps that own everything, `touch-action: none` is the
correct value (already the canvas-root default). The R4
handoff path ensures these values reach the host element
consistently.

## Consequences

### Positive

- **Latency budget preserved** for the majority case: tap on a
  `Button` runs on the compositor with no JS round-trip, hitting
  the 3 – 7 ms band; scroll on a `Panel` is native scroll on the
  compositor thread.
- **Custom-gesture widgets work** without consumer wiring:
  Slider / Carousel / signature pad declare `touch-action: none`
  on their proxy and the router guarantees JS delivery via
  `preventDefault()`.
- **Predictable cross-region gestures** because the intersection
  is latched at `pointerdown`; a swipe that crosses a
  `Button`-inside-`Panel` boundary never flips ownership mid-air.
- **WPT-conformant** by construction: the value set, the
  intersection rule, and the gesture-start-only re-evaluation
  all match Pointer Events Level 2 §6 verbatim.
- **Consumer-debuggable**: the proxy node's `touch-action` is
  visible in DevTools' "Computed" panel exactly as authored.

### Negative

- The intersection table is 100 ordered pairs; getting any one
  wrong is a hard-to-spot bug. We rely on a property test that
  sweeps the table and asserts agreement with the dom-reference
  oracle (#2139) on a synthetic fixture per pair.
- The canvas-root `touch-action: none` default means consumers
  embedding jet inside a scrollable host page must explicitly
  opt their canvas region back into native scroll. This is
  documented but is a real onboarding friction.
- `manipulation`'s 300 ms / 10 px suppression window is timing-
  sensitive; on emulators or under heavy main-thread load the
  double-tap detector can drift. Mitigated by reading the
  platform's `TouchEvent.timeStamp` (compositor-supplied) rather
  than `performance.now()`.

### Neutral

- The router's branch on permitted-vs-forbidden adds one extra
  state-machine arm to ADR-006's dispatcher; isolated behind the
  `gesture-handoff` capability so unit tests for the rest of the
  router don't have to know about it.

## Alternatives considered

### A. Set the canvas to `auto` and never release JS-side

Lowest implementation cost — canvas inherits page semantics, the
router only ever sees what the browser doesn't consume. Rejected
because canvas-drawn carousels, sliders, and signature pads then
*cannot work*: the browser silently swallows the gestures they
need before the router runs. This is the failure mode every
shipped canvas-UI framework hits at least once; we chose to
inherit it intentionally only as a per-region opt-in, never as
the global default.

### B. Set the canvas to `none` and never release back

The other extreme: JS owns every gesture forever, and the router
simulates scroll / pinch / momentum on top of it (which we
already do for nested-scroll in ADR-022). Rejected on
*latency*: 10× the tap response time for the entire app, every
day, to support the minority of widgets that need JS gestures.
Also costs us the compositor-thread isolation that protects
scrolling from main-thread jank. ADR-022's momentum simulator
is a *complement* to the handoff path, not a replacement for it.

### C. Per-event check instead of per-gesture latch

Re-compute the intersection on every `pointermove`. Rejected:
violates the W3C "consulted at gesture start" rule, makes the
user-visible behavior depend on millisecond-level cursor
position (so a swipe that briefly grazes a `none` region in the
middle would freeze), and makes the WPT subset un-passable.

### D. Pure consumer-driven `touch-action` (no per-widget defaults)

Ship no defaults; require every consumer to set
`touch-action` on every widget they care about. Rejected: a
`Button` with no styling that fails to fire `click` until 300 ms
elapsed is exactly the regression the W3C added `manipulation`
to fix. Making consumers re-discover that bug per widget is a
non-starter.

### E. Single-direction pan variants in v1

The spec defines `pan-left` / `pan-right` / `pan-up` /
`pan-down` as single-axis-direction restrictions. Deferred to
v2 — none of the per-widget defaults need them, and the
intersection table doubles in size when they are included. Will
be added when a fixture motivates it (e.g. a "swipe-to-dismiss"
sheet that wants `pan-left` only). The parser accepts the
values today; the intersection algebra rejects them.

## Open questions

- **OS-level double-tap window**: iOS reports the 300 ms / 10 px
  threshold as platform-stable; Android Chrome treats it as
  device-dependent. The R6 implementation hard-codes the spec
  values; if Android conformance tests fail we'll need to read
  the platform's `Touch.radiusX` to widen the move bound.
- **Pointer Events Level 3** is widening the `touch-action`
  grammar (adding `pen-action` for stylus). Will roll into a
  v3 of this ADR when the WPT subset starts gating it.
- **Hover-action interplay**: the ADR-006 router treats hover
  separately from touch. A future ADR may unify them under a
  single `gesture-action` umbrella; for now `touch-action` is
  the only CSS surface the browser consumes pre-JS.

## References

### Specs

- W3C Pointer Events Level 2 — `touch-action` property
  (https://www.w3.org/TR/pointerevents2/#the-touch-action-css-property)
- W3C CSS Touch-Action — value definitions
  (https://www.w3.org/TR/pointerevents/#the-touch-action-css-property)
- WICG Compositor Worker / scroll-on-compositor latency
  notes.

### Related ADRs

- ADR-005 (#2143) — per-widget semantics emitter; this ADR
  extends its proxy-attribute table.
- ADR-006 (#2164) — glass-pane pointer router; the handoff
  path branches off its dispatcher.
- ADR-009 (#2148) — hit-test fixture; intersection rule
  reuses its ancestor-chain output.
- ADR-017 (#2153) — WPT Pointer Events subset; touch-action
  conformance is part of the gated subset.
- ADR-022 (#2167) — nested scroll + momentum; consumes the
  gesture-handoff path when the effective value permits a pan.

### Related issues

- #2168 — this issue (touch-action semantics projection).
- #2137 — parent epic (pointer channel parity).
- #2164 — glass-pane router (handoff branch lives here).
- #2167 — nested scroll delegation (handoff consumer).
- #2158 — per-widget semantics emitter (proxy-attribute table).
- #2139 — dom-reference-runner (oracle for the WPT fixture).
- #2169 — pointer-events CSS (adjacent; out of scope here).
