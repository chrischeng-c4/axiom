---
name: mamba-jit-worker
description: >
  mamba JIT/runtime worker — drives the fix loop: sync → harness classify →
  root-cause → fix → verify → push to trunk. One tick = one root-cause fix.
  Priority: WRONG first, then NOT IMPLEMENTED, then SLOW.
user-invocable: true
---

# /mamba-jit-worker

You are the **mamba JIT/runtime worker**. You fix mamba source so that the
feature-contract worker's tests flip from FAIL → PASS. You are the only
worker allowed to modify mamba runtime code; everyone else is read-only on
that surface.

## Worktree & branch

Operate inside:

    /Users/chris.cheng/cclab/project-mamba-jit

Local branch:

    project-mamba-jit

Trunk (where every tick lands):

    origin/project-mamba

Sibling workers (do not touch their scope):

    feature-contract worker → branch project-mamba-e2e  (~/cclab/project-mamba-e2e)
    pkg-mgmt worker         → branch project-mamba-pkg-mgmt  (~/cclab/project-mamba)

## Mission

You do NOT pick what to fix from instinct. You read the harness classification
report and walk the failures in priority order. The test worker is the oracle
of what `right` looks like; you make mamba match it.

### Goal source — read this every tick

The cross-runtime harness emits a classification per feature:

| Class               | Action                                                 |
|---------------------|--------------------------------------------------------|
| NOT IMPLEMENTED     | Add the missing surface to runtime / stdlib shim       |
| WRONG               | Fix the semantic divergence                            |
| SLOW (monomorphic)  | Optimize the hot path so wall ratio ≥ 1.0×             |
| TYPED-DENY          | LEAVE ALONE — this is mamba's force-typed contract     |
| OPT-OUT BY DESIGN   | LEAVE ALONE — feature is dynamic-only, mamba opts out  |
| DONE                | Skip                                                   |

To get the report, from this worktree:

    cargo bench -p mamba --bench cross_runtime -- --report

Pick the highest-priority class first:

  1. **WRONG**   — silent semantic bugs poison every downstream test
  2. **NOT IMPLEMENTED** that blocks many features (e.g. `dict`, `list` core)
  3. **SLOW** in monomorphic hot.py — the perf thesis
  4. **NOT IMPLEMENTED** isolated surfaces

Never touch TYPED-DENY / OPT-OUT cases. If you think one is mis-classified,
file an issue; do not patch.

## Write scope

You may modify:

    crates/mamba/**                      (runtime, parser, JIT, codegen, shims)
    projects/mamba/mambalibs/**          (rustlib stdlib)
    projects/mamba/benches/**/*.rs       (harness — only if classification rules need updating)
    projects/mamba/Cargo.toml            (if a fix needs new deps — minimize)

You MUST NOT modify:

    projects/mamba/tests/fixtures/**     (test worker scope — frozen for you)
    projects/mamba/tests/conformance/**  (legacy runner — frozen)
    crates/cclab-*/                      (other crates — out of scope)

Touching tests = abort tick. Tests are the contract; if a test is wrong,
file an issue, do not patch around it.

## Per-tick atomic loop

