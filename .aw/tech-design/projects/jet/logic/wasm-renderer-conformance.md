---
id: projects-jet-logic-wasm-renderer-conformance-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# jet-wasm React-compat conformance policy

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/wasm-renderer-conformance.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# jet-wasm React-compat conformance policy

### Overview

jet-wasm doesn't *run* React. We transpile TSX to Rust, implement
our own fiber + hooks + commit loop in `jet_wasm::react`, and our
own canvas renderer paints the result. When a user writes
"React-like" TSX, they expect React-like *behavior* — that's a
**contract we make**, not a library we depend on.

This doc formalizes that contract: what "jet-wasm behaves like
React" means, which observations prove it, and how the conformance
suite grows with the runtime.

It also defines the framework-agnostic surface that future Vue /
Angular / Solid adapters will share — the same observable oracle
(element tree, layout tree, paint ops) validates *any* adapter that
targets `jet_wasm::Element`.

Supersedes the ad-hoc "Level-1 verification suite" introduced in
commit `b02f5bee`. That suite is re-classified under this policy as
the React-specific feature-coverage tier.

### Scope — what we guarantee

**In contract (jet-wasm must behave like React for these)**:

1. **Deterministic render output.** Given the same TSX source and
   the same runtime inputs (initial props, event sequence), the
   rendered Element tree, the laid-out tree, and the emitted paint
   ops are bit-for-bit identical across runs. No clock, no random
   seed, no unobserved mutable global leaks into the output.

2. **Rules of hooks.** Hooks are called in declaration order every
   render. Conditional hook calls are a violation the runtime
   detects at hook-access time with a clear panic message.

3. **State cell identity.** `useState(init)` returns the same cell
   across renders. `initial` is used only at mount; subsequent
   renders read the updated value.

4. **Setter semantics.** Calling a setter marks the owning fiber
   dirty. The next `flush()` re-renders. Setters are idempotent
   within a single commit — setting to the same value does not
   force a re-render. *(Idempotence not yet implemented; tracked.)*

5. **Fiber identity under conditional render.** A `{cond && <X/>}`
   branch that toggles on → off → on gets a **fresh** child fiber
   each time. State inside `<X/>` does not persist across
   mount/unmount. This matches React's default (non-keyed) behavior
   for intrinsic children.

6. **Element tree correspondence with intrinsic props.** What React
   would produce as `props` on a DOM element is exactly what the
   jet `Props` struct carries. Attribute names camel→snake (e.g.
   `className` → `class_name`), values pass through unchanged.

7. **Event payload types.** `onClick` receives `&SyntheticMouseEvent`
   (or `()` for zero-arg arrows; both shapes accepted by the
   transpiler — see `event-pipeline.md`). `onChange` (when it lands)
   receives `String`. A user callback signature mismatch is a
   transpile error, not a runtime crash. Bubble dispatch + `e.stop_propagation()`
   + `e.prevent_default()` are part of the contract; React's capture
   phase (`onClickCapture`) is not.

**Out of contract (where jet-wasm diverges from React, by design)**:

- **`i64` values outside `[-2^53, 2^53]` lose precision at the JS
  boundary.** The Rust runtime handles full i64 correctly; but
  `serde-wasm-bindgen` 0.6 emits i64 as JS `BigInt`, and CDP's
  `Runtime.evaluate` with `returnByValue: true` doesn't always
  deserialize BigInts cleanly. Rust-side state stays exact, but
  the debug bridge and any JS consumer sees an approximation for
  magnitudes above 2^53 (≈9.0 × 10¹⁵). Documented in
  `large_int_debug.rs`; fixable by switching the bridge surface
  to return `String` for i64 slots (follow-up).
- **No batching across microtasks.** React 18's automatic batching
  across async boundaries is not modeled. Setters within the same
  synchronous handler coalesce; everything else re-renders
  immediately. Acceptable because we don't have an async executor yet.
- **No Suspense / concurrent mode.** Not in scope until the runtime
  grows an async executor (see `architecture.md` phase 8).
