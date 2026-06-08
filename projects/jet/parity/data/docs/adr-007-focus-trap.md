# ADR-007: Focus trap for Dialog/Menu/Drawer (MUI FocusTrap parity)

| Field | Value |
|-------|-------|
| Issue | #2154 |
| Parent epic | #2135 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Tabindex-flip on proxy subtree (no DOM moves) + FocusTrapStack for nesting + 4 MUI-derived parity fixtures |

## Context

This ADR is the third concrete deliverable under the jet/MUI parity epic
**#2135 (MUI parity: accessibility primitives)**. It builds directly on
two predecessors:

- **#2152 — Proxy subtree** (ADR-004): jet's "proxies are real DOM
  elements that mirror canvas nodes" architecture. Every interactive
  surface in jet is a canvas node, but every canvas node has a hidden
  DOM proxy (`<button>`, `<a>`, `<input>`, etc.) whose `tabindex`,
  ARIA roles, and event wiring drive the actual keyboard / AT
  experience. The canvas is what users see; the proxy subtree is what
  screen readers and the browser's focus engine touch.
- **#2153 — Tab order** (ADR-006): the rules that linearize the proxy
  subtree into a deterministic Tab sequence. Without that linearization,
  any focus-trap behaviour is undefined — there is no "next focusable"
  to wrap to.

With proxies in place and tab order deterministic, the remaining
keyboard-a11y gap before we can ship parity-grade Dialog / Menu /
Drawer is the **focus trap**: the invariant that while a modal-ish
surface is open, focus cannot escape it via Tab / Shift-Tab, and on
close it lands somewhere predictable.

### MUI's `@mui/base` FocusTrap behaviour summary

MUI's `FocusTrap` (re-exported from `@mui/base/FocusTrap`, used by
`Dialog`, `Modal`, `Menu`, `Drawer`, `Popover`) implements the
following observable contract — derived from reading
`packages/mui-base/src/FocusTrap/FocusTrap.tsx` and the unit-test
suite at `packages/mui-base/src/FocusTrap/FocusTrap.test.tsx` on
`@mui/base@5.0.0-beta.40`:

1. **On open**: moves focus into the trapped subtree via a chain of
   fallbacks (explicit `initialFocus` prop → first focusable
   descendant with `autoFocus` → first focusable descendant → the
   trap root itself with synthesized `tabindex="-1"`).
2. **On Tab at last element**: wraps focus to the first focusable
   descendant.
3. **On Shift-Tab at first element**: wraps focus to the last
   focusable descendant.
4. **Sentinel guards**: MUI inserts two zero-size `<div tabindex="0">`
   "sentinels" before and after the trapped subtree. When focus lands
   on a sentinel (because the browser's Tab handling escaped the
   trapped DOM region), the FocusTrap pushes focus back into the
   subtree.
5. **On close**: restores focus to the element that was
   `document.activeElement` immediately before open, provided it is
   still focusable and still in the DOM; otherwise falls back to
   `document.body`.
6. **Nesting**: multiple FocusTraps can be active at once; the most
   recently opened one "wins" — it owns Tab interception and Esc
   dispatch. When it closes, the next-most-recent regains control.

Jet must match (1)–(6) under the proxy-subtree constraint: **we cannot
move the proxies in the DOM** (doing so would desync canvas-node ↔
proxy correspondence, break tab order, and re-trigger MutationObservers
in assistive tech). Everything must be expressed as `tabindex` flips
and a runtime stack.

## Trap behaviour contract

The jet focus trap is defined by four required behaviours. Any
implementation (now or in a later slice) must satisfy all four. Parity
fixtures in §7 are the executable form of this contract.

| # | Behaviour | Test scenario |
|---|-----------|---------------|
| (a) | **Move initial focus into surface on open.** When a modal/menu/drawer opens, focus must land inside it within the same task (no animation-frame delay), per the initial-focus-target resolution rules in §5. | `mui-dialog-focustrap-cycle-v1` — opens Dialog, asserts `document.activeElement` is the first focusable child of the dialog root. |
| (b) | **Wrap Tab / Shift-Tab at first/last descendant.** Tab on the last focusable descendant moves focus to the first; Shift-Tab on the first focusable descendant moves focus to the last. No focus may land on a proxy outside the trapped subtree. | `mui-menu-focustrap-shift-v1` — opens Menu, Tabs to last item, presses Tab, asserts focus wraps to first menu item. |
| (c) | **Restore prior focus on close.** The element that was `document.activeElement` immediately before open is re-focused on close, per the restore semantics in §6. | `mui-drawer-focustrap-restore-v1` — focuses a `<button id="trigger">`, opens Drawer, Escapes, asserts `document.activeElement === #trigger`. |
| (d) | **Stack nested traps.** Opening a Menu inside a Dialog pushes a new trap onto the stack; the Menu owns Tab interception until it closes; the Dialog then regains control. Esc on the Menu closes the Menu only, not the Dialog. | `mui-nested-dialog-menu-v1` — opens Dialog, opens Menu inside it, asserts Tab cycles within Menu; closes Menu, asserts Tab now cycles within Dialog; closes Dialog, asserts prior focus restored. |

