# ADR-020: Programmatic focus API (`element.focus()` bridging)

| Field | Value |
|-------|-------|
| Issue | #2157 |
| Parent epic | #2135 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Bridge `proxy.focus(options)` and `proxy.blur()` synchronously into the canvas focus controller; bidirectional (`jet.focusWidget(id)` calls `proxy.focus({ preventScroll: true })`); `preventScroll`, `force`, and disabled-no-op honoured; canvas-side `focus`/`blur` channel events re-emit with `jetNodeId`. |

## Context

React refs, MUI internals, Autocomplete-on-open handlers, Dialog
"focus the primary action" effects, wizard "focus the first invalid
field on submit" patterns, snackbar action-button capture — all of
them reach for the same six characters: `.focus()`. The DOM contract
behind those six characters is precise:

- `element.focus(options?)` moves DOM focus to `element`, updates
  `document.activeElement` **synchronously**, and dispatches
  `blur` / `focusout` on the prior `activeElement` followed by
  `focus` / `focusin` on the new one. `focusout` and `focusin`
  bubble; `blur` and `focus` do not.
- `options.preventScroll: true` suppresses the browser's
  scroll-into-view that would otherwise reveal a focused element
  hidden behind a scroll container.
- Calling `.focus()` on a disabled `<input>`, on a detached node,
  or on an element with `tabindex="-1"` that is also `inert`, is a
  silent no-op. The browser does not throw.
- `element.blur()` is symmetric: if `element === document.activeElement`,
  focus moves to `<body>` and the same event quartet fires in
  reverse role.

Jet has to expose this surface on every focusable widget because
calling code does not know — and should not have to know — that a
"button" is a canvas glyph rather than an `<HTMLButtonElement>`. A
React component receives a ref; the ref's `.current` is whatever jet
hands back from its DOM bridge; user code calls `.focus()` on it; the
correct thing must happen.

