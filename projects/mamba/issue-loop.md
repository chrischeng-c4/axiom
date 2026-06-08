---
project: mamba
branch: project-mamba
label: "project:mamba"
repo: chrischeng-c4/cclab
pick_order: oldest-first
build:
  debug: as-needed
  release: perf-only
verify:
  test: cargo test -p cclab-mamba
  perf: required
  perf_baseline: cpython
  perf_threshold: 1.0x
done_gates:
  - test_passes
  - axis1_runner_green
  - perf_ge_cpython
  - realworld_or_typeshed_verified
pr:
  base: main
  merge_strategy: squash
  rebase_after_merge: true
mode: autopilot
---

# Mamba issue loop — per-issue rules

## Branch policy
- All work happens on `project-mamba`. If current branch is anything else, merge into `project-mamba` first.
- One issue → one PR → squash-merge to `main` → rebase `project-mamba` onto `main`. Push.

## Autopilot mode (one issue = one tick)
- Backlog is large (~190 open). Run as Full Autopilot: each iteration takes one issue end-to-end (CRRR + handwrite + `cargo test` + fast-forward merge). No batching, no per-lib CRRR ceremony for conformance-only work.
- Mainthread acts as PM + QA: pick, validate gates, ship. Heavy code/test/spec authorship should go to a subagent (see "When to dispatch a subagent" below). Inline trivial one-liners are fine.

## Build policy
- **Debug**: build only when a runtime change requires it (`cargo build -p cclab-mamba` or `mamba-build-debug` skill). Don't build per issue if tests cover the change.
- **Release**: only required when running perf benches that need an installed `mamba` binary on PATH. Use `mamba-build-release` skill (it bumps patch, builds, installs, tags). Don't release-build for every issue — batch perf runs where possible.

## Definition of done — four gates (all must hold)
1. **Test gate** — `cargo test -p cclab-mamba` passes. For stdlib lib issues, also run the lib's behavior test fixture under `projects/mamba/tests/` or `benches/`.
2. **Axis-1 gate** — `cargo test -p mamba --test cpython_lib_test_runner --release` passes. **Required on every PR**, including ones that don't touch the runner. If a PR's changes flip a previously-green axis-1 seed to red, the PR cannot merge; the fix lands in the same PR. This enforces epic #3331 discipline rules 3 + 4 (no xfail/Stub/ImportPass; growing-only allowlist) — without it, future iterations can silently regress axis 1.
3. **Perf gate** — measured perf ≥ 1.0× CPython wall-time for the relevant workload. **Mamba's primary product is perf+memory > CPython; conformance is secondary.** If the lib can't beat CPython on any reasonable workload, comment on the issue with the numbers and close as `wontfix` or defer to a runtime/JIT followup — don't ship a slower mamba.
4. **Real-world verification** — typeshed type check passes OR a real-world fixture (one of the `ecosystem_fixture_manifest.toml` entries, or a small idiomatic snippet) runs and produces the expected output.

## Known pre-existing blockers (don't re-debug)
- Shared-state SIGABRT and tuple-return-unpack UAF block clean conformance runs. If an issue fails ONLY because of these, comment with the blocker reference and move on — don't try to fix them inline.
- See memory `project_mamba_conformance_blockers.md` for context.

## Known runtime quirks affecting bench authoring
- `import a, b` only binds `a` — use separate `import` lines.
- `import X.Y` then `X.Y.fn` resolves to None — use `from X.Y import fn`.
- `+=` accumulated int may fail `==` vs `*`-built numerically-equal int — use subtraction for equality.
- JIT may silently drop if/else branches AFTER a stdlib module call — put asserts BEFORE such calls when feasible.
- Integer-handle pattern: `+ * - /` lowered to native i64, bypass class.rs dunder dispatch — OOP arith libs (Fraction, Decimal) must expose module-level fns.
- Post-#2100 GC threshold (10k): cap bench iteration size to ≤30k allocations/iter to avoid 0% CPU wedge.

## Stdlib shim authoring conventions
- Dispatcher fn names MUST start with `dispatch_` prefix (matched by `surface.rs::pick_tuple_dispatcher`).
- Integer-handle pattern: thread_local handle table + `class.rs` predicate branches. Use for HMAC, struct.Struct, queue.Queue, io.BytesIO, re.Pattern, etc.
- Handwritten code is acceptable as TEMP state to fix a gap; an issue is "done" only when the code is converted to codegen-driven (regenerates byte-equivalent). For backlog autopilot, ship handwritten + file a followup tracker — don't block the loop on the codegen conversion.

## When to dispatch a subagent
- **Yes (subagent)**: stdlib shim authoring (one full lib), bench writing, multi-file refactors, anything that would exceed ~10 tool calls of focused dev work.
- **No (mainthread inline)**: triage, label/comment management, single-file 1-3 line fixes, perf number collection, gate validation, PR/merge/rebase orchestration.
- Score CRRR (`aw td --apply` / `aw cb --apply`) must NOT be delegated to a subagent — PostToolUse hook lockfile chain doesn't reach the subagent's Bash. If a mamba issue requires the score lifecycle (rare for conformance work), run it on mainthread.

## PR body must include
- Issue link (`Closes #<n>`).
- Test gate output (passing summary).
- Perf numbers: `mamba <T_m> vs cpython <T_c>, ratio <T_c/T_m>×` for the workload tested. Include the bench script or one-liner.
- Real-world / typeshed verification result.
- If shipped as handwritten temp, link the codegen-conversion followup issue.

## Loop exit
- Stop when `gh issue list --label project:mamba --state open` returns 0.
- Defer issues blocked by the pre-existing SIGABRT/UAF runtime bugs to a tracker; don't loop on them.
- Defer issues whose lib cannot beat CPython on any reasonable workload (close as `wontfix` with numbers, or leave for a runtime-JIT improvement to revisit).