The contract is **proxy-aware**: "focusable descendant" means
"proxy whose canvas-node is a descendant of the trapped surface's
canvas-node root, and whose effective `tabindex >= 0` per ADR-006's
tab-order rules". It does NOT mean "DOM descendant of the surface's
proxy" — proxies are siblings in the proxy subtree, not parents and
children. This distinction matters because in jet's architecture, the
Dialog proxy and the button-inside-Dialog proxy are both top-level
children of the proxy subtree root; their parent/child relationship
exists only on the canvas side.

## Proxy-subtree implementation

The implementation strategy is dictated by ADR-004's hard rule:
**proxies do not move**. Two corollaries follow.

**First corollary — focus-channel-driven membership.** When a canvas
modal surface opens (Dialog, Menu, Drawer, etc.), it emits an
`open` event on the focus channel. The focus-trap layer subscribes to
this channel and, on each `open`:

1. Walks the canvas subtree rooted at the modal canvas-node.
2. For each canvas-node encountered, looks up its proxy via the
   canvas-node-id → proxy map (the same map ADR-004 uses for event
   forwarding).
3. Adds every such proxy to the new trap's **focusable set**, in
   ADR-006 tab order.
4. Walks the **complement** — every proxy whose canvas-node is NOT a
   descendant of the modal root — and for each one:
   - Saves its current `tabindex` attribute to a private
     `data-jet-saved-tabindex` attribute (or to a side-table keyed by
     proxy id; the chosen storage is an implementation detail of the
     later slice, but it MUST be round-trippable).
   - Sets `tabindex="-1"`.

On close, the operation is exactly reversed: complement proxies have
their saved tabindex restored; the focusable set is dropped.

**Second corollary — never move proxies in the DOM.** The trap must
be implementable without any `appendChild` / `removeChild` /
`insertBefore` calls on proxies. This rules out the MUI sentinel
technique (sentinels are DOM siblings of the trapped subtree;
inserting them would mutate proxy-subtree structure and re-trigger
canvas-node ↔ proxy reconciliation). Instead, jet uses a global
`keydown` listener at the proxy-subtree root (added at trap push,
removed at pop) that intercepts Tab / Shift-Tab and computes the
next/previous focus from the focusable set directly. See §4.

The "tabindex flip" approach has one observable difference from
sentinels: if some non-jet code calls `someProxy.focus()` on a
complement proxy directly (bypassing Tab), focus DOES land there
because `.focus()` works even on `tabindex="-1"` elements. The trap
MUST handle this by listening to `focusin` at the proxy-subtree root
and, if the event target is not in the focusable set, calling
`.focus()` on the trap root to pull focus back. This is the jet
equivalent of MUI's sentinel-recovery loop.

## Trap stack

`FocusTrapStack` is the runtime data structure that arbitrates
between overlapping traps.

```ts
interface FocusTrapStack {
  push(entry: FocusTrapEntry): void;
  pop(modalSemanticId: string): void;
  top(): FocusTrapEntry | undefined;
  // Internal: re-keyed by modalSemanticId so out-of-order pops
  // (Dialog A → Menu B → close A → close B) are well-defined.
  has(modalSemanticId: string): boolean;
}

interface FocusTrapEntry {
  modalSemanticId: string;          // e.g. "dialog:settings", "menu:user-actions"
  modalCanvasRootId: string;        // canvas-node id
  focusableSet: ProxyId[];          // in ADR-006 tab order
  savedTabindex: Map<ProxyId, string | null>;  // for complement restore
  previouslyActive: Element | null; // for restore-focus on close
  initialFocusRef?: ProxyId;        // explicit override, if provided
}
```

Stack semantics:

- **push (on open)**: append a new entry; install the global Tab
  interceptor and `focusin` recovery listener (idempotent — only the
  first push installs; refcount tracks lifetimes). The just-pushed
  entry becomes `top()`.
- **pop (on close)**: remove the entry by `modalSemanticId` (NOT by
  position — see "out-of-order" caveat below). Restore complement
  tabindex from `savedTabindex`. Restore prior focus per §6. If the
  stack is now empty, uninstall the global listeners.
- **top() ownership**: only the top entry's `focusableSet` is consulted
  for Tab wrapping; only the top entry's `previouslyActive` is consumed
  on its own pop. Esc dispatches `close` on the top trap only — lower
  traps are inert until they become top.

