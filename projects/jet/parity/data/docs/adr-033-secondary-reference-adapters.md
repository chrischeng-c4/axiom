# ADR-033: Secondary reference adapters — `ReferenceAdapter` trait, Angular+Material and Vue+Vuetify scaffolds, divergence-triage routing

| Field | Value |
|-------|-------|
| Issue | #2141 |
| Parent epic | #2133 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Refactor the `jet-parity-oracle` crate (#2139) so the React+MUI runner becomes one implementation of a new `ReferenceAdapter` trait, and add two scaffold-only sibling adapters — `MaterialAdapter` (Angular 17+ + Angular Material 17+, Material 3 design tokens) and `VuetifyAdapter` (Vue 3 + Vuetify 3) — that satisfy the same trait surface but return structured `Err(AdapterError::Unimplemented { adapter_id, method })` from both trait methods. The trait has exactly two methods (`mount(&FixtureIntent) -> AdapterContext`, `capture_channel(Channel, &AdapterContext) -> ChannelArtifact`); no third method may be added without amending the spec. `parity-gating.toml` gains a `[adapter]` section selecting the active adapter set by id (`mui`, `material`, `vuetify`); selecting a scaffold-only adapter for live capture exits 2 with a structured `{ kind: "adapter_stub", adapter_id, missing_methods }` envelope on stderr. The trait-routed React+MUI adapter must produce byte-identical `ChannelArtifact` output to the pre-refactor #2139 runner, enforced by a regression test diffing the two artifact streams over the existing parity corpus. When two or more adapters in the active set produce divergent artifacts for the same `(fixture, channel)` pair, the parity report tags every artifact with its `adapter_id` and routes the divergence into one of three triage buckets: `jet_regression` (Jet disagrees with all reference adapters), `cross_stack_waiver` (one reference adapter disagrees with the others — likely a stack-specific quirk), `wpt_authoritative` (all reference adapters disagree with each other — escalate to WPT as tiebreaker). A new CLI flag `--include-secondary-adapters` (default off) opts a run into executing `material` and `vuetify` alongside `mui`; without the flag the per-PR parity budget covers only React+MUI, and secondary runs charge against an opt-in budget bucket that never blocks PR merge. Panic-placeholder macros (`unimplemented!()`, placeholder `unreachable!()`, any `panic!("TODO")` shorthand) are forbidden in adapter trait method bodies; a CI lint scans adapter crate sources and fails the build on a match. Initial scope = 3 fixture intents (Button, TextField, Checkbox) reusing the MUI corpus from ADR-030; secondary live capture implementations land in follow-up issues, not here. |

## Context

ADR foundation #2139 picked one specific reference stack —
React 18 + ReactDOM + Material-UI v5 — as jet's authoritative
parity oracle. That choice is right for the *primary* gate
(some concrete, falsifiable target is better than none), but it
collapses three independent failure modes into a single signal:

1. **Normative browser behavior.** The DOM does X because the
   HTML/CSS/ARIA specs say so.
2. **React-rendering quirks.** React-DOM does X because of how
   it batches updates, hydrates, or handles synthetic events.
3. **Material-UI styling artifacts.** MUI does X because of its
   emotion-based CSS-in-JS layer, its theme tokens, or its
   component-specific JSX wrappers.

When the primary oracle reports "jet diverges from MUI on
fixture F, channel C", we can't tell from a single oracle which
of those three layers we're diverging from. Today that
forces every divergence into manual triage — an engineer
hand-reads the MUI source, compares to the spec, and decides
whether to file a jet bug, file an MUI bug, or carve a waiver.
That triage cost is the bottleneck on the per-PR parity budget.

The standard remedy in browser-engine teams (WebKit,
Blink, Gecko) is *triangulation*: run the same test in
multiple independent implementations and compare the
disagreements. When all references agree and the engine
under test disagrees, the engine has the bug. When two
references agree and one disagrees, the disagreeing one has
the bug. When all three disagree, the spec is the tiebreaker.

This ADR transplants that pattern into the jet parity
channel. We add *secondary* reference adapters running the
same canonical `FixtureIntent` (component shape, prop shape,
ARIA shape, interaction shape) through alternative DOM stacks:
Angular 17 + Angular Material 17 (Material 3 tokens), and
Vue 3 + Vuetify 3. The choice is deliberate: both are mature
production stacks with their own opinionated component
libraries that *also* claim Material Design conformance, so
the cross-stack comparison is meaningful (we're not comparing
apples to oranges — we're comparing three teams' independent
takes on "what a Material button does").

