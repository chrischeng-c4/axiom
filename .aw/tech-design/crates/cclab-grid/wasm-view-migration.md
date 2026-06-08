# cclab-grid → jet `<WasmView>` Render Layer Migration

## Overview
<!-- type: overview lang: markdown -->

Spec for migrating cclab-grid's render layer from React + DOM
to jet's `<WasmView>` canvas-rendering boundary. Today the grid
ships its full product (data model, formula engine, undo/redo,
DB persistence, collaboration server) on top of
`cclab-grid-wasm`'s WASM data plane, but the render path runs
React + DOM and drops below 30fps at 10K rows with realistic
formatting. The jet wasm renderer (`epic-jet-wasm-canvas-renderer-react-compat`)
is the upgrade path; grid is its first co-design adopter.

This spec is the **target-neutral migration contract** — it pins
the renderer-toggle surface, the data ↔ view boundary, the
per-phase exit criteria, and the cut-over rule, without
committing to renderer internals. Each phase is independently
shippable behind the feature flag (G2).

@spec #1254 — `cclab-grid: migrate render layer to jet <WasmView>
canvas renderer`.

## Slice plan

Six slices. The first is this spec; the next five each correspond
to one of the issue's roadmap phases (POC + Phase A–E). Each is
deployable on its own (G7).

- **Slice 1 (this doc) — migration contract.** Pins the
  renderer-toggle flag, the data ↔ view boundary that stays
  stable across phases, the new `cclab-grid-render-wasm` crate
  layout, the per-phase exit criteria, and the cut-over rule.
  No code yet.
- **Slice 2 — POC (read-only viewport, no edit, no selection).**
  Implements the smallest grid render that covers the issue's
  "POC" line: render a static 10K-row read-only viewport via
  `<WasmView>` + jet renderer primitives. Captures the baseline
  bench against the current DOM path. Scaffolds
  `cclab-grid-render-wasm` and the feature-flag plumbing. Hard
  gate: scroll at 60fps for 10K × 5 columns of formatted text.
- **Slice 3 — Phase A — Read-only parity.** All cell types
  (text, number, date, boolean, formatted), merged cells,
  frozen headers, zoom, column resize, row resize, virtual
  scroll. Still no edit / no selection.
- **Slice 4 — Phase B — Interaction parity.** Selection
  (click / shift-click / drag / keyboard), copy-paste, context
  menu (DOM-rendered per QB), fill handle.
- **Slice 5 — Phase C — Edit parity.** In-cell editor, formula
  bar editor (floating DOM input per QA), IME, autocomplete,
  error indicators.
- **Slice 6 — Phase D — A11y + screen-reader parity.** Shadow
  tree covers the full grid surface including the formula bar;
  screen readers announce cell position, value, formula. This
  slice consumes
  `epic-jet-wasm-canvas-renderer-react-compat#R14`'s a11y
  contract; it does NOT define a grid-local a11y tree.

The "Phase E — Cut-over" line in the issue (remove feature flag,
retire DOM render layer, ship migration guide) is **not a
slice** — it's a release event that fires once Phase D ships and
production telemetry shows the canvas path stable. Filing as a
separate `chore(cclab-grid): retire DOM render layer` PR keeps
the cut-over reviewable on its own.

## Renderer-toggle contract

The flag (G2) is a single config key with two valid values:

```toml
# cclab-grid.toml (or per-instance prop on the React component)

[grid]
renderer = "dom"   # default until Phase E
# renderer = "wasm"  # opt-in during the migration window
```

