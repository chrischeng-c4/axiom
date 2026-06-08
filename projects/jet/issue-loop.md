---
project: jet
branch: project-jet
label: "project:jet"
repo: chrischeng-c4/cclab
pick_order: oldest-first
build: skip
verify:
  test: cargo test -p cclab-jet
  perf: required
done_gates:
  - test_passes
  - perf_verified
pr:
  base: main
  merge_strategy: squash
  rebase_after_merge: true
---

# Jet issue loop — per-issue rules

## Branch policy
- All work happens on `project-jet`. If current branch is anything else, merge it into `project-jet` first (don't escape to a feature branch — `project-jet` IS the working branch).
- One issue → one PR → squash-merge to `main` → `git pull --rebase origin main` back onto `project-jet`. Push.

## Build policy: skip
- **Do not run `cargo build` or any jet build skill (`jet-build-debug`, `jet-build-release`) per issue.**
- Why: user instruction — `cargo test -p cclab-jet` already compiles the test binary; a separate build is wasted time.
- How to apply: rely on `cargo test` for compile-gate. Only build if a test failure suggests a stale artifact AND `cargo clean -p cclab-jet && cargo test -p cclab-jet` doesn't reproduce.

## Definition of done (both must hold)
1. **Test gate** — `cargo test -p cclab-jet` (plus any sub-crate touched, e.g. `cclab-jet-runtime`, `cclab-jet-parity`) passes from a clean state.
2. **Perf gate** — if the issue touches a hot path (runtime, parity oracle, renderer), run the relevant bench/parity-gate and confirm no regression vs `project-jet` HEAD before the change. If the issue is pure refactor/docs/UI-state with no hot-path impact, document why perf check is N/A in the PR body — don't silently skip.

## What "perf verified" means for jet
- Runtime changes: parity-gate `cargo run -p cclab-jet-parity-gate -- <fixture>` or the runtime benches under `projects/jet/data/runtime/benches/`.
- Renderer / TUI changes: visual smoke (snapshot or `conductor-playwright-test` skill if e2e).
- Build/CLI changes: not applicable — note it in PR.

## PR body must include
- Issue link (`Closes #<n>`).
- Test gate output (the passing `cargo test` summary line).
- Perf gate output OR an explicit "perf N/A because …" line.

## Loop exit
- Stop when `gh issue list --label project:jet --state open` returns 0.
- If a single issue can't be closed (blocked by another issue, needs human decision, fails perf), comment on it explaining the block and move to the next. Don't loop on a stuck issue.
