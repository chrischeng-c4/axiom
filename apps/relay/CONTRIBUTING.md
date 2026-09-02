# Contributing to relay

## Brief

How to change `apps/relay`. What it promises and the work roots it owns live in
[README.md](README.md); repository-wide authoring and verification rules live
in the root [CONTRIBUTING.md](../../CONTRIBUTING.md).

Changes here are authored one phase at a time, red first: `e2e` writes
`apps/relay/e2e/`, then `impl` writes `apps/relay/src/`. `/aw-e2e-for`
drives the e2e phase and `/aw-impl-for` drives the impl phase, and every
phase refuses a dirty path outside its own write root.

## Verification

| Gate | Command |
|---|---|
| unit + colocated tests | `cargo test -p relay` |