Resolution order, per QC ("incremental adoption by downstream
apps embedding cclab-grid — can they opt-in per-view or is it a
global flip?" — answered: per-instance):

1. **Per-instance prop** on the React component (`<Grid
   renderer="wasm" ...>`). Overrides the config file. Lets a
   page render two grids side-by-side, one DOM-backed and one
   canvas-backed, for incremental migration in downstream apps.
2. **Workspace config file** (`cclab-grid.toml#grid.renderer`).
   Project-wide default.
3. **Hardcoded default** = `"dom"` until Phase E. After Phase E
   the flag is removed (the DOM branch is gone); attempts to set
   `renderer = "dom"` after cut-over fail loud-fast at config
   load.

Both branches dispatch through the same data-plane API — see
"Data ↔ view boundary" below — so swapping the flag value is
a render-only change.

## Data ↔ view boundary (G3)

The boundary is the **already-clean** API on
`cclab-grid-wasm`'s data plane (`crates/cclab-grid-wasm/src/api.rs`).
This spec **does not renegotiate it**:

```rust
// Existing — stays unchanged across this whole migration.
pub trait DataPlane {
    fn get_viewport_data(&self, viewport: ViewportRect) -> Vec<CellData>;
    fn set_cell_value(&mut self, cell: CellRef, value: CellValue) -> Result<()>;
    fn get_cell_data(&self, cell: CellRef) -> CellData;
    // ... formula bar, undo/redo, collab IPC — all out of scope here.
}
```

Both render layers (`dom` and `wasm`) consume the data plane
via this trait. Slice 2 introduces `cclab-grid-render-wasm` as
a sibling implementation; it does **not** touch the trait or
its impls. Any change the canvas path "needs" to the data plane
is a renderer-side bug per the issue's "co-design signal" rule
("If grid discovers a paint or input need that the renderer
cannot serve, that's a renderer bug, not a grid workaround.").

## New crate layout

```
crates/cclab-grid-render-wasm/         # NEW (Slice 2)
  Cargo.toml                           # depends on jet-wasm + cclab-grid-wasm
  src/
    lib.rs                             # entry: `mount(view: WasmView, data: &dyn DataPlane)`
    viewport.rs                        # virtual scroll + frame culling
    cell_paint.rs                      # per-cell-type paint dispatch
    interaction/                       # Slice 4+ — empty in Slice 2
      selection.rs
      copy_paste.rs
      context_menu.rs                  # DOM-rendered popover handle (per QB)
    edit/                              # Slice 5+ — empty in Slice 2
      in_cell_editor.rs
      formula_bar.rs                   # DOM-floating input handle (per QA)
    a11y/                              # Slice 6 — empty in Slice 2
      shadow_tree.rs
```

Per G4, the new crate **consumes** `<WasmView>` + the jet canvas
renderer's primitives — it does NOT fork the renderer. If a
primitive is missing, file a `jet-wasm-renderer` issue, not a
local fork.

## Per-phase exit criteria

Each phase has hard, falsifiable exit criteria. Promotion to
the next slice requires all bullets green; partial green ships
as a `wip:` increment behind the flag and gates user-facing
opt-in to the previous phase.

### POC (Slice 2)

- 10K rows × 5 columns of formatted text scroll at 60fps
  sustained on a mid-tier laptop.
- Font rendering parity with the DOM path on a side-by-side
  diff against a 100-row screenshot snapshot (manual
  approval — automated text-rendering diffs are flaky, defer).
- Baseline DOM-path frame-time + scroll-fps captured and
  committed to `crates/cclab-grid-render-wasm/bench/baseline.json`.

### Phase A — Read-only parity (Slice 3)

- All cell types render identical to DOM in a fixture
  spanning text / number / date / boolean / formatted.
- Merged cells, frozen headers, zoom, column resize, row
  resize, virtual scroll all work in the canvas path.
- 10K-row scroll bench still hits 60fps with all cell types
  active (not just plain text).

### Phase B — Interaction parity (Slice 4)

- Click / shift-click / drag selection matches DOM behaviour
  on a fixture suite (shape parity, not pixel-perfect).
- Copy-paste round-trips against the existing clipboard
  protocol (`crates/cclab-grid/src/clipboard.rs`).
- Context menu opens as a DOM popover, anchored to the canvas
  cell rect — per QB. Right-click on the canvas finds the cell
  via `<WasmView>` hit-test, then opens the existing DOM
  context-menu component. No canvas-rendered menu items.
- Fill handle drags + commits values via `set_cell_value`.

### Phase C — Edit parity (Slice 5)

- In-cell editor activates on F2 / dblclick. The editor IS a
  floating DOM `<input>` positioned over the canvas cell rect —
  not a canvas-rendered input — per QA (preserves IME +
  autocomplete ergonomics).
- Formula bar editor likewise floats as a DOM input above the
  canvas surface.
- IME composition tested with at least one CJK input
  (Japanese / Traditional Chinese) — sanity for IME boundary.
- Autocomplete dropdown lands as DOM popover anchored to the
  editor.
- Error indicators (red triangle, error tooltip) paint on the
  canvas; tooltip is DOM-rendered.

### Phase D — A11y + screen-reader parity (Slice 6)

- `<WasmView>` shadow tree (per
  `epic-jet-wasm-canvas-renderer-react-compat#R14`) covers
  every visible cell, the formula bar, and any active editor.
- VoiceOver / NVDA announces "<column> <row>: <value>" on
  cursor move (manual smoke-test — automated screen-reader
  testing is brittle, defer).
- jet test-runner `locator` queries resolve against the
  shadow tree (G5).

### Cut-over (release event, no slice)

- Telemetry from a 30-day production opt-in window shows zero
  P0/P1 regressions on the canvas path.
- DOM render code deleted; `renderer = "dom"` config is
  rejected by the loader.
- Migration guide published in `crates/cclab-grid/docs/`.

## Open questions — pinned answers

The issue raised three open questions; this spec freezes the
answers so the implementation slices don't relitigate them:

- **QA — formula-bar editor.** Floating DOM input above the
  canvas. Locked at Phase C. Reason: IME + autocomplete are
  not on the renderer's near-term roadmap; a DOM input gets
  both for free and decouples grid's edit-parity ship date
  from the renderer's text-input phase.
- **QB — context menu.** DOM popover, anchored to the canvas
  cell rect via `<WasmView>` hit-test. Locked at Phase B.
  Reason: native right-click semantics, accessibility, and
  OS-level conventions all live in the OS — losing them for a
  bespoke canvas menu is a regression we don't accept.
- **QC — incremental adoption.** Per-instance opt-in via the
  React component prop, falling back to the workspace config
  default. Locked at the renderer-toggle contract above.
  Reason: lets downstream apps migrate one grid view at a
  time (the side-by-side dual render is the migration's most
  honest A/B).

## Success criteria

Restated from the issue, kept here so the spec is
self-contained:

- 10K-row grid at 60fps sustained scroll, matching or
  beating AG Grid / Handsontable.
- Full grid test suite green against the canvas render path
  (G5).
- Screen reader announces cell position + value correctly
  (G6 / Phase D).
- Zero regression in formula engine / persistence / collab
  behaviour (G3 — the data plane is unchanged).
- JS heap ≤ 1.5× current DOM-path baseline (G6).
- Migration guide + feature-flag cut-over completed; DOM
  render code removed.

## Out of scope

- **Data plane changes.** Zero intrusion outside the render
  layer (G3).
- **Server-side grid rendering.** The cclab-grid server
  product is headless; this migration is browser-side only.
- **Webview / native-app embeds.** Separate follow-up; this
  spec does not commit to a native-shell adapter.

## Cross-references

- Parent epic: `epic-jet-wasm-canvas-renderer-react-compat`.
  Phase 5 (a11y shadow tree) is the hard production-rollout
  blocker for #1254 Phase D; the POC + Phase A unblock once
  the epic's Phase 2 (event loop + hit testing) is ready.
- `.aw/tech-design/crates/jet/wasm-renderer/architecture.md`
  — renderer architecture this migration consumes.
- `.aw/tech-design/crates/jet/wasm-renderer/subset.md` —
  React-compat subset; grid's render code targets this subset.
- `.aw/tech-design/crates/cclab-grid/architecture.md` —
  grid product architecture (data plane stays unchanged).
- `.aw/tech-design/crates/cclab-grid/context-menu-clipboard.md`
  — Phase B reuses this protocol via DOM popover.
- `.aw/tech-design/crates/cclab-grid/formula-bar-redesign.md`
  — Phase C floats this as a DOM input above the canvas.
- `crates/cclab-grid-wasm/src/api.rs` — data plane (unchanged).
- Bench reference:
  `epic-jet-wasm-canvas-renderer-react-compat#R15` — 10K-row
  60fps target; grid IS the canonical consumer.
