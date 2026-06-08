//! Schema gate for the MVP required-correctness profile manifest —
//! closes #2818.
//!
//! Acceptance (issue #2818):
//!
//!   1. Required correctness profile fails if any required fixture is
//!      skipped. `[policy].fail_on_required_skip == true` and
//!      `[policy].excluded_outcomes` includes `skip` (so a skip on a
//!      required entry never tallies as pass).
//!   2. Summary names each failed or missing fixture.
//!      `[summary].include_failed && [summary].include_missing` are
//!      both true; `[summary].fields` carries `family`, `id`, and
//!      `outcome`.
//!   3. Manifest can be consumed by worker scripts. Three input
//!      families (`language_semantics`, `cpython_seeds`,
//!      `stdlib_behavior`) declared with `kind`, `source`, and
//!      `selector`; `[runner_contract]` declares the JSON keys
//!      workers must emit per family; every source path resolves.
//!
//! Cheap test — single TOML read + a handful of field walks. Runs in
//! well under a second; stays in the default `cargo test -p mamba` set.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const EXPECTED_BUCKETS: &[&str] = &["required", "optional", "xfail", "blocker"];

const REQUIRED_FAMILIES: &[&str] = &["language_semantics", "cpython_seeds", "stdlib_behavior"];

const REQUIRED_EXCLUDED_OUTCOMES: &[&str] = &["ImportPass", "Stub", "skip", "xfail"];

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("profiles")
        .join("correctness.toml")
}

fn umbrella_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("mvp.toml")
}

fn crate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn correctness_profile_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("correctness"),
        "correctness.toml `profile` must be \"correctness\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2818),
        "correctness.toml `issue` must record #2818"
    );

    let buckets: BTreeSet<&str> = doc
        .get("buckets")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let expected: BTreeSet<&str> = EXPECTED_BUCKETS.iter().copied().collect();
    assert_eq!(
        buckets, expected,
        "correctness.toml `buckets` must list exactly {EXPECTED_BUCKETS:?}"
    );
}

#[test]
fn correctness_profile_policy_fails_on_required_skip() {
    let doc = load_toml(&manifest_path());
    let policy = doc
        .get("policy")
        .and_then(|v| v.as_table())
        .expect("correctness.toml missing `[policy]` block");

    let fail_on_skip = policy
        .get("fail_on_required_skip")
        .and_then(|v| v.as_bool())
        .expect("`[policy].fail_on_required_skip` must be a bool");
    assert!(
        fail_on_skip,
        "correctness.toml `[policy].fail_on_required_skip` must be true \
         (acceptance: \"Required correctness profile fails if any \
         required fixture is skipped.\")"
    );

    let excluded: BTreeSet<&str> = policy
        .get("excluded_outcomes")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for outcome in REQUIRED_EXCLUDED_OUTCOMES {
        assert!(
            excluded.contains(*outcome),
            "correctness.toml `[policy].excluded_outcomes` must include \
             {outcome:?} (acceptance: \"Exclude import-only, Stub, \
             xfail, and optional fixtures from pass counts.\"); got \
             {excluded:?}"
        );
    }

    let counted: BTreeSet<&str> = policy
        .get("counted_pass_outcomes")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let overlap: Vec<&str> = excluded.intersection(&counted).copied().collect();
    assert!(
        overlap.is_empty(),
        "counted_pass_outcomes and excluded_outcomes must be disjoint; \
         shared: {overlap:?}"
    );

    let required: BTreeSet<&str> = policy
        .get("release_required_buckets")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        required.contains("required"),
        "`[policy].release_required_buckets` must include `required`; \
         got {required:?}"
    );
}

#[test]
fn correctness_profile_summary_names_failures_and_missing() {
    let doc = load_toml(&manifest_path());
    let summary = doc
        .get("summary")
        .and_then(|v| v.as_table())
        .expect("correctness.toml missing `[summary]` block");

    let include_failed = summary
        .get("include_failed")
        .and_then(|v| v.as_bool())
        .expect("`[summary].include_failed` must be a bool");
    assert!(
        include_failed,
        "`[summary].include_failed` must be true (acceptance: \
         \"Summary names each failed or missing fixture.\")"
    );

    let include_missing = summary
        .get("include_missing")
        .and_then(|v| v.as_bool())
        .expect("`[summary].include_missing` must be a bool");
    assert!(
        include_missing,
        "`[summary].include_missing` must be true (acceptance: \
         \"Summary names each failed or missing fixture.\")"
    );

    let fields: BTreeSet<&str> = summary
        .get("fields")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in ["family", "id", "outcome"] {
        assert!(
            fields.contains(required),
            "`[summary].fields` must include {required:?} so the worker \
             names each failed/missing fixture; got {fields:?}"
        );
    }
}

#[test]
fn correctness_profile_families_are_declared_and_consumable() {
    let doc = load_toml(&manifest_path());
    let families = doc
        .get("families")
        .and_then(|v| v.as_table())
        .expect("correctness.toml missing `[families]` table");

    for family in REQUIRED_FAMILIES {
        let entry = families
            .get(*family)
            .and_then(|v| v.as_table())
            .unwrap_or_else(|| panic!("correctness.toml missing `[families.{family}]`"));

        let kind = entry
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("`[families.{family}].kind` must be a non-empty string"));
        assert!(!kind.is_empty(), "family {family}: kind must be non-empty");

        let source = entry
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("`[families.{family}].source` must be set"));
        let resolved = crate_root().join(source);
        assert!(
            resolved.exists(),
            "family {family}: source {} does not exist (acceptance: \
             \"Manifest can be consumed by worker scripts.\")",
            resolved.display()
        );

        let selector = entry
            .get("selector")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("`[families.{family}].selector` must be set"));
        assert!(
            !selector.is_empty(),
            "family {family}: selector must be non-empty so the runner \
             knows how to filter the source"
        );
    }
}

#[test]
fn correctness_profile_runner_contract_covers_every_family() {
    let doc = load_toml(&manifest_path());
    let contract = doc
        .get("runner_contract")
        .and_then(|v| v.as_table())
        .expect("correctness.toml missing `[runner_contract]` block");

    for family in REQUIRED_FAMILIES {
        let keys = contract
            .get(*family)
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| {
                panic!("`[runner_contract].{family}` must be an array of JSON keys")
            });
        let keys: BTreeSet<&str> = keys.iter().filter_map(|v| v.as_str()).collect();
        for required in ["passed", "failed", "missing"] {
            assert!(
                keys.contains(required),
                "`[runner_contract].{family}` must include {required:?} so \
                 worker output is uniform across families; got {keys:?}"
            );
        }
    }
}

#[test]
fn mvp_umbrella_links_to_correctness_manifest() {
    let doc = load_toml(&umbrella_path());
    let entry = doc
        .get("profiles")
        .and_then(|v| v.get("correctness"))
        .and_then(|v| v.as_table())
        .expect("validation/mvp.toml missing `[profiles.correctness]`");

    let manifest = entry
        .get("manifest")
        .and_then(|v| v.as_str())
        .expect("`[profiles.correctness].manifest` must be set");
    assert_eq!(
        manifest, "profiles/correctness.toml",
        "umbrella must point at profiles/correctness.toml; got {manifest:?}"
    );

    let issue = entry
        .get("issue")
        .and_then(|v| v.as_integer())
        .expect("`[profiles.correctness].issue` must record the issue id");
    assert_eq!(issue, 2818, "correctness profile owner issue must be #2818");
}
