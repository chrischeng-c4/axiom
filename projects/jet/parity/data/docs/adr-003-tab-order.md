# ADR-003: Tab order — semantic-tree vs paint order

| Field        | Value |
|--------------|-------|
| Issue        | #2153 |
| Parent epic  | #2135 |
| Status       | accepted |
| Date         | 2026-05-16 |
| Decision     | Semantic-tree order default; `tab_order_override = "paint" \| "manual"` per-subtree opt-in |

## Context

Jet renders interactive widgets onto a canvas surface but must remain a
first-class citizen of the web platform: a real `Tab` press from the user
has to walk jet's widgets in an order that screen readers, automated
accessibility audits, and ported React/MUI components all agree on.

Issue **#2152** (the hidden DOM focus proxy) settled the *mechanism*:
every focusable jet widget is shadowed by an off-screen `<button>` /
`<div tabindex="0">` element living in a hidden DOM subtree. The browser's
own focus engine walks that subtree, and jet mirrors the focus ring back
onto the canvas. That decision left one question unanswered, which is
what this ADR resolves:

> When a user presses `Tab`, **what order** does jet visit its widgets in?

There are two natural answers:

- **Semantic-tree order** — the order widgets appear in the jet semantic
  tree (the canvas-side analogue of the DOM), which is also the order in
  which their hidden-DOM proxies are emitted. This is what
  `tabindex="0"` means in vanilla HTML, what React+MUI assume, and what
  the WPT `html/interaction/focus/sequential-focus-navigation` suite
  asserts.
- **Paint order** — the visual top-to-bottom, left-to-right order in
  which widgets are rasterised onto the canvas. This is appealing for
  freeform canvas layouts (whiteboards, node graphs) where source order
  is arbitrary and visual position is the only meaningful sequence.

The parent epic **#2135** (web-parity for jet input) requires us to pick
one as the default and document the divergence cases that motivate an
override knob. This ADR is the answer.

## Options

### Option A — Semantic-tree order

Walk widgets in the order they appear in the semantic tree (depth-first
pre-order traversal). The hidden-DOM proxies from #2152 are emitted in
the same order, so the browser's native focus engine "just works" —
we never reorder the proxy subtree, and `document.activeElement`
transitions match jet's own focus state machine.

**Absolute positioning.** Widgets with absolute positioning keep their
semantic-tree slot. A widget physically painted at `(800, 100)` but
declared third in source order is still the third Tab stop:

```rust
// Semantic tree (source order):
Column [
    Button("A"),                                  // tab stop 1
    Button("B", position = Absolute(800, 100)),   // tab stop 2 (despite top-right paint)
    Button("C"),                                  // tab stop 3
]
```

**Flex `order:`.** Changing the visual flex `order:` of a child does
**not** change its tab position under Option A. This matches the CSS
specification (`order` affects layout only, not navigation) and the
WPT test `sequential-focus-navigation.html` explicitly asserts this.

```rust
Flex [
    Button("A", flex_order = 3),  // tab stop 1, painted last
    Button("B", flex_order = 1),  // tab stop 2, painted first
    Button("C", flex_order = 2),  // tab stop 3, painted middle
]
// Tab sequence: A → B → C  (source order, NOT 1→2→3 paint order)
```

**CSS grid auto-placement.** Same rule: the auto-placement algorithm may
position grid items in arbitrary cells, but Tab still follows source
order. This is the behaviour Chromium/Firefox/Safari ship, and it is
what assistive technology has been trained on.

**RTL layouts.** In an RTL writing mode the visual sweep flips, but the
DOM/semantic-tree order does **not** flip. Tab continues to follow source
order. This matches `sequential-focus-navigation-rtl.html` in WPT.

### Option B — Paint order

Walk widgets in the order they are rasterised: top-to-bottom by `y`,
then left-to-right by `x` (or right-to-left under RTL), ignoring source
order entirely. Hidden DOM proxies would need to be re-sorted on every
relayout to keep the browser's native focus engine in sync — or jet
would have to intercept `Tab` itself, which breaks screen-reader
integration.

**Absolute positioning.** The absolutely-positioned widget at
`(800, 100)` is now the *first* tab stop, even though it was declared
second:

```rust
Column [
    Button("A"),                                  // painted at (0, 0)    — tab stop 2
    Button("B", position = Absolute(800, 100)),   // painted at (800, 100) — tab stop 3
    Button("C"),                                  // painted at (0, 40)   — tab stop 1? (left of B)
]
// Sort by (y, x): A(0,0) → C(0,40) → B(800,100)
// Tab sequence: A → C → B
```

