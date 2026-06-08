//! Schema gate for the package-manager init fixture — closes #2679.
//!
//! Acceptance (issue #2679):
//!
//!   1. Init creates the expected files exactly once.
//!      `[outputs].must_exist` is non-empty and includes `mamba.toml`.
//!   2. Re-running init fails or is idempotent according to documented
//!      policy.
//!      `[reentry].policy` is set to one of the allowed policy strings,
//!      and `[reentry].must_preserve` declares files the second run
//!      cannot overwrite.
//!   3. No user home or global cache state is modified.
//!      `[isolation]` block forbids writes outside the project and
//!      reads/writes against user home and global cache paths.
//!
//! Also asserts that the profile (`validation/profiles/package_manager.toml`)
//! still points the `init` family at `tests/governance/gates/pkgmgr/init`, so the
//! release-gate runner can find this fixture.
//!
//! Cheap test — two TOML reads + field walks. Stays in the default
//! `cargo test -p mamba` set.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("pkgmgr")
        .join("init")
        .join("manifest.toml")
}

fn profile_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("profiles")
        .join("package_manager.toml")
}

#[test]
fn pkgmgr_init_manifest_header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_init"),
        "manifest.toml `fixture` must be \"pkgmgr_init\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2679),
        "manifest.toml `issue` must record #2679 as the owning issue"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("init"),
        "manifest.toml `family` must be \"init\" (matches the package_manager profile family id)"
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("package_manager"),
        "manifest.toml `profile` must be \"package_manager\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "manifest.toml `network` must be \"offline\" (acceptance: \"Keep the test offline.\")"
    );
}

#[test]
fn pkgmgr_init_outputs_pin_expected_files() {
    let doc = crate::common::load_toml(&manifest_path());

    let outputs = doc
        .get("outputs")
        .and_then(|v| v.as_table())
        .expect("manifest.toml missing `[outputs]` block");

    let must_exist: BTreeSet<&str> = outputs
        .get("must_exist")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        !must_exist.is_empty(),
        "`[outputs].must_exist` must list at least one file \
         (acceptance: \"Init creates the expected files exactly once.\")"
    );
    assert!(
        must_exist.contains("mamba.toml"),
        "`[outputs].must_exist` must include `mamba.toml` — the \
         primary project manifest the init command creates"
    );

    // Must-not-exist pins what init does NOT create on the fresh run.
    let must_not_exist: BTreeSet<&str> = outputs
        .get("must_not_exist")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        must_not_exist.contains("mamba.lock"),
        "`[outputs].must_not_exist` must include `mamba.lock` — init \
         does not lock dependencies on a fresh project"
    );

    // No overlap — a file cannot be both must_exist and must_not_exist.
    let overlap: BTreeSet<&&str> = must_exist.intersection(&must_not_exist).collect();
    assert!(
        overlap.is_empty(),
        "must_exist and must_not_exist sets must not overlap; got {overlap:?}"
    );

    // Defaults block exists and pins at least the python-requires
    // field — the gate needs SOMETHING to assert against the file
    // content rather than only the file's presence.
    let defaults = outputs
        .get("manifest_defaults")
        .and_then(|v| v.as_table())
        .expect("`[outputs.manifest_defaults]` must declare default field values");
    let python_requires = defaults
        .get("python_requires")
        .and_then(|v| v.as_str())
        .expect("`[outputs.manifest_defaults].python_requires` must be set");
    assert!(
        !python_requires.is_empty(),
        "manifest_defaults.python_requires must be a non-empty version spec"
    );
}

#[test]
fn pkgmgr_init_reentry_policy_is_pinned() {
    let doc = crate::common::load_toml(&manifest_path());

    let reentry = doc
        .get("reentry")
        .and_then(|v| v.as_table())
        .expect("manifest.toml missing `[reentry]` block");

    let policy = reentry
        .get("policy")
        .and_then(|v| v.as_str())
        .expect("`[reentry].policy` must be set (acceptance: \
         \"Re-running init fails or is idempotent according to documented policy.\")");
    assert!(
        matches!(
            policy,
            "idempotent_keep_existing" | "idempotent_replace" | "fail_if_initialized"
        ),
        "`[reentry].policy` = {policy:?} must be one of the documented \
         policy strings"
    );

    let must_preserve: BTreeSet<&str> = reentry
        .get("must_preserve")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    if policy.starts_with("idempotent") {
        assert!(
            must_preserve.contains("mamba.toml"),
            "idempotent policies must list `mamba.toml` in \
             `[reentry].must_preserve` so a second run cannot overwrite \
             the user's edits; got {must_preserve:?}"
        );
    }

    let exit_code = reentry
        .get("expected_exit_code")
        .and_then(|v| v.as_integer())
        .expect("`[reentry].expected_exit_code` must be set");
    match policy {
        "idempotent_keep_existing" | "idempotent_replace" => {
            assert_eq!(
                exit_code, 0,
                "idempotent reentry policy must succeed cleanly (exit 0); got {exit_code}"
            );
        }
        "fail_if_initialized" => {
            assert_ne!(
                exit_code, 0,
                "fail_if_initialized policy must report a non-zero exit; got {exit_code}"
            );
        }
        _ => unreachable!(),
    }
}

#[test]
fn pkgmgr_init_isolation_pins_no_global_state() {
    let doc = crate::common::load_toml(&manifest_path());

    let isolation = doc
        .get("isolation")
        .and_then(|v| v.as_table())
        .expect("manifest.toml missing `[isolation]` block \
         (acceptance: \"No user home or global cache state is modified.\")");

    for flag in &[
        "forbid_writes_outside_project",
        "forbid_user_home_reads",
        "forbid_global_cache_reads",
        "forbid_global_cache_writes",
    ] {
        let value = isolation.get(*flag).and_then(|v| v.as_bool());
        assert_eq!(
            value,
            Some(true),
            "`[isolation].{flag}` must be `true` so the gate refuses fixtures \
             that touch global state"
        );
    }
}

#[test]
fn pkgmgr_init_runner_contract_declares_outcome_keys() {
    let doc = crate::common::load_toml(&manifest_path());
    let contract = doc
        .get("runner_contract")
        .and_then(|v| v.as_table())
        .expect("manifest.toml missing `[runner_contract]` block");

    let keys: BTreeSet<&str> = contract
        .get("keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &["outcome", "project_path", "files_created", "exit_code"] {
        assert!(
            keys.contains(*required),
            "`[runner_contract].keys` must include `{required}`; got {keys:?}"
        );
    }

    let outcomes: BTreeSet<&str> = contract
        .get("outcome_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        outcomes.contains("pass") && outcomes.contains("fail"),
        "`[runner_contract].outcome_values` must at least carry `pass` and `fail`; got {outcomes:?}"
    );
}

#[test]
fn pkgmgr_profile_links_to_init_fixture_directory() {
    let doc = crate::common::load_toml(&profile_path());
    let init = doc
        .get("families")
        .and_then(|v| v.get("init"))
        .and_then(|v| v.as_table())
        .expect("validation/profiles/package_manager.toml missing `[families.init]`");

    let source = init
        .get("source")
        .and_then(|v| v.as_str())
        .expect("`[families.init].source` must be set");
    assert_eq!(
        source, "tests/governance/gates/pkgmgr/init",
        "init family must point at tests/governance/gates/pkgmgr/init; got {source:?}"
    );

    let kind = init
        .get("kind")
        .and_then(|v| v.as_str())
        .expect("`[families.init].kind` must be set");
    assert_eq!(
        kind, "pkgmgr_init",
        "init family kind must be `pkgmgr_init` (matches manifest `fixture` field)"
    );
}