Critically, this ADR ships the **trait + scaffolds + routing
+ gating**, not the live capture implementations. The
secondary adapters return structured `Err` from both trait
methods; live capture lands in follow-up issues (one per
adapter). Shipping the foundation separately from the
implementations lets us land the divergence-triage routing,
the CLI flag, the budget accounting, and the byte-equivalence
regression of the refactored React+MUI adapter *without*
waiting on the Angular and Vue toolchain work.

The trait surface is intentionally minimal — two methods.
That ceiling is load-bearing: every additional method
multiplies the cost of bringing up a new adapter, and we
want adding "the fourth stack" (Svelte? SolidJS?) to be a
weekend project, not a quarter.

## Decision

### The trait

Define `ReferenceAdapter` in `projects/jet/parity-oracle` with
exactly two methods:

```rust
pub trait ReferenceAdapter {
    fn adapter_id(&self) -> &'static str;

    fn mount(
        &self,
        fixture: &FixtureIntent,
    ) -> Result<AdapterContext, AdapterError>;

    fn capture_channel(
        &self,
        channel: Channel,
        ctx: &AdapterContext,
    ) -> Result<ChannelArtifact, AdapterError>;
}
```

(`adapter_id` is a `const`-ish accessor, not counted as one of
the two contract methods that R1 pins.) Adding any third
behavior method requires amending the spec at
`.aw/tech-design/projects/jet/specs/jet-parity-secondary-adapters.md`
and a follow-up ADR.

`FixtureIntent` is a stack-agnostic description of the widget
under test — component shape, prop shape, ARIA shape,
interaction shape. The canonical JSON Schema lives at
`projects/jet/data/parity/fixture-intent.schema.json` and is
referenced by every fixture's `parity.toml`.

`AdapterContext` is the opaque per-adapter mount handle (a
Playwright `Page` for `mui`, an Angular `TestBed` reference for
`material`, a Vue `mount()` wrapper for `vuetify`).

`ChannelArtifact` is the 5-channel-compatible output type from
ADR-030 (`screenshot.png` bytes, `ax-tree.json`,
`focus-trace.json`, `pointer-hit-map.json`, `ime-trace.json`)
— same structure across all adapters so the comparator can
diff them without per-adapter shims.

### The three adapter crates

