//! Schema gate for the MVP mambalibs profile manifest — closes #2817.
//!
//! Acceptance (issue #2817):
//!
//!   1. Profile fails if build succeeds but import fails.
//!      `[policy].pair_build_and_import == true` plus a
//!      `[pairs.build_then_import]` block linking the `build` family
//!      to the `import` family with rule
//!      `right_must_succeed_when_left_succeeds`. Both families must
//!      be declared.
//!   2. Profile fails if diagnostics fixtures crash or pass silently.
//!      `[families.diagnostics].outcome_rule ==
//!      "must_emit_diagnostic"` and `[policy].forbid_silent_pass`
//!      includes `diagnostics`; the diagnostics family's
//!      `runner_contract` carries a `silent_pass` key the runner
//!      uses to surface silent passes.
//!   3. Summary names dependency and artifact identity.
//!      `[summary].fields` includes `dependency` and
//!      `artifact_identity`.
//!
//! Also enforces profile-level isolation (no global caches) and the
//! canonical four-bucket layout shared with the other MVP profiles.
//!
//! Cheap test — single TOML read + field walk. Runs in well under a
//! second; stays in the default `cargo test -p mamba` set.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const EXPECTED_BUCKETS: &[&str] = &["required", "optional", "xfail", "blocker"];

const REQUIRED_FAMILIES: &[&str] = &[
    "build",
    "lock",
    "artifact_layout",
    "import",
    "diagnostics",
    "type_roundtrip",
];

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("profiles")
        .join("mambalibs.toml")
}

fn umbrella_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("mvp.toml")
}

#[test]
fn mambalibs_profile_manifest_header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());

    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("mambalibs"),
        "mambalibs.toml `profile` must be \"mambalibs\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2817),
        "mambalibs.toml `issue` must record #2817"
    );

    let buckets: BTreeSet<&str> = doc
        .get("buckets")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let expected: BTreeSet<&str> = EXPECTED_BUCKETS.iter().copied().collect();
    assert_eq!(
        buckets, expected,
        "mambalibs.toml `buckets` must list exactly {EXPECTED_BUCKETS:?}"
    );
}

