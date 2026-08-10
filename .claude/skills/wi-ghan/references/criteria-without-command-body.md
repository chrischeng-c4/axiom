# aw: make change WIs own a revisioned WI-EC-TD-CB lifecycle

## Problem

A `change` WI should own its complete delivery lifecycle, but its issue contract, EC verifier, executable TD, and production CB currently lack one causal revision model. Tracker milestones, local phase state, implicit commits, and verifier results can disagree after restart or repair. The model also reads like a single pass even though CB discoveries return to TD, contract failures return to EC, and semantic WI edits must invalidate every downstream artifact.

AW needs one revision-aware ledger/reducer that starts by accepting the WI contract, advances through EC, TD, and CB with fixed command vocabularies, permits deterministic feedback and parent-only rebinds, and closes only through one current CB terminal commit. Every durable transition must be commit-bound, exact-scope, concurrency-safe across isolated worktrees, recoverable after partial failure, and observable identically through goal, WI show, and health.

### The defect this epic has, measured

R9-R12 name the vocabulary each verb converges *to*. None of them says when the vocabulary it converges *from* stops existing. The result is measurable on `2026-08-10`:

| Verb | leaves today | target | target leaves that exist | legacy leaves still live |
|---|---|---|---|---|
| `aw ec` | 7 | 5 | 2 (`review`, `verify`) | 5 |
| `aw td` | 8 | 5 | 1 (`reconcile`) | 7 |
| `aw cb` | 7 | 4 | 0 | 7 |
| **total** | **22** | **14** | **3** | **19** |

`aw wi` is the worked example of the failure. R12's four leaves — `change`, `test`, `review`, `commit` — have all landed, and `aw wi change --help` prints `Drive R3 of #3363`. The other 21 leaves are still there. The convergence added vocabulary and retired nothing, so the surface moved from 21 leaves to 25.

The 19 legacy `ec`/`td`/`cb` leaves are not free to delete: they carry **517 references in `apps/agentic-workflow/src/**/*.rs`, 60 in `*.toml`, and 411 in `*.md`**. `td check` alone is named in 25 `.toml` files, which are other projects' gate bindings. A convergence that lands its new leaves without a stated retirement order does not converge; it doubles the surface and leaves every downstream issue written against a leaf that may or may not survive.

## Capability Alignment

Capability: `work-item-planning`; `td-cb-lifecycle-automation`; `workflow-root-runner`; `project-local-td-and-ec-gates`

Capability Gap: change WIs do not yet provide a closed `WI -> EC -> TD -> CB` causal lifecycle with immutable revisions, one next command, complete repair loops, four exclusive commit boundaries, canonical tracker projection, independently verified terminal semantics, **and a surface that shrinks as the convergence lands**.

Progress Evidence: public-CLI fixtures complete the happy path plus WI amendment, EC repair with changed and unchanged TD source, CB-driven TD amendment, parent-only CB rebind, infrastructure blocking, concurrent commit races, partial projection recovery, and manual false-green attempts. Fresh goal/show/health processes agree after every event, and only one current CB commit reaches workflow completion. The leaf-count table above is recomputed at every child's close.

## Requirements

