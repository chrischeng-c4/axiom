# ownership-violation evidence — DDD contract

Scope: deterministic evidence for the missing-retain / double-release defect
family at the `MbObject` ownership boundary. This document owns the evidence
model; individual runtime defects remain in their own work items.

Tracked violation: #2585. The current tree has an armed release-path detector
and a positive control, but the four evidence capabilities below are not yet
closed as one reproducible system.

## Bounded context

The `OwnershipEvidence` context translates unsafe pointer ownership into four
machine-observable facts:

1. an invalid object is rejected at the first guarded retain/release boundary;
2. a deliberate invalid pointer proves that rejection remains armed;
3. teardown proves that a green run did not trade a double-release for a leak;
4. a source audit gives every take-ownership constructor site a stable identity.

It does not own the repair of any site. Consumers such as #2539 and #2604 use
these facts to attribute their own before/after result.

## Aggregate root

`OwnershipEvidenceRun` is the aggregate root for one immutable source revision,
build profile, detector mode, and named workload.

```text
Configured
  -> Armed
  -> Executing
  -> Observed
  -> Reconciled
```

`Observed` contains terminal detector events, live-object counters, and the
static site inventory. `Reconciled` is reachable only when every configured
instrument produced complete accounting; absence of output is not a zero.

### Entities and value objects

| Type | Kind | Identity / value |
|---|---|---|
| `OwnershipEvidenceRun` | aggregate root | source revision + profile + workload + detector mode |
| `DetectorEvent` | entity | monotonic event index within a run |
| `OwnershipSite` | entity | semantic path + enclosing symbol + constructor + contract class |
| `LeakBalanceSample` | entity | workload + lifecycle phase + sample index |
| `DetectorMode` | value | `Disarmed` or `Armed` |
| `OwnershipContract` | value | `ConsumesOwned`, `RetainsBorrowed`, or `MixedExplicitRetain` |
| `TerminalClass` | value | `Clean`, `DetectorAbort`, `LeakImbalance`, or `Incomplete` |
| `SiteInventoryDigest` | value | SHA-256 over the normalized sorted site rows |

Line numbers are presentation data, not `OwnershipSite` identity: unrelated
source movement must not manufacture a new site.

## Invariants

1. **One factor per comparison.** A before/after or armed/disarmed pair holds
   source revision, profile, workload, and composition fixed except for the
   declared factor.
2. **Fail closed.** A missing terminal event, truncated process, missing
   counter snapshot, or unparseable site row is `Incomplete`, never `Clean`.
3. **Positive control is independent.** The must-turn-red control uses
   alignment arithmetic, not an invalid Rust enum discriminant or a freed
   allocation whose bytes may be reused.
4. **Attribution precedes abort.** Every detector event identifies the current
   fixture/test and guarded operation before the process terminates.
5. **Detector state is monotonic per process.** An armed process cannot be
   disarmed by a later test; separate processes provide the disarmed control.
6. **Release behavior is unchanged.** Detector state, attribution, and
   counters compile out of non-debug release runtime code.
7. **Balanced does not mean reachable.** A leak balance compares an explicitly
   named workload with its clean control at equivalent lifecycle boundaries;
   GC-tracked count alone is not accepted as total live-object count.
8. **Audit output is deterministic.** The audit enumerates the complete
   constructor surface, classifies every row, sorts by semantic site identity,
   and emits both rows and count. Unclassified rows make the audit red.
9. **A source fix consumes evidence.** A child defect may claim attribution
   only when the detector control still trips, its named workload is balanced,
   and the expected site-count delta reconciles exactly.

## Domain services

### `ReleasePathDetector`

Owned seam: `src/runtime/rc.rs`.

- Reads the explicit process arming control once.
- Guards every selected retain/release dereference before reading object
  fields.
- Emits operation, pointer class, and current fixture/test identity.
- Provides a test-only force-arm function for the positive control.

### `LeakBalanceRecorder`

Owned seam: `src/runtime/rc.rs` plus the narrow runtime teardown seam required
to close one named reproducer.

- Counts heap-object allocation and final deallocation events using debug-only
  counters.
- Snapshots the counter at named lifecycle boundaries.
- Reports signed delta for a clean control and a reproducer.
- Does not change refcounts, retain objects, or call collection to make a
  number look balanced.

### `OwnershipSiteAudit`

Owned seam: one deterministic repository tool under
`apps/mamba/tools/ownership/`.

- Enumerates `new_list`, `new_set`, `new_tuple`, inline/untracked siblings, and
  their borrowed variants under `apps/mamba/src/runtime/`.
- Classifies the value origin feeding each constructor as owned, borrowed, or
  mixed.
- Follows local rebinding instead of matching only the identifier at the call.
- Emits stable rows and a count; the tool's own fixtures include a renamed
  collection and an unclassified counterexample.

The audit is a measurement instrument. It never edits `src/**`.

## Work-item decomposition

| Order | Capability | Exact writable implementation seams | Machine oracle |
|---:|---|---|---|
| 1 | A1 armed detector and attribution reconciliation | AGY: `apps/mamba/src/runtime/rc.rs` only; controller: any required existing harness attribution call | focused unit tests prove default disarmed, explicit armed, attributed abort, and release-profile compile-out |
| 2 | A2 optimization-invariant must-turn-red control | `apps/mamba/src/runtime/rc.rs` test module only | the named positive control passes at test opt-level 0 and 1 because the detector trips with the expected marker |
| 3 | A3 named-workload leak balance | `apps/mamba/src/runtime/rc.rs` plus one narrowly named source-local test module | clean control delta is zero; the reproducer reports zero after the correct ownership path and nonzero under a test-only deliberate leak |
| 4 | A4 deterministic ownership-site audit | no `src/**`; one tool subtree under `apps/mamba/tools/ownership/` plus tool-local fixtures | two runs are byte-identical; rows reconcile to count; an unseen fixture recomputes rather than using pinned totals |

A2 depends on A1. A3 depends on A1 because a green balance without a still-live
violation detector cannot distinguish evidence weakening. A4 is independent of
A2/A3 but must finish before a consuming source-fix ticket claims a site-count
delta.

All `apps/mamba/src/**` changes are AGY-owned. The controller owns this
design, tracker contracts, tools/tests outside `src/**`, independent
verification, commits, and closure.

## Forbidden fixes

- Adding an upstream retain solely to make the crashing suite green.
- Disarming or weakening the detector in the positive-control process.
- Treating a process signal, truncated log, or absent counter line as clean.
- Using a line-number-only inventory or a hard-coded expected count.
- Running collection or retaining leaked objects inside the measurement path.
- Changing the default release profile or selecting fewer tests to hide the
  defect.

## Verification surface

- Focused Rust tests in the owning source module for A1-A3.
- Opt-level 0 and 1 test-profile builds for the detector/control.
- `cargo test -p mamba --release --no-run` for compile-out.
- Tool-local Python tests and two byte-identical audit runs for A4.
- Consumer tickets retain their own profile/full-suite gates; this context
  supplies attribution, not product-parity closure.
