# check exits 0 while type checking is unimplemented

> **Issue**: #1316
> **Crate**: `jet` (`projects/jet/src/cli.rs`)
> **Type**: bug

## Problem

`jet check` is wired up but its handler at
`projects/jet/src/cli.rs:1316-1320` is a stub:

```rust
Some(("check", _)) => {
    println!("Type checking...");
    println!("  This feature is under development");
    Ok(())
}
```

It prints "under development" and **returns `Ok(())`**, so the
process exits 0. Product workspaces that pipe `jet check` through
CI (Cue: `cargo run -p jet -- check` / `cclab jet check` from
`projects/cue/fe`, 2026-05-08) treat the 0 exit as "type checking
passed" and silently mask frontend validation failures.

## Scope

In:

- Make `jet check` fail loudly with an explicit
  "not yet implemented" diagnostic so CI / `&&`-chains break.
- Use `anyhow::bail!` (same pattern as the existing `jet trace`
  "unknown subcommand" and `jet audit` "vulnerabilities found"
  bails) so the exit code is non-zero and the message routes
  through the binary's standard error formatting.
- Add a small extracted helper `check_not_implemented_error()`
  that returns the typed error, so the message is unit-testable
  without spawning the binary.
- Add a unit test that pins the helper's behaviour: returns
  `Err`, message contains the issue link and the words
  `not yet implemented`.

Out:

- Implementing TypeScript type checking. That's the larger
  enhancement; the issue is purely about the silent-zero stub.
- The sibling `jet init` stub at `cli.rs:787-795` ("Initializing
  project … This feature is under development"). Out of scope.

## Interface

```rust
/// The "not yet implemented" error surfaced when a user runs
/// `jet check`. Extracted so the message can be unit-tested
/// without spawning the binary.
fn check_not_implemented_error() -> anyhow::Error;
```

Call site:

```rust
Some(("check", _)) => Err(check_not_implemented_error()),
```

Exit behaviour: `anyhow::Result<()>` propagates through
`execute_async` → `execute` → `main`, which prints the message
to stderr and exits with code 1 (anyhow's standard surface).

## Acceptance Criteria

- [x] `jet check` exits with non-zero status (no longer 0).
- [x] The diagnostic mentions "not yet implemented" and the
      tracking issue (`#1316`).
- [x] `cargo test -p jet --lib cli::check_handler_tests` passes.
- [x] No other CLI handler is touched.

## Reference Context

- `projects/jet/src/cli.rs:1316-1320` — the stub being replaced.
- `projects/jet/src/cli.rs:787-795` — the *sibling* `jet init` stub
  with the same shape; explicitly out of scope.
- `projects/jet/src/cli.rs:928`, `:1102`, `:1421`, `:1690` —
  `anyhow::bail!` precedents this fix follows.
