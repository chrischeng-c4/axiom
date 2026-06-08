//! Schema gate for the MVP package-manager profile manifest —
//! closes #2816.
//!
//! Acceptance (issue #2816):
//!
//!   1. Profile fails on any required offline workflow failure.
//!      `[policy].network == "offline"`, `[policy].index_source ==
//!      "frozen_local"`, `[policy].forbid_required_network == true`,
//!      and all eight workflow families are declared.
//!   2. Live network tests are reported as opt-in, not blocking
//!      defaults. `[live_network]` block sets `bucket = "optional"`
//!      and pins an `enabled_by` opt-in flag; live-network workflows
//!      must NOT appear in `release_required_buckets`.
//!   3. Summary names project, lockfile, and environment paths.
//!      `[summary].fields` includes `project_path`, `lockfile_path`,
//!      and `environment_path`.
//!
//! Cheap test — single TOML read + field walk. Stays in the default
//! `cargo test -p mamba` set; runs in well under a second.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const EXPECTED_BUCKETS: &[&str] = &["required", "optional", "xfail", "blocker"];

const REQUIRED_FAMILIES: &[&str] = &[
    "init", "add", "lock", "sync", "run", "install", "hash", "cache",
];

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("profiles")
        .join("package_manager.toml")
}

fn umbrella_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("mvp.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn package_manager_profile_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("package_manager"),
        "package_manager.toml `profile` must be \"package_manager\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2816),
        "package_manager.toml `issue` must record #2816"
    );

    let buckets: BTreeSet<&str> = doc
        .get("buckets")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let expected: BTreeSet<&str> = EXPECTED_BUCKETS.iter().copied().collect();
    assert_eq!(
        buckets, expected,
        "package_manager.toml `buckets` must list exactly {EXPECTED_BUCKETS:?}"
    );
}

#[test]
fn package_manager_profile_declares_all_workflow_families() {
    let doc = load_toml(&manifest_path());
    let families = doc
        .get("families")
        .and_then(|v| v.as_table())
        .expect("package_manager.toml missing `[families]` table");

    for family in REQUIRED_FAMILIES {
        let entry = families
            .get(*family)
            .and_then(|v| v.as_table())
            .unwrap_or_else(|| panic!("package_manager.toml missing `[families.{family}]`"));
        for required_field in ["kind", "source", "outcome_rule"] {
            let s = entry.get(required_field).and_then(|v| v.as_str());
            assert!(
                s.map(|s| !s.is_empty()).unwrap_or(false),
                "`[families.{family}].{required_field}` must be non-empty"
            );
        }
    }
}

#[test]
fn package_manager_profile_defaults_offline_with_frozen_local_indexes() {
    let doc = load_toml(&manifest_path());
    let policy = doc
        .get("policy")
        .and_then(|v| v.as_table())
        .expect("package_manager.toml missing `[policy]` block");

    let network = policy
        .get("network")
        .and_then(|v| v.as_str())
        .expect("`[policy].network` must be set");
    assert_eq!(
        network, "offline",
        "package_manager.toml `[policy].network` must be \"offline\" \
         (acceptance: \"Profile fails on any required offline workflow \
         failure.\"); got {network:?}"
    );

    let index_source = policy
        .get("index_source")
        .and_then(|v| v.as_str())
        .expect("`[policy].index_source` must be set");
    assert_eq!(
        index_source, "frozen_local",
        "package_manager.toml `[policy].index_source` must be \
         \"frozen_local\" (scope: \"Require all default package-manager \
         fixtures to use local frozen indexes.\"); got {index_source:?}"
    );

    let forbid_required_network = policy
        .get("forbid_required_network")
        .and_then(|v| v.as_bool())
        .expect("`[policy].forbid_required_network` must be a bool");
    assert!(
        forbid_required_network,
        "`[policy].forbid_required_network` must be true so a required \
         workflow can't slip through with a live-network call"
    );

    let required: BTreeSet<&str> = policy
        .get("release_required_buckets")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        required.contains("required"),
        "`[policy].release_required_buckets` must include `required`; got {required:?}"
    );
}

