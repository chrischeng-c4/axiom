//! Schema gate for the MVP release-blocking profile umbrella — closes #2775.
//!
//! Parent: #2526 (MVP test completeness and release gates).
//!
//! #2775 is the umbrella for "MVP release blocking CI profiles". Its six
//! child issues (#2814 ecosystem, #2815 performance, #2816 package_manager,
//! #2817 mambalibs, #2818 correctness, #2819 smoke) each shipped a per-
//! profile manifest. This gate locks the *aggregation* layer:
//! `validation/profiles/release_gate.toml` enumerates the six required
//! profiles, pins their run order, fixes the worker invocation pattern,
//! and locks the release-summary shape grouped by MVP objective.
//!
//! Acceptance (issue #2775):
//!
//!   1. Child leaf issues define profile membership and pass/fail policy.
//!      Every profile referenced in `[profiles.*]` MUST resolve to an
//!      existing manifest file whose `parent_issue` is 2775.
//!   2. CI or worker command names are documented and deterministic.
//!      `[ci_runner].command` + `format_flag` + `exit_codes` are required
//!      and locked to the values the runner (#2821) will consume.
//!   3. Release summary exposes blockers by MVP objective. `[summary]`
//!      MUST declare all four MVP objectives, every profile MUST claim
//!      one, and the per-blocker / per-profile record shapes are pinned.
//!
//! Cheap test — a handful of TOML reads + string checks. Stays in the
//! default `cargo test -p mamba` set; runs well under a second.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const EXPECTED_PROFILES: &[&str] = &[
    "correctness",
    "ecosystem",
    "mambalibs",
    "package_manager",
    "performance",
    "smoke",
];

const EXPECTED_PROFILE_ISSUE: &[(&str, i64)] = &[
    ("correctness", 2818),
    ("ecosystem", 2814),
    ("mambalibs", 2817),
    ("package_manager", 2816),
    ("performance", 2815),
    ("smoke", 2819),
];

const EXPECTED_OBJECTIVES: &[&str] = &["compatibility", "distribution", "ecosystem", "performance"];

const CANONICAL_BUCKETS: &[&str] = &["required", "optional", "xfail", "blocker"];

const NON_PASS_OUTCOMES: &[&str] = &[
    "Fail",
    "ImportPass",
    "Stub",
    "Timeout",
    "blocked",
    "skip",
    "xfail",
];

const UMBRELLA_ISSUE: i64 = 2775;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn manifest_path() -> PathBuf {
    crate_root()
        .join("validation")
        .join("profiles")
        .join("release_gate.toml")
}

fn require_table<'a>(doc: &'a toml::Value, key: &str) -> &'a toml::value::Table {
    doc.get(key)
        .and_then(|v| v.as_table())
        .unwrap_or_else(|| panic!("release_gate.toml missing required table `{key}`"))
}

fn require_str<'a>(table: &'a toml::value::Table, key: &str, ctx: &str) -> &'a str {
    table
        .get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| panic!("{ctx}: missing or empty required string `{key}`"))
}

fn require_int(table: &toml::value::Table, key: &str, ctx: &str) -> i64 {
    table
        .get(key)
        .and_then(|v| v.as_integer())
        .unwrap_or_else(|| panic!("{ctx}: missing required integer `{key}`"))
}

fn require_bool(table: &toml::value::Table, key: &str, ctx: &str) -> bool {
    table
        .get(key)
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| panic!("{ctx}: missing required bool `{key}`"))
}

fn require_array<'a>(table: &'a toml::value::Table, key: &str, ctx: &str) -> &'a Vec<toml::Value> {
    table
        .get(key)
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("{ctx}: missing required array `{key}`"))
}

fn collect_strings(arr: &[toml::Value], ctx: &str) -> BTreeSet<String> {
    arr.iter()
        .map(|v| {
            v.as_str()
                .unwrap_or_else(|| panic!("{ctx}: non-string entry"))
                .to_string()
        })
        .collect()
}

#[test]
fn release_gate_manifest_header_pins_umbrella_issue_and_parent() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(
        doc.get("version").and_then(|v| v.as_integer()),
        Some(1),
        "release_gate.toml `version` must be 1"
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("release_gate"),
        "release_gate.toml `profile` must be \"release_gate\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(UMBRELLA_ISSUE),
        "release_gate.toml `issue` must equal umbrella #2775"
    );
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2526),
        "release_gate.toml `parent_issue` must equal MVP root #2526"
    );
}

