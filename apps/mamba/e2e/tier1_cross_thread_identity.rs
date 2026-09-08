//! Black-box contract: `cargo test -p mamba --test tier1_cross_thread_identity`
//! discovers every `*.py` fixture under
//! `apps/mamba/e2e/tier1/concurrency/identity/**` — closure (already ported by
//! issue #4242), generator, hashlib, io, and random — judges each by the
//! `self-verdict` kind its own PEP 723 header names, and exits 0 only once
//! every one of them prints `concurrency: PASS`. Today the target does not
//! exist (`error: no test target named \`tier1_cross_thread_identity\` in
//! \`mamba\` package`, exit 101); once it does, five of the ten fixtures print
//! a `concurrency: FAIL:` line under `target/debug/mamba run --compile`: the
//! four parent-to-child handoff fixtures for generator, hashlib, io, and
//! random (`TypeError: 'int' object is not iterable`,
//! `TypeError: 'NoneType' object is not callable` twice,
//! `AttributeError: 'int' object has no attribute 'write'`), plus
//! `io/child_opened_file_writes_in_child.py`'s distinct `with open(...) as
//! fh:` defect (`UnboundLocalError: cannot access local variable 'fh' where
//! it is not associated with a value`) — where `python3.12` is not consulted
//! at all, because a concurrency fixture judges itself (issue #4243).
//!
//! # What this case owns
//!
//! Discovery, header parsing, and the `self-verdict` judge live in
//! `tier1/harness/mod.rs`, included below through
//! `#[path = "tier1/harness/mod.rs"] mod harness;` — a fixture, not a case,
//! under the engine's own rule that only a `.rs` file directly under `e2e/`
//! is a case (`apps/aw/src/aw/scripts/e2e.py:234`). This file owns the
//! restriction to the `concurrency/identity` prefix and the fail-closed
//! checks over it: walk the whole `tier1` tree, keep only fixtures whose path
//! falls under `concurrency/identity`, judge every one of them, report every
//! failing path, and fail closed if the prefix or any of its sub-buckets
//! contributed nothing.
//!
//! # The observation point
//!
//! `cargo test -p mamba --test tier1_cross_thread_identity`'s own exit code
//! and, on failure, the panic message's list of every fixture path whose
//! judge rejected it — never a name-filtered subset, matching the issue's
//! `## Never` rule against running this target with `--skip` or a filter.
//!
//! # Facets
//!
//! - **Behavior**: `apps/mamba/e2e/tier1_cross_thread_identity.rs:143` — the
//!   aggregate `assert!(failures.is_empty(), …)` fails today naming five
//!   fixture paths — `generator/next_in_child_yields_all_values.py`,
//!   `hashlib/parent_hash_object_hexdigests_in_child.py`,
//!   `io/parent_file_handle_writes_in_child.py`,
//!   `io/child_opened_file_writes_in_child.py`, and
//!   `random/parent_random_instance_draws_in_child.py` — whose judge
//!   (`apps/mamba/e2e/tier1/harness/mod.rs`'s `judge_self_verdict`) records a
//!   missing `concurrency: PASS` stdout line; the other five discovered
//!   fixtures (the two already-ported closure fixtures and the three
//!   `child_built_*` controls) already print it. This is the red the
//!   `apps/mamba/src/runtime/{iter.rs,file_io.rs,stdlib/hashlib_mod.rs,stdlib/random_mod.rs}`
//!   registry moves and the child-thread `with … as` binding fix (impl
//!   phase, issue #4243) must turn green.
//! - **Security**: `apps/mamba/e2e/tier1_cross_thread_identity.rs:157` and
//!   `:170` — this case reads a fixture sub-tree
//!   (`apps/mamba/e2e/tier1/concurrency/identity/**`) that did not hold eight
//!   of its ten files before this change, a file input this crate now walks
//!   and judges; the trust boundary is the same one `tier1_exit.rs` closes
//!   for the three top-level areas, but `tier1_exit`'s per-area loop only
//!   requires the whole `concurrency` area to be non-empty, so an empty
//!   `concurrency/identity` prefix, or an empty `generator`, `hashlib`,
//!   `io`, or `random` sub-bucket specifically, would still pass that loop
//!   as long as some other concurrency fixture existed anywhere (`closure`,
//!   `safety/re`, …) — a coverage gap for exactly the sub-tree this issue
//!   populates. `apps/mamba/e2e/tier1_cross_thread_identity.rs:157` asserts
//!   the `concurrency/identity` prefix itself contributed at least one
//!   fixture, and `:170` asserts each of its five sub-buckets
//!   (`closure`, `generator`, `hashlib`, `io`, `random`) did too — an
//!   accidentally empty or missing bucket must fail this rollup closed
//!   rather than pass by vacuous truth over zero fixtures. Both ride after
//!   the behavior assertion in the same `#[test] fn`, so today's observed
//!   red is the behavior panic above and these assertions are unreached —
//!   they re-arm, still checked, on every later green, the same pattern
//!   `tier1_exit.rs:126` uses for its own per-area loop.
//! - **Performance**: the change reaches
//!   `apps/mamba/src/runtime/{iter.rs,file_io.rs,stdlib/hashlib_mod.rs,stdlib/random_mod.rs}`
//!   (impl phase) through the CLI subcommand `mamba run --compile <file>` —
//!   a build/compile-and-run step a developer waits on, invoked here against
//!   every discovered `concurrency/identity` fixture. No document names a
//!   budget for this path: the only README performance promise,
//!   `cpu-and-memory-under-cpython` (`apps/mamba/README.md:971`,
//!   `apps/mamba/README.md:993`–`1001`), is scoped to "the pinned perf
//!   fixtures" gated by `cargo test -p mamba --release --test perf_pin` — a
//!   different path than the compile-and-run invocations this case performs,
//!   and the same gap `tier1_exit.rs` already reports for the sibling
//!   `tier1_exit` rollup over the `core`/`type`/`concurrency` tree. This case
//!   asserts no timing; the missing budget is a gap for `mamba-pm` to draft
//!   for the `mamba run --compile` build-step path this issue's own change
//!   points reach, nearest the existing `cpu-and-memory-under-cpython`
//!   capability (`apps/mamba/README.md:971`) it could extend or sit beside.
//!   Measured in this run over an already-green fixture at HEAD (a
//!   `child_built_*` control, never one of the five red handoff fixtures, so
//!   the timing reflects a full compile-and-run rather than an early error
//!   exit),
//!   `apps/mamba/e2e/tier1/concurrency/identity/generator/child_built_generator_yields_all_values.py`:
//!   `/usr/bin/time -l target/debug/mamba run --compile
//!   apps/mamba/e2e/tier1/concurrency/identity/generator/child_built_generator_yields_all_values.py`
//!   (debug build) → `0.33 real 0.29 user 0.00 sys`, `69419008` maximum
//!   resident set size (bytes).

