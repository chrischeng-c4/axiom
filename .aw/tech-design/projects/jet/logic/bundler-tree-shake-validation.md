---
id: projects-jet-logic-bundler-tree-shake-validation-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Bundler Tree-Shake Validation Suite

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/bundler-tree-shake-validation.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# Bundler Tree-Shake Validation Suite

### Overview

Spec for the fixture-based test suite that pins tree-shaker
behavior in `crates/jet/src/bundler/tree_shake.rs` against a
documented matrix of real-world DCE patterns. The suite is the
safety net for upcoming work that intensifies tree-shaker stress
(MFE shared-deps federation, scope-hoisting follow-up). Closed
precedents `bug-dce-and-minifier-assume-ascii-only-source-audit-al`
and `bug-mangler-scope-analysis-fails-on-large-bundles-267` are
the kind of regressions this suite must catch in CI before they
reach a user project.

@spec #1249 — `jet: bundler — tree-shaking validation suite`.

### Slice plan

The validation suite is delivered in named slices so each tick
can land an independent, mergeable artifact. Slice 1 is this
spec; Slices 2+ materialize fixtures one pattern at a time so
each tree-shaker bug surfaced is its own follow-up issue (per
the spec's "fixing surfaced bugs is out of scope" rule).

- **Slice 1 (this doc) — spec + fixture layout.** Pins pattern
  matrix, fixture directory shape, snapshot tooling, and the
  regression-gate baseline file format. No code yet.
- **Slice 2 (shipped) — fixture harness + first pattern.** Adds
  the `tree_shaking` integration-test entry as the single-file
  `crates/jet/tests/tree_shaking.rs` (deviation from the
  originally-planned `tree_shaking/` directory layout — the
  pure-functional surface of `tree_shake.rs` is small enough
  that a directory + `Fixture` runner would be over-structure;
  promote to a directory in Slice 9 when bundle-driven on-disk
  fixtures arrive). Drives the analyzer + shaker directly
  against synthetic in-memory `(PathBuf, String)` modules
  rather than full `Bundler::bundle` runs — keeps tests
  hermetic and millisecond-fast. Snapshot machinery is the
  inlined `snapshot_eq` helper from `tests/common/snapshot.rs`
  (R2's "insta or equivalent" — the helper writes
  canonical-JSON snapshots under `tests/__snapshots__/`,
  with `JET_SNAPSHOT_UPDATE=1` for accept). First pattern:
  `unused_named_exports` — proven by the `analysis` snapshot
  (helper records only the `used` export) plus the
  `shaken_helper` snapshot (post-`shake_module` source has
  the unused declaration dropped).
- **Slice 3 (shipped — folded into Slice 2 commit) —
  side-effect matrix.** Adds two fixtures alongside Slice 2's
  `unused_named_exports` so the side-effect polarity is pinned
  end-to-end: `side_effect_free_module_is_eliminated` (orphan
  module with `sideEffects: false` and no top-level statements
  → lands in `eliminated_modules`) and
  `side_effect_full_module_is_preserved` (polyfill with a
  top-level assignment → kept regardless of decl). The
  side-effect-full test also pins the documented bundler
  semantic that `sideEffects: false` overrides code analysis
  when an explicit `SideEffectsDecl::None` is passed to
  `module_has_side_effects`.
- **Slice 4 (shipped — baseline pinned, gap filed) — re-export
  chains.** Adds the `reexport_chain_partial_usage_baseline`
  fixture: `barrel.js` re-exports from `a.js` + `b.js`; entry
  uses only `a`. The analyzer is **currently conservative on
  re-exports** — `extract_import_bindings` only walks lines
  starting with `import `, so `export { x } from './y'` does
  NOT thread the "used" signal to the leaf module. Snapshot
  pins the under-shaken result (both `a.js` and `b.js` land in
  `eliminated_modules`); fix tracked as a separate bug at
  #1342 (`bug(jet): tree-shaker drops re-export chain leaves`).
  Per the issue's "fixing surfaced bugs is out of scope" rule
  the suite captures the regression baseline rather than
  blocking on the fix.
- **Slice 5 (shipped — baseline pinned, gap filed) — dynamic
  import retention.** Adds the `dynamic_import_retained_baseline`
  fixture: entry calls `import('./lazy')` but never references a
  named export. The analyzer's static-import matcher uses
  `starts_with("import ")` (with the trailing space), so dynamic
  `import(...)` call expressions don't match — `lazy.js` is
  wrongly eliminated. Snapshot pins the under-shaken result;
  fix tracked at #1344 (`bug(jet): tree-shaker eliminates
  dynamically-imported modules`). Same calibration-baseline
  pattern as Slice 4: a future fix shrinks the snapshot diff
  (lazy.js leaves `eliminated_modules` and gains `["*"]`); a
  regression in the other direction (e.g. eliminating entry.js)
  fails the suite.
- **Slice 6 (shipped — "not supported" baseline) — class with
  unused methods.** Adds the
  `class_unused_methods_documented_baseline` fixture: a class
  whose `add` method is called and `sub` method is unused. The
  current `crates/jet/src/bundler/{dce.rs,mangle.rs}` do NOT do
  method-level DCE — `tree_shake.rs` is export-level only. The
  test pins this "not supported" baseline (per the spec rule
  "remove if mangler supports; else document as not supported"):
  the class export `Calc` is marked used, the module survives
  intact, and `shake_module` preserves the unused `sub` method
  verbatim. Future mangler work that adds method-level DCE will
  produce a snapshot diff (`sub` disappears from
  `class_unused_methods__shaken_calc`); until then the snapshot
  calibrates today's behavior so accidental over- or under-
  shaking shows up loud.
- **Slice 7 (shipped — analyzer correct, shaker baseline) —
  mixed ESM/CJS interop.** Adds the
  `mixed_esm_cjs_interop_baseline` fixture: ESM entry imports
  `prod` from a CJS lib whose `exports.prod` and `exports.dev`
  are assigned on different branches of a `process.env.NODE_ENV`
  check. The analyzer correctly threads ESM-import → CJS-export
  binding (`prod` marked used, `dev` left out). The shaker is
  the documented limitation: `shake_module` only strips ESM
  `export ` lines, so the unused `exports.dev = …` line
  survives. Snapshot pins both the (correct) analysis result
  and the (limitation) shaker output. Future work to extend
  the shaker to CJS branches produces a clean snapshot diff.
- **Slice 8 (shipped) — conditional-exports pruning.** Adds
  the `conditional_exports_pruning_browser_condition` fixture:
  the resolver picks `b.js` under the `browser` condition while
  `n.js` (the unreachable node alternate) also sits in the
  module set. The analyzer marks only `b.js`'s `pkg_browser`
  export as used and lands `n.js` in `eliminated_modules` — the
  "n.js's exports do not leak into the bundle" property
  reduced to its analyzer-observable form. Resolver-side
  selection lives in `crate::resolver` tests; this fixture is
  the tree-shaker's safety net for the case where a buggy
  build pipeline accidentally pulls both alternates into the
  graph.
- **Slice 9 (shipped — closes #1249) — mini-react e2e + bundle-
  size regression gate.** Adds `mini_react_e2e_baseline`: a
  five-module synthetic-but-realistic React-flavored graph
  (`react/index.js` + `react-dom/index.js` + `app/hooks.js`
  re-export barrel + `app/main.js` entry + `app/devtools.js`
  side-effect-free orphan). Two snapshots ship: the analyzer
  result (R4 — proves the suite handles a multi-module graph
  end-to-end) and a `post_shake_bytes_total` +
  `post_shake_bytes_per_module` payload computed by running
  `shake_module` over every module and summing surviving
  bytes (R3 — any analyzer or shaker change that flips a
  module's keep/drop decision moves the number, and the
  snapshot diff fails CI; `JET_SNAPSHOT_UPDATE=1` accepts
  intentional changes). The fixture is hermetic and
  millisecond-fast (synthetic in-memory modules, no
  `npm install`, no real `Bundler::bundle`); when bundle-output
  snapshots are wanted later they ship as a follow-up issue
  rather than blocking #1249's close. Today's baseline reflects
  the under-shaken state from #1342 (re-export chain leaves)
  and #1344 (dynamic imports — not exercised here); when
  either fix lands the size baseline shrinks and the snapshot
  updates in the same PR that flips the analyzer behavior.

With Slice 9 shipped the suite owns nine fixtures covering
every pattern in the issue's `### In Scope` list (R1), uses
the `snapshot_eq` machinery for every assertion (R2), pins
a regression gate on the post-shake byte count (R3), and
closes on a realistic mini-react graph (R4). #1249 closes.

Slice 9 closes the matrix; the suite then owns ten fixtures
covering every pattern in the issue's `### In Scope` list.

### Fixture layout

```
crates/jet/tests/tree_shaking/
  mod.rs                    # the integration-test entry; one #[test] per fixture
  fixture.rs                # the Fixture runner (loads inputs, drives Bundler, asserts)
  baselines.toml            # regression-gate baseline file (Slice 2+)
  snapshots/                # insta snapshot directory
    unused_named_exports.snap
    side_effects_false_eliminates_module.snap
    ...
  fixtures/
    unused_named_exports/
      input/
        entry.ts            # entry point (always `entry.ts`)
        utils.ts            # supporting modules
      package.json          # only present when the fixture pins package metadata
      jet.config.toml       # only present when the fixture pins build config
      assertions.toml       # per-fixture expected outcomes — see schema below
    side_effects_false_eliminates_module/
      input/
        ...
      package.json          # `"sideEffects": false`
      assertions.toml
    ...
```

Conventions:

1. **Entry is always `entry.ts`.** The Fixture runner finds it
   by name; no fixture overrides the entry path.
2. **Per-fixture `package.json` is optional.** Most fixtures
   exercise inline source patterns; only the side-effect and
   conditional-exports fixtures need a real package boundary.
3. **`jet.config.toml` is optional.** Default config is the
   workspace root's; per-fixture overrides only apply when the
   pattern requires it (e.g. resolver conditions for Slice 8).
4. **Snapshots use `insta`.** R2 calls out
   "`insta` or equivalent" — `insta` is the workspace-pinned
   snapshot crate and aligns with existing usage in
   `projects/agentic-workflow-codegen/`.

### Per-fixture assertions schema

`assertions.toml` is the typed contract each fixture's snapshot
diffs against. The Fixture runner deserializes it, runs the
bundle, and asserts the snapshot + the typed fields side-by-side
so a structural assertion (e.g. "export X is eliminated")
fails distinctly from a snapshot-mismatch assertion ("the
emitted bundle text drifted").

```toml
# crates/jet/tests/tree_shaking/fixtures/<name>/assertions.toml

# Modules the tree-shaker is expected to eliminate entirely.
# Path is relative to the fixture's `input/` dir.
eliminated_modules = ["utils_unused.ts"]

# Modules expected to retain at least one export.
retained_modules = ["entry.ts", "utils.ts"]

# Exports expected to NOT appear in the final bundle text. Each
# entry is a substring search (no regex) so authors can pin
# user-visible names without coupling to mangler output.
forbidden_substrings = ["unused_named_helper"]

# Exports / identifiers expected to remain in the bundle text.
# Same substring-match rule.
required_substrings = ["used_helper"]

# Regression-gate: max bundle size in bytes. Authored by
# Slice 2+ then auto-updated only via `cargo test --features
# update-tree-shaking-baselines` (a separate baseline writer
# entry-point — keeps drift visible in PRs).
max_bundle_bytes = 4096
```

### Regression-gate file format

`baselines.toml` is the source-of-truth size-baseline file for
the suite. One section per fixture. CI reads it, runs the
Fixture runner with `enforce_baseline = true`, and fails if any
fixture's bundle exceeds its recorded `max_bundle_bytes`.

```toml
# crates/jet/tests/tree_shaking/baselines.toml
# AUTO-GENERATED — edit via `cargo test --features
#   update-tree-shaking-baselines` and commit the diff.

[unused_named_exports]
max_bundle_bytes = 412

[side_effects_false_eliminates_module]
max_bundle_bytes = 280
# ...
```

The `update-tree-shaking-baselines` feature is a workspace-level
opt-in that flips the runner from "assert" to "rewrite" mode.
PRs that bump a baseline carry a visible diff to this file,
which is the audit trail the regression gate (R3) needs.

### Snapshot scope

Per-fixture snapshots record:

1. **Bundle text** — `*.snap` file with the full minified
   bundle. Catches mangler regressions, ASCII-only assumption
   regressions (the closed
   `bug-dce-and-minifier-assume-ascii-only-source-audit-al`
   precedent), and re-export-chain drift.
2. **`TreeShakeResult` debug** — a derived snapshot capturing
   `eliminated_modules` (sorted) + `eliminated_bytes`. Catches
   "we kept the byte count but ate the module" regressions
   that pure-text snapshots miss.

Snapshot text is post-minified to keep diffs stable across
mangler whitespace tweaks. The runner pins minify on (default
production config) for exactly this reason.

### Out of scope (this suite)

- **Fixing tree-shaker bugs the suite surfaces.** Each becomes
  its own `bug(jet)` follow-up issue. The suite's job is to
  make the bug visible and reproducible.
- **CI perf tracking across runs.** A separate concern (issue
  body §"Out of Scope"); the regression gate is per-PR
  bundle-size only, not per-run wall-clock.
- **Snapshot churn rules.** When to bump a snapshot vs. file a
  bug is reviewer judgment; this spec doesn't try to encode it.
  The audit trail is the diff to `baselines.toml` + the snapshot
  file in the PR.

### Cross-references

- `crates/jet/src/bundler/tree_shake.rs` — module under test.
- `crates/jet/src/bundler/mangle.rs` — interacts with DCE; the
  class-unused-methods fixture (Slice 6) is the boundary case.
- `.aw/tech-design/crates/jet/bundler.md` §"Tree Shaking" —
  current production behavior; this suite snapshots it.
- Closed precedents:
  - `bug-dce-and-minifier-assume-ascii-only-source-audit-al`
  - `bug-mangler-scope-analysis-fails-on-large-bundles-267`
  - `enhancement-jet-build-expand-mini-react-example-with-advanced`
- Feeds: `epic-module-federation-config-container-manifest-shared`
  (#1121) — federated shared deps depend on aggressive DCE to
  avoid duplicated code across remotes; this suite is the gate
  that lets the federation work move fast.