#[test]
fn release_gate_manifest_lists_exactly_the_six_release_blocking_profiles() {
    let doc = crate::common::load_toml(&manifest_path());
    let profiles = require_table(&doc, "profiles");
    let got: BTreeSet<&str> = profiles.keys().map(|s| s.as_str()).collect();
    let want: BTreeSet<&str> = EXPECTED_PROFILES.iter().copied().collect();
    assert_eq!(
        got,
        want,
        "release_gate.toml `[profiles.*]` must be exactly the six \
         release-blocking profiles. Missing: {:?}. Extra: {:?}.",
        want.difference(&got).collect::<Vec<_>>(),
        got.difference(&want).collect::<Vec<_>>()
    );
}

#[test]
fn each_referenced_profile_manifest_file_exists_and_points_back_to_2775() {
    // Acceptance 1: "Child leaf issues define profile membership and
    // pass/fail policy." We assert every referenced manifest resolves
    // to a real file AND that file's `parent_issue` is 2775, closing
    // the umbrella loop.
    let doc = crate::common::load_toml(&manifest_path());
    let profiles = require_table(&doc, "profiles");
    let dir = manifest_path()
        .parent()
        .expect("release_gate.toml has a parent")
        .to_path_buf();

    for (name, value) in profiles.iter() {
        let table = value
            .as_table()
            .unwrap_or_else(|| panic!("profile `{name}` is not a TOML table"));
        let ctx = format!("profile `{name}`");
        let manifest_rel = require_str(table, "manifest", &ctx);
        let manifest_abs = dir.join(manifest_rel);
        assert!(
            manifest_abs.is_file(),
            "profile `{name}`: manifest path {} does not exist",
            manifest_abs.display()
        );
        let child = crate::common::load_toml(&manifest_abs);
        let child_parent = child
            .get("parent_issue")
            .and_then(|v| v.as_integer())
            .unwrap_or_else(|| {
                panic!(
                    "child manifest {} missing `parent_issue`",
                    manifest_abs.display()
                )
            });
        assert_eq!(
            child_parent,
            UMBRELLA_ISSUE,
            "child manifest {} `parent_issue` = {child_parent}, must be {UMBRELLA_ISSUE}",
            manifest_abs.display()
        );
    }
}

#[test]
fn each_profile_entry_carries_required_fields_and_correct_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let profiles = require_table(&doc, "profiles");
    let expected: BTreeMap<&str, i64> = EXPECTED_PROFILE_ISSUE.iter().copied().collect();

    for (name, value) in profiles.iter() {
        let table = value.as_table().expect("profile is a table");
        let ctx = format!("profile `{name}`");
        let issue = require_int(table, "issue", &ctx);
        let want_issue = expected
            .get(name.as_str())
            .copied()
            .unwrap_or_else(|| panic!("unexpected profile `{name}`"));
        assert_eq!(
            issue, want_issue,
            "profile `{name}` declared issue {issue}, expected {want_issue}"
        );
        let objective = require_str(table, "objective", &ctx);
        assert!(
            EXPECTED_OBJECTIVES.contains(&objective),
            "profile `{name}` objective {objective:?} not in expected set {:?}",
            EXPECTED_OBJECTIVES
        );
        let required = require_bool(table, "required", &ctx);
        assert!(
            required,
            "profile `{name}` `required` must be true — release_gate.toml lists \
             only blocking profiles"
        );
        let _description = require_str(table, "description", &ctx);
    }
}

#[test]
fn every_mvp_objective_is_claimed_by_at_least_one_profile() {
    // Acceptance 3: "Release summary exposes blockers by MVP objective."
    // Every declared objective in `[summary].objectives` must be the
    // objective of at least one profile, else the summary has dead keys.
    let doc = crate::common::load_toml(&manifest_path());
    let summary = require_table(&doc, "summary");
    let declared = collect_strings(
        require_array(summary, "objectives", "[summary]"),
        "[summary].objectives",
    );
    assert_eq!(
        declared,
        EXPECTED_OBJECTIVES
            .iter()
            .map(|s| s.to_string())
            .collect::<BTreeSet<_>>(),
        "[summary].objectives must equal the canonical four MVP objectives"
    );

    let profiles = require_table(&doc, "profiles");
    let claimed: BTreeSet<String> = profiles
        .values()
        .filter_map(|v| v.as_table())
        .filter_map(|t| t.get("objective").and_then(|v| v.as_str()))
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        claimed, declared,
        "every declared objective must be claimed by at least one profile (and \
         no profile may declare an objective outside the canonical set)"
    );
}