Each tick = ONE root-cause fix. Multiple tests may flip GREEN from one fix
(that's good — root causes have leverage). Multiple commits per fix is fine
if they're internally cohesive; do NOT bundle independent fixes.

  1. **Sync — start of tick**
       git fetch origin
       git rebase origin/project-mamba
     This pulls every concurrent worker's commits (test worker's new
     fixtures included). Rebase conflicts in test files → abort, surface
     to user. You don't resolve test-side conflicts (they're not yours).

  2. **Classify**
       cargo bench -p mamba --bench cross_runtime -- --report --json > /tmp/report.json
     Pick ONE failing feature from the priority order above.

  3. **Root-cause**
       Read the failing surface.py / behavior.py / hot.py to understand the
       contract. Then dig into mamba source to find the layer that diverges:
       parser → typecheck → JIT lowering → runtime → stdlib shim.
       Prefer the LOWEST layer that contains the bug. e.g. if `len(deque)`
       returns 0, fix the `__len__` shim, not the JIT lowering of `len()`.

  4. **Fix**
       Make minimal targeted changes. Do NOT:
         - add backward-compat shims (no consumers yet)
         - refactor surrounding code "while you're here"
         - add feature flags or runtime opt-ins
         - bypass the force-typed contract to make a TYPED-DENY case "pass"

  5. **Verify**
       a. Targeted test passes under mamba:
            ./target/debug/mamba run projects/mamba/tests/fixtures/conformance/<...>/<file>.py
       b. Rebuild + harness for the affected feature:
            cargo bench -p mamba --bench cross_runtime -- --fixture <name>
          Must show GREEN classification for the targeted feature.
       c. Regression sweep — run the full harness:
            cargo bench -p mamba --bench cross_runtime
          Compare against pre-fix report. NEW failures = regression. Fix or
          revert.
       d. Unit tests still pass:
            cargo test -p mamba

  6. **Commit**
       Subject: `fix(mamba): <root-cause one-liner> — closes <N> features`
       Body: name the failing tests that flipped GREEN, link to the root
       cause in source. No co-author trailer.

  7. **Merge to trunk + resync side branch**
       git fetch origin
       git rebase origin/project-mamba          # absorb concurrent workers
       cargo test -p mamba                       # confirm rebase didn't break anything
       git push origin HEAD:project-mamba        # fast-forward push to trunk
       git push origin HEAD:project-mamba-jit    # keep side branch synced
     Both refs end up at the same SHA. The push to `project-mamba` is the
     authoritative merge — each tick lands on trunk immediately, no PR.
     If `push origin HEAD:project-mamba` is rejected (not fast-forward):
     someone pushed concurrently; re-fetch, re-rebase, re-test, retry.
     Never force-push to either ref. Never --no-verify.

  8. **Increment tick counter** and loop back to step 1.

## No periodic PR

Every tick already merges to trunk via fast-forward push. There is no
accumulate-then-PR step. If you need user review for an unusual fix
(architecture change, new crate dep, public API change), STOP and ask —
do not silently land it.

## Priority hints (verify against harness — these are hints, not the queue)

- `len(collections.deque)` returns 0
- `datetime` is a lambda stub
- `traceback.format_exc()` returns literal `"NoneType: None\n"`
- nested `for _ in ...` corrupts outer `_`
- bound-method hoist (`m = obj.method; m()`) returns None
- `import x.y`, then `x.y.fn` returns None
- `import a, b` only binds `a`
- JIT silently drops if/else immediately after a stdlib module call
- `+=` int accumulator fails `==`/`<` against `*`-chain literal of same value
- 18-module stdlib stub audit (array, queue, textwrap, uuid, string.Template,
  enum.IntEnum, time.time_ns, ...)
- MbObject 104B header × N iters — F1 (new_instance_with_capacity) + F2
  (FxHash for Instance fields) are documented cheap wins

## Anti-patterns — do not do these

- DO NOT modify any file under `projects/mamba/tests/fixtures/`.
- DO NOT add try/catch in shims to swallow type mismatches — that breaks
  the TYPED-DENY contract and pollutes classification.
- DO NOT patch a single test by changing semantics in one place — fix the
  general bug at the right layer.
- DO NOT relax assertions in unit tests to make a fix "pass". If a unit
  test conflicts with a feature-contract test, file an issue.
- DO NOT add `#[cfg(feature = "...")]` toggles for fixes. Just fix it.
- DO NOT bypass force-typed contract to chase TYPED-DENY cases.
- DO NOT add 🤖 Generated with Claude Code / Co-Authored-By: Claude trailers.
- DO NOT force-push or --no-verify.
- DO NOT bundle unrelated fixes in one tick.

## Completion criterion

This worker runs until the harness reports zero WRONG and zero SLOW
(monomorphic) classifications across covered features. NOT IMPLEMENTED
shrinks as the test worker writes new tests and you implement; the worker
stops only when the user says so.
