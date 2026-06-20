//! CI lint enforcing the DDD shape of `tests/`.
//!
//! `tests/` is organized by capability domain so agents can learn where to act
//! from paths alone:
//!
//!   - `cpython/`    — CPython replacement fixtures, config, conventions, and tools.
//!   - `harness/`    — execution, collection, and reporting harnesses.
//!   - `mambalibs/`  — mamba-native library gates.
//!   - `pkgmgr/`     — package-manager CLI gates.
//!   - `governance/` — meta-gates over manifests and test policy.
//!
//! Cargo only auto-discovers top-level `tests/*.rs`, so this project uses
//! explicit `[[test]]` entries in `Cargo.toml`. New integration tests should
//! land under a domain directory and be wired through a domain entrypoint.
//! Canonical authoring rules live in `CONTRIBUTING.md` under "Mamba test
//! architecture: DDD, boundary-first"; this file enforces the cheap structural
//! subset.
//!
//! This lint is intentionally structural and fast. Runtime behavior belongs in
//! the domain harnesses; this file prevents the test tree from drifting back
//! into mixed per-issue files or unexplained top-level binaries.

use std::fs;
use std::path::{Path, PathBuf};

use toml::Value;

const DOMAIN_DIRS: &[&str] = &["cpython", "harness", "mambalibs", "pkgmgr", "governance"];
const MAX_LEGACY_CPYTHON_MONOLITHS: usize = 511;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn top_level_tests_directory_contains_only_domain_dirs_and_docs() {
    let tests_dir = manifest_dir().join("tests");
    let mut violations = Vec::new();

    for entry in fs::read_dir(&tests_dir).expect("read tests/ failed") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("<non-utf8>");

        let allowed = if path.is_dir() {
            DOMAIN_DIRS.contains(&name)
        } else {
            matches!(name, "README.md" | "PRODUCTION-GATE.md")
        };

        if !allowed {
            violations.push(name.to_string());
        }
    }

    assert!(
        violations.is_empty(),
        "unexpected entries at tests/ root; add them under a capability domain: {violations:?}"
    );
}

#[test]
fn domain_roots_contain_only_entrypoints_and_taxonomy_dirs() {
    let mut violations = Vec::new();

    // Dimension-first fixture layout (tests/harness/cpython/conventions/
    // FIXTURE-LAYOUT.md): the cpython domain root holds one dir per facet,
    // the flat walls, the no-record `_regression` tree, and the residual
    // legacy bucket dirs (`core`, `std-libs`, `3rd-libs`) that still carry
    // referenced manifests / bench placeholders.
    collect_domain_root_violations(
        "cpython",
        &[
            ".cache",
            "_regression",
            "3rd-libs",
            "behavior",
            "bench",
            "concurrency",
            "core",
            "errors",
            "perf",
            "real_world",
            "security",
            "security-matrix",
            "std-libs",
            "surface",
            "type",
        ],
        &[],
        |_| false,
        &mut violations,
    );
    collect_domain_root_violations("harness", &["cpython"], &[], |_| false, &mut violations);
    collect_domain_root_violations(
        "governance",
        &["gates", "mvp_gates", "schema_gates"],
        &[
            "ci_guard.rs",
            "common.rs",
            "mvp_gates.rs",
            "schema_gates.rs",
        ],
        |_| false,
        &mut violations,
    );
    collect_domain_root_violations(
        "mambalibs",
        &["fixtures"],
        &["runner.rs"],
        |name| name.starts_with("mambalibs_") && name.ends_with(".rs"),
        &mut violations,
    );
    collect_domain_root_violations(
        "pkgmgr",
        &[],
        &[
            "add.rs",
            "cache.rs",
            "hash.rs",
            "init.rs",
            "install.rs",
            "lock.rs",
            "remove.rs",
            "run_preflight.rs",
            "run_stdin.rs",
            "runner.rs",
            "sync.rs",
            "validate.rs",
        ],
        |_| false,
        &mut violations,
    );

    assert!(
        violations.is_empty(),
        "unexpected entries at test domain roots; move cases under the domain taxonomy: {violations:?}"
    );
}

fn collect_domain_root_violations(
    domain: &str,
    allowed_dirs: &[&str],
    allowed_files: &[&str],
    allow_file: impl Fn(&str) -> bool,
    violations: &mut Vec<String>,
) {
    let root = manifest_dir().join("tests").join(domain);
    for entry in
        fs::read_dir(&root).unwrap_or_else(|err| panic!("read {} failed: {err}", root.display()))
    {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("<non-utf8>");

        let allowed = if path.is_dir() {
            allowed_dirs.contains(&name)
        } else {
            allowed_files.contains(&name) || allow_file(name)
        };

        if !allowed {
            violations.push(format!("{domain}/{name}"));
        }
    }
}

#[test]
fn cargo_test_targets_are_domain_entrypoints() {
    let cargo_toml_path = manifest_dir().join("Cargo.toml");
    let raw = fs::read_to_string(&cargo_toml_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", cargo_toml_path.display()));
    let parsed: Value = raw
        .parse()
        .unwrap_or_else(|err| panic!("parse {}: {err}", cargo_toml_path.display()));
    let tests = parsed
        .get("test")
        .and_then(Value::as_array)
        .expect("Cargo.toml must have [[test]] entries");

    let mut violations = Vec::new();
    for test in tests {
        let name = test
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("<missing-name>");
        let path = test
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or("<missing-path>");

        if !is_allowed_test_target_path(path) {
            violations.push(format!("{name}: {path}"));
        }
        let full_path = manifest_dir().join(path);
        assert!(
            full_path.exists(),
            "Cargo test target `{name}` points at missing {}",
            full_path.display()
        );
    }

    assert!(
        violations.is_empty(),
        "test targets must point at domain entrypoints, not ad-hoc files: {violations:?}"
    );
}

fn is_allowed_test_target_path(path: &str) -> bool {
    path.starts_with("tests/harness/cpython/")
        || matches!(
            path,
            "tests/mambalibs/runner.rs"
                | "tests/pkgmgr/runner.rs"
                | "tests/governance/schema_gates.rs"
                | "tests/governance/mvp_gates.rs"
                | "tests/governance/ci_guard.rs"
        )
}

#[test]
fn cpython_legacy_monolith_count_does_not_increase() {
    let fixtures = manifest_dir().join("tests/cpython");
    let mut count = 0_usize;
    walk_py_files(&fixtures, &mut |path| {
        if matches!(
            path.file_name().and_then(|s| s.to_str()),
            Some("surface.py" | "behavior.py" | "errors.py")
        ) {
            count += 1;
        }
    });

    assert!(
        count <= MAX_LEGACY_CPYTHON_MONOLITHS,
        "legacy CPython monolith fixtures increased to {count}; target shape is <facet>/<bucket>/<lib>/<case>.py"
    );
}

fn walk_py_files(root: &Path, visit: &mut impl FnMut(&Path)) {
    for entry in
        fs::read_dir(root).unwrap_or_else(|err| panic!("read {} failed: {err}", root.display()))
    {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            walk_py_files(&path, visit);
        } else if path.extension().and_then(|s| s.to_str()) == Some("py") {
            visit(&path);
        }
    }
}
