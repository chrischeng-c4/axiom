# Contributing to jet

## Brief

How to change `apps/jet`. What it promises and the work roots it owns live in
[README.md](README.md); repository-wide authoring and verification rules live
in the root [CONTRIBUTING.md](../../CONTRIBUTING.md).

Changes here are authored one phase at a time, red first: `e2e` writes
`apps/jet/e2e/`, then `impl` writes `apps/jet/src/`. `/aw-e2e-for-wi`
drives the e2e phase and `/aw-impl-for-wi` drives the impl phase, and every
phase refuses a dirty path outside its own write root.

## Verification

| Gate | Command |
|---|---|
| unit + colocated tests | `cargo test -p jet` |
