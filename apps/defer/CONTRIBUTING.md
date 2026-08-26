# Contributing to defer

## Brief

How to change `apps/defer`. What it promises and the work roots it owns live in
[README.md](README.md); repository-wide authoring and verification rules live
in the root [CONTRIBUTING.md](../../CONTRIBUTING.md).

Changes here are authored one phase at a time, red first: `e2e` writes
`apps/defer/e2e/`, then `unit` and `logic` write `apps/defer/src/`.
`/aw-go-tdd-for-change` drives the ladder, and every phase refuses a dirty path
outside its own write root.

## Verification

| Gate | Command |
|---|---|
| unit + colocated tests | `cargo test -p defer` |
