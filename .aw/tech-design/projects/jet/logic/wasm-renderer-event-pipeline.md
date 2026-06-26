---
id: projects-jet-logic-wasm-renderer-event-pipeline-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Event pipeline — synthetic events for the canvas runtime

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/wasm-renderer-event-pipeline.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# Event pipeline — synthetic events for the canvas runtime

### Overview

The event pipeline owns the path from a native browser event landing on
the `<canvas>` to a transpiled Rust handler closure firing inside the
React fiber. Today (`logic/wasm-renderer-paint-runtime.md § Click event loop`) that path
exists for `click` but the handler payload is unit (`Callback<()>`) and
only the deepest hit fires — there is no synthetic event object, no
bubble, and no `preventDefault` / `stopPropagation`. This spec adds
those three pieces, scoped to `click` events.

What this spec defines:

1. `SyntheticMouseEvent` — the per-dispatch event object passed to a
   one-arg arrow handler (`onClick={(e) => …}`).
2. `HitPath` — the deepest-first vector of layout nodes a click hits,
   replacing today's single-deepest `hit_test_on_click`.
3. Dispatch algorithm — bubble through the path, fire each handler with
   `e.currentTarget` updated, halt on `e.stop_propagation()`.
4. `dispatch_click` entrypoint — single function the canvas listener
   and any future replay path (SSR hydration queue, programmatic
   dispatch in tests) call into.
5. Transpiler lowering — emit `EventCallback::new(move |e: &SyntheticMouseEvent| …)`
   when the source arrow is one-arg; keep `move |_e| …` when zero-arg.

What this spec does **not** define:

- Any event other than `click`. `mousedown` / `keydown` / `focus` etc.
  ship in sibling specs once this spine stabilises.
- Capture phase. Bubble-only is the v1 contract.
- Event pooling / `event.persist()`. Every dispatch builds a fresh
  `SyntheticMouseEvent`.
- DOM-backed renderer. Layout-index-based `target` works because the
  canvas renderer owns the layout tree; a future DOM backend would
  fill `SyntheticMouseEvent` from a real `MouseEvent` and feed the
  same `dispatch_click`.
- Replay queue / hydration. The `dispatch_click` entrypoint is shaped
  to support a queue (R7's `native: Option<&MouseEvent>` admits a
  no-native replay path), but the queue itself is a separate spec.

Parent: `logic/wasm-renderer-architecture.md`. Sibling specs touched:
`logic/wasm-renderer-paint-runtime.md`, `logic/wasm-renderer-subset.md`,
`logic/wasm-renderer-transpiler.md`, `logic/wasm-renderer-conformance.md`.

Module: `crates/jet-wasm/src/{lib.rs, react/mod.rs, react/canvas_app.rs,
renderer/mod.rs}` + `crates/jet/src/tsx_to_rust/emit.rs`.

### Design Contract

```mermaid
---
id: jet-wasm-renderer-event-requirements
entry: E1
---
requirementDiagram
    requirement E1 { id: E1 text: SyntheticMouseEvent lives in framework agnostic substrate risk: high verifymethod: inspection }
    requirement E2 { id: E2 text: prevent_default and stop_propagation take shared reference via Cell state risk: high verifymethod: test }
    requirement E3 { id: E3 text: target and current target layout indices mirror React semantics risk: high verifymethod: test }
    requirement E4 { id: E4 text: EventCallback is distinct from owned Callback risk: medium verifymethod: test }
    requirement E5 { id: E5 text: Props on_click stores EventCallback SyntheticMouseEvent risk: high verifymethod: test }
    requirement E6 { id: E6 text: hit_test_path returns deepest first bubble path risk: high verifymethod: test }
    requirement E7 { id: E7 text: dispatch_click walks path invokes handlers and halts on stop propagation risk: high verifymethod: test }
    requirement E8 { id: E8 text: prevent_default propagates to native event when present risk: medium verifymethod: test }
    requirement E9 { id: E9 text: transpiler accepts zero arg and one arg onClick arrows risk: high verifymethod: test }
    requirement E10 { id: E10 text: transpiler maps accepted mouse event fields and rejects others risk: high verifymethod: test }
    requirement E11 { id: E11 text: canvas click listener is thin shim around dispatch_click risk: medium verifymethod: inspection }
```

