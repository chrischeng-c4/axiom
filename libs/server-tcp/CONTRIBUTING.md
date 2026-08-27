# Contributing to server-tcp

## Brief

How to change `libs/server-tcp`. What it promises and the work roots it owns
live in [README.md](README.md); repository-wide authoring and verification
rules live in the root [CONTRIBUTING.md](../../CONTRIBUTING.md).

There is no phase ladder here. `leg.leg_root` resolves `apps/<project>` and
nothing else, so a change to this crate is authored and committed directly
rather than driven through `e2e -> impl`.

## Verification

| Gate | Command |
|---|---|
| unit + colocated tests | `cargo test -p server-tcp` |