**Keyed by modal semantic id.** The stack key is jet's modal-semantic
id (`dialog:settings`, `menu:user-actions`, `drawer:nav`), NOT raw
array position. This handles the legal-but-unusual case where modals
close out of LIFO order (e.g. programmatic close of an outer Dialog
while a Menu inside it is still open). On out-of-order pop:

1. The inner Menu's trap remains intact and on top — it still owns
   Tab interception.
2. The outer Dialog's `savedTabindex` map is applied to its complement
   immediately, which may legitimately re-enable tabindex on proxies
   that the Menu's complement also disabled.
3. To resolve the conflict, the Menu's `savedTabindex` is reconstructed
   from "current tabindex AFTER Dialog's pop" — i.e. Menu's restore
   data is rebased. This is implemented by walking the stack from
   bottom to top and recomputing each entry's `savedTabindex` against
   the previous entry's post-pop state.

This rebase is O(stack_depth × proxy_count) and runs only on
out-of-order pops, which are rare. The common LIFO case is O(1).

## Initial-focus-target resolution

When a trap is pushed, focus must move into the trapped surface. The
resolution order, matching MUI:

1. **Explicit `initialFocus` ref.** If the surface props include
   `initialFocus={proxyRef}`, use that proxy. This is the escape
   hatch for surfaces that need to focus, say, the "Cancel" button
   instead of the first focusable.
2. **First focusable descendant with `autoFocus`.** Walk
   `focusableSet` in tab order; pick the first whose canvas-node
   carries the `autoFocus` semantic prop. (Note: `autoFocus` is a
   jet semantic-layer prop, not the DOM attribute — the DOM
   attribute is consumed and discarded at proxy-creation time per
   ADR-004.)
3. **First focusable descendant.** Walk `focusableSet` in tab order;
   pick `focusableSet[0]` if non-empty.
4. **The modal root itself.** If `focusableSet` is empty (a modal
   with no interactive content — e.g. an alert with only text), the
   modal's own proxy is synthesized into a tabbable target by
   temporarily setting `tabindex="-1"` on it (saved + restored
   symmetrically to complement handling) and calling `.focus()`. The
   `tabindex="-1"` is required because `.focus()` on an otherwise
   non-focusable element is a no-op in some browsers.

Resolution runs synchronously in the same microtask as `push()`. No
`requestAnimationFrame` delay, because (per MUI parity tests)
keyboard users who Tab immediately after open expect focus to
already be inside.

## Restore-focus semantics

On trap push, the trap captures `previouslyActive = document.activeElement`.
On trap pop, restore proceeds as:

1. If `previouslyActive` is `null` (initial document had nothing
   focused, e.g. immediately after page load) → focus
   `document.body`.
2. Else if `previouslyActive` is no longer in the DOM (was unmounted
   while the modal was open — e.g. a route change inside the modal
   removed the trigger) → focus `document.body`.
3. Else if `previouslyActive` is in the DOM but its computed
   `tabindex` is `< 0` AND it is not natively focusable (`<input>`,
   `<button>`, etc.) → focus `document.body`.
4. Else → call `previouslyActive.focus()` and verify
   `document.activeElement === previouslyActive`. If verification
   fails (e.g. the element is `disabled`, or in an inert subtree),
   fall back to `document.body`.

`document.body` as the final fallback is deliberate: it matches MUI,
it's always in the DOM, and screen readers handle the "focus on body"
state predictably (it announces the page context). The alternative —
leaving focus where the modal's last-focused element was — would
leak focus into a now-hidden subtree and is rejected.

The "still focusable + still in DOM" check is run via the same
predicate ADR-006 uses for tab-order eligibility, ensuring restore
and Tab agree on what counts as focusable.

## Parity fixtures

Four new fixtures land under
`projects/jet/data/parity/fixtures/mui/`. Each fixture is a pair of files:

- `<fixture-id>.tsx` — JSX placeholder using jet's
  Dialog/Menu/Drawer primitives (the placeholders compile but do not
  yet implement the trap — the fixtures will go from `pending` to
  `passing` when the implementation slice ships).
- `<fixture-id>.expected_focus_trace.json` — the expected sequence
  of `document.activeElement` values (identified by stable
  `data-testid` attributes) as the test driver issues Tab /
  Shift-Tab / Esc / open / close events.

### `mui-dialog-focustrap-cycle-v1`

JSX: Dialog with three buttons `[ok, cancel, more]` and an external
`<button id="trigger">Open</button>`. Driver sequence: focus trigger
→ click trigger → Tab × 4. Expected trace:
`[trigger, ok, cancel, more, ok]` — Tab from `more` wraps to `ok`.

