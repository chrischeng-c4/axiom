---
name: Axis 1 — executable assertion seed
about: Add or promote a Py3.12 executable seed to AssertionPass
title: "test(mamba): axis1 <category> <seed-stem> AssertionPass seed"
labels: ["project:mamba", "axis:1", "type:test"]
---

Epic: #3331

## Seed location
Manifest case under `projects/mamba/tests/harness/cpython/config/manifests/<bucket>/<lib>.toml`;
generated fixture under `projects/mamba/tests/cpython/<dimension>/<bucket>/<lib>/<case>.py`.

## Surface (what the seed asserts)
- [ ] <feature 1 / API 1>
- [ ] <feature 2 / API 2>
- [ ] ... (at least 5 real assertions)

## Acceptance
- [ ] Manifest case authored and generated fixture filled with real assertions, no stubs
- [ ] Emits `MAMBA_ASSERTION_PASS: <stem> <n> asserts` on exit 0
- [ ] `[tool.mamba]` record is schema-clean under the current CPython harness
- [ ] `cargo test -p mamba --test conformance_cpython_lib_test` green under the debug build
- [ ] If runtime gap blocks AssertionPass: separate runtime-gap issue filed + linked; THIS issue stays open

## Discipline (non-negotiable, see #3331)
- NO `xfail` / `Stub` / `ImportPass`. Only `AssertionPass` counts.
- NO TOML-schema theater. The Rust test that walks this seed is `tests/harness/cpython/lib_test.rs` via `cargo test -p mamba --test conformance_cpython_lib_test`; it executes `mamba run`.
- Real-world preferred. Distilled minimal repros of bugs outrank synthetic features.