#[test]
fn run_order_is_a_permutation_of_declared_profiles() {
    // Run order must reference exactly the profiles declared in
    // [profiles.*] — no missing entries (would silently skip a profile)
    // and no extras (typo would be a no-op).
    let doc = crate::common::load_toml(&manifest_path());
    let run_order = require_table(&doc, "run_order");
    let sequence = require_array(run_order, "sequence", "[run_order]");
    let listed: Vec<String> = sequence
        .iter()
        .map(|v| {
            v.as_str()
                .expect("[run_order].sequence entries must be strings")
                .to_string()
        })
        .collect();
    let listed_set: BTreeSet<&str> = listed.iter().map(|s| s.as_str()).collect();
    let want: BTreeSet<&str> = EXPECTED_PROFILES.iter().copied().collect();
    assert_eq!(
        listed_set, want,
        "[run_order].sequence must be a permutation of the six release-blocking \
         profile ids"
    );
    assert_eq!(
        listed.len(),
        EXPECTED_PROFILES.len(),
        "[run_order].sequence must not contain duplicates"
    );
    assert_eq!(
        listed.first().map(|s| s.as_str()),
        Some("smoke"),
        "smoke must run first so a broken compile blocks every deeper gate"
    );
}

#[test]
fn policy_bans_skip_xfail_stub_and_importpass_as_pass() {
    // Acceptance: "release-blocking rules do not count skips, xfails, or
    // stubs as pass" (from #2775 scope).
    let doc = crate::common::load_toml(&manifest_path());
    let policy = require_table(&doc, "policy");
    let banned = collect_strings(
        require_array(policy, "non_pass_outcomes", "[policy]"),
        "[policy].non_pass_outcomes",
    );
    let want: BTreeSet<String> = NON_PASS_OUTCOMES.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        banned, want,
        "[policy].non_pass_outcomes must be exactly {:?} — skip/xfail/Stub/ImportPass \
         MUST be banned across every required profile",
        NON_PASS_OUTCOMES
    );
    let required_bucket = require_str(policy, "required_bucket", "[policy]");
    assert_eq!(
        required_bucket, "required",
        "[policy].required_bucket must be \"required\""
    );
    let canonical = collect_strings(
        require_array(policy, "canonical_buckets", "[policy]"),
        "[policy].canonical_buckets",
    );
    let want_buckets: BTreeSet<String> = CANONICAL_BUCKETS.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        canonical, want_buckets,
        "[policy].canonical_buckets must equal {:?}",
        CANONICAL_BUCKETS
    );
    let crash_blocker = require_bool(policy, "treat_required_crash_as_blocker", "[policy]");
    assert!(
        crash_blocker,
        "[policy].treat_required_crash_as_blocker must be true — a crashed \
         required profile MUST count as a blocker, not a skip"
    );
}

#[test]
fn ci_runner_block_locks_command_format_flag_and_exit_codes() {
    // Acceptance 2: "CI or worker command names are documented and
    // deterministic." Lock the entire `[ci_runner]` contract so the
    // runner (#2821) can be wired against fixed names.
    let doc = crate::common::load_toml(&manifest_path());
    let ci = require_table(&doc, "ci_runner");
    assert_eq!(
        require_str(ci, "command", "[ci_runner]"),
        "run-profile",
        "[ci_runner].command must be \"run-profile\""
    );
    assert_eq!(
        require_str(ci, "format_flag", "[ci_runner]"),
        "--format json",
        "[ci_runner].format_flag must be \"--format json\""
    );
    let _script = require_str(ci, "script", "[ci_runner]");

    let exit = ci
        .get("exit_codes")
        .and_then(|v| v.as_table())
        .expect("[ci_runner].exit_codes table missing");
    assert_eq!(
        exit.get("pass").and_then(|v| v.as_integer()),
        Some(0),
        "[ci_runner].exit_codes.pass must be 0"
    );
    assert_eq!(
        exit.get("blocker").and_then(|v| v.as_integer()),
        Some(1),
        "[ci_runner].exit_codes.blocker must be 1"
    );
    assert_eq!(
        exit.get("harness_error").and_then(|v| v.as_integer()),
        Some(101),
        "[ci_runner].exit_codes.harness_error must be 101"
    );
}