(The exact answer depends on the y-banding tolerance — does row 0 mean
"y < 20" or "y < 40"? Paint order needs to pick a band height and bake
it into the spec.)

**Flex `order:`.** Flex `order:` *does* affect paint position, so under
Option B it would also affect Tab order — the opposite of every browser.
Porting MUI's `<Tabs>` would silently break.

**CSS grid auto-placement.** Likewise, any layout reshuffle (window
resize, container query, dynamic content insertion) reorders tab stops.
Keyboard-only users would discover the tab sequence changes underneath
them whenever the viewport changes.

**RTL layouts.** Paint order under RTL must flip the x-axis comparator,
which is doable, but now the implementation has to consult the writing
mode at every focus advance.

## Divergence cases

The two options agree on a plain top-to-bottom column. They diverge
whenever visual order differs from source order:

**Case 1 — Absolute-positioned "skip link".**

```html
<div>
  <button>Main content button</button>
  <button style="position: absolute; top: 0; right: 0">Skip to footer</button>
</div>
```

- Option A (semantic-tree): `Main content button` → `Skip to footer`.
- Option B (paint): `Skip to footer` → `Main content button` (top-right is painted first under a top-down sweep).

WPT asserts Option A; assistive technology assumes Option A.

**Case 2 — Flex `order:` reversal.**

```html
<div style="display: flex">
  <button style="order: 3">A</button>
  <button style="order: 1">B</button>
  <button style="order: 2">C</button>
</div>
```

- Option A: `A → B → C` (source order).
- Option B: `B → C → A` (paint order).

**Case 3 — Grid auto-placement.**

```html
<div style="display: grid; grid-template-columns: repeat(3, 1fr)">
  <button style="grid-column: 3">A</button>  <!-- painted top-right  -->
  <button style="grid-column: 1">B</button>  <!-- painted top-left   -->
  <button style="grid-column: 2">C</button>  <!-- painted top-middle -->
</div>
```

- Option A: `A → B → C` (source).
- Option B: `B → C → A` (paint, left-to-right).

**Case 4 — RTL paragraph with inline button.**

```html
<p dir="rtl">
  <button>اولاً</button>
  <button>ثانياً</button>
  <button>ثالثاً</button>
</p>
```

- Option A: source order `اولاً → ثانياً → ثالثاً` (matches WPT RTL test).
- Option B: paint order `ثالثاً → ثانياً → اولاً` (visually right-to-left).

In Cases 1-4 the WPT suite codifies Option A. Diverging from that means
either failing WPT or vendoring a fork of every assistive-technology
heuristic that already assumes the spec answer.

## Decision

**Adopt Option A — semantic-tree order — as jet's default Tab sequence.**

Rationale, in priority order:

1. **WPT parity.** `html/interaction/focus/sequential-focus-navigation`
   in WPT is the conformance suite the web platform uses. Matching it
   is the cheapest way to inherit a decade of edge-case work.
2. **React + MUI compatibility.** Every React component library on the
   market assumes source-order Tab navigation. Diverging silently
   breaks `<Tabs>`, `<Menu>`, `<Dialog>`'s focus trap, and `<Autocomplete>`.
3. **Screen-reader compatibility.** NVDA / JAWS / VoiceOver follow the
   accessibility tree, which in turn follows the DOM/semantic tree.
   Paint-order Tab would desynchronise the keyboard sweep from the
   screen-reader sweep — a known a11y antipattern.
4. **Stability under layout changes.** Source order is a property of the
   author's code; paint order is a property of the rendered frame. Users
   who memorise a keyboard route should not have it rearranged by a
   window resize.

Paint order remains useful — but as an **opt-in** for specific subtrees,
not the global default.

## Override knob

We introduce a per-subtree attribute on jet widget semantics:

```rust
pub enum TabOrderOverride {
    /// Default — inherit from parent; root inherits `SemanticTree`.
    Inherit,
    /// Semantic-tree order (the global default).
    SemanticTree,
    /// Paint order: top-to-bottom, left-to-right (RTL-aware).
    Paint,
    /// Caller supplies an explicit ordering callback.
    Manual(Arc<dyn TabOrderProvider>),
}

pub trait TabOrderProvider: Send + Sync {
    /// Given the focusable children of the subtree, return them in the
    /// order Tab should visit them. The callback runs once per
    /// relayout, not once per keypress.
    fn order(&self, children: &[WidgetId], ctx: &LayoutContext) -> Vec<WidgetId>;
}
```

