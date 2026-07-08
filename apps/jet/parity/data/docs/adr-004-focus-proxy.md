# ADR-004: Hidden DOM focus proxy (<jet-semantics> shadow subtree)

| Field | Value |
|-------|-------|
| Issue | #2152 |
| Parent epic | #2135 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Open shadow root on root canvas; one <jet-focus-proxy> per focusable widget, visually hidden via opacity+clip-path, position:absolute over hit-rect |

## Context

Jet renders its UI on a single `<canvas>` element. Every widget — buttons,
text fields, list rows, menu items — is painted by the renderer, not the
browser. From the DOM's perspective the entire app is one opaque
rectangle. This is fast and gives us total control over paint, but it
collides head-on with how the platform delivers **focus** and
**accessibility**: both are DOM-shaped.

Three concrete consequences:

1. **Keyboard focus has no home.** The browser's focus model walks DOM
   nodes by `tabindex`. A canvas has at most one focus stop (itself).
   We cannot Tab from "Send" to "Cancel" because, to Chrome, there is no
   "Send" and no "Cancel" — there is one `<canvas>`.
2. **Screen readers see nothing.** VoiceOver, NVDA, Narrator and Orca
   read the accessibility tree, which is derived from the DOM + ARIA.
   A bare canvas exposes one generic "image" node. The user hears
   silence where the app is.
3. **Browser-native UX is unreachable.** Focus rings (`:focus-visible`),
   IME composition popovers, password-manager affordances, OS-level
   "find on page", and devtools' "Inspect element" all key off DOM
   nodes. Without a DOM shadow of our widget tree we forfeit all of
   them.

The parent epic **#2135 (parity: keyboard + a11y on canvas)** frames the
full problem space. ADR-004 is the substrate that the rest of the
parity slices stand on:

| Issue | Depends on this ADR for |
|-------|-------------------------|
| #2153 | Focus traversal order — needs DOM nodes to walk |
| #2154 | Focus traps (modals/menus) — needs a DOM subtree to scope `inert` |
| #2155 | Synthetic focus events bridged to canvas widgets |
| #2156 | `:focus-visible` ring painting driven off proxy `:focus` state |
| #2157 | Live-region announcements (`aria-live`) hosted on the proxies |

Each of those slices needs a DOM node it can attach a `tabindex`, a
`role`, an `aria-label`, or an event listener to. ADR-004 says: that
node exists, here is what it looks like, here is who owns its
lifecycle.

## The <jet-semantics> shadow subtree

We attach an **open shadow root** to the root `<canvas>` element and
mount a single host element inside it:

```
<canvas id="jet-root">
  #shadow-root (open)
    <jet-semantics>
      <jet-focus-proxy data-jet-semantic-id="app/header/menu-button" ...></jet-focus-proxy>
      <jet-focus-proxy data-jet-semantic-id="app/header/search-input" ...></jet-focus-proxy>
      <jet-focus-proxy data-jet-semantic-id="app/main/list/row-0/title" ...></jet-focus-proxy>
      ...
    </jet-semantics>
</canvas>
```

Element names:

- **`<jet-semantics>`** — the host element. Exactly one per jet root
  canvas. It is a flat container; no nesting. Nesting would force us to
  reconstruct DOM ancestor chains that mirror the widget tree, which
  buys nothing for focus or AX and costs us a mutation per
  reparent. Instead, hierarchy is encoded in the `semantic_id` string
  (see § Canvas ↔ proxy ID protocol).
- **`<jet-focus-proxy>`** — one per focusable widget. Custom element
  registered under that tag name; the registration is bare (it carries
  no behaviour of its own — all behaviour is driven from the
  semantics host).

Why an **open** shadow root (not closed, not light DOM):