- R1: Define WI, EC, TD, and CB causal artifact revisions plus an append-only event ledger and pure deterministic reducer. Revision identity includes content and causal parents; every state has exactly one obligation/`next.command`, and unchanged content under a changed parent can materialize a rebind while a total no-op cannot create an event.
- R2: Project only committed recoverable lifecycle milestones into the tracker. Keep AW-owned phase/status/comments/receipts outside the canonical WI contract digest; use event/head compare-and-set and close the tracker only from the committed CB terminal event.
- R3: Verify TD with behavior/security and CB with behavior/security/efficiency/stability. Record exactly one failure owner: contract to EC, design to TD, implementation to CB, infrastructure to blocked same-verifier retry; invalid evidence fails closed.
- R4: Order post-CB TD reconciliation after CB test/review and before CB EC verification. `no_change` preserves TD and advances to `ec verify cb`; `amended` routes to TD change and invalidates affected CB evidence, while reusable source still requires causal rebind/fresh evidence.
- R5: Make `aw goal wi`, `aw wi show`, and `aw health` consume one reducer-derived read model and report identical WI/EC/TD/CB revisions, evidence, invalidations, blocker, iteration, event/projection state, next command, and terminal value across restart.
- R6: Migrate supported legacy phase/local-loop state without false green and independently prove happy, WI-repair, EC-repair, TD-amend, rebind, dimension/failure, race/recovery, manual-manipulation, and terminal paths through frozen Python external contracts.
- R7: Make `aw wi commit`, `aw ec commit`, `aw td commit`, and `aw cb commit` the exclusive lifecycle Git authority. Every other lifecycle leaf cannot inspect lifecycle Git history, stage lifecycle paths, create commits, project tracker state, or independently advance the ledger.
- R8: Bind each commit to an AW-owned prepared operation, project/worktree lease, expected ledger head, deterministic event ID, fixed declared paths/message/receipt, and ledger compare-and-set. Manual/forged Git history has no authority; identical races converge once, competing candidates have one winner, and post-commit failures recover through the same leaf/event/OID without duplicate durable effects.
- R9: Converge EC to `change`, `test`, `verify {td|cb}`, `review`, and `commit`, all WI-bound. EC owns a testable verifier project; self-tests do not prove target conformance; initial and repaired EC revisions follow the reducer and support WI-parent-only binding plus current-TD impact verification.
- R10: Converge TD to `change`, `test`, `review`, `reconcile`, and `commit`, all WI-bound. TD is an executable full-typed Python project; internal tests, EC behavior/security, independent review, parent-only rebind, no-change/amended reconciliation, and commit each retain distinct evidence semantics.
- R11: Converge CB to `change`, `test`, `review`, and `commit`, all WI-bound. Generation/fill/repair are internal change modes; the sole order is test, review, TD reconcile, four-dimensional EC verify, then CB commit, which is the only terminal event and tracker-close authority.
- R12: Converge the change-WI contract to `change`, `test`, `review`, and `commit` after issue creation. Canonicalize semantic issue fields separately from AW projection metadata; any semantic post-commit drift invalidates all downstream evidence, and manual CLI/tracker close cannot complete an active change.
- R13: **A convergence is two halves, and a child that lands only the first is not done.** The first half is that the target leaf exists and is the documented path. The second half is that the leaf it replaces has left the clap surface, and the evidence is that `aw <verb> --help` lists exactly the target set. Between the halves a leaf is *deprecated*: it still parses, it prints one line naming its replacement, and the chain validator refuses any new emit site for it. No child in R9-R12 closes while a leaf it replaces is still live.
- R14: **Retire in blast-radius order, cheapest first, and migrate the callers in the same change that removes the leaf.** The order is fixed by the Retirement Schedule below and is not re-argued per child. A leaf bound in another project's `aw.toml` cannot be removed until `aw conf` rewrites those bindings, because removing it turns a green project gate into a parse error in a repository this epic does not own.
- R15: **Measure this epic by the leaf table, not by child terminality.** Progress is `aw ec/td/cb` at 22 leaves converging to 14, recomputed by one command. A child that closes without moving the count has not moved the epic. Child terminality is not an acceptance signal for this epic: the label was demonstrated to accumulate work that belongs elsewhere.
## Scope

### In Scope

- WI/EC/TD/CB causal revisions, immutable events, deterministic forward/feedback/rebind reducer, transitive evidence invalidation, and one next command.
- Canonical WI snapshot, independent artifact reviews, distinct internal-test versus external-verification evidence, and fixed lifecycle surfaces.
- Tracker milestone/terminal projection with semantic digest isolation, compare-and-set, drift detection, and exact recovery.
- Four prepared-operation commit gates with fixed scope/message/receipt, worktree lease/CAS, manual-history rejection, parent-only rebind, and partial-failure recovery.
- Goal/show/health convergence, command registry migration, legacy-state migration, and independent Python E2E/negative coverage.
- The deprecation and removal of the 19 leaves in the Retirement Schedule, and the caller migration each removal requires.

### Out of Scope

