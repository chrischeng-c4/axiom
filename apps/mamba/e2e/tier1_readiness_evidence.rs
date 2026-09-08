//! Black-box contract: `cargo test -p mamba --test tier1_readiness_evidence`
//! runs the readiness fixture
//! `apps/mamba/e2e/tier1/concurrency/readiness/container_mutation_rounds.py`
//! — four threads each performing 2000 `list.append`, `dict.__setitem__`,
//! and `set.add` calls into fresh containers, repeated for 5 rounds and
//! again for 50 rounds under `target/debug/mamba run --compile` — and
//! measures from the Rust side, via `libc::wait4` on the child's own pid,
//! that the 50-round run's peak `ru_maxrss` is no more than 4x the 5-round
//! run's, that each run's wall time stays under the fixture header's
//! `max_wall_s`, and that each run's CPU time (`ru_utime + ru_stime`) is
//! reported. Today the target does not exist (`error: no test target named
//! \`tier1_readiness_evidence\` in \`mamba\` package`, exit 101); once it
//! does, both runs reach `readiness: PASS` and stay inside `max_wall_s` —
//! 3.463s and 29.693s, both under the header's 120s — but the 50-round
//! peak RSS measures 1414594560 bytes against the 5-round peak of
//! 217317376 bytes, a 6.51x ratio over the 4x bound, because containers a
//! completed round drops are retained across rounds (issue #4244).
//!
//! # What this case owns
//!
//! Discovery and header parsing live in `tier1/harness/mod.rs`, included
//! below through `#[path = "tier1/harness/mod.rs"] mod harness;` — a
//! fixture, not a case, under the engine's own rule that only a `.rs` file
//! directly under `e2e/` is a case (`apps/aw/src/aw/scripts/e2e.py:234`).
//! This file owns the restriction to the `concurrency/readiness` prefix,
//! the 5-round/50-round double run and its ratio check, the wall-time
//! bound check, and the fail-closed checks over an empty prefix and a
//! malformed `max_wall_s` header field.
//!
//! # The observation point
//!
//! `cargo test -p mamba --test tier1_readiness_evidence`'s own exit code
//! and, on failure, the panic message naming the fixture path and the two
//! measured peak-RSS values with their ratio.
//!
//! # Facets
//!
//! - **Behavior**: `apps/mamba/e2e/tier1_readiness_evidence.rs:189` —
//!   the `assert!(fifty.maxrss <= 4 * five.maxrss, …)` fails today: the
//!   fixture's 50-round run measures a peak RSS of 1414594560 bytes
//!   against the 5-round run's 217317376 bytes, a 6.51x ratio, over the 4x
//!   bound (`apps/mamba/e2e/tier1/harness/mod.rs`'s `run_readiness_rounds`,
//!   reading the child's own `rusage` via `libc::wait4` rather than
//!   `getrusage(RUSAGE_CHILDREN)`). The four assertions before it in the
//!   same loop — `apps/mamba/e2e/tier1_readiness_evidence.rs:155` and
//!   `:161` (`readiness: PASS` at 5 and 50 rounds) and `:171` and
//!   `:177` (each run's wall time under the fixture's own
//!   `max_wall_s`, 3.463s and 29.693s against a 120s bound) — already pass
//!   at HEAD, so correctness and the deadlock guard both hold; only the
//!   leak bound is red. This is the red `apps/mamba/src/runtime/` (impl
//!   phase, the allocation path for a round-scoped `list`, `dict`, and
//!   `set` value, located by this red) must turn green.
//! - **Security**: `apps/mamba/e2e/tier1_readiness_evidence.rs:207`
//!   closes the same trust boundary `tier1_cross_thread_identity.rs:157`
//!   closes for `concurrency/identity`: this case reads a fixture sub-tree
//!   (`apps/mamba/e2e/tier1/concurrency/readiness/**`) that did not exist
//!   before this change, a file input this crate now walks and judges,
//!   and `tier1_exit.rs`'s per-area loop only requires the whole
//!   `concurrency` area to be non-empty, so an empty or missing
//!   `concurrency/readiness` prefix specifically would still pass that
//!   coarser check as long as some other concurrency fixture existed
//!   anywhere (`identity`, `safety/re`, …) — a coverage gap for exactly
//!   the sub-tree this issue populates.
//!   `apps/mamba/e2e/tier1_readiness_evidence.rs:221` and `:227`
//!   close a second, new boundary this change opens:
//!   `apps/mamba/e2e/tier1/harness/mod.rs`'s new `parse_max_wall_s` (line
//!   205) reads a fixture-header-supplied field — file content, not this
//!   crate's own bytes — and must fail closed rather than silently
//!   default when the field is missing or non-numeric, the same no-skip
//!   contract `parse_kind` (line 179) already holds for the header's
//!   `kind` field, extended here to `max_wall_s`. Exercised directly
//!   against synthetic header text rather than a second fixture file,
//!   since the readiness prefix's fixture count is itself pinned to
//!   exactly one file (issue #4244 Acceptance row 4). All three ride
//!   after the behavior assertions in the same `#[test] fn`, so today's
//!   observed red is the behavior panic above and these assertions are
//!   unreached — they re-arm, still checked, on every later green, the
//!   same pattern `tier1_exit.rs:126` and
//!   `tier1_cross_thread_identity.rs:157` use for their own fail-closed
//!   checks.
//! - **Performance**: the change reaches `apps/mamba/src/runtime/` (impl
//!   phase) through the CLI subcommand `mamba run --compile <file>` — a
//!   build/compile-and-run step a developer waits on, invoked here twice
//!   per discovered readiness fixture (5 rounds, then 50). No document
//!   names a budget for this path: the only README performance promise,
//!   `cpu-and-memory-under-cpython` (`apps/mamba/README.md:971`,
//!   `apps/mamba/README.md:993`–`1001`), is scoped to "the pinned perf
//!   fixtures" gated by `cargo test -p mamba --release --test perf_pin` —
//!   a different path than the compile-and-run invocations this case
//!   performs, and the same gap `tier1_exit.rs` and
//!   `tier1_cross_thread_identity.rs` already report for the sibling
//!   `core`/`type`/`concurrency` rollup and the `concurrency/identity`
//!   prefix. This case asserts no timing budget — the ratio bound above is
//!   the contract's own leak bound, not a speed or memory ceiling drawn
//!   from any document; the missing budget is a gap for `mamba-pm` to
//!   draft for the `mamba run --compile` path this issue's own change
//!   points reach, nearest the existing `cpu-and-memory-under-cpython`
//!   capability (`apps/mamba/README.md:971`) it could extend or sit
//!   beside. Measured in this run, the same `run_readiness_rounds` call
//!   this case makes at 5 rounds: `mamba run --compile
//!   apps/mamba/e2e/tier1/concurrency/readiness/container_mutation_rounds.py`
//!   with `MAMBA_FIXTURE_ROUNDS=5` (debug build) → cpu 3.846s
//!   (`ru_utime + ru_stime`), wall 3.463s, maxrss 217317376 bytes
//!   (`ru_maxrss`, via `libc::wait4`).