Wire-up rules:

- The attribute lives on the **container** widget, not on individual
  focusables. Setting `tab_order_override = Paint` on a `Canvas` makes
  *its direct focusable descendants* sort by paint order.
- The override is **scoped to one subtree**. A `Paint`-ordered canvas
  embedded inside a `SemanticTree`-ordered page does not leak paint
  ordering up to the page level: when the browser's Tab traversal
  enters the canvas, jet hands it the paint-sorted proxy slice; when
  it exits, the outer page resumes semantic-tree order.
- `Manual` mode is the escape hatch for whiteboard-style apps where
  neither source nor paint order makes sense (e.g. a node graph that
  wants Tab to follow data-flow edges). Returning a `Vec` that omits
  children is allowed — omitted widgets become un-Tab-reachable but
  remain click-focusable.
- Mixing modes at sibling level is allowed and well-defined: each
  subtree contributes its own locally-sorted slice in the order its
  parent visits it.

This isolates "paint-ordered freeform canvas" as a labeled, audited
choice rather than a silent global setting.

## WPT gating

The conformance gate for this ADR is checked in alongside it:

- File: `projects/jet/data/parity/wpt/focus-sequential.manifest.toml`
- Path subset gated: `html/interaction/focus/sequential-focus-navigation`
- Tests gated (whitelist):
  - `sequential-focus-navigation.html` — baseline source-order traversal.
  - `sequential-focus-navigation-iframe.html` — Tab crosses iframe
    boundaries in source order; relevant once jet supports embedded
    sub-surfaces.
  - `sequential-focus-navigation-rtl.html` — RTL writing mode does not
    flip Tab direction (Case 4 above).
  - `tabindex-negative-not-in-sequence.html` — `tabindex="-1"` widgets
    are programmatically focusable but excluded from sequential Tab.
  - `tabindex-positive-order.html` — positive `tabindex` values create
    a parallel ordered group ahead of the natural `tabindex="0"`
    sequence.

Other WPT focus tests (focus visibility, `:focus-visible`, focus rings)
are out of scope for #2153 and tracked under #2142.

The gate ships as **soft-gate** (`gating.blocking = false`) in v1: CI
records pass/fail per test, but a regression does not block merge. We
flip to `blocking = true` once the four follow-up issues below are
resolved and we have a clean baseline.

## Consequences

- jet widgets inherit a Tab order that matches every mainstream web
  framework out of the box. Porting MUI components costs nothing extra.
- The hidden-DOM focus proxy subtree from #2152 stays a simple linear
  list — no per-frame re-sort — which keeps the per-frame focus cost
  at O(focusables-changed) rather than O(focusables).
- Authors of freeform canvas surfaces (whiteboard, node graph, drawing
  tool) must explicitly opt their root into `Paint` or `Manual` mode.
  This is a one-line annotation, not a global config.
- Screen-reader audits (axe-core, Lighthouse a11y) pass without
  jet-specific waivers, because the proxy DOM matches the visual
  semantics.
- WPT failures in `sequential-focus-navigation` are now jet bugs, not
  policy disagreements. The soft-gate buys us a window to drive the
  initial failure list to zero.

## Follow-ups

These are the consumer issues that build on this ADR. Suggested
`aw wi` candidates:

- **#2154 — focus-trap primitive.** Implement `<DialogTrap>`-style
  cycle-within-subtree behaviour on top of the semantic-tree default,
  with explicit re-entry on close.
- **#2155 — roving tabindex pattern.** Provide a `RovingGroup`
  container (radio groups, listbox, toolbar) that exposes exactly one
  tab stop and uses arrow keys internally — the canonical WAI-ARIA
  pattern.
- **#2156 — paint-order canvas opt-in audit.** Audit jet's
  whiteboard/graph demos and explicitly set `tab_order_override = Paint`
  where it makes sense; verify nothing else inherits it.
- **#2157 — flip WPT gate to blocking.** Once #2154/#2155/#2156 land
  and the WPT soft-gate baseline is clean, flip
  `gating.blocking = true` in the manifest.
- **#2158 — Manual ordering callback ergonomics.** Design a higher-level
  builder API on top of `TabOrderProvider` so layout authors don't have
  to hand-roll `Arc<dyn ...>` for common cases (e.g. "follow data-flow
  edges in a DAG").