### `mui-menu-focustrap-shift-v1`

JSX: Menu with four items `[item-a, item-b, item-c, item-d]` and an
external `<button id="trigger">Open</button>`. Driver sequence: focus
trigger → click trigger → Shift-Tab. Expected trace:
`[trigger, item-a, item-d]` — focus opens on `item-a`, Shift-Tab
wraps to `item-d`.

### `mui-drawer-focustrap-restore-v1`

JSX: Drawer with two buttons `[close, save]` and an external
`<button id="trigger">Open</button>`. Driver sequence: focus trigger
→ click trigger → Escape. Expected trace:
`[trigger, close, trigger]` — Esc closes the drawer and restores
focus to the trigger.

### `mui-nested-dialog-menu-v1`

JSX: Dialog with `[ok, open-menu, cancel]`; clicking `open-menu`
opens a Menu with `[m1, m2, m3]`. External
`<button id="trigger">Open Dialog</button>`. Driver sequence: focus
trigger → click trigger → Tab → click open-menu → Tab × 3 → Escape
→ Tab × 2 → Escape. Expected trace:
`[trigger, ok, open-menu, m1, m2, m3, m1, open-menu, cancel, ok, trigger]`.
This asserts: (1) Dialog trap pushes; (2) Menu trap pushes on top
and wraps within `[m1, m2, m3]`; (3) Esc on Menu pops only the
Menu, returning Tab to the Dialog focusable set; (4) Esc on Dialog
pops the Dialog and restores prior focus.

Fixture format follows the convention established by the
ADR-005 / ADR-006 fixtures (`<fixture-id>.tsx` + sidecar JSON), so
the existing parity test runner picks them up without runner
changes.

## Out of scope

The following are explicitly NOT decided by this ADR. They are
flagged as separate issues so reviewers do not block on them.

- **Actual FocusTrap implementation in jet** — this ADR is design
  only. The implementation slice (writing `FocusTrapStack`, wiring
  the focus channel, installing global listeners, handling
  `focusin` recovery) ships in a follow-up issue. The fixtures
  authored here will be `pending` until that slice lands.
- **`:focus-visible` ring** — issue **#2156**. Focus-visible is the
  *visual* indication that an element has keyboard focus; this ADR
  is about *which* element has focus. The two compose but are
  orthogonal.
- **Roving tabindex** — issue **#2155**. Roving tabindex is the
  pattern for composite widgets (toolbar, listbox, tablist) where
  only one descendant is tabbable at a time and arrow keys move
  focus. The trap operates on top of whatever tab-order ADR-006 and
  the eventual roving-tabindex ADR produce; it doesn't compete with
  them.
- **Inert / `aria-hidden` of the background** — handled by ADR-008
  (modal backdrop / inert background), to be authored next. The
  trap and the inert background are separate concerns: the trap
  controls keyboard focus, inert controls AT navigation and pointer
  events.
- **iframe and shadow-root traversal** — jet does not yet ship
  surfaces that embed iframes or open shadow roots. When it does,
  the focusable-set walk will need to recurse into them; deferred
  until a real surface needs it.

## Follow-ups

1. **Implementation slice (#2154-impl)** — write `FocusTrapStack`,
   wire focus-channel `open`/`close` events, install global Tab
   interceptor + `focusin` recovery; turn the four parity fixtures
   green.
2. **ADR-008: modal backdrop + inert background** — the natural
   pair to this ADR. Specifies how `inert` / `aria-hidden` is
   applied to the complement subtree so AT users see only the open
   modal.
3. **Focus-trap interaction with `roving-tabindex` (#2155)** —
   when a Menu with roving tabindex is the top trap, Tab should
   exit the menu (per ARIA APG), not cycle within it. Wrapping
   behaviour (b) needs an opt-out for roving-tabindex surfaces.
   Spec this as part of #2155.
4. **Programmatic focus during trap** — what happens if app code
   calls `someProxy.focus()` on a complement proxy mid-trap? §3
   says we pull focus back via `focusin` recovery, but this could
   cause an infinite loop if the app keeps re-calling. Add a
   one-frame debounce and a console warning when recovery fires
   more than N times per second.
5. **Trap stack inspector** — a dev-tools panel that visualizes
   the current `FocusTrapStack`, each entry's focusable set, and
   the live `document.activeElement`. Invaluable for debugging
   parity failures.
6. **MUI ContainerFocusTrap parity** — MUI's `Modal` accepts a
   `container` prop that re-parents the modal into an arbitrary
   DOM node. Jet's canvas-first model makes this trivial on the
   canvas side, but the proxy subtree always lives at a single
   well-known root. Decide whether jet exposes a `container`
   equivalent (and what proxy-side semantics it has) or whether
   the canvas-side answer is sufficient.