ADR-004 (#2152) gave us the substrate: every focusable widget is
shadowed by a `<jet-focus-proxy>` inside `<jet-semantics>`. ADR-003
(#2153) wired Tab traversal through those proxies. ADR-007 (#2154)
scoped traps to a proxy subtree. ADR-006 (#2156) drove
`:focus-visible` paint from proxy state. ADR-020 closes the loop:
the **caller-initiated** path. `jetNode.focus(opts)` runs the same
pipeline as a `Tab` keypress, but the trigger is a synchronous JS call
inside arbitrary user code, and the contract that JS call has to
honour is the DOM's, not jet's.

## Decision

We expose `focus(options?)` and `blur()` on every jet node bound to a
focusable proxy, route both methods synchronously through the proxy,
and re-emit the resulting browser events on the canvas-side focus
channel with `jetNodeId` identity preserved.

### Method shape

```ts
interface JetFocusableNode {
  focus(options?: JetFocusOptions): void;
  blur(): void;
}

type JetFocusOptions = {
  preventScroll?: boolean;  // DOM-standard
  force?: boolean;          // jet extension; bypass active trap
};
```

`options` defaults to `{}`. Both `preventScroll` and `force` default
to `false`. Unknown keys are silently ignored (forward-compatible with
future DOM additions like `focusVisible`).

### Outbound bridge — `jetNode.focus(opts)`

The implementation is a thin two-step:

```ts
function focus(this: JetNode, opts: JetFocusOptions = {}) {
  const proxy = registry.lookup(this.jetNodeId);
  if (!proxy) return;                       // unmounted; no-op
  if (this.disabled) return;                 // R-disabled
  if (trapActive() && !this.insideTrap() && !opts.force) {
    log.warn("focus outside active trap suppressed", { jetNodeId: this.jetNodeId });
    return;
  }
  proxy.focus({ preventScroll: true });     // see § preventScroll
  if (!opts.preventScroll) {
    canvas.scrollWidgetIntoView(this.jetNodeId);
  }
}
```

Three properties matter and are load-bearing:

1. **Synchronous.** `proxy.focus()` returns only after
   `document.activeElement === proxy`. This is what makes
   `:focus-visible` modality detection work (R6): if the caller is
   inside a `keydown` handler, the browser's modality heuristic still
   sees the keyboard event as the "trigger" of the focus change. If
   we deferred to a microtask the heuristic would be lost and every
   programmatic focus would render as `:focus-visible: false`.
2. **The proxy is always called with `preventScroll: true`.** The
   proxy is a 1×1px clipped element pinned over the widget's hit-rect
   (ADR-004); the browser's scroll-into-view would scroll the page to
   reveal the proxy's actual DOM position, which is correct for the
   proxy and wrong for the user (the proxy is invisible). Canvas-side
   scroll-into-view is a separate, larger animation against the
   canvas viewport.
3. **Canvas-side scroll-into-view is the caller's `preventScroll`.**
   `preventScroll: true` skips it; `false` (the default) runs it.

### Inbound bridge — `jet.focusWidget(id)` and channel observation

Canvas-initiated focus must keep `document.activeElement` canonical so
that user code reading `document.activeElement`, ancestor CSS
selectors like `:focus-within`, and the AX tree all agree. The
inbound path is symmetric to the outbound:

```ts
function focusWidget(id: JetNodeId) {
  const proxy = registry.lookup(id);
  if (!proxy) return;
  proxy.focus({ preventScroll: true });
  // browser fires blur/focus on proxies → focus controller observes
}
```

The focus controller (ADR-004 §4) already listens for `focusin` /
`focusout` on the `<jet-semantics>` host (capture-phase, single
delegated listener). Whether the trigger was `Tab`, a programmatic
`proxy.focus()` from user code, or a programmatic `proxy.focus()`
from `jet.focusWidget`, the controller reacts identically:

1. Resolve `event.target` → `jetNodeId` via the registry.
2. Update internal `focused: JetNodeId | null` state within the
   current rAF tick.
3. Schedule a repaint of the focus ring per ADR-006 / #2156.
4. Re-emit on the canvas-side `focus` channel with payload
   `{ jetNodeId, modality: lastInputModality() }`.

This is the single-pipeline guarantee: every focus change, no matter
its origin, threads through one observer and produces one channel
event. User code listening on the channel receives a uniform stream.

### `blur()` — symmetric

```ts
function blur(this: JetNode) {
  const proxy = registry.lookup(this.jetNodeId);
  if (!proxy) return;
  if (document.activeElement !== proxy) return;  // no-op per DOM
  proxy.blur();
}

function blurFocused() {
  const id = focusController.activeJetNodeId();
  if (id == null) return;
  const proxy = registry.lookup(id);
  proxy?.blur();
}
```

`blur()` is the cheap direction: there is no `preventScroll` and no
trap interaction (releasing focus is always allowed). The
focus controller observes `focusout` on the proxy and emits a canvas
`blur` channel event with the same `jetNodeId` payload.

### Event order

The DOM order for a transfer from proxy A to proxy B is fixed by
HTML §6.5.3:

1. `blur` on A (non-bubbling)
2. `focusout` on A (bubbling)
3. `focus` on B (non-bubbling)
4. `focusin` on B (bubbling)

Because the focus controller's listeners are capture-phase on the
`<jet-semantics>` host, it observes all four events in order. The
re-emitted canvas channel events therefore also fire in this order:
canvas `blur` on `jetNodeId(A)` → canvas `focusout` walking A's
focusable ancestor chain → canvas `focus` on `jetNodeId(B)` → canvas
`focusin` walking B's chain. R8 is an event-order parity test that
asserts this byte-for-byte against the DOM equivalent.

From the program's view (the line after `jetNode.focus()`) the
transfer is **synchronous**: `document.activeElement` is already B.
From the canvas-paint view it is **microtask-async**: the focus-ring
repaint lands within the same rAF but does not block the JS return.

### `:focus-visible` modality

R6 is the subtle one. The contract is: programmatic focus inside a
keyboard event handler must paint a focus ring; the same call inside
a mouse handler must not. The browser's heuristic for `:focus-visible`
on the proxy uses the **currently-being-processed input event** as
the modality signal. Because `proxy.focus()` is called synchronously
inside the user's handler, the browser sees a keyboard event in
progress (or a mouse event, or nothing) and computes `:focus-visible`
accordingly. The focus controller then reads
`proxy.matches(':focus-visible')` after the focus change and forwards
the boolean into the canvas-side ring paint.

The "no async work between input event and `proxy.focus()`"
constraint is the reason the outbound bridge is the tight two-step
shown above — anything longer (a Promise, a deferred handler, a
`requestAnimationFrame`) breaks R6.

### Disabled, detached, and trapped

- **Disabled** — `this.disabled` is set from the canvas widget's
  semantic model. Disabled widgets get `tabindex="-1"` and
  `aria-disabled="true"` on their proxies already (ADR-004); the
  guard in `focus()` short-circuits before touching the proxy, so
  no `focus`/`blur` event quartet fires. Matches `<button disabled>`.
- **Detached** — `registry.lookup(id)` returns `undefined` for
  unmounted widgets. The method returns. Matches the DOM behaviour
  of `detachedNode.focus()` (silent no-op).
- **Trapped** — when a `FocusTrap` (ADR-007) is active and the target
  is outside its subtree, the call is intercepted. Default: no-op +
  `warn`. Opt-in `{ force: true }`: bypass. This is the same surface
  as MUI's `FocusTrap`'s `disableEnforceFocus`.

### `focusChannel.activeJetNodeId()`

Read accessor: returns the `jetNodeId` whose proxy is currently
`document.activeElement`, or `null` if focus is on a non-jet element
(including `<body>`). Implemented as a registry reverse-lookup on
`document.activeElement`. Used by parity tests and by jet-internal
code that needs to know whether the canvas currently holds focus.

## Consequences

**Wins**

- React refs Just Work. `<JetButton ref={r} />; r.current.focus()`
  produces the same observable behaviour as `<button ref={r} />`.
- MUI internals (Dialog, Autocomplete, Snackbar, Wizard) port
  unchanged. None of them touch the proxy directly; they call
  `.focus()` on a ref and the bridge does the rest.
- `document.activeElement` is canonical at all times. Ancestor CSS
  selectors (`:focus-within`, `:has(:focus)`) and the AX tree both
  reflect the canvas focus state correctly.
- Single pipeline. Tab traversal, programmatic focus, and click-focus
  all converge on the same controller — every focus change emits one
  channel event.

**Costs**

- The disabled / trap / detached checks must run on the **outbound**
  side before touching the proxy, because once `proxy.focus()` fires
  it is too late: the browser will commit the focus change and emit
  events that the controller will then have to undo. The outbound
  guard is the source of truth for "is this focus call allowed".
- `force: true` is a jet-only option not present in the DOM. Callers
  porting from web get it for free (they never pass it); callers
  writing jet-aware code can use it deliberately. Documented in the
  parity gap manifest.
- The synchronous-call requirement for R6 leaks into the channel
  contract: middleware that wants to intercept programmatic focus
  cannot defer the decision asynchronously without losing
  `:focus-visible` modality. We exposed a `beforeFocus` sync hook
  for this; any async intercept must explicitly opt out of modality
  parity.

**Risks**

- Browsers may revise `:focus-visible` heuristics. We pin behaviour
  to the WPT subset (#2163) so regressions surface in CI rather than
  in user reports.
- Re-entrancy: a `focus` channel event handler that calls
  `jetNode.focus()` on a different widget could re-enter the
  controller mid-paint. The controller serializes via a single rAF
  tick; nested `focus()` calls within the same tick collapse to the
  last write wins. Documented; mirrors how `<button onFocus={...}>`
  recursion behaves in the DOM (browser permits it, last write wins).

## Alternatives considered

1. **Asynchronous bridge** — defer `proxy.focus()` to a microtask
   to give middleware time to intercept. Rejected: breaks R6
   (`:focus-visible` modality requires synchronous call inside the
   originating input event).
2. **Pure canvas-side focus, no `document.activeElement` update** —
   keep focus state entirely on the canvas; emit a canvas focus
   channel but never call `proxy.focus()`. Rejected: breaks
   `document.activeElement`, `:focus-within`, screen-reader focus
   tracking, password-manager affordances, and every MUI component
   that reads `document.activeElement`.
3. **`focus()` returning a Promise** — make the API explicitly async
   to discourage the "synchronous DOM mutation" model. Rejected:
   breaks parity (`HTMLElement.focus` returns `void`); MUI and React
   refs assume sync; would force every caller to `await`.
4. **Use the canvas's own focus state without a proxy** — assign
   `tabindex` to `<canvas>` and synthesize events. Rejected for the
   same reasons as ADR-004 (#2152): no AX, one focus stop, no
   `:focus-visible`, no IME caret.
5. **`force: true` always implicit** — drop the trap-interception
   default and let every programmatic focus bypass traps. Rejected:
   would silently break MUI Dialogs, command palettes, and every
   "focus shouldn't escape this modal" UX. The default has to match
   the DOM's `FocusTrap`-enforcing behaviour.

## Open questions

- Should `focus()` accept `{ visible: true | false | "auto" }` to
  override the browser's `:focus-visible` heuristic? Chromium has
  shipped a non-standard hint; we are watching the spec discussion.
  For now: omit; track in #2135.
- `autofocus` HTML-attribute parity is intentionally deferred. The
  trigger differs (mount-time) but the mechanism is the same; spin
  a follow-up issue once #2157 lands.
- How does this interact with iframe-hosted jet roots that lose focus
  to a sibling frame? Out of scope per epic; document the boundary
  in the integration guide.

## References

- Issue #2157 (this ADR)
- Parent epic #2135 — parity: keyboard + a11y on canvas
- ADR-004 (#2152) — `<jet-semantics>` shadow subtree (proxy substrate)
- ADR-003 (#2153) — focus channel + Tab order
- ADR-007 (#2154) — focus trap
- ADR-006 (#2156) — `:focus-visible` paint
- WHATWG HTML §6.5.3 — focusing steps and event order
- W3C CSSWG `:focus-visible` — modality heuristic
- WPT `html/interaction/focus/the-autofocus-attribute/` — focus
  conformance subset; the `element.focus()` cases are the targeted
  acceptance for this ADR.
