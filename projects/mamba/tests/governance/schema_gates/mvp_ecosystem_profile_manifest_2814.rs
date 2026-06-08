//! Schema gate for the MVP ecosystem profile manifest — closes #2814.
//!
//! Acceptance (issue #2814):
//!
//!   1. Required ecosystem fixture failure blocks the profile.
//!      `[policy].release_required_buckets` must include `required`.
//!   2. Optional failures are reported but do not block.
//!      `[policy].report_only_buckets` must include `optional`.
//!   3. xfail/blocker items do not count as pass. `[policy]
//!      .non_pass_outcomes` must include `xfail` (and any other
//!      currently-known non-pass outcomes).
//!   4. Default execution offline and deterministic.
//!      `[policy].offline` and `[policy].deterministic` are both true.
//!
//! Also enforces consistency with the source-of-truth fixture manifest
//! (`ecosystem_fixture_manifest.toml`, #2551): every entry's
//! `expected_outcome` must be a key in this profile's `[outcome_map]`,
//! and any `xfail` entry must declare a blocker (per #2555).
//!
//! Cheap test — two TOML reads + table walk. Runs in well under a
//! second; stays in the default `cargo test -p mamba` set.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const EXPECTED_BUCKETS: &[&str] = &["required", "optional", "xfail", "blocker"];

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("profiles")
        .join("ecosystem.toml")
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
fn ecosystem_profile_manifest_declares_canonical_buckets() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("ecosystem"),
        "ecosystem.toml `profile` must be \"ecosystem\""
    );

    let buckets = doc
        .get("buckets")
        .and_then(|v| v.as_array())
        .expect("ecosystem.toml missing required `buckets` array");
    let buckets: BTreeSet<&str> = buckets.iter().filter_map(|v| v.as_str()).collect();
    let expected: BTreeSet<&str> = EXPECTED_BUCKETS.iter().copied().collect();
    assert_eq!(
        buckets, expected,
        "ecosystem.toml `buckets` must list exactly {EXPECTED_BUCKETS:?}; \
         issue #2814 acceptance requires required/optional/xfail/blocker categories"
    );
}

#[test]
fn ecosystem_profile_policy_satisfies_acceptance() {
    let doc = load_toml(&manifest_path());
    let policy = doc
        .get("policy")
        .and_then(|v| v.as_table())
        .expect("ecosystem.toml missing `[policy]` block");

    let offline = policy
        .get("offline")
        .and_then(|v| v.as_bool())
        .expect("ecosystem.toml `[policy].offline` must be a bool");
    assert!(
        offline,
        "ecosystem profile must default to offline (acceptance: \
         \"Default execution offline and deterministic.\")"
    );

    let deterministic = policy
        .get("deterministic")
        .and_then(|v| v.as_bool())
        .expect("ecosystem.toml `[policy].deterministic` must be a bool");
    assert!(
        deterministic,
        "ecosystem profile must default to deterministic"
    );

    let required: BTreeSet<&str> = policy
        .get("release_required_buckets")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        required.contains("required"),
        "ecosystem.toml `[policy].release_required_buckets` must include \
         `required` (acceptance: \"Required ecosystem fixture failure \
         blocks the profile.\"); got {required:?}"
    );

    let report_only: BTreeSet<&str> = policy
        .get("report_only_buckets")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        report_only.contains("optional"),
        "ecosystem.toml `[policy].report_only_buckets` must include \
         `optional` (acceptance: \"Optional failures are reported but do \
         not block.\"); got {report_only:?}"
    );

    let non_pass: BTreeSet<&str> = policy
        .get("non_pass_outcomes")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        non_pass.contains("xfail"),
        "ecosystem.toml `[policy].non_pass_outcomes` must include \
         `xfail` (acceptance: \"xfail/blocker items do not count as \
         pass.\"); got {non_pass:?}"
    );

    let buckets_required: BTreeSet<&str> = doc
        .get("buckets")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let overlap: Vec<&str> = required.intersection(&report_only).copied().collect();
    assert!(
        overlap.is_empty(),
        "release_required_buckets and report_only_buckets must be disjoint; \
         shared: {overlap:?}"
    );
    let coverage: BTreeSet<&str> = required.union(&report_only).copied().collect();
    let uncovered: Vec<&str> = buckets_required.difference(&coverage).copied().collect();
    assert!(
        uncovered.is_empty(),
        "every declared bucket must be either release-required or \
         report-only; unassigned: {uncovered:?}"
    );
}