| Crate | Adapter id | Stack | Status this ADR |
|-------|------------|-------|-----------------|
| `jet-parity-adapter-mui` | `mui` | React 18 + ReactDOM + MUI v5 (existing #2139 runner) | refactored to implement `ReferenceAdapter`; byte-equivalent output regression-tested |
| `jet-parity-adapter-material` | `material` | Angular 17 + Angular Material 17 (Material 3 tokens) | scaffold; both methods `Err(AdapterError::Unimplemented)` |
| `jet-parity-adapter-vuetify` | `vuetify` | Vue 3 + Vuetify 3 | scaffold; both methods `Err(AdapterError::Unimplemented)` |

Scaffold adapters MUST return:

```rust
Err(AdapterError::Unimplemented {
    adapter_id: "material",          // or "vuetify"
    method: "mount",                 // or "capture_channel"
})
```

Panic-placeholder macros (`unimplemented!()`,
`unreachable!()` as a placeholder, `panic!("TODO")`,
`todo!()`) are forbidden in adapter trait method bodies.
Rationale: a panicking scaffold crashes the parity runner
mid-corpus and corrupts the divergence-triage signal that the
rest of this ADR depends on. A CI lint
(`scripts/parity/lint-no-panic-placeholders.sh`) greps the
three adapter crate source trees for the forbidden macros and
fails the build on a match.

### Fixture intent and per-stack fixtures

Each fixture under `projects/jet/data/parity/fixtures/<adapter>/<fixture>/`
declares the same `FixtureIntent` in stack-native syntax:

```
projects/jet/data/parity/fixtures/
├── mui/
│   ├── button-primary-v1/
│   │   ├── parity.toml         # fixture_intent = "../../intents/button-primary-v1.json"
│   │   ├── App.tsx             # React + MUI JSX
│   │   └── package.json        # pins react@18.2, @mui/material@5.x
│   ├── textfield-outlined-v1/
│   └── checkbox-basic-v1/
├── material/
│   ├── button-primary-v1/
│   │   ├── parity.toml         # same fixture_intent
│   │   ├── app.component.ts    # Angular + Material template
│   │   └── package.json        # pins @angular/core@17, @angular/material@17
│   ├── textfield-outlined-v1/
│   └── checkbox-basic-v1/
├── vuetify/
│   ├── button-primary-v1/
│   │   ├── parity.toml
│   │   ├── App.vue
│   │   └── package.json        # pins vue@3, vuetify@3
│   ├── textfield-outlined-v1/
│   └── checkbox-basic-v1/
└── intents/                    # the canonical FixtureIntent JSON files
    ├── button-primary-v1.json
    ├── textfield-outlined-v1.json
    └── checkbox-basic-v1.json
```

Each adapter's fixture brings its own `node_modules` lockfile
to its own sub-project. Bumping `material`'s `@angular/material`
pin triggers a `material`-only re-baseline (when live capture
lands); the `mui` and `vuetify` corpora are untouched. This
isolation is what makes the per-adapter version-bump cost
bounded.

### `parity-gating.toml [adapter]` selection

```toml
[adapter]
# Active adapter set. Order matters for divergence-triage tiebreaking.
active = ["mui"]                            # default — only primary
# active = ["mui", "material", "vuetify"]   # opt-in via CLI flag

[adapter.budget]
# Per-PR budget covers only "mui". "material" and "vuetify" charge
# against the opt-in bucket, which never blocks PR merge.
primary_seconds = 90
secondary_seconds = 600                     # nightly job ceiling
```

Selecting a scaffold-only adapter (`material` or `vuetify`)
for a *live capture* invocation exits with code 2 and prints to
stderr:

```json
{
  "kind": "adapter_stub",
  "adapter_id": "material",
  "missing_methods": ["mount", "capture_channel"]
}
```

This gives operators a clean, machine-readable failure when
they configure `active = ["material"]` before the live capture
implementation lands.

### Divergence-triage routing

When the active set has ≥ 2 adapters and at least one
disagrees on a `(fixture, channel)` pair, the parity report
tags every artifact with its `adapter_id` and emits a
divergence record:

| Pattern | Bucket | Routing |
|---------|--------|---------|
| Jet ≠ all references; all references agree | `jet_regression` | Block PR; file as a jet bug. |
| Jet = one reference; that reference ≠ the others | `cross_stack_waiver` | Don't block; record as a stack-specific quirk; surface in the cross-stack waiver dashboard. |
| All references disagree with each other | `wpt_authoritative` | Don't block; route to WPT (#2154) as the spec-level tiebreaker. |

Cross-adapter expectations are *not* byte-equal — Vuetify's
button paints with a different default elevation shadow than
MUI's. The tolerance widening per `(channel, tier)` pair for
adapter-to-adapter comparison is declared in
`projects/jet/data/parity/adapters.toml`. The primary oracle
(`mui`) is the canonical baseline; secondary adapters serve as
*divergence detectors*, not byte-equal reference targets.

### `--include-secondary-adapters` flag

```
score parity oracle button-primary-v1                              # mui only (default)
score parity oracle --include-secondary-adapters button-primary-v1 # mui + material + vuetify
```

Default-off behavior preserves the per-PR parity budget. The
flag is wired into CI's nightly job (not the per-PR job), so
secondary-adapter runs accumulate over time without blocking
the merge queue.

### CI strategy

- **Per-PR:** `mui` adapter only, full corpus, parity gate
  active. Budget: `primary_seconds = 90`.
- **Nightly:** `mui` + `material` + `vuetify` (once live
  capture lands), full corpus, results posted to the
  divergence-tracking dashboard. Budget:
  `secondary_seconds = 600`.
- **On-demand:** any developer can run
  `--include-secondary-adapters` locally to triangulate a
  divergence they're triaging.

### Byte-equivalence regression

R2's invariant — trait-routed `mui` produces byte-identical
output to the pre-refactor #2139 runner — is enforced by
`projects/jet/parity-oracle/tests/trait_refactor_byte_equiv.rs`:

```rust
#[test]
fn trait_routed_mui_byte_equiv_to_pre_refactor() {
    let pre = run_pre_refactor_mui_runner(CORPUS);
    let post = run_trait_routed_mui_adapter(CORPUS);
    assert_byte_equal(pre, post);
}
```

The pre-refactor runner is preserved as a frozen reference
implementation under
`projects/jet/parity-oracle/src/_legacy_pre_refactor.rs` and
deleted only after this test has been green for 30 days on
the main branch.

## Consequences

**Positive.**

- Divergence triage stops being O(engineer-hours) per
  divergence: the routing buckets mechanize the first cut.
- Adding a fourth stack (Svelte? SolidJS?) is a new crate
  implementing two methods plus a `node_modules` lockfile —
  no changes to the oracle, the comparator, or the CI gate.
- The byte-equivalence regression on `mui` makes the trait
  refactor safe to land: we can prove we didn't accidentally
  change the primary baseline while extracting the trait.
- The per-PR budget is unchanged. Secondary-adapter cost is
  scheduled, not paid on every commit.

**Negative.**

- Carrying three `node_modules` trees in `fixtures/`
  increases CI install time even when only `mui` runs (the
  installer doesn't know which adapters will be selected).
  Mitigated by per-adapter lockfile caching keyed on
  `package.json` hash.
- The `cross_stack_waiver` bucket can be abused as a
  catch-all for "this is too hard to fix" — needs human
  review on the dashboard, not auto-approval.
- Tolerance widening for adapter-to-adapter comparison
  (`adapters.toml`) is a new knob and a new source of
  bugs.

**Neutral.**

- Live capture implementations of `material` and `vuetify`
  are deferred to follow-up issues; this ADR ships the
  foundation only. Until those land, the divergence-triage
  routing has nothing to triangulate — but the trait, the
  flag, the budget accounting, and the CI lint are in place.

## Alternatives considered

1. **Single oracle, manual triage.** Keep `mui` as the only
   reference and accept that every divergence requires
   hand-reading MUI source. Rejected — the per-PR budget can't
   absorb O(engineer-hours) per divergence at corpus scale.
2. **WPT as the only reference.** Skip Angular and Vue, route
   every divergence to WPT directly. Rejected — WPT covers
   browser primitives, not component-library composition.
   "Does MUI's `<Button variant="contained">` paint the
   right shadow" is not a WPT question.
3. **Pixel-only secondary adapters (screenshot diffs).** Skip
   AX-tree, focus-trace, hit-test, IME — just compare PNGs
   across stacks. Rejected — pixel diffs are the noisiest
   channel and would generate `cross_stack_waiver` records
   on every fixture (different stacks paint at slightly
   different sub-pixel offsets). The triangulation signal
   needs the structured channels.
4. **Trait with `n` methods (one per channel).** `mount`,
   `capture_screenshot`, `capture_ax`, `capture_focus`,
   `capture_pointer`, `capture_ime`. Rejected — R1 pins the
   trait at two methods. The channel selection is a
   parameter to `capture_channel`, not a method-per-channel
   explosion, because adding a 6th channel (#2173, IME
   compositional events) should not break adapter contracts.
5. **Ship live capture for `material` in this issue.**
   Rejected — bringing up Angular 17 + Angular Material 17
   in-tree (toolchain, lockfile, Playwright integration, CDP
   binding) is its own multi-week project. Decoupling the
   trait/scaffold/routing from the per-adapter
   implementation lets us land the divergence-triage
   foundation now.

## Open questions

- **Vuetify's Material conformance.** Vuetify 3 claims
  Material 3 conformance, but its component-level
  interpretations diverge from MUI's in non-trivial ways
  (default elevations, ripple animation timing). The
  `adapters.toml` tolerance ladder for `(channel, tier)`
  cross-adapter widening is unstaffed; #2141's follow-ups
  will calibrate it empirically once live capture lands.
- **Angular Material 17 vs MUI v5 design-token drift.**
  Material 3 (Angular Material 17) and Material You (MUI v5
  still on Material Design 2 with partial M3) are
  *different design systems*. We may need a v6 MUI bump
  before the `material` adapter can run side-by-side
  meaningfully.
- **Fourth adapter governance.** When does adding a
  fifth/sixth/seventh stack become "too many"? No threshold
  is set in this ADR — we'll revisit when an adapter
  proposal lands.
- **Tolerance authority.** Who owns `adapters.toml`? Should
  changes to it require an ADR amendment, or just code
  review? Deferred.

## References

- #2141 — this issue (secondary reference adapters foundation)
- #2133 — parent epic (parity channel)
- #2138 — parent of #2141 (parity foundation roadmap)
- #2139 — ADR foundation: React+MUI DOM reference runner (refactored in R2)
- #2154 — WPT integration (`wpt_authoritative` bucket downstream)
- ADR-002 — per-renderer baselines (channel directory roots)
- ADR-030 — MUI corpus snapshot protocol (5-channel artifact bundle this trait emits)
- `.aw/tech-design/projects/jet/specs/jet-parity-secondary-adapters.md` — tech-design spec authorized by this ADR
- `.aw/tech-design/projects/jet/specs/jet-parity-dom-reference-runner.md` — #2139 runner spec (byte-equivalence anchor for R2)
- `projects/jet/data/parity/fixture-intent.schema.json` — canonical `FixtureIntent` JSON Schema
- `projects/jet/data/parity/adapters.toml` — cross-adapter tolerance widening per `(channel, tier)`
