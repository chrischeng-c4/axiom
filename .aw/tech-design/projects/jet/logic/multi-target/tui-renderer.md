---
id: projects-jet-logic-multi-target-tui-renderer-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# jet-tui-renderer — ratatui-backed `TargetRenderer` for jet

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/multi-target/tui-renderer.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# jet-tui-renderer — ratatui-backed `TargetRenderer` for jet

> Issue: #1241 — `enhancement(jet): prototype ratatui-backed TUI
> renderer for Jet apps`. Sibling of #1238 (renderer-neutral contract)
> and #1239 (`--target tui` build plumbing).

### Goal

Implement the renderer-neutral [`TargetRenderer`] trait for a terminal
target, paint laid-out frames into a ratatui surface, and translate
crossterm input into [`SyntheticEvent`]s. The end-state demo is the
small Cue navigation/list/detail/log workflow from #1241's acceptance
criteria.

### Slices

The crate ships in four hand-written slices:

- **Slice 1 (shipped) — buffer-only spike.** `TuiRenderer::paint_into`
  walks `&[LaidOutNode]` and writes glyphs into a caller-provided
  `ratatui::buffer::Buffer`. T1 pixel→cell math (default 8 px/col,
  16 px/row) is the only honored degradation rule. `mount` records
  the substrate's event callback but does not yet drive a poll loop.
  Ten unit tests cover the math + paint shapes without touching real
  IO. Crate compiles on stable Rust without a wasm toolchain.
- **Slice 2 (input-translation shipped) — crossterm → SyntheticEvent.**
  `event_in::translate_event(crossterm::event::Event, CellSize) ->
  Option<SyntheticEvent>` is a pure function: feed it crossterm
  events from any source (poll thread, test fixture), get back the
  contract-shape synthetic event. Twelve unit tests cover Enter / Esc
  / Tab / arrows / function keys / letter (shifted) / digits / Resize
  → Lifecycle / mouse Left-Down / modifier-only filtering / Release
  filtering / mouse Up-Drag-Scroll filtering.
- **Slice 2b (shipped) — internal Buffer + headless input pump +
  ratatui Terminal integration.** `TuiRenderer` now carries an
  `internal: Option<Buffer>` so the trait-level `paint(frame)` is no
  longer a no-op: `mount` pre-sizes the buffer from
  `LayoutTree::root_rect` (falling back to 80×24 when the rect is
  zero), and trait `paint` resets between frames and writes into it.
  `with_size(cols, rows)` is the explicit-sizing builder for callers
  that don't go through `mount`. `pump_event(&CtEvent) -> bool`
  drives `event_in::translate_event` through the stored callback so
  tests (and any future poll thread) can feed input headlessly. A
  free function `render_with_backend<B: Backend>(terminal, cell,
  frame)` wraps `Terminal::draw(...)` so the renderer composes with
  any ratatui backend (including `TestBackend` in tests). Eight new
  unit tests cover the buffer-sizing paths, the inter-frame reset,
  the pump's three return-values, and the full render-through-
  `TestBackend` round-trip. The crossterm poll loop that ties this
  to a real terminal is still pending (Slice 2c).
- **Slice 2c (shipped) — `RealTerminal` lifecycle guard.** New
  `runtime` module ships `RealTerminal`, the production driver that
  wraps `Terminal<CrosstermBackend<Stdout>>` plus the alt-screen
  / raw-mode lifecycle every real-tty session needs. `enter(cell)`
  arms raw mode + alt screen and rolls back partial setup on error
  so callers never observe a half-armed terminal. `render(frame)`
  is a thin passthrough to the Slice-2b `render_with_backend`
  helper. `poll_event(timeout)` wraps `crossterm::event::{poll,
  read}` + `event_in::translate_event` and returns
  `Ok(None)` / `Ok(Some(None))` / `Ok(Some(Some(event)))` so
  callers can distinguish poll-timeout, dropped-event, and
  translated-event without re-importing crossterm. The
  alt-screen + raw-mode teardown lives in a private
  `LifecycleGuard` state-machine: `Drop` and explicit
  `RealTerminal::leave()` both route through `run_disarm`, which
  flips `armed` and runs the disarm closure exactly once (verified
  by 5 unit tests covering armed-on-construct, single-disarm,
  second-disarm-noop, error-still-flips-armed, no-retry-after-
  error). Tests don't construct a `RealTerminal` itself —
  `enable_raw_mode()` requires a tty, so the body is exercised by
  Slice 4's Cue demo end-to-end. `ratatui` gained the `crossterm`
  feature in `Cargo.toml` to unlock `CrosstermBackend`.
- **Slice 3 (shipped) — degradation rules T4 (color quantize) +
  T5 (font drop).** New `style` module parses CSS-like `style:
  "..."` strings from `jet_wasm::Props.style` into `StyleHints {
  fg, bg }`. T4: `color`, `background`, `background-color` (rgb /
  rgba / `#RGB` / `#RRGGBB` syntax) quantize to one of the 16
  ANSI colors via nearest-neighbor squared-distance against a
  VGA palette. T5: `font-family` / `font-weight` / `font-style`
  / `font-size` parse without error and produce no hint — the
  terminal owns the font. Painters (`draw_border`, `write_text`)
  apply the resulting `ratatui::style::Style` per cell. 24 new
  tests (20 in `style::tests` covering parse / quantize / case
  / lenient-skip / mixed-prop ordering, 4 in `tests` covering
  end-to-end paint with fg / bg / font-only / mixed style
  strings). `indexed-256` opt-in is deferred until terminal
  capability detection lands in Slice 4+.