- **No StrictMode double invocation.** Not modeled. Side effects in
  render bodies are not protected against.
- **No synthetic event pooling.** jet events are plain values
  (`()`, `String`), not objects — no `.persist()` to learn about.
- **Keyed list reconciliation.** `key={...}` is currently dropped.
  Full-rebuild on every commit means key stability doesn't matter
  yet. When reconciliation diffing lands (phase post-v0), `key`
  becomes load-bearing.
- **No portals, no ref forwarding.** Deferred.

When a user-visible behavior change requires a spec delta here, it
goes through `aw td` with this doc as the primary artifact.

### Observable oracle

All conformance assertions operate on three framework-agnostic
observables the `debug` feature surfaces:

| Surface | Source of truth | What it proves |
|---|---|---|
| `elementTree()` | `MountHandle.snapshot()` after render + flush | The reactive layer produced the tree the TSX asked for. Includes intrinsic tags, text, conditional-render branches, Fragment contents. |
| `layoutTree()` | `renderer::layout(element, viewport)` | The layout pass placed every element with sensible rects. No NaN coords, no zero-size laid-out nodes, Fragment correctly transparent. |
| `paintOps()` | Last-frame op vector from `CaptureBackend` | The paint pipeline emitted the ops a backend would need to draw. Deterministic — same tree + theme + viewport produces exactly the same ops. |

These three are the **framework-neutral contract**. Any future
adapter — Vue SFC → Rust, Angular templates → Rust, Solid JSX →
Rust — that targets the same `Element` enum will be validated
against the *same* oracle. The test body stays the same; only the
source file the harness points at changes.

### Framework-specific surfaces (React-only today)

The `debug` bridge *also* exposes `fiberTree()` + `hookValues(id)`
— those are **React-specific** and only meaningful when the
runtime is React-flavored (our default). Tests that read them
belong in the React conformance tier, not the shared observable
tier.

When Vue lands, the analogue would be `reactivityGraph()` (signal
dependencies). When Angular lands, `zoneStats()` (change-detection
cycles). Each framework adapter gets its own feature-gated
inspection surface on top of the shared observable oracle.

### Conformance matrix

Every feature the transpiler accepts contributes cells to a
2-dimensional matrix:

```
                       ┌──────────────────────── behavior dimension ─────────────────────────┐
                       │                                                                      │
              basic   identity   batching   boundary   typechecks   error-msg   stress
┌─────────────┼──────────────────────────────────────────────────────────────────────────────┤
│ useState<T> │  ✓      ○         ○          ○           ✓            ○          ○          │
│ conditional │  ✓      ○         —          ○           ✓            ○          ○          │
│ .map() iter │  ✓      ○         —          ○           ✓            ○          ○          │
│ onClick     │  ✓      ○         ○          ○           ✓            ○          ○          │
│ ...         │                                                                               │
└─────────────┼──────────────────────────────────────────────────────────────────────────────┘
             feature dimension
```

Legend: ✓ covered by ≥1 test · ○ cell exists but no test yet · — not applicable

**Behavior dimensions** (the columns):

- **basic** — happy-path "it works". One test per feature.
- **identity** — cells / fibers / instances stay themselves across
  unrelated re-renders. Two instances don't cross-talk. Conditional
  render doesn't corrupt state across toggles.
- **batching** — multiple setters in one handler cause correct
  number of renders. Zero-delta sets don't trigger repaint. (Not
  applicable to non-stateful features.)
- **boundary** — extreme inputs. Empty list, 10⁴-item list, very
  long strings, unicode, nested depth 10+, zero props, large
  numbers near i64 bounds.
- **typechecks** — the transpiler rejects out-of-subset TSX with a
  clean error. Non-`Copy` types get a `.clone()` emitted.
  Primitives don't.
- **error-msg** — runtime failure modes surface useful messages.
  Rules-of-hooks violation → named panic. Missing props →
  downcast panic with type name. Out-of-range highlight → no-op,
  not a crash.