#[test]
fn package_manager_profile_live_network_is_opt_in_only() {
    let doc = load_toml(&manifest_path());

    let live = doc
        .get("live_network")
        .and_then(|v| v.as_table())
        .expect("package_manager.toml missing `[live_network]` block");

    let bucket = live
        .get("bucket")
        .and_then(|v| v.as_str())
        .expect("`[live_network].bucket` must be set");
    assert_eq!(
        bucket, "optional",
        "`[live_network].bucket` must be \"optional\" so live network \
         workflows never block (acceptance: \"Live network tests are \
         reported as opt-in, not blocking defaults.\"); got {bucket:?}"
    );

    let enabled_by = live
        .get("enabled_by")
        .and_then(|v| v.as_str())
        .expect("`[live_network].enabled_by` must name the opt-in flag");
    assert!(
        !enabled_by.is_empty() && enabled_by.starts_with("--"),
        "`[live_network].enabled_by` must be a CLI flag (e.g. \
         \"--include-live-network\"); got {enabled_by:?}"
    );

    let workflows: BTreeSet<&str> = live
        .get("workflows")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        !workflows.is_empty(),
        "`[live_network].workflows` must list at least one workflow"
    );

    let policy = doc
        .get("policy")
        .and_then(|v| v.as_table())
        .expect("policy block missing");
    let required_buckets: BTreeSet<&str> = policy
        .get("release_required_buckets")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        !required_buckets.contains(bucket),
        "live network bucket {bucket:?} must NOT appear in \
         `[policy].release_required_buckets` (would make live-network \
         failures gate the release); got {required_buckets:?}"
    );
}

#[test]
fn package_manager_profile_summary_names_project_lockfile_environment() {
    let doc = load_toml(&manifest_path());
    let summary = doc
        .get("summary")
        .and_then(|v| v.as_table())
        .expect("package_manager.toml missing `[summary]` block");

    let fields: BTreeSet<&str> = summary
        .get("fields")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in ["project_path", "lockfile_path", "environment_path"] {
        assert!(
            fields.contains(required),
            "`[summary].fields` must include {required:?} (acceptance: \
             \"Summary names project, lockfile, and environment \
             paths.\"); got {fields:?}"
        );
    }
}

#[test]
fn package_manager_profile_runner_contract_covers_every_family() {
    let doc = load_toml(&manifest_path());
    let contract = doc
        .get("runner_contract")
        .and_then(|v| v.as_table())
        .expect("package_manager.toml missing `[runner_contract]` block");

    for family in REQUIRED_FAMILIES {
        let keys: BTreeSet<&str> = contract
            .get(*family)
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        for required in ["passed", "failed", "missing"] {
            assert!(
                keys.contains(required),
                "`[runner_contract].{family}` must include {required:?}; got {keys:?}"
            );
        }
    }
}

#[test]
fn mvp_umbrella_links_to_package_manager_manifest() {
    let doc = load_toml(&umbrella_path());
    let entry = doc
        .get("profiles")
        .and_then(|v| v.get("package_manager"))
        .and_then(|v| v.as_table())
        .expect("validation/mvp.toml missing `[profiles.package_manager]`");

    let manifest = entry
        .get("manifest")
        .and_then(|v| v.as_str())
        .expect("`[profiles.package_manager].manifest` must be set");
    assert_eq!(
        manifest, "profiles/package_manager.toml",
        "umbrella must point at profiles/package_manager.toml; got {manifest:?}"
    );

    let issue = entry
        .get("issue")
        .and_then(|v| v.as_integer())
        .expect("`[profiles.package_manager].issue` must record the issue id");
    assert_eq!(
        issue, 2816,
        "package_manager profile owner issue must be #2816"
    );
}
