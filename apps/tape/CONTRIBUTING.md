# Contributing to tape

## Brief

How to change `apps/tape`. What it promises and the work roots it owns live in
[README.md](README.md); the per-surface support state is in
[STATUS.md](STATUS.md); repository-wide authoring and verification rules live
in the root [CONTRIBUTING.md](../../CONTRIBUTING.md).

Changes here are authored one phase at a time, red first: `e2e` writes
`apps/tape/e2e/`, then `impl` writes `apps/tape/src/`. `/aw-e2e-for-wi` drives
the e2e phase and `/aw-impl-for-wi` drives the impl phase, and every phase
refuses a dirty path outside its own write root.

`Cargo.toml` declares `autotests = false` and one `[[test]]` stanza per
`e2e/*.rs` file. A new case is not run until its stanza exists, so add the
stanza in the same `e2e` phase that adds the file. Targets behind a feature
carry `required-features` and only run with that feature enabled.

## Verification

| Gate | Command |
|---|---|
| Declared e2e targets plus colocated unit tests | `cargo test -p tape` |
| Operator and backup feature targets | `cargo test -p tape --features operator,backup` |
| Release-mode performance ceiling | `cargo test --release -p tape --test tape_perf_gate` |
| Product document contract | `uv run --python 3.13 --no-project scripts/meta/project_docs_contract.py check apps/tape --format json` |
| Local kind acceptance (manual) | `bash apps/tape/scripts/kind-e2e.sh` |

Run the document contract check after editing `README.md`, `STATUS.md`,
`ROADMAP.md`, or `clients/README.md`; it resolves every gate above to a
declared target and refuses a bare test-name filter.
