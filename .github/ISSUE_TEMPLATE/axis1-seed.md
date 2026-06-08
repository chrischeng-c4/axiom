---
name: Axis 1 — executable assertion seed
about: Add or promote a Py3.12 executable seed to AssertionPass
title: "test(mamba): axis1 <category> <seed-stem> AssertionPass seed"
labels: ["project:mamba", "axis:1", "type:test"]
---

Epic: #3331

## Seed location
`projects/mamba/tests/fixtures/cpython_lib_test/<seed|3p|realworld>/<stem>.py`

## Surface (what the seed asserts)
- [ ] <feature 1 / API 1>
- [ ] <feature 2 / API 2>
- [ ] ... (at least 5 real assertions)

## Acceptance
- [ ] Seed authored, real assertions, no stubs
- [ ] Emits `MAMBA_ASSERTION_PASS: <stem> <n> asserts` on exit 0
- [ ] Pinned at `outcome = "AssertionPass"` in `cpython_lib_test_baseline.toml`
- [ ] Pinned at `minimum_outcome = "AssertionPass"` in `cpython_lib_test_allowlist.toml`
- [ ] `cargo test -p mamba --test cpython_lib_test_runner --release` green
- [ ] If runtime gap blocks AssertionPass: separate runtime-gap issue filed + linked; THIS issue stays open

## Discipline (non-negotiable, see #3331)
- NO `xfail` / `Stub` / `ImportPass`. Only `AssertionPass` counts.
- NO TOML-schema theater. The Rust test that walks this seed is `cpython_lib_test_runner.rs` — it executes `mamba run`.
- Real-world preferred. Distilled minimal repros of bugs outrank synthetic features.
