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

`red` on the expanded DOM production-build corpus (see the dashboard in
`projects/jet/README.md`). Build is intentionally third in the Basic
replacement order, behind package management and Browser Bridge.

## Open gaps before "=== vite/webpack" is claimable

- Remaining corpus fixtures failing correctness or size/speed thresholds.
- Keep runtime trace, gzip size, CSS/public/static artifact, and contract
  self-test checks green across the whole corpus, not just the
  representative fixture.