- **Slice 4 — Cue demo (split into 4a–4f, each independently
  shippable).** The minimum work-item-list / detail / status-timeline
  / log-stream / command-input surface from #1241's acceptance
  criteria, exercised end-to-end in CI via the existing conformance
  harness against the TUI capability profile.
  - **Slice 4a (shipped) — work-item-list, read-only.** Hand-crafted
    `LayoutTree` (header bar + 5 work-item rows) painted via
    `render_with_backend(&mut Terminal<TestBackend>, …)`. Asserts
    every row's `"[<id>] <title>"` prefix lands at its expected cell
    row, the empty-list case renders only the header, and a 200-char
    title clips at the 80-col viewport edge. Demo IS the integration
    test at `crates/jet-tui-renderer/tests/cue_work_item_list_demo.rs`
    — no separate `examples/` binary, since CI exercises tests by
    default. No event loop, no selection, no scroll.
  - **Slice 4b (shipped) — input + selection on the work-item-list.**
    `WorkItemListApp { items, cursor, selected }` lives in the demo
    test file behind `Rc<RefCell<…>>`; the mount callback feeds
    `SyntheticEvent::Key` into `app.on_key(&KeyEvent)`. ArrowDown
    advances cursor (clamped at `items.len() - 1`), ArrowUp retreats
    (saturating at 0), Enter appends `items[cursor].id` to a selected
    log. Cursor row renders with `> ` prefix, non-cursor rows with
    `  ` so column alignment stays stable. Six new tests pin: cursor
    caret prefix, ArrowDown advance, ArrowDown clamp, ArrowUp clamp,
    Enter-records-id, and a `pump → render_with_backend` round-trip
    that asserts the caret moves on the painted buffer. There is no
    `SyntheticEvent::Custom` variant yet — the demo's selected log
    is direct app-state in lieu of a substrate-level dispatch; once
    `Custom` lands, 4b can re-target the substrate path without a
    change to the renderer's surface.
  - **Slice 4c (shipped) — work-item detail panel.** `layout_split`
    splits the 80-col viewport: list pane on cols 0..39 (titles
    clipped at the divider via per-row `Rect.w = 320 px`), detail
    pane on cols 40..79 carrying `Detail` header + `id:` + `title:`
    + a body fixture. Body comes from `sample_body(id)` — a small
    in-test fixture; the real Cue dogfood (#1240) supplies bodies
    via the data plane. No new public renderer surface; reuses the
    same `LaidOutNode { Text | Intrinsic }` primitives. Four new
    tests pin: detail panel id/title/body for cursor=0, repaint on
    cursor move (cursor=2 ⇒ detail shows `#103`), list-pane title
    clip at the divider (200-char title doesn't overwrite detail
    pane on the same row), and a `pump → split-render` round-trip
    (Down ⇒ detail pane focuses `#102`). Tests use a `cell_substr`
    helper because the demo header `Cue — Work Items` carries a
    multi-byte em-dash — byte-slicing the row String would skip
    cells.
  - **Slice 4d (shipped) — status-timeline.** `layout_split_with_timeline`
    appends a stage stack below the detail body in the right pane.
    `Status` header at row 11; the seven `Lifecycle-Stage:` values
    (`Create` → `Fill-Requirements` → `Fill-Scope` →
    `Fill-ReferenceContext` → `Review` → `Revise` → `Merge`) stack
    from row 13. Active stage uses the same `> ` caret prefix as the
    list cursor in 4b; inactive stages pad with `  `. Active-stage
    fixture lives in `active_stage_index(id)` — the real Cue dogfood
    (#1240) threads `Lifecycle-Stage:` git-trailer history through
    the data plane. Five new tests pin: all-stages render with caret
    on active (cursor=0 ⇒ Review highlighted), repaint on cursor
    move (cursor=2 ⇒ Merge highlighted, all others flat), caret on
    first stage when active=Create (cursor=4), no left-pane bleed
    (timeline `Rect.x = 320 px` stays right of the divider), and a
    `pump → timeline-render` round-trip (Down ⇒ Fill-Requirements
    highlighted).
  - **Slice 4e (shipped) — log-stream.** Bottom-anchored log surface
    on the left pane: `Log` header on row 18, up to 5 visible lines
    on rows 19..23. `LogStream { lines: VecDeque<String>, capacity }`
    in the demo file caps at `capacity` (oldest rolls off when full)
    and exposes `tail(count)` for the visible window. Bottom-anchored
    means newest line lands on row 23; if entries < visible budget,
    top rows stay blank. Six new tests pin: capacity drop (push 4
    into capacity-3 ⇒ tail = b/c/d), bottom-anchored render with
    blank top rows when underfilled, full-budget render when entries
    == visible rows, overflow render keeps only the last 5 visible
    when 8 entries pushed into capacity-8, no left-pane bleed (long
    log line clips at the col-40 divider, doesn't overwrite right
    pane), and a `pump → log-render` round-trip (Down ⇒ "cursor -> 1"
    appears on the bottom log row).
  - **Slice 4f (shipped) — command-input.** Self-contained sibling
    file `crates/jet-tui-renderer/tests/cue_command_input_demo.rs`
    (split from the main demo file to keep both under the 1000-line
    CLAUDE.md ceiling). `CommandInputApp { buffer, submitted }`
    handles printable chars (append), Backspace (pop, no-op on
    empty), Enter (push buffer onto `submitted`, clear). The bottom
    row (`COMMAND_ROW = 23`) renders as `> <buffer>█` — `█` indicates
    the insert position, painted as a plain `Text` node since no
    text-input primitive exists on the renderer side yet (a future
    renderer slice can lower `<text_input>` into a real primitive
    transparently to this app shape). Eleven tests pin: typing
    appends, Backspace pops + no-op on empty, Enter dispatches and
    clears, empty-buffer Enter dispatches "", input row renders
    prompt+buffer+caret, empty buffer renders just `> █`, pump→render
    round-trip, modifier-only events (Esc/Tab) don't mutate the
    buffer, 200-char buffer clips at terminal width, header on row 0
    stays intact when the command row paints.

This closes #1241's first acceptance criterion ("a small Jet demo
app renders and handles input in terminal") across all five Cue
surfaces — work-item-list (4a) + cursor/selection (4b) + detail
panel (4c) + status-timeline (4d) + log-stream (4e) + command-input
(4f). Total 35 demo tests across two files.

### Degradation reference (closing notes for #1241 AC#2)

`target-profiles.yaml` is the source of truth for the T1-T10 rules;
this table captures **where each rule is honored inside this crate**
so a Cue/jet UI author can predict what visuals survive the TUI hop.
The `severity` column mirrors `degradation_rules` in the YAML.

| Rule | Subject                             | Renderer state | Where                                                                                  |
|------|-------------------------------------|----------------|----------------------------------------------------------------------------------------|
| T1   | pixel dimensions → cells            | shipped        | `pixel_to_cell` (Slice 1 public surface). Deterministic round-to-cell.                 |
| T2   | percent dimensions                  | n/a            | Resolved at layout (`jet_wasm::layout`); the renderer only sees absolute `Rect`s.      |
| T3   | auto dimensions                     | n/a            | Same as T2; renderer-side pass-through.                                                |
| T4   | rgba color → ANSI-16                | shipped        | `style::quantize_to_ansi` (Slice 3). VGA-palette nearest-neighbor in squared distance. |
| T5   | font-family / weight / style / size | shipped        | `style::parse_style` lenient-skip (Slice 3). No hint emitted; terminal owns the font.  |
| T6   | text-shadow / box-shadow            | drop-silent    | No painter consumes these props. Effective behavior matches the YAML rule.             |
| T7   | border-radius                       | drop-silent    | `draw_border` paints square corners regardless. Effective behavior matches the rule.   |
| T8   | CSS transitions / transforms        | drop-silent    | TUI paint loop is `poll`+30 Hz (no rAF); animation props are dropped at lower.         |
| T9   | mouse → keyboard fallback           | partial        | `event_in::translate_event` translates real mouse events (Slice 2). The "every mouse handler MUST have a keyboard equivalent" check is a *build-pipeline* concern (#1239), not a renderer concern. |
| T10  | unsupported `LaidOutKind::Intrinsic` props | passthrough | Unrecognized intrinsic tags paint as no-op cells. Build-side rejection lives in `--target tui` source-validation (#1239). |

Two rules are deliberately *not* the renderer's responsibility: T9's
keyboard-equivalent enforcement and T10's unsupported-element error.
Both are source-validation passes in the build pipeline (#1239) — by
the time a frame reaches the renderer, the source has already been
rejected (or accepted, with `drop-silent` rules acknowledged). The
renderer's job is to be **the loudest about T1/T4/T5** (visible,
deterministic) and **silent about T6/T7/T8** (decorative, no-op).

### Spike → stable contract findings (closing notes for #1241 AC#3)

After Slices 1–4f, these are the design choices the spike validated
that should outlive #1241 and become contract for any future renderer
on the same `TargetRenderer` shape (web, desktop, future CLI/native):

- **`paint_into(buf)` vs trait `paint(frame)` is the right split.**
  `paint_into` is the deterministic, test-only path that writes into
  a caller-owned `ratatui::buffer::Buffer`; trait `paint` is the
  substrate-driven path that composes with `Terminal::draw`. Tests
  (35 demo + ~50 unit) never touch a real tty. Future renderers can
  ship the same dual surface — a "paint into a caller-owned target"
  primitive plus a "paint as the substrate told you" trait method.
- **Crossterm-side input translation is a pure function.**
  `event_in::translate_event(CtEvent, CellSize) -> Option<SyntheticEvent>`
  composes with *any* input source (real poll thread, headless
  `pump_event` for tests, future replay harness). New renderers
  should keep their input translators pure functions for the same
  testability win — keep the IO at the edges.
- **`Rc<RefCell<App>>` is the agreed app-state pattern inside the
  `Box<dyn Fn(SyntheticEvent)>` mount callback.** The trait's `Fn`
  (not `FnMut`) bound is intentional — multiple renderers will need
  to dispatch the same callback re-entrantly. App state goes through
  interior mutability; the demo files in `tests/` are the reference.
- **Multi-byte glyphs require char-indexed buffer assertions.** The
  `Cue — Work Items` em-dash exposed a footgun: `&row[40..46]`
  byte-slices the row String and skips cells. Test helpers across
  the demo files use a `cell_substr(row, start)` that does
  `.chars().skip(start).collect()`. Future renderer test suites
  (web visual-regression, future native/CLI) should adopt the same
  cell-indexed assertion vocabulary up front.
- **`LayoutTree::root_rect`-based mount sizing with an 80×24
  fallback works.** Renderers that don't know their physical size
  at construction time (TUI, future native) can default-size from
  the layout tree's root rect; if it's zero, fall back to a
  platform-sane default. `with_size(cols, rows)` exists for callers
  that *do* know.
- **No `SyntheticEvent::Custom` is needed for v1 demos.** The Slice
  4 demos thread app-internal "selected" / "submitted" state
  directly through the `Rc<RefCell<App>>` rather than dispatching
  custom synthetic events. When `Custom` lands, demos can re-target
  the substrate path with no change to the renderer surface — the
  demos are intentionally non-coupled to a custom-event mechanism.
- **Lifecycle armor lives in a private state-machine, not the
  trait.** `RealTerminal::LifecycleGuard` (Slice 2c) flips `armed`
  exactly once via `run_disarm`; `Drop` and explicit `leave()` both
  route through it. Future renderers needing teardown ordering (GPU
  context release, audio device release) should ship the same
  shape *inside* the renderer crate — the trait stays minimal.

These findings are the contract that #1238's renderer-neutral surface
needs to keep stable; if a future renderer slice forces any of them
to change, that's a contract-level change, not a renderer-local one.

### Public surface (Slice 1)

```rust
pub struct TuiRenderer { /* ... */ }
pub struct CellSize { pub px_per_col: u16, pub px_per_row: u16 }
pub fn pixel_to_cell(px_x: f32, px_y: f32, px_w: f32, px_h: f32, cell: CellSize) -> ratatui::layout::Rect;

impl TuiRenderer {
    pub fn new() -> Self;
    pub fn with_cell_size(self, cell: CellSize) -> Self;
    pub fn paint_into(&mut self, frame: &[LaidOutNode], buf: &mut Buffer);
    pub fn paints(&self) -> usize;
    pub fn is_torn_down(&self) -> bool;
}

impl jet_multi_target::renderer::TargetRenderer for TuiRenderer {
    fn profile(&self) -> &'static TargetProfile;
    fn mount(&mut self, root: &LayoutTree, on_event: Box<dyn Fn(SyntheticEvent)>);
    fn paint(&mut self, frame: &[LaidOutNode]);
    fn teardown(&mut self);
}
```

`paint_into` is the deterministic test-only path; the in-Slice-1 trait
`paint` only bumps a counter so substrate-side commit pulses are
observable without real terminal IO.

### Cross-references

- `element-contract.md` — the `TargetRenderer` trait and the C1-C10
  invariants this implementation must respect.
- `target-profiles.md` — the TUI capability profile + the T1-T10
  degradation table this crate progressively honors per slice.
- `build-targets.md` — `--target tui` selection + cargo-feature wiring
  in `jet-multi-target`.
- `crates/jet-multi-target/src/web.rs` — the WebRenderer adapter
  shipped in #1238; mirror of the same trait shape for reference.