- **Open** so Playwright, devtools, and assistive tech that crawls
  shadow boundaries (Chromium's a11y tree does) can see the proxies.
  A closed root would hide them from the AX tree.
- **Shadow** (not light DOM) so canvas paint, CSS resets in the host
  page, and `document.querySelectorAll('*')` in user code don't
  interfere with our subtree. Style encapsulation is mandatory: the
  visual-hiding CSS must not be overridable by the embedder.

Why mount on the **root canvas** and not on `document.body`:

- Co-locates the semantics tree with the canvas it describes — a page
  hosting two jet roots gets two independent shadow trees.
- Lets us tear down semantics atomically when the canvas unmounts.
- Keeps the embedder's DOM clean: from the outside, jet is still one
  `<canvas>` node.

## Proxy element contract

Every `<jet-focus-proxy>` carries exactly these attributes:

| Attribute | Purpose | Example |
|-----------|---------|---------|
| `tabindex` | Participation in sequential focus | `tabindex="0"` (in order) / `tabindex="-1"` (programmatic only) |
| `role` | ARIA role mirroring the canvas widget | `role="button"` / `role="textbox"` / `role="option"` |
| `aria-label` | Accessible name | `aria-label="Send message"` |
| `data-jet-semantic-id` | Stable id linking back to the canvas widget | `data-jet-semantic-id="app/composer/send"` |

Additional ARIA state attributes (`aria-pressed`, `aria-expanded`,
`aria-checked`, `aria-disabled`, `aria-selected`, `aria-current`) are
mirrored on demand by the widget's state binding — they are not part of
the proxy's required attribute set, but the proxy is the canonical
host for them when they apply.

**Positioning.** Each proxy is `position: absolute` and sized to match
the widget's canvas hit-rect (the bounding rectangle the canvas
hit-tester uses for pointer events). This matters because the browser
paints the system focus ring (and screen-reader visual-focus
indicators, and Windows Magnifier's follow-focus rectangle) around the
focused DOM node's box. If the proxy is the wrong size or in the wrong
place, AT users get a focus indicator that drifts away from the actual
widget.

**Visual-hiding strategy.** The proxy must be invisible to the eye and
non-interactive for the mouse, but **still focusable**. The combination
that satisfies all four constraints:

```css
jet-focus-proxy {
  position: absolute;
  opacity: 0;
  pointer-events: none;
  color: transparent;
  clip-path: inset(50%);
  /* width/height/top/left set per-instance from the canvas hit-rect */
}
```

Why each rule:

- `opacity: 0` — invisible to the eye, but the element still
  participates in layout and remains focusable. (Contrast `visibility:
  hidden`, which strips focusability.)
- `pointer-events: none` — mouse and touch events fall through to the
  canvas. Pointer hit-testing is the canvas's job; the proxy is for
  keyboard and AT only.
- `color: transparent` — any accidental text content (we keep proxies
  empty, but defence-in-depth) renders invisibly.
- `clip-path: inset(50%)` — collapses the painted box to a zero-area
  region without removing it from the layout tree. This is the
  standard "visually hidden but focusable" trick. We deliberately do
  **not** use:
  - `display: none` — strips the element from the focus order entirely.
  - `visibility: hidden` — same problem.
  - `width: 0; height: 0` — some browsers skip zero-sized boxes during
    focus traversal.
  - the `hidden` attribute — same as `display: none`.

`clip-path: inset(50%)` is preferred over the older `clip: rect(0 0 0
0)` because `clip` only works on `position: absolute` elements (we
already are, but the modern property is unconditional) and is on a
deprecation track.

**Content.** Proxies are empty elements. The accessible name comes
from `aria-label`, not from text content. Empty content keeps the
DOM small (one widget = one tag, no children) and avoids paint
work the browser would otherwise do for descendant text nodes.

## Lifecycle

The semantics host owns a **`SemanticsHostMutation` queue**: a
per-frame batched list of DOM operations driven by canvas-tree
changes. Every widget mount/unmount/move/attribute-change pushes one
entry; the queue flushes once per frame, just before the next paint.

```
canvas tree change ─┐
                    ├─> SemanticsHostMutation queue ─> rAF flush ─> DOM writes
attribute change ───┘
```

Mutation kinds:

| Kind | Trigger | DOM effect |
|------|---------|------------|
| `Mount` | Focusable widget added to canvas tree | `appendChild(<jet-focus-proxy>)` with full attribute set |
| `Unmount` | Widget removed | `removeChild` matching `data-jet-semantic-id` |
| `Rebox` | Widget hit-rect changed (layout, scroll) | Update `style.top/left/width/height` |
| `Relabel` | `aria-label` or `role` changed | Update the affected attribute |
| `StateMirror` | `aria-pressed`/`aria-expanded`/etc. changed | Update the ARIA state attribute |
| `Reorder` | DOM order must change to match canvas tab order | Move proxy to new position among siblings |

**Batching rules.**

1. Mutations coalesce per widget per frame: ten `Rebox` events for the
   same widget in one frame collapse to one DOM write of the final box.
2. `Mount` + `Unmount` for the same `semantic_id` in the same frame
   cancel out.
3. Flush order within a frame: `Unmount` → `Mount` → `Reorder` →
   `Rebox` → `Relabel` → `StateMirror`. This minimises layout
   thrash (additions/removals first, then in-place updates).
4. The flush runs inside the same `requestAnimationFrame` callback
   that drives canvas paint, **before** paint. This guarantees that
   when paint commits, the DOM already reflects the same world the
   pixels show — preventing one-frame focus/paint drift.

**Tear-down.** When the root canvas unmounts, the shadow root goes
with it; the entire proxy tree is reclaimed atomically. No per-proxy
cleanup is required.

## Canvas ↔ proxy ID protocol

The link between canvas widget and DOM proxy is a single stable
string: the **`semantic_id`**.

**Format.** Kebab-case, slash-separated, parent-prefixed:

```
app/header/menu-button
app/main/list/row-0/title
app/composer/send
```

Rules:

- Lowercase ASCII letters, digits, `-` within a segment, `/` between
  segments.
- Each segment names the widget within its parent's scope.
- Repeated siblings get a stable suffix (`row-0`, `row-1`, ...) drawn
  from the list's key, not the array index — so reordering doesn't
  shuffle ids.
- Total length capped at 256 chars to keep DOM attribute size
  predictable.

**Bidirectional mapping.** Two lookup tables, both keyed by
`semantic_id`:

| Direction | Source of truth | How it's written |
|-----------|----------------|------------------|
| canvas widget → proxy element | canvas-side `SemanticIndex` | proxy created with `data-jet-semantic-id` at `Mount` |
| proxy element → canvas widget | DOM-side `data-jet-semantic-id` | resolved via `host.querySelector(...)` or a cached `Map<string, HTMLElement>` |

**Focus flow.**

- *Canvas-initiated focus* (programmatic, e.g. autofocus on mount, or
  arrow-key navigation inside the canvas): canvas state advances →
  semantics host looks up the proxy for the new `semantic_id` → calls
  `proxy.focus()`. The DOM focus event then drives the browser-native
  side effects (focus ring, AT announcement, scroll-into-view).
- *DOM-initiated focus* (user Tabs into the canvas, or AT calls
  `accessibilityPerformAction`): the proxy's `focusin` handler fires
  → handler reads `data-jet-semantic-id` → semantics host writes
  the matching canvas focus state → next paint shows the canvas-side
  focus treatment.

The two paths are **idempotent**: a canvas-initiated focus that calls
`proxy.focus()` will fire `focusin`, which will write the canvas state
that's already set — the second write is a no-op. We do not need
re-entrance guards.

**Why a string and not an opaque handle.** The semantic id has to
survive a round-trip through the DOM attribute (which is a string),
through the AX tree (which serialises strings), and through
Playwright selectors (`[data-jet-semantic-id="app/composer/send"]`).
Opaque numeric handles would force a translation layer at every
boundary.

## Test surface

Concrete, automatable assertions. All of these are Playwright-driven
against a real Chromium and run as part of the parity suite.

1. **Proxy cardinality.**
   `count(<jet-focus-proxy>) === count(focusable canvas widgets)`. The
   test enumerates the canvas widget tree via a test-only inspector
   API and counts those with `is_focusable() == true`; the DOM side
   counts `host.querySelectorAll('jet-focus-proxy')`. Equal.

2. **Every proxy is focusable.** For each proxy in document order:
   `await proxy.focus(); expect(proxy).toBeFocused();`. A proxy that
   fails this assertion has been hidden with a focus-stripping CSS
   property (`display:none`, `visibility:hidden`, `inert`, etc.) — a
   regression on the visual-hiding contract.

3. **Tab traversal walks all proxies.** Starting from `body`, press
   `Tab` N times where N = proxy count; assert each proxy is focused
   exactly once and in the order dictated by canvas tab-order. (This
   overlaps with #2153's surface; ADR-004 owns the cardinality half,
   #2153 owns the ordering half.)

4. **AX tree reflects proxy roles.** Via the `#2160` AX-tree snapshot
   facility: every proxy with `role="button"` appears as a button
   node; every `role="textbox"` appears as an edit node; etc. No
   proxy appears as "generic" or "unknown".

5. **`focusin` round-trips to canvas.** Focus a proxy directly
   (`proxy.focus()`); assert the canvas-side `focused_widget_id`
   equals the proxy's `data-jet-semantic-id`.

6. **Canvas focus drives `proxy.focus()`.** Programmatically focus a
   canvas widget via the inspector API; assert
   `document.activeElement` (descending into the shadow root) is the
   matching proxy.

7. **`SemanticsHostMutation` queue flushes within one frame.** Mount a
   widget; on the next `requestAnimationFrame` callback, assert the
   proxy exists. The queue must not require multiple frames to drain.

8. **Hit-rect tracking.** Move a widget; on the next frame, assert
   the proxy's bounding rect matches the new hit-rect within 1px
   tolerance.

## Out of scope

- **Implementation.** This ADR is a design contract. The Rust/TS code
  that mounts `<jet-semantics>`, manages the mutation queue, and
  wires the canvas ↔ proxy id map lands in a follow-up slice driven
  off this document.
- **`:focus-visible` ring painting.** When a proxy receives focus,
  the canvas needs to paint a focus ring on the matching widget. The
  paint logic, ring style, and the heuristic that decides
  keyboard-vs-mouse focus all live in **#2156**. ADR-004 only
  guarantees that proxy focus state is observable.
- **Focus traps (modals, menus).** Scoping `Tab` traversal inside a
  modal subtree is handled by **#2154**, which will apply `inert` to
  sibling proxies. ADR-004 makes that possible by giving #2154 DOM
  nodes to mark inert.
- **Synthetic event bridging.** Mapping a proxy's `keydown` to a
  canvas-side widget event lives in **#2155**.
- **Live regions / announcements.** Aria-live announcements that ride
  on proxies are owned by **#2157**.
- **AX tree introspection.** Snapshot/diff tooling for the
  accessibility tree is **#2160**'s deliverable; ADR-004 just makes
  the tree non-empty.
- **Non-focusable semantics.** Static text, decorative regions, and
  landmark roles (`role="main"`, `role="navigation"`) may need DOM
  representation too, but they are not focusable and are out of scope
  for this ADR. A future ADR may extend `<jet-semantics>` with a
  parallel `<jet-semantic-region>` element.

## Follow-ups

Candidate `aw wi` items to file once ADR-004 lands:

1. **`feat(jet): implement <jet-semantics> shadow root mount`** —
   ship the open shadow root attachment on canvas mount and the
   `<jet-semantics>` host element. Empty queue, no proxies yet. Type:
   enhancement. Project: jet. Priority: p1.
2. **`feat(jet): SemanticsHostMutation queue with rAF flush`** —
   implement the per-frame batched DOM-write queue with the coalescing
   rules from § Lifecycle. Type: enhancement. Priority: p1.
3. **`feat(jet): semantic_id index on canvas widget tree`** — add
   stable, parent-prefixed, kebab-case ids to every focusable widget;
   expose a test-only inspector API. Type: enhancement. Priority: p1.
4. **`feat(jet): <jet-focus-proxy> custom element + visual-hiding CSS`** —
   register the element, ship the style sheet inside the shadow
   root. Type: enhancement. Priority: p1.
5. **`test(jet): parity assertions 1-8 from ADR-004`** — author the
   Playwright suite covering cardinality, focusability, traversal, AX
   roles, round-trip, and hit-rect tracking. Type: test. Priority: p1.
6. **`refactor(jet): extract focus-proxy lifecycle into reusable hook`** —
   once #2153–#2157 have landed against the proxy contract, refactor
   shared lifecycle plumbing out of each consumer. Type: refactor.
   Priority: p2.