#[test]
fn ecosystem_profile_source_manifest_resolves() {
    let doc = load_toml(&manifest_path());
    let source = doc
        .get("source_manifest")
        .and_then(|v| v.as_str())
        .expect("ecosystem.toml missing required `source_manifest` field");

    let resolved = manifest_path().parent().unwrap().join(source);
    let canonical = resolved
        .canonicalize()
        .unwrap_or_else(|e| panic!("source_manifest {source:?} cannot be resolved: {e}"));
    assert!(
        canonical.exists(),
        "ecosystem.toml `source_manifest` ({}) must point at an existing file",
        canonical.display()
    );

    // Sanity check that the path lands inside the mamba project — guards
    // against accidental `..` escapes that would resolve outside the crate.
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .expect("crate root canonicalize failed");
    assert!(
        canonical.starts_with(&crate_root),
        "source_manifest resolved outside crate root: {} not under {}",
        canonical.display(),
        crate_root.display()
    );
}

#[test]
fn ecosystem_profile_outcome_map_covers_source_manifest() {
    let profile_doc = load_toml(&manifest_path());
    let outcome_map: BTreeSet<&str> = profile_doc
        .get("outcome_map")
        .and_then(|v| v.as_table())
        .expect("ecosystem.toml missing `[outcome_map]`")
        .keys()
        .map(|s| s.as_str())
        .collect();

    let source_rel = profile_doc
        .get("source_manifest")
        .and_then(|v| v.as_str())
        .expect("ecosystem.toml missing `source_manifest`");
    let source_path = manifest_path().parent().unwrap().join(source_rel);
    let source_doc = load_toml(&source_path);

    let fixtures = source_doc
        .get("fixtures")
        .and_then(|v| v.as_table())
        .expect("source manifest missing `[fixtures]` table");

    let mut unmapped: Vec<String> = Vec::new();
    let mut xfail_missing_blocker: Vec<String> = Vec::new();
    for (id, entry) in fixtures {
        let table = match entry.as_table() {
            Some(t) => t,
            None => continue,
        };
        let outcome = table
            .get("expected_outcome")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !outcome_map.contains(outcome) {
            unmapped.push(format!("{id} -> {outcome:?}"));
        }
        if outcome == "xfail"
            && !table
                .get("blocker")
                .and_then(|v| v.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false)
        {
            xfail_missing_blocker.push(id.clone());
        }
    }

    assert!(
        unmapped.is_empty(),
        "source manifest entries have outcomes missing from \
         ecosystem.toml `[outcome_map]`:\n  - {}",
        unmapped.join("\n  - ")
    );
    assert!(
        xfail_missing_blocker.is_empty(),
        "xfail source manifest entries must declare a `blocker` \
         (acceptance: \"xfail/blocker items do not count as pass.\"):\n  - {}",
        xfail_missing_blocker.join("\n  - ")
    );
}

#[test]
fn mvp_umbrella_links_to_ecosystem_manifest() {
    let doc = load_toml(&umbrella_path());
    let entry = doc
        .get("profiles")
        .and_then(|v| v.get("ecosystem"))
        .and_then(|v| v.as_table())
        .expect("validation/mvp.toml missing `[profiles.ecosystem]`");

    let manifest = entry.get("manifest").and_then(|v| v.as_str()).expect(
        "validation/mvp.toml `[profiles.ecosystem].manifest` must be set \
             so workers can locate ecosystem.toml",
    );
    assert_eq!(
        manifest, "profiles/ecosystem.toml",
        "umbrella must point at profiles/ecosystem.toml; got {manifest:?}"
    );

    let resolved = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join(manifest);
    assert!(
        resolved.exists(),
        "ecosystem manifest path {} does not exist",
        resolved.display()
    );

    let issue = entry
        .get("issue")
        .and_then(|v| v.as_integer())
        .expect("validation/mvp.toml `[profiles.ecosystem].issue` must record the issue id");
    assert_eq!(issue, 2814, "ecosystem profile owner issue must be #2814");
}