#[test]
fn mambalibs_profile_declares_all_pipeline_families() {
    let doc = crate::common::load_toml(&manifest_path());
    let families = doc
        .get("families")
        .and_then(|v| v.as_table())
        .expect("mambalibs.toml missing `[families]` table");

    for family in REQUIRED_FAMILIES {
        let entry = families
            .get(*family)
            .and_then(|v| v.as_table())
            .unwrap_or_else(|| panic!("mambalibs.toml missing `[families.{family}]`"));
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
fn mambalibs_profile_pairs_build_with_import() {
    let doc = crate::common::load_toml(&manifest_path());

    let policy = doc
        .get("policy")
        .and_then(|v| v.as_table())
        .expect("mambalibs.toml missing `[policy]` block");

    let pair_flag = policy
        .get("pair_build_and_import")
        .and_then(|v| v.as_bool())
        .expect("`[policy].pair_build_and_import` must be a bool");
    assert!(
        pair_flag,
        "`[policy].pair_build_and_import` must be true (acceptance: \
         \"Profile fails if build succeeds but import fails.\")"
    );

    let pair = doc
        .get("pairs")
        .and_then(|v| v.get("build_then_import"))
        .and_then(|v| v.as_table())
        .expect("mambalibs.toml missing `[pairs.build_then_import]`");

    let left = pair
        .get("left")
        .and_then(|v| v.as_str())
        .expect("`[pairs.build_then_import].left` must be set");
    let right = pair
        .get("right")
        .and_then(|v| v.as_str())
        .expect("`[pairs.build_then_import].right` must be set");
    let rule = pair
        .get("rule")
        .and_then(|v| v.as_str())
        .expect("`[pairs.build_then_import].rule` must be set");

    assert_eq!(left, "build", "build_then_import.left must be `build`");
    assert_eq!(right, "import", "build_then_import.right must be `import`");
    assert_eq!(
        rule, "right_must_succeed_when_left_succeeds",
        "build_then_import.rule must enforce `right_must_succeed_when_left_succeeds`"
    );

    // Both families referenced by the pair must exist as declared
    // families.
    let families = doc
        .get("families")
        .and_then(|v| v.as_table())
        .expect("families table missing");
    for f in [left, right] {
        assert!(
            families.contains_key(f),
            "`[pairs.build_then_import]` references family {f:?} which is not declared"
        );
    }
}

#[test]
fn mambalibs_profile_diagnostics_must_not_silently_pass() {
    let doc = crate::common::load_toml(&manifest_path());

    let policy = doc
        .get("policy")
        .and_then(|v| v.as_table())
        .expect("mambalibs.toml missing `[policy]` block");

    let forbid_silent: BTreeSet<&str> = policy
        .get("forbid_silent_pass")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        forbid_silent.contains("diagnostics"),
        "`[policy].forbid_silent_pass` must include `diagnostics` \
         (acceptance: \"Profile fails if diagnostics fixtures crash or \
         pass silently.\"); got {forbid_silent:?}"
    );

    let diag = doc
        .get("families")
        .and_then(|v| v.get("diagnostics"))
        .and_then(|v| v.as_table())
        .expect("families.diagnostics missing");
    let rule = diag
        .get("outcome_rule")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert_eq!(
        rule, "must_emit_diagnostic",
        "`[families.diagnostics].outcome_rule` must be \
         `must_emit_diagnostic` so a silent exit-0 fails the profile; \
         got {rule:?}"
    );

    let diag_contract: BTreeSet<&str> = doc
        .get("runner_contract")
        .and_then(|v| v.get("diagnostics"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        diag_contract.contains("silent_pass"),
        "`[runner_contract].diagnostics` must include `silent_pass` so \
         the runner surfaces silent-pass counts to the summary; got \
         {diag_contract:?}"
    );
}

#[test]
fn mambalibs_profile_summary_names_dependency_and_artifact() {
    let doc = crate::common::load_toml(&manifest_path());
    let summary = doc
        .get("summary")
        .and_then(|v| v.as_table())
        .expect("mambalibs.toml missing `[summary]` block");

    let fields: BTreeSet<&str> = summary
        .get("fields")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    for required in ["dependency", "artifact_identity"] {
        assert!(
            fields.contains(required),
            "`[summary].fields` must include {required:?} (acceptance: \
             \"Summary names dependency and artifact identity.\"); got \
             {fields:?}"
        );
    }
}

#[test]
fn mambalibs_profile_isolates_from_global_caches() {
    let doc = crate::common::load_toml(&manifest_path());
    let policy = doc
        .get("policy")
        .and_then(|v| v.as_table())
        .expect("mambalibs.toml missing `[policy]` block");

    let isolated: BTreeSet<&str> = policy
        .get("isolated_from")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        !isolated.is_empty(),
        "`[policy].isolated_from` must list at least one cache the \
         runner clears before each run (scope: \"Keep profile isolated \
         from global caches.\")"
    );
}

#[test]
fn mvp_umbrella_links_to_mambalibs_manifest() {
    let doc = crate::common::load_toml(&umbrella_path());
    let entry = doc
        .get("profiles")
        .and_then(|v| v.get("mambalibs"))
        .and_then(|v| v.as_table())
        .expect("validation/mvp.toml missing `[profiles.mambalibs]`");

    let manifest = entry
        .get("manifest")
        .and_then(|v| v.as_str())
        .expect("`[profiles.mambalibs].manifest` must be set");
    assert_eq!(
        manifest, "profiles/mambalibs.toml",
        "umbrella must point at profiles/mambalibs.toml; got {manifest:?}"
    );

    let issue = entry
        .get("issue")
        .and_then(|v| v.as_integer())
        .expect("`[profiles.mambalibs].issue` must record the issue id");
    assert_eq!(issue, 2817, "mambalibs profile owner issue must be #2817");
}