| id | Requirement | Verifies |
|----|-------------|----------|
| E1 | `SyntheticMouseEvent` lives in the framework-agnostic substrate (`crates/jet-wasm/src/lib.rs`), not the `react` module. Vue / Solid adapters reuse it. | Type definition next to `Element` / `Props` / `Callback`. |
| E2 | `SyntheticMouseEvent::prevent_default(&self)` and `stop_propagation(&self)` take `&self` (not `&mut self`) so handler closures hold `&SyntheticMouseEvent`. State is in `Cell<bool>`. | Calling both inside an `Fn(&SyntheticMouseEvent)` closure compiles. |
| E3 | `target_layout_index` is the leaf hit; `current_target_layout_index` is updated per-dispatch step to the node currently firing. Mirrors React's `e.target` vs `e.currentTarget`. | Conformance test scenario (d) — handler at depth 1 reads both indices. |
| E4 | `EventCallback<E>` is a separate type from `Callback<P>`. `Callback<P>` (owned payload) stays for `on_change: Callback<String>` and similar; `EventCallback<E>` (borrowed payload) is for synthetic events. | Two type defs with non-overlapping use sites. |
| E5 | `Props.on_click: Option<EventCallback<SyntheticMouseEvent>>` — owned payload variant retired for click. | Compile failure if any caller stores `Callback<()>` for on_click. |
| E6 | `LayoutTree::hit_test_path(&self, point: Point) -> Vec<HitNode>` returns the bubble path **deepest first**, with `HitNode { layout_index: usize, on_click: Option<&EventCallback<SyntheticMouseEvent>> }`. Empty vec = no hit. | Unit test on a 3-level tree at three click points. |
| E7 | `dispatch_click(layout_tree, point, native: Option<&MouseEvent>) -> bool` walks the hit path, invokes each handler with a fresh `SyntheticMouseEvent`, updates `current_target_layout_index` per step, halts on `e.propagation_stopped`. Returns `true` iff any handler ran. | Unit test exercising bubble + stop_propagation. |
| E8 | After dispatch, if `e.default_prevented` and `native.is_some()`, call `native.prevent_default()`. Canvas has no useful native default; the contract still propagates so a future DOM backend honours it. | Unit test: handler calls `e.prevent_default()`, mock native receives the call. |
| E9 | The transpiler accepts zero-arg AND one-arg arrow handlers for `onClick`. Multi-arg keeps the existing `bail!`. The one-arg parameter name is bound to a Rust `e: &SyntheticMouseEvent`. | Snapshot tests over `onClick={() => …}` and `onClick={(e) => e.prevent_default()}`. |
| E10 | The transpiler maps `e.clientX`/`e.clientY` (TSX camelCase) to `e.client_x`/`e.client_y` (Rust snake_case) and `e.preventDefault()`/`e.stopPropagation()` to `e.prevent_default()`/`e.stop_propagation()`. The mapping table is enumerated and closed — no other property/method on `e` is allowed in the v1 subset. | Snapshot test: TSX `e.clientX` round-trips; TSX `e.target` rejected with `bail!`. |
| E11 | `canvas_app::install_click_listener` becomes a thin shim around `dispatch_click`. The listener's responsibility is reduced to (a) compute `Point` from the native MouseEvent + canvas rect, (b) call `dispatch_click`, (c) if `true` returned, run `flush + repaint`. | Diff: `install_click_listener` body shrinks; logic moves into `dispatch_click`. |

### SyntheticMouseEvent — struct + methods

```rust
use std::cell::Cell;

/// Synthetic event payload for click handlers.
///
/// Mirrors React's `SyntheticEvent<MouseEvent>` shape, narrowed to
/// the fields jet-wasm can supply from the canvas hit path:
/// - coordinates from the native `MouseEvent`,
/// - `target` / `currentTarget` as opaque indices into the cached
///   `LayoutTree` (the canvas backend's analog of a DOM element),
/// - propagation + default flags as interior-mutable `Cell`s so
///   handler closures take `&SyntheticMouseEvent`.
pub struct SyntheticMouseEvent {
    /// Viewport-relative X, in CSS pixels (post `ctx.scale(dpr, dpr)`).
    pub client_x: f32,
    /// Viewport-relative Y, in CSS pixels.
    pub client_y: f32,
    /// Layout-tree index of the deepest node that was hit.
    /// Stable for the lifetime of the dispatch.
    pub target_layout_index: usize,
    /// Layout-tree index of the node whose handler is currently
    /// firing. Bubble dispatch updates this between handlers.
    pub current_target_layout_index: Cell<usize>,
    /// Set by `prevent_default()`. Inspected after dispatch.
    default_prevented: Cell<bool>,
    /// Set by `stop_propagation()`. Inspected between handlers.
    propagation_stopped: Cell<bool>,
}

impl SyntheticMouseEvent {
    pub fn prevent_default(&self) {
        self.default_prevented.set(true);
    }

    pub fn stop_propagation(&self) {
        self.propagation_stopped.set(true);
    }

    pub fn is_default_prevented(&self) -> bool {
        self.default_prevented.get()
    }

    pub(crate) fn is_propagation_stopped(&self) -> bool {
        self.propagation_stopped.get()
    }
}
```

`Cell<bool>` — not `RefCell` — because the flags are `Copy` and we
never need a borrow. `&self` methods + `Cell` is the canonical
no-allocation, no-overhead pattern for "interior-mutable scalar".