- Replacing GitHub as the human tracker, forbidding repository owners from using Git outside AW, rewriting existing product TD/EC content, or publishing a tracker milestone for every authoring edit.
- Changing capability core/non-core classification (#2887), retiring `aw capability` (#3254), or redefining the Typer/uv language and lock foundation (#2926).
- Rebinding the broad existing EC inventories in #3302/#3305 before this lifecycle contract lands.
- The dispatch harness under `.claude/`, which is #3540.
- The emitted-command chain validator, which is #3541. A leaf this epic retires is one that epic validates; the retirement order is stated here, and #3541 consumes it.
- Retiring the 21 non-lifecycle `aw wi` leaves. R12 converges the change-WI *lifecycle* after issue creation; `aw wi show`, `list`, `create`, and the planning leaves are preserved by R5 and by `aw wi`'s own authoring surface.

## Acceptance Criteria

- AC1: Every active change has one durable causal WI/EC/TD/CB ledger head and exactly one next command; stale/conflicting/legacy state fails closed.
- AC2: Tracker projection names committed event/revision/evidence/next owner without altering WI contract identity; only CB terminal projection closes.
- AC3: TD executes exactly two and CB exactly four EC dimensions; every red/blocked verdict has one deterministic owner route.
- AC4: Reconciliation has one reducer position and one result; no-change creates no TD revision, while amended invalidates and rebinds/rebuilds affected CB targets.
- AC5: Goal, show, and health are observationally identical and read-only across every forward, feedback, blocked, drift, recovery, and terminal state.
- AC6: Legacy and full public-CLI external scenarios cannot reach false green from stale evidence, manual Git, manual close, partial dimensions, self-oracle, or incomplete projection.
- AC7: Only four commit leaves can reach lifecycle history/staging/commit and each uses the prepared current obligation with fixed scope/format.
- AC8: Same-event race/retry creates one commit; competing candidates have one CAS winner; every partial failure recovers exactly once by event/OID.
- AC9: EC exposes five leaves, keeps self-test separate from target proof, and supports WI/EC repair plus unchanged-TD causal rebind.
- AC10: TD exposes five leaves, follows test/EC-verify/review/commit, supports parent-only rebind, and preserves no-change as attestation only.
- AC11: CB exposes four leaves, follows test/review/reconcile/EC-verify/commit, and only its current commit produces terminal completion.
- AC12: WI exposes four lifecycle leaves after creation, hashes only canonical semantic contract data, invalidates all downstream state on semantic drift, and rejects manual close as completion.
  ```
  for v in ec td cb; do printf '%s %s\n' "$v" \
    "$(aw $v --help | sed -n '/^Commands:/,/^Options:/p' | grep -E '^  [a-z]' | awk '$1!="help"' | wc -l)"; done
  ```
  The epic is not terminal while this prints anything other than `ec 5`, `td 5`, `cb 4`.

## Reference Context

### Backlog reset, 2026-08-10

This epic carried 65 children. They were reset, not abandoned:

- **25 relabelled** to the epics that actually own them — 16 to #3540 (dispatch harness) and 9 to #3541 (chain validator). Neither is lifecycle work; both had been landing under this label because no other label existed.
- **6 closed as duplicate.** Each was a VOIDed dispatch re-filed under a new number instead of continuing the existing one. #3395's own title says so: *"Recover real linked-worktree fixture admission after void #3394"*.
- **26 closed as reset.** They were written against leaves the Retirement Schedule deletes, so their baselines were not merely stale — they were scheduled for deletion. They are re-derived against the surface that exists once each Tier lands.

Every child, open and closed, is recorded with its defect statement in the snapshot: https://github.com/chrischeng-c4/axiom/issues/3346#issuecomment-5237666061

Children are re-filed per Tier, at the point that Tier starts. The label is no longer a place to park work whose owner is undecided; per R15 it is not the progress metric either.

### Related Specs

| Path | Why |
|---|---|
| `apps/agentic-workflow/src/cli/issues.rs` | WI authoring, canonicalization, tracker projection, and close boundary. |
| `apps/agentic-workflow/src/cli/loop_state.rs` | Existing local lifecycle state to supersede/migrate. |
| `apps/agentic-workflow/src/cli/ec.rs` | EC project, review, target verification, and commit inputs. |
| `apps/agentic-workflow/src/cli/td.rs` | Executable TD, reconciliation, implicit commits, and history probes. |
| `apps/agentic-workflow/src/cli/cb.rs` | Production candidate, generation/fill/check, commits, and terminal handling. |
| `apps/agentic-workflow/src/cli/goal.rs` | Canonical root invoke/completion envelope. |
| `apps/agentic-workflow/src/cli/health.rs` | Read-only readiness projection. |
| `apps/agentic-workflow/src/cli/conf.rs` | Rewrites the `.toml` gate bindings Tier C removal depends on. |
| `apps/agentic-workflow/external-contracts/` | Independent frozen Python black-box lifecycle cases. |
| `#2926` | Python/uv foundation consumed at child boundaries. |
| `#3302`, `#3305` | Broad EC inventories to bind after the new lifecycle settles. |
| `#3540`, `#3541` | Sibling epics; neither owns lifecycle vocabulary. |

### Canonical lifecycle

```text
wi change -> wi test -> wi review -> wi commit
  -> ec change -> ec test -> ec review -> ec commit
  -> td change -> td test -> ec verify td -> td review -> td commit
  -> cb change -> cb test -> cb review -> td reconcile
       amended  -> td change
       no_change -> ec verify cb
         contract       -> ec change
         design         -> td change
         implementation -> cb change
         infrastructure -> blocked ec verify cb
         green          -> cb commit -> terminal/close
```

A semantic WI edit returns to WI change and invalidates EC/TD/CB. An EC edit returns through current-TD impact verification; unchanged downstream source advances only through a material causal-parent rebind and fresh required evidence.

### Spec Plan

| Spec ID | Action | Main Spec Ref |
|---|---|---|
| `revisioned-change-wi-ledger` | create | WI/EC/TD/CB causal revisions, immutable events, reducer, invalidation, and hydration. |
| `change-wi-milestone-projection` | create | Canonical/projection separation, recoverable milestones, and terminal close. |
| `ec-target-failure-routing` | create | Two/four-dimension profiles and closed owner taxonomy. |
| `td-post-cb-reconciliation` | create | Ordered no-change/amended and affected-target rebind. |
| `artifact-commit-authority` | create | Four prepared commit gates, fixed receipts, lease/CAS, and exact recovery. |
| `wi-lifecycle-surface` | create | Four-leaf canonical WI contract acceptance and close protection. |
| `ec-lifecycle-surface` | create | Five-leaf EC project, review, commit, and target verification. |
| `td-lifecycle-surface` | create | Five-leaf executable TD, rebind, and reconcile surface. |
| `cb-lifecycle-surface` | create | Four-leaf production and terminal surface. |
| `lifecycle-leaf-retirement` | create | Deprecation state, tiered removal order, caller migration, and the leaf-count gate. |
| `revisioned-lifecycle-observability` | create | Shared goal/show/health read model. |
| `revisioned-lifecycle-e2e` | create | Migration, all feedback/rebind/race/recovery/negative scenarios. |

## Retirement Schedule

Reference counts measured `2026-08-10` on `app/aw`: `rust` is `grep -rF "aw <leaf>" apps/agentic-workflow/src --include='*.rs'`, `toml` and `docs` are the same literal repo-wide excluding `target/` and `.aw-wi/`.

### Tier A — no config binding; retire first, and prove the deprecation mechanism on them

| Leaf | rust | toml | docs | Replaced by |
|---|---|---|---|---|
| `ec record` | 2 | 2 | 0 | `ec commit` receipt |
| `ec doc` | 6 | 0 | 2 | `ec change` output |
| `td ast` | 2 | 0 | 9 | internal to `td test` |
| `td claim` | 4 | 0 | 4 | ledger lease (R8) |
| `td migrate-mermaid` | 9 | 0 | 11 | one-shot migration; delete outright |
| `cb materialize` | 2 | 1 | 0 | `cb change` mode |
| `cb publish` | 3 | 1 | 0 | `cb commit` |
| `cb promote` | 4 | 0 | 3 | `cb commit` |

Tier A is 32 Rust references total. It is the cheapest possible proof that R13's two halves work, and it must land before any Tier B leaf is deprecated.

### Tier B — documented but not bound in project config

| Leaf | rust | toml | docs | Replaced by |
|---|---|---|---|---|
| `ec draft` | 7 | 4 | 5 | `ec change` |
| `ec lock` | 12 | 2 | 9 | `ec commit` |
| `td lock` | 6 | 1 | 10 | `td commit` |
| `td audit-record` | 13 | 2 | 14 | `td commit` receipt |
| `cb gen-source` | 8 | 2 | 11 | `cb change` mode |
| `cb fill` | 37 | 1 | 17 | `cb change` mode |

### Tier C — load-bearing; other repositories' gates name these

| Leaf | rust | toml | docs | Replaced by |
|---|---|---|---|---|
| `ec check` | 38 | 8 | 31 | `ec test` |
| `td create` | 115 | 5 | 88 | `td change` |
| `td check` | 69 | 25 | 97 | `td test` |
| `cb gen` | 74 | 6 | 59 | `cb change` mode |
| `cb check` | 106 | 0 | 41 | `cb test` |

Tier C removal is blocked on `aw conf` rewriting the 44 `.toml` bindings. Until that lands these five stay live, and a fix to any of them is maintenance of a leaf with a scheduled end, not work against this epic.

## Child Work Items

Reset `2026-08-10`; see the Backlog reset section. Children are re-filed per Tier, at the point that Tier starts, against the surface that exists then. The dependency order below is retained because it is the ordering constraint, not a claim that these issues exist.

| Role | Depends On |
|---|---|
| R1: causal ledger and deterministic reducer | - |
| R2: canonical-safe tracker projection | R1 |
| R3: EC dimensions and failure owner routing | R1 |
| R4: ordered post-CB TD reconciliation | R1, R3 |
| R7-R8: four commit authorities, lease/CAS, receipts, recovery | R1, R2 |
| R13: deprecation state and the `--help` leaf gate | R7-R8 |
| R14 Tier A: 8 leaves, no config binding | R13 |
| R12: WI contract lifecycle surface | R1, R2, R7-R8 |
| R9: EC lifecycle surface | R3, R7-R8, R12 |
| R10: TD lifecycle surface | R4, R7-R8, R9 |
| R11: CB lifecycle and terminal surface | R3, R4, R7-R10 |
| R14 Tier B: 6 leaves, docs-only callers | R9, R10, R11, Tier A |
| R14 Tier C: 5 leaves, plus `aw conf` binding rewrite | Tier B |
| R5: goal/show/health convergence | R1-R4, R7-R12 |
| R6: migration and complete E2E matrix | R1-R5 |
| R15: leaf-count gate wired as the epic's acceptance signal | R13 |

## Verification Inventory

| Requirement | Gate | Oracle | Depends On |
|---|---|---|---|
| R1 | `cargo test -p agentic-workflow --lib revisioned_change_wi_ledger` and `revisioned_change_wi_reducer` | Causal revisions, events, invalidation, rebind, blocked, repair, terminal, and hydration fixtures always yield one next command. | - |
| R2 | `cargo test -p agentic-workflow --lib revisioned_change_wi_milestone_projection` | Only committed events project; AW status preserves contract digest; conflict/retry/close is event-idempotent. | R1 |
| R3 | `cargo test -p agentic-workflow --lib ec_target_verification_profiles` and `revisioned_change_wi_verdict_routing` | TD runs two, CB four dimensions, and every failure has one owner/block route. | R1 |
| R4 | `cargo test -p agentic-workflow --lib revisioned_change_wi_td_reconciliation` | Reconcile is ordered, no-change preserves TD, and amended invalidates/rebinds affected CB. | R1, R3 |
| R5 | `uv run --frozen --offline --project apps/agentic-workflow/external-contracts python apps/agentic-workflow/external-contracts/src/runner.py --case revisioned-change-wi-lifecycle-observability` | Fresh goal/show/health processes agree after every event, drift, block, recovery, and terminal state. | R1-R4, R7-R12 |
| R6 | External cases `revisioned-change-wi-lifecycle-migration`, `-feedback`, `-verdict-routing`, `-race-recovery`, and `-terminal-authority` | Legacy plus all forward/repair/rebind/race/manual-negative paths converge or fail closed independently. | R1-R5, R7-R12 |
| R7 | `cargo test -p agentic-workflow --lib lifecycle_commit_boundary` | Only WI/EC/TD/CB commit leaves reach lifecycle Git operations. | R1-R4 |
| R8 | `cargo test -p agentic-workflow --lib lifecycle_commit_concurrency` and `lifecycle_commit_recovery` | Prepared operation, scope, CAS, race winner, manual rejection, and exact recovery are enforced. | R1, R2, R7 |
| R9 | `cargo test -p agentic-workflow --lib ec_lifecycle_surface` | EC exposes five leaves, separates self-test/target proof, and completes initial/repair/rebind sequences. | R1, R3, R7, R8, R12 |
| R10 | `cargo test -p agentic-workflow --lib td_lifecycle_surface` | TD exposes five leaves, follows fixed evidence order, and distinguishes parent rebind from reconcile no-change. | R1, R4, R7-R9 |
| R11 | `cargo test -p agentic-workflow --lib cb_lifecycle_surface` | CB exposes four leaves, follows reconcile then four-dimension verify, and solely commits terminal. | R1-R4, R7-R10 |
| R12 | `cargo test -p agentic-workflow --lib wi_contract_lifecycle_surface` | WI exposes four leaves, hashes only canonical semantics, invalidates downstream drift, and rejects manual close. | R1, R2, R7, R8 |
| R13 | `cargo test -p agentic-workflow --lib lifecycle_leaf_deprecation` | A deprecated leaf parses, names its replacement on stderr, and is refused as a new emit site; a removed leaf is absent from `--help` and from clap. | R7, R8 |
| R14 | External case `lifecycle-leaf-retirement-tiers` | Each Tier's removal lands with its caller migration; a `.toml` gate green before a Tier C removal is green after it. | R13, R9-R11 |
| R15 | `aw ec/td/cb --help` leaf count prints `ec 5`, `td 5`, `cb 4` | The epic's own terminal condition, recomputable in one command and unfakeable: an unretired leaf is visibly unretired. | R13, R14 |
