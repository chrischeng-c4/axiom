# Contributing to lumen

## Brief

How to change `apps/lumen`. What it promises and the work roots it owns live in
[README.md](README.md); repository-wide authoring and verification rules live
in the root [CONTRIBUTING.md](../../CONTRIBUTING.md).

Changes here are authored one phase at a time, red first: `e2e` writes
`apps/lumen/e2e/`, then `unit` and `logic` write `apps/lumen/src/`.
`/aw:wi-tdd` drives the ladder, and every phase refuses a dirty path outside
its own write root.

## Verification

| Gate | Command |
|---|---|
| unit + colocated tests | `cargo test -p lumen` |
| feature-gated e2e targets | `cargo test -p lumen --features "operator delegated-auth"` |

Both rows are required, and neither run is a superset of the other.

`default = []`, and no `[[test]]` stanza in `Cargo.toml` declares
`required-features`. So the first command does not skip the 11 targets whose
files open with `#![cfg(feature = …)]` — it compiles each of them into an empty
binary that prints `test result: ok. 0 passed` and exits 0. Ten of the eleven
are gated on `operator`; `e2e/cli_client_ksa_token.rs:29` is gated on
`all(unix, feature = "delegated-auth", feature = "backup")`, and `operator`
pulls in `backup` but never `delegated-auth`. `"operator delegated-auth"` is
the smallest feature string that compiles all eleven, and
`e2e/feature_gated_targets_are_registered.rs` is the list it has to satisfy.

The second row does not replace the first, because a feature that is on
compiles away the code that runs when it is off. `e2e/auth_e2e.rs:278` is
`#[cfg(not(feature = "delegated-auth"))]` with a live twin at `:285`, and
`src/bin/lumen.rs` carries 15 `#[cfg(not(feature = …))]` refusal stubs. Nothing
in the featured run can refuse a regression in any of them.

`--all-features` is not the second row, even though it does compile all eleven.
It also turns on `jieba`, and `e2e/jieba_bigram_fallback_e2e.rs` was written to
prove that CJK matching still works with `jieba` compiled out. That file states
the requirement in its `//!` but carries no `cfg` to enforce it, so
`--all-features` makes it fail for the same reason it exists.

Measured on the S8 tree (parent `db414b17db`), both with `--no-fail-fast`, 82
test binaries each: row 1 is `629 passed; 4 failed; 28 ignored` and row 2 is
`929 passed; 4 failed; 28 ignored`, over the same three red targets —
`capability_shared_ownership`, `retired_credential_surface` and `spec_gen_e2e`
— failing with byte-identical messages, so the second row introduces no new
failure. Of its `+300` executed tests, 177 are the eleven binaries that were
empty under row 1.

Row 1 can show a fourth red target that is not a regression.
`e2e/auth_e2e.rs:182` gives a spawned `lumen serve` 20 seconds to refuse and
exit, and panics at `:210` if it has not. When that deadline expires the panic
prints the child's own stderr — which is the correct refusal — so the message
says the server "kept running" while quoting the proof that it did not. Row 1
read `627 passed; 6 failed; 43 ignored` over 4 red targets at `d1f407f6cf`,
whose tree differs from `db414b17db` only in three `.md` files; that run shared
the machine and the workspace `target/` with a second session. On an isolated
`CARGO_TARGET_DIR` the target finishes in ~1.1 s, three runs of three.