#[test]
fn summary_blocker_record_required_fields_are_pinned() {
    // Acceptance 3 detail: each blocker record MUST carry profile, id,
    // outcome — the minimum a downstream consumer needs to identify
    // and act on a blocker.
    let doc = crate::common::load_toml(&manifest_path());
    let summary = require_table(&doc, "summary");
    let br = summary
        .get("blocker_record")
        .and_then(|v| v.as_table())
        .expect("[summary.blocker_record] missing");
    let required = collect_strings(
        require_array(br, "required_fields", "[summary.blocker_record]"),
        "[summary.blocker_record].required_fields",
    );
    let want: BTreeSet<String> = ["profile", "id", "outcome"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        required,
        want,
        "[summary.blocker_record].required_fields must be exactly {:?}",
        ["profile", "id", "outcome"]
    );

    let pr = summary
        .get("profile_record")
        .and_then(|v| v.as_table())
        .expect("[summary.profile_record] missing");
    let pr_required = collect_strings(
        require_array(pr, "required_fields", "[summary.profile_record]"),
        "[summary.profile_record].required_fields",
    );
    let pr_want: BTreeSet<String> = ["profile", "objective", "passed", "blockers"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        pr_required,
        pr_want,
        "[summary.profile_record].required_fields must be exactly {:?}",
        ["profile", "objective", "passed", "blockers"]
    );
}

#[test]
fn summary_carries_objective_grouping_fields() {
    // The runner needs to know what shape the per-objective record
    // takes; lock the four mandatory fields.
    let doc = crate::common::load_toml(&manifest_path());
    let summary = require_table(&doc, "summary");
    let fields = collect_strings(
        require_array(summary, "fields", "[summary]"),
        "[summary].fields",
    );
    let want: BTreeSet<String> = ["objective", "profile_ids", "blocker_count", "blockers"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        fields,
        want,
        "[summary].fields must be exactly {:?}",
        ["objective", "profile_ids", "blocker_count", "blockers"]
    );
}

#[test]
fn mvp_umbrella_links_release_gate_to_this_manifest() {
    // The MVP umbrella `validation/mvp.toml` must point at this
    // manifest so workers reading the umbrella discover the
    // release-gate aggregation layer.
    let umbrella = crate_root().join("validation").join("mvp.toml");
    let doc = crate::common::load_toml(&umbrella);
    let profiles = require_table(&doc, "profiles");
    let release_gate = profiles
        .get("release_gate")
        .and_then(|v| v.as_table())
        .expect("[profiles.release_gate] missing from mvp.toml");
    assert_eq!(
        require_str(release_gate, "manifest", "mvp.toml [profiles.release_gate]"),
        "profiles/release_gate.toml",
        "mvp.toml [profiles.release_gate].manifest must point at \
         profiles/release_gate.toml"
    );
    assert_eq!(
        require_int(release_gate, "issue", "mvp.toml [profiles.release_gate]"),
        UMBRELLA_ISSUE,
        "mvp.toml [profiles.release_gate].issue must equal #2775"
    );
    assert!(
        require_bool(release_gate, "required", "mvp.toml [profiles.release_gate]"),
        "mvp.toml [profiles.release_gate].required must be true"
    );
}

#[test]
fn references_block_links_to_dependent_release_gate_issues() {
    // The aggregation manifest must forward-reference the release-
    // summary schema (#2820) and the release-gate runner (#2821) so
    // a reviewer reading this file alone can find both consumers.
    let doc = crate::common::load_toml(&manifest_path());
    let refs = require_table(&doc, "references");
    let summary_ref = refs
        .get("release_summary_schema")
        .and_then(|v| v.as_table())
        .expect("[references.release_summary_schema] missing");
    assert_eq!(
        require_int(summary_ref, "issue", "[references.release_summary_schema]",),
        2820,
        "release_summary_schema reference must point at #2820"
    );
    let runner_ref = refs
        .get("release_gate_runner")
        .and_then(|v| v.as_table())
        .expect("[references.release_gate_runner] missing");
    assert_eq!(
        require_int(runner_ref, "issue", "[references.release_gate_runner]"),
        2821,
        "release_gate_runner reference must point at #2821"
    );
    let baseline_ref = refs
        .get("baseline_update_policy")
        .and_then(|v| v.as_table())
        .expect("[references.baseline_update_policy] missing");
    assert_eq!(
        require_int(baseline_ref, "issue", "[references.baseline_update_policy]",),
        2823,
        "baseline_update_policy reference must point at #2823"
    );
}
