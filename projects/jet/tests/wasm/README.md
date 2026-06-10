# Block: FE-on-WASM — match the FE-on-DOM oracle

**Claim.** The Advanced toolchain sinks TS/TSX, CSS, and the HTML-like host
tree into Jet's Rust/WASM runtime and paints via canvas/WebGPU. For the same
app and gesture, behavior must match the Basic FE-on-DOM oracle; any
divergence is a Jet bug (DOM-oracle-first rule).

## Gates

| Gate | Command | Covers |
|---|---|---|
| WASM build e2e | `cargo test -p jet --test wasm_build_end_to_end` | `jet build --wasm`, WebGPU scaffold default, runtime status/visual probe |
| DOM oracle conformance | `cargo test -p jet --test react_dom_oracle_conformance` | React-DOM-vs-WASM observable oracle |
| MUI visual regression | `cargo test -p jet --test mui_visual_regression` | real MUI DOM/WASM visual parity |
| TSX→Rust lowering | `cargo test -p jet --test tsx_to_rust_imports` (+ sibling `tsx_to_rust_*` targets) | typed lowering of imports, state, events, i18n |
| Parity oracle link | `cargo test -p jet --test parity_oracle_reexport` | `jet_parity_oracle::run_fixture` resolves |
| Dev smoke | `cargo test -p jet --test wasm_dev_smoke` | `jet dev --wasm` serves the counter demo |

Aggregate script: `projects/jet/scripts/verify-advanced-wasm-gates.sh`.

## In this folder

- `tsx_to_rust_*.rs` — typed TSX→Rust lowering fixtures (TSX inputs in
  `../fixtures/tsx_to_rust_*.tsx`).
- `*_debug.rs` — single-behavior runtime verification cells (hooks, lists,
  unicode, nesting, handlers, ...), built on `../common/` and JSON snapshots
  in `../__snapshots__/`.
- Oracle/visual: `react_dom_oracle_conformance.rs`, `mui_visual_regression.rs`,
  `parity_oracle_reexport.rs`.
- Toolchain: `wasm_build_end_to_end.rs`, `wasm_dev_smoke.rs`.

## Open gaps

- Replayable DOM-vs-WASM traces for layout, text, selection, clipboard,
  scroll, context menu, a11y, and post-load performance budgets (see the
  Advanced contract-family table in `projects/jet/README.md`).