#[path = "tier1/harness/mod.rs"]
mod harness;

use std::collections::BTreeMap;
use std::path::Path;

#[test]
fn tier1_cross_thread_identity() {
    if let Err(reason) = harness::check_python312() {
        panic!("python3.12 precondition failed: {reason}");
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("e2e/tier1");
    let identity_root = root.join("concurrency").join("identity");
    let fixtures: Vec<harness::Fixture> = harness::discover_fixtures(&root)
        .into_iter()
        .filter(|fixture| fixture.path.starts_with(&identity_root))
        .collect();

    let mut failures: Vec<String> = Vec::new();
    let mut buckets: BTreeMap<String, usize> = BTreeMap::new();
    for fixture in &fixtures {
        let bucket = fixture
            .path
            .strip_prefix(&identity_root)
            .ok()
            .and_then(|rel| rel.components().next())
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .unwrap_or_default();
        *buckets.entry(bucket).or_insert(0) += 1;
        if let Err(reason) = harness::judge_fixture(&fixture.path) {
            failures.push(format!("{}: {reason}", fixture.path.display()));
        }
    }

    // Behavior: every discovered concurrency/identity fixture must print
    // `concurrency: PASS`. Five parent-to-child handoff fixtures are today's
    // named red (issue #4243): the generator, hashlib, io (parent handle),
    // and random registries are thread-local, and the io child-opened `with
    // ... as fh:` binding is independently broken, so this assertion fails
    // today naming all five paths.
    assert!(
        failures.is_empty(),
        "{} of {} concurrency/identity fixtures failed their judge:\n{}",
        failures.len(),
        fixtures.len(),
        failures.join("\n")
    );

    // Security: the concurrency/identity prefix itself must fail closed on
    // an empty or missing set -- a fixture tree that silently contributed
    // zero fixtures must not let this restricted rollup pass by vacuous
    // truth. Reached only once every discovered fixture has already passed
    // its judge above, so it re-arms on every later green the same way the
    // behavior assertion does.
    assert!(
        !fixtures.is_empty(),
        "concurrency/identity contributed zero fixtures; an empty or missing \
         prefix must fail the rollup closed, not pass it"
    );

    // Security: each sub-bucket under concurrency/identity must also fail
    // closed on its own -- tier1_exit.rs's per-area loop only requires the
    // whole `concurrency` area to be non-empty, so an empty bucket here
    // would otherwise hide behind an unrelated concurrency fixture
    // elsewhere (closure, safety/re, ...) passing that coarser check.
    for bucket in ["closure", "generator", "hashlib", "io", "random"] {
        let count = buckets.get(bucket).copied().unwrap_or(0);
        assert!(
            count > 0,
            "concurrency/identity bucket {bucket:?} contributed zero fixtures; \
             an empty or missing bucket must fail the rollup closed, not pass it"
        );
    }
}
