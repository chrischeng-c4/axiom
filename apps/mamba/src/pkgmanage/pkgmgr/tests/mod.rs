// Colocated unit tests for `pkgmgr`, one file per module under test.
// `is_test_file` (a repo-wide tooling check) recognizes a file literally
// named `tests.rs` or `tests/mod.rs`; this file exists so the ladder's C0
// gate can see the phase wrote a colocated test, without renaming any test's
// own module path -- each `mod` below matches the module names this crate's
// suite has always run.
mod installer;
mod mvp_package_manager_umbrella_gate;
mod pypi_index_client;
mod resolver;
mod venv_phase_gate;
