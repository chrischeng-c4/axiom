# Braid Held-WIP Landing Plan

Issue: #1102

This playbook is for landing verified fixes that currently ride on top of
deliberately held user WIP. The landing must preserve the held WIP, stage only
the intended braid, and prove the committed state instead of accidentally
passing on dirty-worktree state.

## HOLD Invariants

- HOLD: never `git add -A`.
- Do not clean up, revert, or restack unrelated user-owned WIP.
- Preserve the user's dirty state exactly as found outside the bounded landing
  hunks.
- If a required hunk overlaps unrelated dirty edits and cannot be staged
  surgically, stop and ask the main thread to resolve the overlap instead of
  widening scope.
- Do not strand a dependency where a committed file references an uncommitted symbol, vendored module, helper, or `stdlib_sigs` wall.

## Preflight Inventory

Run these before any staging so the landing braid is explicit:

```bash
git status --short
git diff --name-only
git diff --stat
git diff -- projects/mamba/src/lower
git diff -- projects/mamba/src/runtime/stdlib
git diff -- projects/mamba/src/types/stdlib_sigs.rs
git diff -- projects/mamba/src/types/stdlib_sigs_generated.rs
rg -n "#897|#953|#976|#977|#1014|#1015" projects/mamba/src projects/mamba/tests
```

Main thread evidence capture for already-closed fixes from 2026-07-04 and
2026-07-05:

```bash
gh issue view 897 --comments
gh issue view 953 --comments
gh issue view 976 --comments
gh issue view 977 --comments
gh issue view 1014 --comments
gh issue view 1015 --comments
```

Use those close comments to confirm the exact files and regions that must land
together.

## Lockstep Landing Checklist

The following items are one braid and must land in the same commit:

- `lower:: walrus=WIP-owned`
- `lower:: mutated-defaults=#897`
- `lower:: raw-int-param-ordering=#953`
- Vendored flips tied to their lowering dependencies:
  `VENDORED_MODULES` + no-op registers + `py_src` files + curated
  `stdlib_sigs` walls
- Dependency issue family that must stay coherent:
  `#953/#976/#977/#1014/#1015`

Do not split the vendored flips away from the lowering fixes they depend on. A
standalone vendored commit can regress the committed tree even if the dirty
worktree is green.

## Surgical Staging Recipe

Stage only the braid hunks. Do not execute broad staging commands.

```bash
git diff > /tmp/1102-full.diff
git diff -- projects/mamba/src/lower > /tmp/1102-lower.diff
git diff -- projects/mamba/src/runtime/stdlib/vendor_lib.rs > /tmp/1102-vendor.diff
git diff -- projects/mamba/src/runtime/stdlib/py_src > /tmp/1102-py-src.diff
git diff -- projects/mamba/src/types/stdlib_sigs.rs > /tmp/1102-stdlib-sigs.diff
git diff -- projects/mamba/src/types/stdlib_sigs_generated.rs > /tmp/1102-stdlib-sigs-generated.diff
```

Build the staged patch by copying only the required `@@` hunks into a filtered
patch, then stage with:

```bash
git apply --cached /tmp/1102-filtered.patch
git diff --cached --stat
git diff --cached
```

If the filtered patch stages a file that references an unstaged symbol, helper,
vendored module, or signature wall, unstage it and rebuild the filtered patch
before proceeding.

## Committed-State Verification

Prove the committed state, not the dirty worktree.

Preferred path after creating the landing commit:

```bash
git rev-parse HEAD
git worktree add /tmp/mamba-1102-verify <commit-sha>
cd /tmp/mamba-1102-verify/projects/mamba
cargo build -p mamba
cargo test -p mamba lower:: -- --nocapture
```

If a fresh worktree is temporarily unavailable, use an index-only or
commit-specific proof path that still avoids pulling in unrelated dirty files.
The key rule is that verification must target the staged-or-committed braid
state, not the original dirty checkout.

## Post-Landing Gates

Run the post-landing checks on the committed state:

```bash
cargo build -p mamba
cargo test -p mamba lower:: -- --nocapture
```

Focused lockstep proof:

- The 3 known-red `lower::` tests are green.
- No committed file references an uncommitted symbol.

Conformance follow-up at baseline:

```bash
python3 projects/mamba/tests/harness/cpython/tools/replacement_readiness.py --help
python3 projects/mamba/tests/harness/cpython/tools/platform_readiness.py --help
```

Run the normal conformance baseline or sweep command used by the main thread
after landing; do not claim completion until the post-landing sweep is back at
baseline.

Commit messages must reference the landing issue and the dependency fixes they
carry, for example:

```text
mamba: land held-WIP braid for lowering and vendored stdlib fixes (#1102, #897, #953, #976, #977, #1014, #1015)
```

## Abort / Rollback

If the braid cannot be staged or validated cleanly, abort without disturbing the
held WIP:

```bash
git diff --cached
git restore --staged <path>
git status --short
```

- Only unstage the paths involved in the attempted braid.
- Do not run cleanup or revert commands against unrelated dirty files.
- Leave the worktree exactly as a held-WIP checkout, then hand control back to
  the main thread with the overlap or dependency called out explicitly.

## Acceptance Criteria

- AC-1: committed state builds clean in a fresh worktree.
- AC-2: baseline post-landing conformance sweep is back to expected state.
- AC-3: the 3 lockstep `lower::` tests are green together.