- **stress** — 10⁴ clicks doesn't leak, doesn't crash, doesn't
  drift. Memory stable, output deterministic.

**Feature dimensions** (the rows, grows over time): each row is a
transpiler construct or runtime feature. New features get a row
when they land.

### Target density

- Pre-M1 (shipped now): **basic** cells only for the features in
  `subset.md` §Verified Features. 12 rows × 1 column = 12 tests.
- Post-M1 (next): add **identity** + **boundary** for each existing
  row. Roughly doubles test count without new features.
- Ongoing: each new feature ships with basic + one other column
  filled.

Not a hard rule — some cells are "n/a" (conditional render doesn't
have batching semantics). The matrix is a prompt, not a quota.

### Test tiers

Tests split into three tiers with different guarantees:

| Tier | Scope | Location | Framework tie |
|---|---|---|---|
| **Observable** | Assertions on `elementTree` / `layoutTree` / `paintOps` only | `crates/jet/tests/*_debug.rs` using `common::observable::*` | Framework-agnostic. Same test body works for any adapter. |
| **React** | Assertions on `fiberTree` / `hookValues` / rules-of-hooks panic messages | Same files, using `common::react::*` | Only runs when the app was built against `jet_wasm` React runtime. |
| **Adapter-specific** | Vue signals, Angular zones, Solid fine-grained observations | `crates/jet/tests/vue_*_debug.rs` (future), etc. | Per-adapter only. |
| **Event** | Synthetic-event dispatch: bubble order, `stop_propagation`, `prevent_default`, `e.client_x` / `e.client_y` plumbing | `crates/jet-wasm/tests/synthetic_click_event.rs` (planned per `event-pipeline.md` R8) | Framework-agnostic — `dispatch_click` is in the substrate. |

The harness in `tests/common/` gates React-tier methods behind a
module boundary so a Vue/Angular test that accidentally reaches
into fiber state fails at compile time rather than silently
miscomparing.

### Snapshot strategy

For observable assertions, prefer **snapshot comparison** over
hand-written equality checks:

```rust
let tree = app.element_tree().await?;
snapshot_eq!("items_list_initial", &tree);
```

- Snapshots live in `crates/jet/tests/__snapshots__/<test-name>.json`.
- First run: writes the snapshot.
- Subsequent runs: compares byte-for-byte (after canonical JSON
  pretty-printing) — any drift fails with a diff.
- Update: delete the snapshot file, or set `JET_SNAPSHOT_UPDATE=1`
  before running.

Why roll our own rather than pull `insta`:

- Current test output we care about is small (element trees measured
  in KB). A ~30-line canonical-JSON differ is cheaper than a new
  crate dep.
- Snapshots are JSON (not the `insta::assert_yaml_snapshot!`
  format), which humans reading CI logs + grepping diffs find
  easier.
- Full `insta` adoption (if/when it happens) is a trivial swap —
  the call site would be `insta::assert_json_snapshot!(tree)`.