### EventCallback — borrowed-payload callback

```rust
use std::rc::Rc;

/// Callback over a borrowed event payload.
///
/// `Callback<P: Clone>` is retained for owned-payload handlers
/// (`Callback<String>` for `on_change`, `Callback<()>` for the rare
/// no-payload case). `EventCallback<E>` is the synthetic-event
/// flavour: the closure takes `&E` and is invoked via
/// `EventCallback::call(&evt)`. Bifurcating the type avoids forcing
/// a lifetime parameter onto every storage site of `Callback`, which
/// would be the cost of trying to handle both shapes with one type.
pub struct EventCallback<E>(Rc<dyn Fn(&E)>);

impl<E> EventCallback<E> {
    pub fn new(f: impl Fn(&E) + 'static) -> Self {
        Self(Rc::new(f))
    }

    pub fn call(&self, evt: &E) {
        (self.0)(evt)
    }
}

impl<E> Clone for EventCallback<E> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<E> std::fmt::Debug for EventCallback<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EventCallback").finish()
    }
}
```

`Props.on_click` migrates to `Option<EventCallback<SyntheticMouseEvent>>`.
The transpiler emits `EventCallback::new(...)` instead of
`Callback::new(...)` for click handlers.

### HitPath

```rust
pub struct HitNode<'a> {
    pub layout_index: usize,
    pub on_click: Option<&'a EventCallback<SyntheticMouseEvent>>,
}

impl LayoutTree {
    /// Bubble path from a click point.
    ///
    /// Walks `nodes` in reverse (child-on-top z-order) and collects
    /// every layout node whose rect contains `point`. Nodes are
    /// returned **deepest first** — index 0 is the leaf hit, the
    /// tail is the document root.
    ///
    /// `on_click` is borrowed from the laid-out node's props; the
    /// returned vec borrows the layout tree, so it cannot outlive
    /// the dispatch (which is the desired lifetime — handlers fire
    /// inline, the path is dropped before `flush + repaint`).
    pub fn hit_test_path(&self, point: Point) -> Vec<HitNode<'_>> {
        // (algorithm in § Hit-test algorithm)
    }
}
```

The existing `hit_test_on_click(point) -> Option<&Callback<()>>` is
removed. Its single caller (`canvas_app::install_click_listener`)
moves to the new path-based API. Tests that exercise the old API
migrate to `hit_test_path(p).first()` for the deepest-only behaviour
(or full-path scenarios where appropriate).

### Hit-test algorithm

```
hit_test_path(layout_tree, point):
  result = []
  for node in layout_tree.nodes.iter().rev():
    # Reverse order — child layouts come AFTER parents in `nodes`,
    # so reverse iteration gives child-first which is the desired
    # bubble order (child at result[0], root at result[-1]).
    if node.rect.contains(point):
      on_click = match &node.kind:
        Intrinsic { props, .. } => props.on_click.as_ref(),
        Text { .. } | Component { .. } | Empty | Fragment { .. } => None
      result.push(HitNode {
        layout_index: node.index_in_layout_tree,
        on_click,
      })
  return result
```

`node.rect.contains(point)` is the same point-in-rect test
`hit_test_on_click` already uses (`renderer/mod.rs:208`). The change is
collecting all hits instead of returning on first match.

Empty vec — no node hit — is a frequent case: the user clicked outside
any laid-out node (margins, gaps). The dispatcher returns `false`
immediately and the canvas listener skips `flush + repaint`.

### Dispatch algorithm

```
dispatch_click(layout_tree, point, native: Option<&MouseEvent>) -> bool:
  path = layout_tree.hit_test_path(point)
  if path.is_empty():
    return false

  evt = SyntheticMouseEvent {
    client_x: point.x,
    client_y: point.y,
    target_layout_index: path[0].layout_index,
    current_target_layout_index: Cell::new(path[0].layout_index),
    default_prevented: Cell::new(false),
    propagation_stopped: Cell::new(false),
  }

  fired_any = false
  for hit in &path:
    if let Some(cb) = hit.on_click:
      evt.current_target_layout_index.set(hit.layout_index)
      cb.call(&evt)
      fired_any = true
      if evt.is_propagation_stopped():
        break

  if evt.is_default_prevented() and let Some(n) = native:
    n.prevent_default()

  return fired_any
```

Key invariants:

- **Fresh event per dispatch.** No pooling, no reuse. The
  `SyntheticMouseEvent` is dropped at the end of `dispatch_click`.
- **Bubble order is deepest-first.** Matches React semantics. The
  hit path is already ordered correctly by `hit_test_path`.
- **Propagation halts the loop, not the function.** `default_prevented`
  is still inspected after the break; both flags are independent.
