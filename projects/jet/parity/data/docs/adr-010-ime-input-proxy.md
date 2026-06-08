# ADR-010: IME hidden input proxy (per-focused-widget composition channel)

| Field | Value |
|-------|-------|
| Issue | #2171 |
| Parent epic | #2138 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Mount exactly one hidden `<input type="text">` (or `<textarea>` for multiline) inside `<jet-semantics>` on widget focus; unmount on blur; reposition over the canvas-side caret each frame within 1 CSS pixel; `opacity:0` + `pointer-events:none` + `caret-color:transparent` hiding strategy |

## Context

ADR-004 (#2152) gave jet a shadow-DOM substrate (`<jet-semantics>`) so
keyboard focus and the accessibility tree have somewhere to live. ADR-006
(#2164) added the glass-pane input router so pointer events reach the
right canvas widget. Both ADRs handle the **dispatch** side — getting the
browser to point its focus and pointer events at a jet widget. Neither
ADR addresses the **text-input** side, and text input on the modern web
is not a `keydown` stream. It is an IME composition session.

When a user types Japanese, Chinese, Korean, Vietnamese, or any of the
emoji / accented-Latin paths that go through `CompositionEvent`, the
browser does three things that a `<canvas>` cannot participate in:

1. It reads the focused element's bounding rect and asks the OS to anchor
   the **candidate window** (the popup that shows kanji choices, emoji
   suggestions, dead-key combiners) directly under that rect's caret.
2. It buffers the not-yet-committed text in the focused element's
   `value` and fires `compositionstart` / `compositionupdate` /
   `compositionend` events on it. The composing text is **not** in the
   `keydown` stream — Chromium and WebKit both suppress synthetic
   `keydown`s while a composition is active.
3. It exposes a TSF / IMK / IBus / fcitx surface to assistive tech (so
   a screen reader can read the composing text, not just the committed
   glyphs).

A `<canvas>` cannot do any of those — it has no `value`, no caret rect
the OS can read, and no `CompositionEvent` target. The parent epic
**#2138 (parity: text input + IME on canvas)** frames the problem; this
ADR is the substrate the rest of the slices stand on:

| Issue | Depends on this ADR for |
|-------|-------------------------|
| #2172 | `compositionstart/update/end` → canvas widget bridge |
| #2173 | Selection range mirror (so the OS sees the canvas selection) |
| #2174 | Mobile keyboard surface (`inputmode`, `enterkeyhint`) |
| #2175 | Dead-key / Option-key combiner replay |
| #2176 | Password-manager autofill via real `<input type=password>` |

Each slice needs a DOM `<input>` to attach a listener to, ask the browser
"where is your caret?", or hand off to `autocomplete`. ADR-010 says:
that input exists, here is what it looks like, here is who owns its
lifecycle, and here is the contract its position must satisfy so the OS
candidate window pins to the right spot on screen.

## Decision

For every focused jet canvas text widget, mount **exactly one** hidden
DOM input element inside the `<jet-semantics>` shadow subtree. Unmount it
on blur. Reposition it over the canvas-side caret on every animation
frame. The canvas widget remains the source of truth for `value` and
selection; the proxy is a transient composition channel, not a model.

### Proxy shape

```
<jet-semantics>
  <!-- existing <jet-focus-proxy> nodes from ADR-004 -->
  <input
    is="jet-ime-proxy"
    type="text"
    data-jet-widget-id="app/main/composer/body"
    autocomplete="off"
    autocorrect="off"
    autocapitalize="off"
    spellcheck="false"
    style="
      position:absolute;
      left:<caret.x>px; top:<caret.y>px;
      width:1px; height:<caret.height>px;
      opacity:0;
      pointer-events:none;
      caret-color:transparent;
      background:transparent;
      border:0;
      padding:0;
      margin:0;
      outline:0;
      font: <widget.font>;
    "
  />
</jet-semantics>
```

`<textarea>` is substituted when the focused widget declares
`multiline: true` in its semantic descriptor (so newline keys produce a
literal `\n` in the composition buffer instead of an `Enter` keydown
that bypasses composition). Everything else in the contract is
identical.

### Hiding strategy

The proxy must be **visually invisible** but **OS-visible**. The
following four properties achieve that and are jointly mandatory:

- `opacity: 0` — removes the proxy from the visual layer.
- `pointer-events: none` — keeps the glass pane (ADR-006) as the sole
  pointer target. The proxy never participates in hit-testing.
- `caret-color: transparent` — suppresses the browser's blinking
  caret. The canvas widget paints its own caret; rendering two would
  be visible as a glitch in the millisecond before opacity takes
  effect, and on browsers with hardware-accelerated caret rendering
  the native caret can leak through `opacity: 0`.
- non-zero size (`width: 1px` minimum, `height: caret.height`) —
  some IME stacks consult the focused element's
  `getBoundingClientRect()` to decide candidate-window placement. A
  zero-sized rect collapses to `(0, 0)` on Safari, parking the popup
  in the top-left corner of the viewport regardless of where the
  canvas caret actually is.

What we **must not** use:

- `display: none` — removes the element from the focus tree;
  `document.activeElement` returns `<body>`, no composition events
  fire.
- `visibility: hidden` — same focus-tree consequence on Firefox
  ≤ 122 and on WebKit nightly: the element is focusable but does
  not receive `compositionstart` because the IME pipeline skips
  hidden subtrees.
- `aria-hidden="true"` — hides from assistive tech (which is the
  opposite of what #2173 needs) and on Safari with VoiceOver
  enabled also detaches the element from the TSF surface, so the
  candidate window anchors to the previous focus target.
- `position: fixed; left: -9999px` (the classic "offscreen" pattern)
  — moves the candidate window offscreen with it. The OS does not
  know the visible caret is on the canvas; it puts the popup where
  the input rect is.

The hiding strategy is therefore a single-point decision: any deviation
breaks at least one OS × browser × AT triple. ADR-010 pins it.

## Lifecycle

The proxy is **per-focus-session**, not per-widget. At any moment the
DOM contains either zero or one `<input is="jet-ime-proxy">`:

1. **Focus.** When a canvas text widget receives focus (via tab order
   per ADR-003, click via ADR-006, programmatic `widget.focus()`, or
   restore-on-mount), the focus controller:
   1. Computes the current caret rect in the canvas's CSS coordinate
      space.
   2. Creates the `<input>` (or `<textarea>`) element with the style
      block from § Proxy shape, sized over the caret rect.
   3. Appends it to `<jet-semantics>`.
   4. Calls `proxy.focus({ preventScroll: true })`. The
      `preventScroll` is mandatory — without it, browsers will scroll
      the document so the (invisible) proxy is in view, which is
      visually identical to "the page randomly jumped" because the
      canvas does not move.
   5. Synchronously mirrors the widget's current `value` into
      `proxy.value` so password-managers / browser autofill / OS
      replacement-text features see the full string, not an empty
      input.
2. **Composition / input.** Listeners (wired by #2172) translate
   `compositionstart / compositionupdate / compositionend` and
   non-composition `input` events into canvas-widget mutations. The
   widget remains the source of truth; the proxy is the **channel**.
3. **Re-mirror.** After every `compositionend` and after every
   non-composition `input`, the proxy's `value` is re-synced from the
   widget's authoritative state. This collapses any drift introduced
   by the widget's own validation (e.g. a numeric field that strips
   non-digits) so the next composition starts from a clean buffer.
4. **Blur.** When the widget loses focus (Tab away, click away,
   programmatic blur, widget unmount), the focus controller:
   1. Fires `proxy.blur()` to terminate any pending composition.
   2. Detaches the proxy from `<jet-semantics>` and drops the
      registry entry (§ Registry).
   3. Drops the rAF reposition callback (§ Per-frame reposition).

At-most-one is enforced structurally: the focus controller (ADR-004's
single owner of `<jet-semantics>`) refuses to mount a second proxy. A
focus change is `unmount → mount`, never `mount → mount → unmount-old`.
This rules out a class of bugs where two inputs briefly co-exist and
the OS arbitrarily picks one as the IME target.

## Per-frame caret-rect repositioning

The OS candidate window anchors to the proxy's bounding rect
**at the moment the IME asks** — which is unpredictable from
JavaScript. Some IMEs ask on `compositionstart` only. Others
re-query every `compositionupdate`. A few (macOS Live Conversion,
recent versions of Microsoft IME) re-query on every keystroke. We
therefore cannot lazily reposition "when a composition starts" — we
must keep the proxy's rect overlapping the canvas caret rect **at all
times** the widget is focused.

The repositioning contract:

- A `requestAnimationFrame` callback is registered while the proxy is
  mounted. On every frame it:
  1. Reads the widget's caret rect in CSS pixels:
     `(caret.x, caret.y, caret.width, caret.height)`. The width is
     typically `1` (the caret bar) but for IME composition we use the
     rect of the **composition span** (the underlined region) when
     one exists — the OS prefers anchoring under the composition span,
     not the insertion point.
  2. Writes `proxy.style.left`, `top`, `width`, `height` to that rect.
- The reposition is **idempotent**: if the rect has not changed since
  last frame, the style write is skipped (cached in a `lastRect`
  field) to avoid layout thrash.
- The acceptance bound is **1 CSS pixel**: at the end of every
  animation frame, `proxy.getBoundingClientRect()` must overlap the
  canvas-side caret rect within ±1 px on each edge. This is the
  R8 acceptance test from #2171.

Scroll and resize are handled by the same per-frame callback — no
special `scroll` / `resize` listeners are needed. Within one rAF after
either event, the canvas widget's caret rect has been recomputed (it
is derived from widget layout, which the renderer recomputes on
scroll/resize), and the proxy reposition then converges in that same
frame. This is R9 from #2171.

The `font` shorthand on the proxy is mirrored from the widget's
typography descriptor so that, when the OS asks the input for its
font metrics (some Japanese IMEs do this to choose a candidate-window
font), it sees the same family + size as the canvas-painted glyphs.

## Registry

`<jet-semantics>` owns a bidirectional registry exposed on its
instance:

```ts
class JetSemantics extends HTMLElement {
  imeProxyByWidget: Map<WidgetId, HTMLInputElement | HTMLTextAreaElement>;
  widgetByImeProxy: WeakMap<HTMLElement, WidgetId>;
}
```

Lookups are O(1) in both directions. The `WeakMap` direction lets event
handlers go from a raw DOM event (`event.target`) back to the widget id
without iterating. The forward `Map` lets the focus controller answer
"is there a proxy for widget X?" — which it asks on every focus change
to enforce the at-most-one invariant.

Consumers (#2172, #2174, #2176) read this registry; they do not mutate
it. Mutation is the focus controller's exclusive responsibility.
Registry drift is the canonical bug for this whole subsystem, so
locating the writer at one site is non-negotiable.

## Opt-out

A canvas host that wants to disable the IME channel sets
`data-jet-ime-channel="off"` on the `<canvas>` element. The focus
controller observes this attribute when promoting a focus event to a
proxy mount; on `"off"`, mount is skipped and the widget falls back to
**keydown-only** input.

Keydown-only is a real degradation, not a graceful one: IME composition
does not work, dead-keys do not combine, and the OS candidate window
never appears. The opt-out exists for two narrow cases:

1. **Games / non-text canvases** that want to capture raw key codes
   for hotkeys and explicitly do not want a text-input affordance.
2. **Embedded contexts** (jet inside a Storybook frame, jet inside a
   Playwright test that needs deterministic keydown timing) where the
   IME's asynchronous composition events would corrupt the test's
   event log.

The attribute is read on every focus mount, not cached. Toggling it at
runtime takes effect on the next focus change.

## Acceptance tests

R8 and R9 from #2171, encoded as parity gate checks:

- **R8 — focus mounts the proxy.** `widget.focus()` programmatically;
  after one rAF, assert:
  1. Exactly one `<input is="jet-ime-proxy">` is in
     `<jet-semantics>`.
  2. `document.activeElement` is that input (traversing through the
     shadow root with `getRootNode().activeElement`).
  3. `proxy.getBoundingClientRect()` overlaps the widget's caret
     rect within 1 px on each edge.
- **R9 — survives scroll / resize.** From the R8 state, dispatch a
  `window.scrollBy(0, 100)` and a `window.resizeTo(new_w, new_h)`
  in sequence. After exactly one rAF, the three R8 assertions still
  hold. The "exactly one rAF" budget is what makes this a parity
  test and not a "settle eventually" check — the canvas widget and
  the proxy must converge in lock-step.

Both tests live as a new fixture
`projects/jet/data/parity/fixtures/ime-proxy/` consumed by the existing
parity gate (#2144) under a `diff_kind: dom_attribute_diff` channel.

## Consequences

- **One DOM node per focus session** — measured cost is sub-millisecond
  on every browser we ship to. The `<input>` is never reflowed beyond
  its own 1-px box, and it never participates in pointer hit-testing
  (ADR-006 owns that).
- **The renderer must expose a caret-rect API** to the focus
  controller. This API already exists for the canvas's own caret
  painting; we are surfacing it, not inventing it.
- **The widget remains the source of truth for `value`.** Anyone
  reading the proxy's `value` to "read the widget's text" is wrong;
  the proxy mirror is one-way and one-shot (sync on mount, after
  `compositionend`, after `input`). The mirror exists for OS / AT /
  autofill consumption, not for app logic.
- **Password managers see real inputs.** #2176 builds on this by
  substituting `<input type="password">` when the widget's semantic
  role is `password-field`. The hiding strategy is identical.
- **Composition pipeline has a single owner.** `<jet-semantics>` is
  the only DOM subtree that contains IME proxies. Tests, devtools, and
  the parity gate can all use a single selector
  (`jet-semantics input[is="jet-ime-proxy"]`).
- **The keydown-only fallback is intentionally lossy.** Sites that
  set `data-jet-ime-channel="off"` are accepting that international
  users cannot type CJK / emoji into their canvas. We document that
  loudly in the opt-out's surface in the jet API.

## Alternatives considered

- **Always-mounted hidden input pool.** Pre-mount N proxies and
  rotate them on focus. Rejected: the OS candidate window key off the
  focused input's identity, so reusing a proxy across widgets confuses
  some IMEs (the candidate window flickers as the same DOM node moves).
  At-most-one with mount-on-focus is the cleanest contract.
- **One persistent proxy at the document root.** Mount once at jet
  initialization, never unmount; reposition it for every focused
  widget. Rejected for the same reason — and because moving a single
  proxy across `<jet-semantics>` boundaries (multiple jet roots on
  one page) requires shuttling it between shadow subtrees, which
  breaks the encapsulation ADR-004 paid for.
- **`contenteditable` div instead of `<input>` / `<textarea>`.** A
  `contenteditable` host gets richer composition events but loses
  autofill, password-manager hooks, and `inputmode` /
  `enterkeyhint` (#2174's surface). The text-input epic needs all of
  those.
- **Native `TextInput` element from the WICG `EditContext` proposal.**
  Right shape, wrong year. `EditContext` is Chromium-only as of
  2026-05 and the spec is still moving. We track it as a follow-up
  (§ Open questions) but cannot ship on it.
- **Hidden-by-`clip-path`** (the same strategy ADR-004 uses for
  `<jet-focus-proxy>`). Tested on Safari 17; the candidate-window
  anchor sometimes uses the **clipped** rect (zero area) rather than
  the layout rect, so the popup lands at (0, 0). `opacity: 0` is the
  only hiding mode all three engines agree on.

## Open questions

1. **`EditContext` migration path.** Once `EditContext` ships in
   WebKit and Gecko (currently behind flags), the proxy can be
   replaced by a `EditContext`-bound canvas, eliminating the DOM
   node entirely. Track under a future issue; the migration is
   transparent because the consumer surface (compositionstart →
   widget mutation) is the same.
2. **Multiple `<jet-semantics>` on one page.** ADR-004 permits one
   per canvas, but two co-existing jet roots can each focus a widget
   simultaneously (e.g. a host page and an iframe-less embed). The
   at-most-one invariant is per-shadow-root, so two proxies can
   exist globally; the OS will follow `document.activeElement`,
   which is single. Verify in fixtures.
3. **Composition state during widget unmount.** If a canvas widget
   is destroyed (React unmount, jet view swap) while a composition
   is active, we currently `proxy.blur()` and lose the in-flight
   text. A future enhancement could surface the partial composition
   to the destruction hook so the host can decide (commit / discard).
   Tracked under a follow-up to #2172.
4. **iOS Safari `inputmode` quirks.** Mobile Safari triggers the
   on-screen keyboard from a `focus()` only inside a user-gesture
   stack frame. Tab-on-key-down restores focus outside that frame
   and the keyboard fails to appear. #2174 will deal with this with
   a synchronous-focus shim; ADR-010 does not.

## References

- #2171 — this slice (IME hidden input proxy).
- #2138 — parent epic: text input + IME on canvas.
- #2152 / ADR-004 — `<jet-semantics>` shadow subtree (the substrate).
- #2164 / ADR-006 — glass-pane input router (pointer events stay off
  the proxy via `pointer-events: none`).
- #2153 / ADR-003 — tab order (focus arrives at the widget that mounts
  this proxy).
- #2172 — composition event bridge (consumer of this proxy).
- #2174 — mobile keyboard surface (consumer of this proxy).
- #2176 — password-manager autofill (consumer of this proxy).
- WICG EditContext proposal — future migration target.