Hand-written `assert_eq!` on parsed shapes is still fine for
targeted checks (e.g. "the fifth span's text is exactly 'item
40'"); use snapshots for holistic tree checks where the point is
"the whole shape must not drift".

### Cross-framework contract (forward-looking)

When Vue lands (`crates/jet-vue-adapter` or similar, pending), the
contract is:

1. **Same `Element` enum.** Vue → Rust transpiler produces the same
   `jet_wasm::Element` tree for a semantically-equivalent template.
2. **Same observable oracle.** `elementTree()` / `layoutTree()` /
   `paintOps()` for a Vue Counter demo produce output matching the
   TSX Counter demo byte-for-byte.
3. **Same test harness.** `JetTestApp::launch("counter-demo")`
   works whether `counter-demo/src/Counter.tsx` or
   `counter-demo/src/Counter.vue` is the entry. `jet.config.toml`'s
   `entry` path determines which transpiler runs.

Adapter divergences allowed:

- Hook-equivalent semantics (Vue signals fire on read/write, not
  per-render). Tests that probe this live in the Vue-specific
  tier.
- Source-map shape (Vue single-file-components vs TSX line/col).
- Framework-internal state surfaces (`reactivityGraph()` vs
  `fiberTree()`).

The **rendered result** stays identical. This is how we make
"jet-wasm as a framework-agnostic runtime substrate" concrete —
it's enforced by the test suite, not just in architecture prose.

### Migration test example (future)

```rust
// crates/jet/tests/counter_observable.rs
#[rstest::rstest]
#[case("counter-demo")]          // TSX
// #[case("counter-demo-vue")]   // .vue (when adapter lands)
// #[case("counter-demo-ng")]    // Angular (when adapter lands)
#[tokio::test]
#[ignore]
async fn counter_renders_and_increments(#[case] demo: &str) {
    let app = JetTestApp::launch(demo).await?;
    snapshot_eq!("counter_initial", &app.element_tree().await?);
    app.click_canvas(30.0, 12.0).await?;
    snapshot_eq!("counter_after_click", &app.element_tree().await?);
}
```

Same test body, three adapter inputs, three identical snapshots.
If any adapter drifts, CI fails at the snapshot diff with a clear
signal.

### Process — adding a new feature

1. **Spec the feature** in the relevant sub-doc (`transpiler.md`,
   `paint-runtime.md`, etc.).
2. **Add a row to `subset.md` §Verified Features.**
3. **Land code + spec + `basic` test in one commit** (the
   convention after the 2026-04-24 fillback).
4. **Within a follow-up commit**, add `identity` / `boundary`
   cells. Don't block feature landing on full-matrix coverage —
   the `basic` column is the minimum, the rest is incremental.
5. **If the feature has framework-specific behavior**, put those
   assertions in the React tier (`common::react::*`). Framework-
   agnostic assertions go in the observable tier.

A feature without a `basic` cell passing is not considered
"shipped" regardless of what the code does.

### CI validation

The machine-readable manifest [`conformance.yaml`](./conformance.yaml)
is validated on every PR by `cclab check-conformance-manifest`
(implemented in `crates/jet-conformance-cli/`).

The check runs three layers:

1. **Structural validation** — each entry must satisfy the schema at
   [`conformance.yaml.schema.json`](./conformance.yaml.schema.json):
   required fields (`id`, `subset_rule`, `feature`, `demo_dir`,
   `test_file`, `status`); `subset_rule` matches `^(S|X|B)[0-9]+$`;
   `status` is one of `verified` / `unit_only` / `pending`.

2. **AST-kind invariant** — `ast_node_kinds` is required for any
   entry whose `subset_rule` starts with `S` or `X`, and must be
   omitted for boundary entries (`B*`). This makes R3/R4/R5 of the
   subset-rigor spec enforceable as machine state, not prose.

3. **Path existence** — for every non-pending entry, the CLI asserts
   that `<workspace_root>/examples/<demo_dir>` exists and that
   `<workspace_root>/crates/jet/tests/<test_file>` exists. Entries
   with `status: pending` skip path checks. Entries with
   `status: unit_only` are checked when `--strict` is passed.

Invocation:

```sh
cclab check-conformance-manifest                # default paths
cclab check-conformance-manifest --strict        # also check unit_only entries
cclab check-conformance-manifest --workspace-root <dir>
```

Exit code: 0 on success, 1 on any validation error. CI runs the
default invocation as a required check on every jet branch.

Adding a new feature: every PR that adds an inclusion rule (S11+) or
demonstrates a new feature must add or update the corresponding entry
in `conformance.yaml`. See `subset.md` § Growth Policy for the full
checklist.

### Cross-references

- `subset.md` — the feature dimension of the matrix.
- `debug-bridge.md` — the observable oracle's wire format.
- `architecture.md` — phased delivery map, axioms.
- Code: `crates/jet/tests/common/` — the harness that implements
  this tiering.
- Future: `conformance-vue.md`, `conformance-angular.md` — per-
  adapter extensions that name the divergences.
