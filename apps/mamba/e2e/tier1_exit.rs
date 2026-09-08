//! Black-box contract: `cargo test -p mamba --test tier1_exit` discovers
//! every `*.py` fixture under `apps/mamba/e2e/tier1/**`, judges each by the
//! kind its own PEP 723 `[tool.mamba]` header names, and exits 0 only once
//! all of them pass — including the new core-semantics fixture
//! `tier1/core/module_loop_rebinds_global_with_def_present.py`, whose
//! module defines a function and then accumulates in a module-scope `for`
//! loop. Today the target does not exist (`error: no test target named
//! \`tier1_exit\` in \`mamba\` package`, exit 101); once it does, the core
//! fixture's program prints `0` under `mamba run --compile` where
//! `python3.12` prints `3` (issue #4242).
//!
//! # What this case owns
//!
//! Discovery, header parsing, and the three judges (`oracle`,
//! `type-strict`, `self-verdict`) live in `tier1/harness/mod.rs`, included
//! below through `#[path = "tier1/harness/mod.rs"] mod harness;` — a
//! fixture, not a case, under the engine's rule that only a `.rs` file
//! directly under `e2e/` is a case (`apps/aw/src/aw/scripts/e2e.py:234`).
//! This file owns the rollup itself: walk every area, judge every
//! discovered fixture, report every failing path, and fail closed if an
//! area contributed nothing.
//!
//! # The observation point
//!
//! `cargo test -p mamba --test tier1_exit`'s own exit code and, on
//! failure, the panic message's list of every fixture path whose judge
//! rejected it — never a name-filtered subset, matching the issue's
//! `## Never` rule against running this target with `--skip` or a filter.
//!
//! # Facets
//!
//! - **Behavior**: `apps/mamba/e2e/tier1_exit.rs:110` — the aggregate
//!   `assert!(failures.is_empty(), …)` fails today naming
//!   `tier1/core/module_loop_rebinds_global_with_def_present.py`, whose
//!   judge (`apps/mamba/e2e/tier1/harness/mod.rs`'s `judge_oracle`) records
//!   a stdout mismatch: `mamba run --compile` prints `0`, `python3.12`
//!   prints `3` — the exact defect bisected to
//!   `apps/mamba/src/lower/hir_to_mir.rs:7367`. The same run also fails
//!   `tier1/concurrency/safety/re/match_group_in_child_does_not_corrupt_heap.py`,
//!   whose own docstring documents a deliberate, already-tracked crash
//!   (`source = "#3103"` in its header) unrelated to loop lowering; the
//!   assertion reports both paths and keeps checking every discovered
//!   fixture on every later run, so it stays the one gate for the whole
//!   tree rather than only the fixture this work item's impl phase
//!   targets.
//! - **Security**: `apps/mamba/e2e/tier1_exit.rs:126` — the
//!   per-area `assert!(count > 0, …)` loop over `["core", "type",
//!   "concurrency"]` closes the trust boundary this rollup opens by
//!   reading a fixture tree it did not exist to read before this change
//!   (`apps/mamba/e2e/tier1/**`, a file input this crate now walks and
//!   judges): an empty or missing area directory must fail the rollup
//!   closed rather than pass by vacuous truth over zero fixtures. It rides
//!   after the behavior assertion in the same `#[test] fn`, so today's
//!   observed red is the behavior panic and this assertion is unreached —
//!   it re-arms, still checked, on every later green. This is the same
//!   fail-closed rule the issue's own Mutation B exercises by deleting
//!   `apps/mamba/e2e/tier1/core/` and requiring row 1 to fail naming the
//!   empty `core` area.
//! - **Performance**: the change reaches
//!   `apps/mamba/src/lower/hir_to_mir.rs`'s for-loop lowering (impl phase)
//!   through the CLI subcommand `mamba run --compile <file>` — a
//!   build/compile step a developer waits on, invoked here against every
//!   discovered fixture. No document names a budget for this path: the
//!   only README performance promise, `cpu-and-memory-under-cpython`
//!   (`apps/mamba/README.md:971`, `apps/mamba/README.md:993`–`1001`), is
//!   scoped to "the pinned perf fixtures" gated by
//!   `cargo test -p mamba --release --test perf_pin` — a different path
//!   than the compile-and-run invocations this rollup performs. This case
//!   asserts no timing; the missing budget is a gap for `mamba-pm` to
//!   draft for the `mamba run --compile` build-step path
//!   `apps/mamba/src/lower/hir_to_mir.rs` and this case's own change point
//!   reach, nearest the existing `cpu-and-memory-under-cpython` capability
//!   (`apps/mamba/README.md:971`) it could extend or sit beside. Measured
//!   in this run over a file already at HEAD,
//!   `apps/mamba/tests/cpython/type/core/arg_annotation/default_int_arg_uses_str_default.py`:
//!   `/usr/bin/time -l target/debug/mamba run --compile
//!   apps/mamba/tests/cpython/type/core/arg_annotation/default_int_arg_uses_str_default.py`
//!   (debug build) → `0.24 real 0.16 user 0.03 sys`, `41844736` maximum
//!   resident set size (bytes).

#[path = "tier1/harness/mod.rs"]
mod harness;

use std::collections::BTreeMap;
use std::path::Path;

#[test]
fn tier1_exit() {
    if let Err(reason) = harness::check_python312() {
        panic!("python3.12 precondition failed: {reason}");
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("e2e/tier1");
    let fixtures = harness::discover_fixtures(&root);

    let mut failures: Vec<String> = Vec::new();
    let mut areas: BTreeMap<String, usize> = BTreeMap::new();
    for fixture in &fixtures {
        *areas.entry(fixture.area.clone()).or_insert(0) += 1;
        if let Err(reason) = harness::judge_fixture(&fixture.path) {
            failures.push(format!("{}: {reason}", fixture.path.display()));
        }
    }

    // Behavior: every discovered tier1 fixture must pass its judge. The
    // core fixture `module_loop_rebinds_global_with_def_present.py` is
    // today's named red (issue #4242): `mamba run --compile` prints `0`,
    // the python3.12 oracle prints `3`, so this assertion fails today,
    // naming that fixture's path in the report below.
    assert!(
        failures.is_empty(),
        "{} of {} tier1 fixtures failed their judge:\n{}",
        failures.len(),
        fixtures.len(),
        failures.join("\n")
    );

    // Security: fixture discovery must fail closed on an empty or missing
    // top-level area -- a directory tree that silently contributed zero
    // fixtures for `core`, `type`, or `concurrency` must not let the
    // rollup pass by vacuous truth. Reached only once every discovered
    // fixture has already passed its judge above, so it re-arms on every
    // later green the same way the behavior assertion does.
    for area in ["core", "type", "concurrency"] {
        let count = areas.get(area).copied().unwrap_or(0);
        assert!(
            count > 0,
            "tier1 area {area:?} contributed zero fixtures; an empty or \
             missing area must fail the rollup closed, not pass it"
        );
    }
}
