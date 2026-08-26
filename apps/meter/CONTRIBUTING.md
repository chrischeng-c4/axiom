# Contributing to meter

## Brief

How to change `apps/meter`. What it promises and the work roots it owns live in
[README.md](README.md); repository-wide authoring and verification rules live
in the root [CONTRIBUTING.md](../../CONTRIBUTING.md).

Changes here are authored one phase at a time, red first: `e2e` writes
`apps/meter/e2e/`, then `unit` and `logic` write `apps/meter/src/`.
`/aw-go-tdd-for-change` drives the ladder, and every phase refuses a dirty path
outside its own write root.

## Verification

| Gate | Command |
|---|---|
| unit + colocated tests | `cargo test -p meter` |