#[path = "tier1/harness/mod.rs"]
mod harness;

use std::path::Path;

#[test]
fn tier1_readiness_evidence() {
    if let Err(reason) = harness::check_python312() {
        panic!("python3.12 precondition failed: {reason}");
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("e2e/tier1");
    let readiness_root = root.join("concurrency").join("readiness");
    let fixtures: Vec<harness::Fixture> = harness::discover_fixtures(&root)
        .into_iter()
        .filter(|fixture| fixture.path.starts_with(&readiness_root))
        .collect();

    for fixture in &fixtures {
        let text = std::fs::read_to_string(&fixture.path)
            .unwrap_or_else(|e| panic!("{}: cannot read fixture: {e}", fixture.path.display()));
        let max_wall_s = harness::parse_max_wall_s(&text)
            .unwrap_or_else(|reason| panic!("{}: {reason}", fixture.path.display()));

        let five = harness::run_readiness_rounds(&fixture.path, 5).unwrap_or_else(|reason| {
            panic!("{}: 5-round run failed: {reason}", fixture.path.display())
        });
        let fifty = harness::run_readiness_rounds(&fixture.path, 50).unwrap_or_else(|reason| {
            panic!("{}: 50-round run failed: {reason}", fixture.path.display())
        });

        println!(
            "{}: 5 rounds  -> maxrss={} cpu={:.3}s wall={:.3}s",
            fixture.path.display(),
            five.maxrss,
            five.cpu_s,
            five.wall_s
        );
        println!(
            "{}: 50 rounds -> maxrss={} cpu={:.3}s wall={:.3}s",
            fixture.path.display(),
            fifty.maxrss,
            fifty.cpu_s,
            fifty.wall_s
        );

        // Behavior: both runs must reach their own PASS verdict -- the
        // container-mutation counts must come out exact every round,
        // regardless of round count.
        assert!(
            five.stdout.lines().any(|l| l.trim() == "readiness: PASS"),
            "{}: 5-round run did not print `readiness: PASS`: stdout={:?}",
            fixture.path.display(),
            five.stdout
        );
        assert!(
            fifty.stdout.lines().any(|l| l.trim() == "readiness: PASS"),
            "{}: 50-round run did not print `readiness: PASS`: stdout={:?}",
            fixture.path.display(),
            fifty.stdout
        );

        // Behavior: neither run may deadlock or run away -- the fixture
        // header's own max_wall_s bound (a deadlock guard, not a speed
        // budget -- Frozen decisions, issue #4244).
        assert!(
            five.wall_s <= max_wall_s,
            "{}: 5-round wall time {:.3}s exceeded max_wall_s {max_wall_s:.3}s",
            fixture.path.display(),
            five.wall_s
        );
        assert!(
            fifty.wall_s <= max_wall_s,
            "{}: 50-round wall time {:.3}s exceeded max_wall_s {max_wall_s:.3}s",
            fixture.path.display(),
            fifty.wall_s
        );

        // Behavior: the leak bound -- today's named red. The 50-round run's
        // peak RSS must be no more than 4x the 5-round run's; today dropped
        // round containers are retained across rounds, so this fails,
        // naming the measured ratio.
        let ratio = fifty.maxrss as f64 / five.maxrss as f64;
        assert!(
            fifty.maxrss <= 4 * five.maxrss,
            "{}: 50-round peak RSS {} exceeded 4x the 5-round peak RSS {} (ratio {ratio:.2}x)",
            fixture.path.display(),
            fifty.maxrss,
            five.maxrss
        );
    }

    // Security: the concurrency/readiness prefix itself must fail closed on
    // an empty or missing set -- the same pattern
    // tier1_cross_thread_identity.rs applies to concurrency/identity, and
    // for the same reason: tier1_exit.rs's per-area loop only requires the
    // whole `concurrency` area to be non-empty, so an empty or missing
    // `concurrency/readiness` prefix specifically would still pass that
    // coarser check as long as some other concurrency fixture existed
    // anywhere. Reached only once every discovered fixture has already
    // passed every assertion above, so it re-arms on every later green.
    assert!(
        !fixtures.is_empty(),
        "concurrency/readiness contributed zero fixtures; an empty or \
         missing prefix must fail the rollup closed, not pass it"
    );

    // Security: a readiness fixture header that is missing or malforms
    // max_wall_s must fail closed, never silently default -- the same
    // no-skip contract `parse_kind` already holds for the `kind` field
    // (harness/mod.rs module doc), extended here to this change's new
    // field. Exercised directly against synthetic header text rather than
    // a second fixture file, since the readiness prefix's fixture count is
    // itself pinned to exactly one file (issue #4244 Acceptance row 4).
    let missing_field = "# /// script\n#\n# [tool.mamba]\n# kind = \"readiness\"\n# ///\n";
    assert!(
        harness::parse_max_wall_s(missing_field).is_err(),
        "a readiness header missing max_wall_s must fail closed, not \
         silently default"
    );
    let non_numeric = "# /// script\n#\n# [tool.mamba]\n# kind = \"readiness\"\n# max_wall_s = \"soon\"\n# ///\n";
    assert!(
        harness::parse_max_wall_s(non_numeric).is_err(),
        "a readiness header with a non-numeric max_wall_s must fail \
         closed, not silently default"
    );
}
