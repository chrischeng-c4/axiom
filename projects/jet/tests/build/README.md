# Block: build — match Vite/Webpack output, never bigger, always faster

**Claim.** `jet build` is interchangeable with a Vite/Webpack production
build for real DOM apps, in this order of importance:

1. **Correctness** — the built app must behave identically (runtime smoke in
   a real browser, CSS/public/static artifacts present and correct).
2. **Bundle size** — gzip bundle size must not exceed the Vite/Webpack
   output for the same fixture.
3. **Speed** — build wall-clock must beat both Vite and Webpack.

## Gates

**The block owner is `build_parity_gate.rs`**: run
`cargo test -p jet --test build_parity_gate -- --nocapture` to verify the
whole claim. It drives the Jet/Vite/Webpack corpus and asserts per fixture,
in priority order: runtime smoke green (through `jet bb`), static
functional checks green, gzip ≤ the smallest baseline (2% measurement
tolerance), and build duration strictly faster than the fastest baseline.

| Gate | Command | Covers |
|---|---|---|
| Block owner | `cargo test -p jet --test build_parity_gate -- --nocapture` | correctness + size + speed claim over the whole corpus |
| Jet/Vite/Webpack corpus | `node projects/jet/scripts/compare-dom-build-corpus.mjs --runtime-smoke required` (also driven by `verify-basic-dom-gates.sh`) | per-fixture build success, runtime smoke via `jet bb`, speed and gzip-size comparison vs Vite and Webpack |
| Production-build regression | `cargo test -p jet --test production_build_regression` | representative React/MUI fixture stays buildable and runnable |
| Tree shaking | `cargo test -p jet --test tree_shaking` | snapshot suite incl. `__snapshots__/tree_shaking__mini_react_e2e__bundle_size_baseline.json` size baseline |
| Monorepo bundling | `cargo test -p jet --test bundler_monorepo` | workspace-internal dependency bundling |
| HMR rebuild bench | `cargo test -p jet --test incremental_rebuild_bench` | incremental rebuild latency budget (R6, #1250) |

Corpus fixtures: `../fixtures/dom-production-build/` (react-bench, mui,
antd, tailwind, styled-components, ...); regression fixture:
`../fixtures/production-build-regression/`.

## Current status

Correctness is green across the whole corpus (runtime smoke through
`jet bb` plus static functional checks on all six fixtures, 2026-06-10).
The remaining violations are size-only on three fixtures; duration is
won on five of six. Latest gate snapshot (ratio vs the best of
Vite/Webpack; bar: gzip ≤ 1.0 + 2% tolerance, duration < 1.0):

| Fixture | duration | gzip |
|---|---|---|
| react-bench | 0.98 ✓ | 1.04 |
| dom-production-assets | 0.87 ✓ | 1.04 |
| mui-visual-demo | 1.58 | 1.21 |
| antd-visual-demo | 0.94 ✓ | 0.69 ✓ |
| tailwind-visual-demo | 0.45 ✓ | 1.01 ✓ |
| styled-components-visual-demo | 0.91 ✓ | 1.15 |

## Open gaps before "=== vite/webpack" is claimable

All remaining gaps are CJS-interop scaffolding and residual unshaken
code; the per-fixture diagnosis (with byte counts) lives in the round-2
workflow output:

- Interop scaffolding reaches the final output: `__esModule`
  defineProperty markers per consumed-as-namespace module, inline
  `$(n)["default"]||$(n)` interop thunks (726x on mui), registry slots
  `var X={exports:{}}`, and string-keyed export assignments — ~60KB raw
  on mui and the whole react-bench/assets 1.04 gap. Fix lives in the
  flatten emission (scope_hoist.rs): emit markers only for namespace
  consumers, cache one interop var per target, skip slots for fully
  flattened modules.
- mui residual: CJS+ESM duplicate package copies (~23KB pre-minify) and
  within-module dead code vite's DCE removes.
- styled residual: dead `var`/`const` initializer declarations of unused
  exports (the orphan collector currently handles `function`
  declarations only) and the remaining dev-map ternary.
- mui duration 1.58 tracks its size; closes as the bundle shrinks.
