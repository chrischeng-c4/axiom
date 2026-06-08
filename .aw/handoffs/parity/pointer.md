# Pointer Event Observation Channel

## Goal

The pointer channel proves that **every click, tap, scroll, drag, and hover that lands on jet's `<canvas>` routes to the same logical widget that a DOM-rendered React/MUI build would route it to**. From the browser's point of view there is exactly one event target — the single `<canvas id="jet-root">` glass pane — so every aspect of "which thing did the user touch?" is implemented by jet's WASM-side hit-tester rather than by the DOM. If this channel drifts, the visible pixels can be byte-perfect and the app will still feel broken: buttons under modals get clicked, swipes scroll the page instead of the carousel, drag-and-drop silently no-ops, and `cursor: pointer` never appears.

User impact spans every input modality: mouse (hover affordance, right-click, wheel + shift-wheel horizontal scroll), touch (tap, long-press, pan, pinch, multi-finger), pen (pressure, tilt), and keyboard-driven activation (covered in #2135 focus, not here). The two highest-risk surfaces are **nested scroll/momentum** — the historical #1 Flutter Web bug class (see flutter/flutter#75180 lineage) — and **HTML5 drag-and-drop**, which fundamentally cannot work from inside a canvas without a real DOM bridge.

## Architecture

```
                 browser
                    │
                    ▼
        ┌──────────────────────────┐
        │  <canvas id="jet-root">  │   touch-action: <emitted per region>
        │   single glass pane      │   tabindex=0, role=application
        └──────────────────────────┘
                    │   pointerdown / move / up / cancel
                    │   wheel, click, contextmenu
                    │   touchstart / move / end / cancel  (legacy compat)
                    │   dragstart / over / drop  (via proxy bridge)
                    ▼
        ┌──────────────────────────┐
        │  jet input router (JS)   │   normalize, coalesce, sanitize buttons
        │  thin shim, no logic     │   forward as flat struct into WASM
        └──────────────────────────┘
                    │   postMessage-free direct call
                    ▼
        ┌──────────────────────────┐
        │  WASM hit-tester         │   walk render tree, respect z, transforms,
        │  + capture table         │   pointer-events:none equivalent, clip rects
        └──────────────────────────┘
                    │
                    ▼
        ┌──────────────────────────┐
        │  widget event handlers   │   onClick, onPointerDown, onScroll, ...
        └──────────────────────────┘
```

This mirrors Flutter Web's `<flt-glass-pane>` model implemented in
[`pointer_binding.dart`](https://github.com/flutter/engine/blob/main/lib/web_ui/lib/src/engine/pointer_binding.dart):
listeners are attached to the root element for `down`/`leave`/`wheel` and to
`window` for `move`/`up` so a drag that leaves the canvas still resolves a
matching `pointerup`. Flutter's `_ButtonSanitizer` and `ClickDebouncer`
patterns are directly relevant — jet needs the same button-state
normalization across browsers and the same disambiguation between
browser-synthesized clicks and pointer-derived clicks.

The WASM hit-tester is the analogue of Flutter's
[`RenderObject.hitTest` / `BoxHitTestResult`](https://api.flutter.dev/flutter/rendering/RenderBox/hitTest.html):
a depth-first walk over the render tree that pushes matching objects onto a
result list, deepest-first. For jet, the **parity oracle** is the browser's
own [`document.elementFromPoint(x, y)`](https://developer.mozilla.org/en-US/docs/Web/API/Document/elementFromPoint)
running against the equivalent React+DOM build — jet's reported semantic
target must equal the DOM target at the same `(x, y)` for any layout the
harness can render.

## Sub-issues

| #     | Priority | Subject                                                                   | Depends on            |
|-------|----------|---------------------------------------------------------------------------|-----------------------|
| #2164 | P1       | Single glass-pane input router (root canvas captures all events)          | —                     |
| #2165 | P1       | Hit-test correctness fixture (1000 random clicks parity grid)             | #2164                 |
| #2166 | P1       | Adopt WPT `pointerevents/` subset (~600 tests)                            | #2164, #2142          |
| #2167 | P1       | Nested scroll & momentum (Drawer-in-Dialog, MUI DataGrid)                 | #2164, #2165          |
| #2168 | P2       | `touch-action` semantics (browser consumes before JS)                     | #2164                 |
| #2169 | P2       | HTML5 drag-and-drop bridge (proxy element under cursor)                   | #2164                 |
| #2170 | P2       | Hover & cursor style parity (`cursor: pointer` on Button)                 | #2164                 |

## Technical approach

- **Single glass-pane setup (#2164).** Exactly one `<canvas id="jet-root">`
  in the DOM; CSS `touch-action` emitted per-region by the layout pass and
  collapsed onto the canvas's computed value (browser intersects up to the
  first scroll container — see
  [MDN `touch-action`](https://developer.mozilla.org/en-US/docs/Web/CSS/touch-action)).
  JS shim attaches listeners on `_viewTarget` (the canvas) for
  `pointerdown`/`pointerleave`/`wheel`/`contextmenu` and on `window` for
  `pointermove`/`pointerup`/`pointercancel`, exactly like Flutter's
  `_PointerAdapter`. No per-widget `addEventListener` — all routing is done
  in WASM. Sub-widgets that semantically want `pointer-events: none` are a
  flag in the render tree, not a CSS property.

- **Hit-test algorithm (#2165).** Depth-first, painter's-order reverse walk
  over the rendered render objects; transforms inverted on the way down,
  clip rects honored. The hit-tester must agree with
  `document.elementFromPoint` for the equivalent DOM tree on the fixture
  matrix: overlapping z-index, CSS transforms (rotate, scale, translate),
  `overflow: hidden` clip, fixed/sticky positioning, modal overlays.
  Oracle: render the same layout in React+MUI, run 1000 deterministic
  `(x, y)` samples through `document.elementFromPoint`, compare to jet's
  reported semantic target keyed by `data-testid`. Diff = parity bug.

- **WPT compliance (#2166).** Vendor
  [`web-platform-tests/wpt/pointerevents/`](https://github.com/web-platform-tests/wpt/tree/master/pointerevents)
  via foundation #2142 — the directory ships ~600 conformance tests
  organized into `bugs/`, `compat/`, `crashtests/`, `html/`, `parsing/`,
  `persistentDeviceId/`, `pointerlock/`. The subset jet targets covers
  capture (`setPointerCapture` / `lostpointercapture` / implicit-capture on
  direct-manipulation devices), boundary events (`pointerover`/`out`/
  `enter`/`leave` ordering and non-bubbling semantics for enter/leave),
  coalesced events (`getCoalescedEvents()` returns the dropped intermediate
  samples), predicted events (`getPredictedEvents()`), primary-pointer
  rules ("first pointer of each type is primary; only primary generates
  compatibility mouse events"), button-state sanitization, and
  `pointerType` normalization across mouse/touch/pen. See
  [W3C Pointer Events Level 3](https://www.w3.org/TR/pointerevents3/).

- **`touch-action` delegation (#2168).** Browser consumes `touch-action`
  **before any JS event fires** — by the time the WASM hit-tester sees a
  `pointermove`, the browser has already decided whether it is scrolling
  the page or letting the app handle the gesture. The layout pass must
  therefore emit a `touch-action` value onto the canvas root that reflects
  the gesture surface under the pointer at the time of `pointerdown`.
  Carousels declare `pan-y` so vertical page scroll still works while jet
  consumes horizontal swipes; modal scrim declares `none`; default is
  `manipulation` (removes the iOS 300 ms tap delay). If the browser takes
  over mid-gesture, jet sees a `pointercancel` event and must unwind any
  in-flight gesture state — same contract as native.

- **Scroll chain & momentum (#2167).** Define jet's scroll-chain rules
  explicitly: a `pointermove` delta is consumed by the deepest scrollable
  widget under the press; remainder bubbles to the next ancestor scroller;
  fling/momentum is applied per-axis with platform-matching deceleration
  curves (iOS rubber-band, Android over-scroll glow). Fixture pair:
  MUI `DataGrid` (nested vertical scroll inside a page scroll) and a
  `Drawer` opened inside a `Dialog` (two stacked modal scroll surfaces).
  These are the historical Flutter Web pain points called out by
  [flutter/flutter#75180](https://github.com/flutter/flutter/issues/75180)
  and its lineage of nested-scroll bugs. Wheel + Shift = horizontal axis
  swap (the original #75180 desktop convention) is in-scope.

- **Drag-and-drop bridge (#2169).** HTML5 DnD is special: only real DOM
  elements with `draggable="true"` fire `dragstart`/`dragend`, and only
  real DOM elements receive `dragover`/`drop` — see
  [MDN HTML Drag-and-Drop API](https://developer.mozilla.org/en-US/docs/Web/API/HTML_Drag_and_Drop_API).
  A canvas cannot participate. Bridge design: on `pointerdown` over a
  widget whose render-tree node has `draggable=true`, jet spawns a 1×1
  transparent `<div draggable="true">` under the cursor, listens for
  `dragstart` on it, populates `DataTransfer` from the widget's payload,
  and tears it down on `dragend`. Drop zones get a sibling
  `<div data-jet-droptarget>` overlay sized to the WASM-side widget
  bounds, with `dragover`/`drop` forwarded into WASM. Fixture: MUI's
  `react-dnd`-based file-upload zone and reorderable list.

- **Hover, cursor & primary pointer (#2170).** `cursor: pointer` is a
  CSS property of a DOM element — jet's hit-tester must update
  `canvas.style.cursor` on every `pointermove` to reflect the widget
  currently under the pointer (`pointer`, `text`, `grab`, `not-allowed`,
  etc.). The browser only honors the canvas's current cursor; there is
  no per-pixel cursor API. Same shim updates `aria-busy`/tooltip state
  if applicable. Hover affordance is the user's main signal that
  "this is clickable" — getting it wrong feels like dead pixels.

## Dependencies

- **Foundation:**
  - #2139 — parity runner provides side-by-side jet/React render harness
  - #2142 — WPT vendor (must include `wpt/pointerevents/`, `wpt/uievents/`,
    `wpt/touch-events/`, `wpt/css/css-ui/` for `touch-action`)
  - #2143 — fixture loader and `data-testid` plumbing
  - #2144 — manifest of pointer parity fixtures and expected-diff waivers

- **Cross-channel coupling:**
  - **Pixel (#2134):** hover/cursor mutations cause visible pixel deltas
    (focus rings, ripple effects, cursor sprite). Hover fixtures must be
    excluded from the strict pixel-diff matrix, or evaluated with a
    cursor-region mask.
  - **Focus (#2135):** `click` synthesizes focus on the target — focus
    channel asserts the focused widget after each parity click.
  - **IME (#2138):** DnD's proxy element lifecycle interacts with the
    IME proxy lifecycle; both spawn ephemeral DOM under the canvas, and
    the order of `pointerup` → `focus` → `compositionstart` matters.
  - **a11y (#2136):** `pointerdown` on a `role=button` must fire the same
    accessibility event the DOM build would; primary-pointer
    compatibility-click is what AT relies on.

- **External:**
  - [WPT `pointerevents/`](https://github.com/web-platform-tests/wpt/tree/master/pointerevents)
    (~600 tests)
  - [W3C Pointer Events Level 3](https://www.w3.org/TR/pointerevents3/)
  - [W3C UI Events](https://www.w3.org/TR/uievents/) for click synthesis
    rules
  - [CSS Touch-Action](https://drafts.csswg.org/css-ui/#touch-action)
  - [HTML5 Drag-and-Drop](https://html.spec.whatwg.org/multipage/dnd.html)

## Success criteria

- **Glass-pane router (#2164):** exactly one `<canvas>` in the DOM; no
  per-widget listeners; all 9 W3C pointer event types and the 4 legacy
  touch events forwarded into WASM with sanitized button state across
  Chromium / WebKit / Firefox.
- **Hit-test correctness (#2165):** 1000-click parity grid over the
  fixture matrix (overlapping z-index, transforms, scrolled containers,
  modal overlays) shows **100% target-id match** against
  `document.elementFromPoint`. Any mismatch is either a fixture-author
  bug or a waiver entry — no silent divergence allowed.
- **WPT (#2166):** vendored `pointerevents/` subset (capture + boundary
  + coalesced + primary-pointer + touch-action) passes at **≥ 95%**.
  The remaining ≤ 5% must be enumerated as explicit waivers with a
  rationale (typically pointer-lock and persistentDeviceId, which are
  out of scope).
- **Scroll & momentum (#2167):** MUI `DataGrid` parity scroll test
  passes (vertical pan inside a page scroll, no scroll-leak); Drawer
  inside Dialog passes (inner pan does not move outer scrim);
  Shift+wheel horizontal scroll works on desktop.
- **`touch-action` (#2168):** carousel horizontal-swipe fixture
  succeeds while page vertical scroll continues to work; map-pan
  fixture works while pinch-zoom remains available; `pointercancel`
  fires when the browser takes over a gesture mid-flight, and jet's
  in-flight gesture state is correctly unwound.
- **DnD (#2169):** MUI `react-dnd` file-drop fixture and reorderable
  list both pass — drag-image visible, `dataTransfer` round-trips,
  `dragover`/`drop` fire on the correct WASM-side widget.
- **Hover & cursor (#2170):** MUI `Button` and `Link` hover fixtures
  show `cursor: pointer` set on the canvas; `cursor: text` over an
  `Input`; `cursor: not-allowed` over a disabled control. Sampled at
  ≥ 60 Hz under continuous `pointermove`, no missed frames where the
  cursor lags the underlying widget by more than one frame.

## Out of scope / waivers

- **Pointer lock** (`element.requestPointerLock()`): used for FPS-style
  games; not needed for MUI/Material/Vuetify parity. Out of scope —
  matches Flutter Web.
- **`persistentDeviceId`** (WPT subdir): privacy-gated, not part of the
  parity surface. Waived.
- **Browser-native context menu on long-press (mobile):** Flutter Web
  doesn't support custom long-press context menus reliably either;
  accept that long-press shows browser-native menu on mobile, suppress
  via `touch-action: manipulation` only where the design demands.
- **Force-touch / 3D Touch:** WebKit-only legacy; not exposed by W3C
  Pointer Events 3. Waived.
- **Coalesced events on Safari < 16:** `getCoalescedEvents()` returned
  an empty list historically. Treated as a "browser bug, not a jet bug"
  — the parity oracle uses the same browser, so jet's behavior matches.
- **Pen tilt parity** beyond `tiltX`/`tiltY`: spherical
  `altitudeAngle`/`azimuthAngle` is Pointer Events 3 only and is not
  required for MUI surface parity.

## Prior art and references

- [Flutter `pointer_binding.dart`](https://github.com/flutter/engine/blob/main/lib/web_ui/lib/src/engine/pointer_binding.dart)
  — the canonical single-glass-pane input router; `_PointerAdapter`
  registers on root + window, `_ButtonSanitizer` normalizes button
  state across browsers, `ClickDebouncer` disambiguates browser clicks
  from pointer-derived clicks.
- [Flutter `RenderBox.hitTest`](https://api.flutter.dev/flutter/rendering/RenderBox/hitTest.html)
  and `BoxHitTestResult` — the depth-first painter's-order algorithm
  jet's WASM hit-tester mirrors.
- [W3C Pointer Events Level 3](https://www.w3.org/TR/pointerevents3/) —
  authoritative spec for event types, capture, coalesced/predicted
  events, primary pointer, normalization across mouse/touch/pen.
- [WPT `pointerevents/`](https://github.com/web-platform-tests/wpt/tree/master/pointerevents)
  — ~600 conformance tests; the de facto interoperability bar.
- [MDN `document.elementFromPoint`](https://developer.mozilla.org/en-US/docs/Web/API/Document/elementFromPoint)
  and `elementsFromPoint` — the browser-native hit-test oracle jet's
  parity grid compares against; explicitly skips
  `pointer-events: none` elements.
- [MDN `touch-action`](https://developer.mozilla.org/en-US/docs/Web/CSS/touch-action)
  — defines browser consumption order ("intersects up to the first
  scrolling element", fires `pointercancel` if browser takes over,
  changes don't apply mid-gesture).
- [MDN HTML Drag-and-Drop API](https://developer.mozilla.org/en-US/docs/Web/API/HTML_Drag_and_Drop_API)
  — establishes why a canvas cannot participate in DnD without a real
  DOM proxy element with `draggable="true"`.
- [flutter/flutter#75180](https://github.com/flutter/flutter/issues/75180)
  — Shift+wheel horizontal scroll on desktop, archetype of the
  "nested scroll & momentum" bug class that has haunted Flutter Web;
  jet must not repeat the pattern.
- [CSSWG Scroll Snap / Scroll Chaining](https://drafts.csswg.org/css-scroll-snap/)
  — informs the scroll-chain bubbling rules jet implements in #2167.
- [MUI `react-dnd` integration](https://github.com/mui/material-ui/tree/master/docs/data/material/components/tables)
  — the concrete user-facing DnD surface jet's bridge (#2169) must
  satisfy.