- **`fired_any` drives repaint.** The canvas listener calls
  `handle.flush()` only when `fired_any && handle.flush()` — same
  shape as today's `if handle.flush() { repaint(...) }` but gated on
  the dispatcher reporting work happened. Without `fired_any`, even a
  click that hit a non-handler node could trigger a redundant flush.
- **`native` is `Option`.** The canvas listener passes `Some(&native)`.
  A future replay path (cold-start hydration queue) passes `None` and
  loses only the `prevent_default()` callback to the browser, which is
  correct: a replay can't propagate to a native event that already
  fired and finished.

### Transpiler lowering

The transpiler (`crates/jet/src/tsx_to_rust/emit.rs:1073-1108`) currently
matches `onClick` and emits a zero-arg closure. The change widens that
match to accept one parameter and lowers the parameter accesses.

**Accepted shapes**:

| TSX source | Emitted Rust |
|------------|--------------|
| `onClick={() => body}` | `on_click: Some(EventCallback::new(move \|_e: &::jet_wasm::SyntheticMouseEvent\| { body_rust }))` |
| `onClick={(e) => body}` | `on_click: Some(EventCallback::new({ let cb = move \|e: &::jet_wasm::SyntheticMouseEvent\| { body_rust }; cb }))` |
| `onClick={(evt) => body}` | identical to `(e) =>`; the parameter name is preserved verbatim. |

**Property / method mapping** when the body references the parameter:

| TSX expression on `e` | Rust expression |
|-----------------------|-----------------|
| `e.clientX` | `e.client_x` |
| `e.clientY` | `e.client_y` |
| `e.preventDefault()` | `e.prevent_default()` |
| `e.stopPropagation()` | `e.stop_propagation()` |
| `e.target` | (rejected — not in v1 subset; see § Out of scope) |
| `e.currentTarget` | (rejected — same) |
| Anything else (`e.button`, `e.shiftKey`, …) | `bail!("e.{prop} not in synthetic-event v1 subset")` |

Mapping is implemented as a small lookup applied during expression
lowering when the receiver is the bound event parameter. Out-of-subset
property access fails loud at transpile time, consistent with the
existing "spike subset" `bail!` policy.

**Rejected shapes** (keep existing `bail!`):

- `onClick={handler}` — bare identifier; not an arrow.
- `onClick={(a, b) => …}` — multi-arg.
- `onClick={async (e) => …}` — async arrow; out of subset.
- `onClick={function(e) { … }}` — function expression; out of subset.

### Out of scope

- All event types other than `click` (mousedown, mouseup, mouseenter, mouseleave, mousemove, keydown, keyup, focus, blur, submit, change for non-input intrinsics, wheel, touch*, pointer*, drag*).
- Capture phase. React's `onClickCapture` is not supported in v1; bubble-only.
- Event pooling / `e.persist()`.
- `e.relatedTarget` (only meaningful for `mouseenter` / `mouseleave`).
- `e.button` / `e.buttons` / `e.shiftKey` / `e.ctrlKey` / `e.altKey` / `e.metaKey`. Click-based UIs that need modifier keys land with the rest of the keyboard event surface.
- `e.target` / `e.currentTarget` as references to original DOM elements. The synthetic event exposes layout-tree indices instead. A future DOM backend would surface DOM elements through the same field types.
- Touch + pointer event coalescing.
- Cold-start replay queue (the SSR hydration motivation). `dispatch_click` is shaped to admit it via `native: Option<...>`, but a queue + replay state machine is a separate spec.
- Migrating `Props.on_change` (`Callback<String>`) to a synthetic event. Lands when an input handler needs `e.target.value` semantics that today's `String` payload does not cover.
- DOM-backed renderer.

### Test strategy

- Unit tests in `crates/jet-wasm/src/renderer/mod.rs` cover `hit_test_path` over a 3-level tree at four click points (leaf, intermediate, root, miss).
- Unit tests in `crates/jet-wasm/src/react/mod.rs` cover `dispatch_click` with: no-handler hit, single-handler hit, bubble (parent + child both have handlers), `stop_propagation`, `prevent_default`.
- Integration test `crates/jet-wasm/tests/synthetic_click_event.rs` (R8) exercises end-to-end: mount fixture component, drive `dispatch_click` directly, assert handler invocation order + flag effects.
- Snapshot tests in `crates/jet/tests/transpile_*.rs` cover zero-arg + one-arg `onClick` lowering and the property mapping table.
- The conformance harness (`logic/wasm-renderer-conformance.md § Event tier`) gets a new `synthetic_click_event` row.

### Changes

```yaml
changes:
  - date: 2026-04-27
    impl_mode: hand-written
    description: Initial spec authored alongside enhancement-synthetic-event-pipeline-canvas-click-fiber-handle.
    refs: [R1, R2, R3, R4, R5, R6, R7, R8, R9]
```
