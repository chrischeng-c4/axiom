# Contributing to cube

## Brief

How to change `apps/cube`. What it promises and the work roots it owns live in
[README.md](README.md); repository-wide authoring and verification rules live
in the root [CONTRIBUTING.md](../../CONTRIBUTING.md).

Changes here are authored one phase at a time, red first: `e2e` writes
`apps/cube/e2e/`, then `impl` writes `apps/cube/src/`. `/aw-e2e-for-wi`
drives the e2e phase and `/aw-impl-for-wi` drives the impl phase, and every
phase refuses a dirty path outside its own write root.

## Verification

Nothing to verify yet. `apps/cube` holds its configuration, its `tech-design/`
bucket and these two documents -- no source, no crate, and so no gate command
that would not be a fiction. The first change opens the ladder above, and this
section gets its first row then.
