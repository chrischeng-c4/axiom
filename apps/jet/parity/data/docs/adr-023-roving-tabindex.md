# ADR-023: Roving tabindex composite navigation (toolbar / listbox / menu / radiogroup / tablist / grid)

| Field | Value |
|-------|-------|
| Issue | #2155 |
| Parent epic | #2135 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Expose a `RovingTabindexGroup` primitive on the focus channel that maintains the one-active-descendant invariant across a set of canvas-side member nodes; exactly one member's `<jet-focus-proxy>` carries `tabindex="0"` while all siblings carry `tabindex="-1"`; the primitive exposes a pure `moveActive(direction)` API plus `setActive(jetNodeId)`, and when DOM focus is inside the group it re-issues `proxy.focus({ preventScroll: true })` per ADR-020 so `document.activeElement` follows the canvas-side active descendant. Per-role key maps (toolbar / listbox / menubar / radiogroup / tablist / grid) live on the adapter, not the primitive. |

## Context

The WAI-ARIA Authoring Practices Guide (APG) prescribes a single
keyboard contract for composite widgets — toolbars, listboxes, menus,
menubars, radio groups, tab lists, trees, and grids — that diverges
from native HTML's "every focusable is a tab stop" default in three
specific ways:

1. **One tab stop per composite.** Pressing `Tab` enters the widget on
   exactly one descendant — the *active descendant* — and the next
   `Tab` leaves the widget entirely. The user does not have to step
   through every toolbar button or every listbox row to reach the
   next outer focusable.
2. **Arrow keys move within.** Once focus is inside the composite,
   arrow keys (and `Home` / `End`, sometimes `PageUp` / `PageDown` /
   `Ctrl+Home` / `Ctrl+End`) re-target the active descendant without
   leaving the composite.
3. **The tab stop is remembered.** After the user `Tab`s out and
   `Shift+Tab`s back in, focus lands on the last-active descendant —
   not the first one. This is the property that lets a user "park" on
   a particular toolbar button or grid cell while bouncing in and out
   of surrounding inputs.

The DOM implementation idiom for (1)–(3) is *roving tabindex*: at any
instant, exactly one descendant of the composite has `tabindex="0"`
and every other descendant has `tabindex="-1"`. Tab order traversal
(ADR-003 / #2153) finds the single `tabindex="0"` member when entering
the widget; arrow key handlers re-assign the `0` to the new active
descendant and call `.focus()` on it so `document.activeElement` —
and the screen reader's announcement cursor — follows. MUI uses this
verbatim across `ToolBar`, `MenuList`, `TabList`, `RadioGroup`, and
`DataGrid` cell navigation, layered on top of `@mui/base`'s
`useToolbar`, `useListbox`, and `useRadioGroup` primitives.

Jet has the same APG contract to honour, but the widget itself is
canvas-rendered. The substrate that carries `tabindex` is the
`<jet-focus-proxy>` mirror inside `<jet-semantics>` (ADR-004 / #2152);
the substrate that decides "what is the next tab stop?" is the focus
channel's tab-order pipeline (ADR-003 / #2153); and the substrate
that synchronously moves `document.activeElement` to a chosen proxy
is the programmatic focus bridge (ADR-020 / #2157). What is missing
is the *coordination layer* — a primitive that knows "these N proxies
belong to one composite, exactly one of them is the tab stop, and
arrow keys rotate which one." Without it, every composite widget
(toolbar, listbox, grid) would reinvent the same state machine and
get the contract subtly wrong (different wrap policy, different
disabled-skip behaviour, different remembered-tab-stop semantics),
and parity with MUI would drift in an unauditable way.

ADR-023 specifies that coordination layer.

## Decision

We introduce a single primitive on the focus channel —
`RovingTabindexGroup` — that owns the one-active-descendant invariant
for a named set of member jet nodes. Per-role keyboard semantics are
**not** baked into the primitive; they ride on thin adapter modules
(`ToolBarAdapter`, `ListBoxAdapter`, `MenuBarAdapter`,
`RadioGroupAdapter`, `TabListAdapter`, `GridAdapter`) that map their
role's APG key contract onto the primitive's pure `moveActive`
surface. This split is non-negotiable: putting key bindings inside
the primitive would force every consumer to either accept the
default map or bypass the primitive entirely; both options have
historically caused drift in MUI's own composite widgets.

### Primitive shape

```ts
type Direction = 'next' | 'prev' | 'first' | 'last';

interface RovingTabindexGroupOptions {
  /** Initial active descendant. */
  initialActive: JetNodeId | 'first' | 'last';
  /** Wrap policy at the ends. */
  wrap: boolean;
  /** Whether disabled members are skipped by moveActive. */
  skipDisabled: boolean;
  /**
   * Reported on `aria-orientation` proxies (informational); does not
   * affect moveActive semantics, which are purely linear.
   */
  orientation: 'horizontal' | 'vertical' | 'both';
}

interface RovingTabindexGroup {
  /** Add a member (idempotent). */
  addMember(id: JetNodeId, opts?: { disabled?: boolean }): void;
  /** Remove a member; if it was active, active rolls to the next non-disabled. */
  removeMember(id: JetNodeId): void;
  /** Set the active descendant by id. */
  setActive(id: JetNodeId): void;
  /** Move the active descendant along an axis-agnostic direction. */
  moveActive(direction: Direction): void;
  /** Read the current active id. */
  getActive(): JetNodeId | null;
  /** Tear down — removes all members and clears proxy tabindex. */
  dispose(): void;
}
```

### Invariant

At every observable tick boundary (i.e., between dispatched events):

- Exactly one member's proxy has `tabindex="0"`.
- Every other member's proxy has `tabindex="-1"`.
- The single `0`-bearing proxy is the canvas-side active descendant
  reported by `getActive()`.

`addMember` / `removeMember` re-establish the invariant in the same
frame; we never observe a transient state where two members are both
`0` or where the group has zero `0`-members.

### Focus synchronization (the ADR-020 hand-off)

`setActive(id)` always rewrites proxy `tabindex`. It additionally
calls `newActive.proxy.focus({ preventScroll: true })` if and only if
the *previously*-active member's proxy currently holds DOM focus —
i.e., the user is actively inside the widget. If the group is
unfocused (the user is somewhere else on the page), we move the tab
stop silently without dragging focus back into the widget. This is
the behaviour MUI gets out of the box because in their world the
old active descendant *is* `document.activeElement`; we have to
replicate it explicitly because canvas-side active state and DOM
focus are decoupled.

### Per-role key maps (adapter layer)

The primitive does not bind keys. Adapters bind keys and call
`moveActive`. The APG key contract per role:

| Role | Prev | Next | First | Last | Notes |
|------|------|------|-------|------|-------|
| toolbar | `ArrowLeft` (h) / `ArrowUp` (v) | `ArrowRight` (h) / `ArrowDown` (v) | `Home` | `End` | Wrap optional; APG defaults to no wrap. |
| listbox (single-col) | `ArrowUp` | `ArrowDown` | `Home` | `End` | Type-ahead jumps to first member matching prefix. |
| listbox (multi-col) | `ArrowUp` / `ArrowLeft` | `ArrowDown` / `ArrowRight` | `Home` | `End` | 2D geometry mapped onto linear member order by row-major scan. |
| menubar | `ArrowLeft` | `ArrowRight` | `Home` | `End` | `ArrowDown` opens submenu (out of scope — owned by menu issue). |
| radiogroup | `ArrowUp` / `ArrowLeft` | `ArrowDown` / `ArrowRight` | n/a | n/a | Move *also* changes selection (different from listbox: selection follows focus). |
| tablist (horizontal) | `ArrowLeft` | `ArrowRight` | `Home` | `End` | Automatic vs manual activation configurable; in `automatic` mode focus = select. |
| tablist (vertical) | `ArrowUp` | `ArrowDown` | `Home` | `End` | Honour `aria-orientation`. |
| grid | `ArrowUp` / `ArrowDown` / `ArrowLeft` / `ArrowRight` | (same) | `Ctrl+Home` | `Ctrl+End` | `PageUp` / `PageDown` jump 10 rows; 2D semantics owned by `GridAdapter`. |

`aria-orientation` on the composite's proxy is informational for
assistive tech only; the adapter is responsible for choosing which
keys map to `prev` / `next`. (A horizontal toolbar adapter ignores
`ArrowUp` / `ArrowDown`; a vertical listbox adapter ignores
`ArrowLeft` / `ArrowRight` unless explicitly multi-column.)

### Disabled members

`skipDisabled: true` (the default) makes `moveActive('next')` skip
over members whose `addMember(id, { disabled: true })` flag is set,
matching MUI's `ToolBar` / `MenuList` behaviour and APG's "disabled
items remain focusable but are not part of arrow navigation" carve-
out for some composites — note that *toolbars* in APG do let users
focus disabled buttons via arrow, while *menus* skip them. Adapters
override `skipDisabled` for their role.

### Wrap policy

`wrap: false` (default) clamps at the ends — `moveActive('next')` on
the last member is a no-op. `wrap: true` rolls over to the first
member (and `'prev'` from the first rolls to the last). Toolbars
default to no wrap; menus default to wrap; grid is no-wrap on row
and column axes. The adapter picks the default; the consumer can
override per group.

### Activate vs focus (tablist auto-vs-manual)

Tablist uniquely distinguishes:

- **Automatic activation**: focus moves *and* the corresponding tab
  panel is displayed — implemented by the adapter calling
  `setActive` *and* its own `selectTab` in the same handler.
- **Manual activation**: arrow keys only move the tab stop; the user
  must press `Enter` or `Space` to switch panels. Implemented by the
  adapter calling only `setActive` on arrow; the `Enter` / `Space`
  handler reads `getActive()` and calls `selectTab`.

The primitive is agnostic; the choice lives in `TabListAdapter`
configuration.

### Remembered tab stop

Because `setActive` is the only mutator of proxy `tabindex` within a
group, and because it is called only on arrow / `Home` / `End` / `setActive`
APIs, the `0` member persists when DOM focus leaves the group (Tab
out, click elsewhere, focus dialog). When the user `Shift+Tab`s back
in, ADR-003 tab-order traversal finds the same `0`-member and lands
on it — no extra bookkeeping required. The invariant is self-
preserving.

### Adapters shipped in this TD

Per issue R5, three adapters land alongside the primitive:

- `ToolBarAdapter` — horizontal default, `Home` / `End`, no wrap, no
  type-ahead, focusable-when-disabled.
- `ListBoxAdapter` (covers `MenuListAdapter`) — vertical default,
  wrap, type-ahead via accessible-name lookup (a11y channel), skip
  disabled.
- `GridAdapter` — 2D cell navigation, no wrap on either axis,
  `Ctrl+Home` / `Ctrl+End` corner jumps; `PageUp` / `PageDown` 10-row
  jump deferred to grid-specific follow-up.

`MenuBarAdapter`, `RadioGroupAdapter`, `TabListAdapter` are
specified above but ship in follow-up issues that own the per-role
selection semantics.

## Consequences

### Positive

- One auditable state machine. Any composite widget that wires
  itself through `RovingTabindexGroup` automatically gets the APG
  contract right; drift from MUI is bounded by adapter code, not
  by per-widget reimplementation.
- Pure `moveActive(direction)` keeps the primitive testable in
  isolation — unit tests don't have to simulate keyboard events.
- The "remembered tab stop" invariant falls out for free from
  ADR-003 traversal, no extra storage.
- Disabled-skip and wrap policies are first-class config, not
  per-adapter overrides bolted on.
- Adapter split lets us add new composite roles (e.g., tree,
  combobox listbox) without touching the primitive.

### Negative

- Two-layer model (primitive + adapter) adds an indirection over
  "just bind arrow keys on the widget" that's easy to skip when
  prototyping a new widget. We address this by making the primitive
  the *only* way to write a `tabindex` on a proxy — direct mutation
  is rejected by the focus channel's invariant guard.
- Type-ahead requires the a11y channel's accessible-name source,
  which is out of scope for this issue; `ListBoxAdapter` ships with
  type-ahead disabled and is feature-flagged on once the a11y
  channel lands.
- 2D grid mapping onto a linear member order is row-major-only in
  this TD; column-major or skip-column-spanning behaviours are
  deferred.

### Neutral

- Wrap defaults differ per role; consumers occasionally have to
  remember which role wraps. We mitigate by documenting defaults on
  each adapter's TD spec.

## Alternatives considered

### A. Per-widget tabindex management (no primitive)

Each composite widget (toolbar, listbox, grid) implements its own
roving-tabindex state. Rejected: this is exactly what MUI did
pre-`@mui/base`, and the resulting drift across components is the
canonical case study for *why* MUI extracted `useToolbar` /
`useListbox` / `useRadioGroup` as hooks. Repeating that mistake in
jet would force per-widget parity tests instead of one primitive
test, and would make ADR-003's tab-order traversal harder to
reason about (it would have to trust N independent state machines
instead of one).

### B. `aria-activedescendant` instead of roving tabindex

APG permits `aria-activedescendant` as an alternative: a single
`tabindex="0"` *container* element holds DOM focus permanently, and
the active descendant is identified by id reference. Rejected for
the substrate: jet's substrate is the proxy mirror, and we want
`document.activeElement` to identify the *active descendant proxy*
(not its container) so that ADR-020 `proxy.focus()` semantics and
the screen reader's "what element is focused?" question converge on
the same node. Roving tabindex makes those two questions structur-
ally identical; `aria-activedescendant` requires every SR to consult
an attribute hop, which weaker AT often gets wrong. We leave room
for `aria-activedescendant`-style widgets (combobox listbox most
notably) in a follow-up; the primitive is compatible because
`setActive` mutates only proxy `tabindex`, and an `aria-active-
descendant` consumer can drive the same primitive without focus
moving.

### C. Bind keys inside the primitive

`RovingTabindexGroup` ships its own `ArrowRight` / `ArrowDown` /
etc. handler instead of exposing `moveActive`. Rejected:
configurations multiply combinatorially (orientation × wrap ×
type-ahead × skip-disabled × auto-vs-manual-activation), and per-
role key maps would have to be encoded as primitive flags. The
adapter split keeps the primitive small and lets each role's key
map live in code that is also responsible for its selection
semantics — closer to MUI's `useToolbar` / `useListbox` shape.

### D. Skip the primitive; let ADR-003 traversal alone enforce the invariant

Reject one-of-many-`tabindex="0"` at traversal time instead of at
mutation time. Rejected: traversal would have to pick one winner
arbitrarily on every Tab, breaking the "remembered tab stop"
guarantee (R6); the invariant has to be maintained at the proxy-
attribute level for SR consistency, not just at jet's internal
traversal level.

## Open questions

- **OQ1 — Multi-grid composite navigation.** A grid embedded inside
  a toolbar (an APG-legal but rare pattern) creates two nested
  `RovingTabindexGroup`s. Does `Tab` from inside the grid exit to
  the toolbar's next member, or does it exit the whole composite?
  Per APG: exits the whole composite. The primitive should support
  *nested* groups with a `parent: RovingTabindexGroup` pointer so
  that focus leaving the inner group restores the outer group's
  active descendant. Deferred to grid follow-up TD.
- **OQ2 — Type-ahead buffer timeout.** APG's type-ahead pattern
  accumulates keystrokes within a ~500 ms window then resets.
  `ListBoxAdapter` will need a configurable timeout; the exact
  default (500 ms vs MUI's 500 ms vs Windows's 1 s) is owned by
  the listbox adapter's TD.
- **OQ3 — IME keystrokes during type-ahead.** What happens when a
  user types a CJK character into a listbox's accessible-name
  search? Per ADR-013/-014/-016 the IME composition phase doesn't
  surface keystrokes until commit; type-ahead therefore sees only
  the committed accessible-name prefix. We should document this
  in `ListBoxAdapter`.
- **OQ4 — Reduced-motion `preventScroll`.** ADR-020 always passes
  `preventScroll: true` on the primitive-initiated focus call. If a
  caller wants the browser to scroll a freshly-active grid cell
  into view (a sensible default for grids), they must `setActive`
  then call `proxy.scrollIntoView()` manually. We may want a
  `scrollIntoViewOnActive` adapter option for `GridAdapter`;
  deferred.
- **OQ5 — Disabled-yet-tabbable toolbar items.** APG says toolbar
  *items* may be focused even when disabled (so screen-reader users
  can read why they're disabled). Our `skipDisabled` default for
  `ToolBarAdapter` is therefore `false`. This contradicts MUI's
  current default but matches APG; we'll document the divergence
  and let consumers override.

## Acceptance

Tied to issue #2155 requirements R1–R9:

- R1: `RovingTabindexGroup` primitive shipped with `(memberIds, options)`
  signature, `initialActive` / `orientation` / `wrap` honoured.
- R2: Invariant test — every tick boundary, exactly one member has
  `tabindex="0"`. Includes add/remove cases.
- R3: `setActive(id)` calls `proxy.focus({ preventScroll: true })`
  if and only if the previously-active proxy is `document.active-
  Element`. Unit test isolates both branches.
- R4: `moveActive(direction)` is pure (no key binding). Test rig
  drives it directly.
- R5: `ToolBarAdapter`, `ListBoxAdapter`, `GridAdapter` adapters
  ship in this TD.
- R6: Parity test (R9) covers the remembered tab stop.
- R7: Type-ahead hook exposes consumer interface; ships disabled
  pending a11y channel.
- R8: Parity test — MUI `ToolBar` × 5 `IconButton` vs jet `ToolBar`;
  five `ArrowRight`s then five `ArrowLeft`s; assert byte-identical
  active-index sequence and identical post-Tab outer landing.
- R9: Parity test — Tab in, `ArrowDown` × 2, Tab out, Tab in;
  third member active in both implementations.

## References

- [ARIA APG — Keyboard Interaction: Roving tabindex](https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/#kbd_roving_tabindex)
- [ARIA APG — Toolbar pattern](https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/)
- [ARIA APG — Listbox pattern](https://www.w3.org/WAI/ARIA/apg/patterns/listbox/)
- [ARIA APG — Menu and Menubar pattern](https://www.w3.org/WAI/ARIA/apg/patterns/menubar/)
- [ARIA APG — Radio Group pattern](https://www.w3.org/WAI/ARIA/apg/patterns/radio/)
- [ARIA APG — Tabs pattern](https://www.w3.org/WAI/ARIA/apg/patterns/tabs/)
- [ARIA APG — Grid pattern](https://www.w3.org/WAI/ARIA/apg/patterns/grid/)
- [`@mui/base` — `useToolbar`, `useListbox`, `useRadioGroup`](https://mui.com/base-ui/)
- ADR-003 / #2153 — focus channel tab order
- ADR-004 / #2152 — `<jet-semantics>` shadow subtree (proxy substrate)
- ADR-007 / #2154 — focus traps
- ADR-020 / #2157 — programmatic `proxy.focus()` bridging
- Issue #2155 — this ADR's parent
- Epic #2135 — focus parity rollup
