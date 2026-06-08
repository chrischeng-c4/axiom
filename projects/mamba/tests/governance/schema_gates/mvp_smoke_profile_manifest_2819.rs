//! Schema gate for the MVP smoke profile manifest — closes #2819.
//!
//! Acceptance (issue #2819):
//!
//!   1. Missing smoke profile entry fails validation. Required commands
//!      are `compile_list`, `inventory_summary`, and `skip_debt_summary`;
//!      removing any one of them must fail this test.
//!   2. Smoke profile output includes command names and pass/fail status.
//!      The manifest carries `id` (command name) plus `outcome_field`
//!      (JSON key the worker reads for pass/fail). Both are required and
//!      non-empty for every entry.
//!   3. The profile excludes long-running benchmarks. `kind` is closed
//!      to {compile_list, inventory, skip_debt}; the `[policy]` block
//!      caps `max_wall_seconds` at 60 and forbids bench/perf/ecosystem
//!      kinds explicitly.
//!
//! Cheap test — single TOML read + a handful of string checks. Stays in
//! the default `cargo test -p mamba` set; runs in well under a second.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const REQUIRED_COMMANDS: &[&str] = &["compile_list", "inventory_summary", "skip_debt_summary"];

const ALLOWED_KINDS: &[&str] = &["compile_list", "inventory", "skip_debt"];

const FORBIDDEN_KINDS: &[&str] = &["bench", "perf", "ecosystem", "real_world"];

const SMOKE_WALL_CEILING_SECS: i64 = 60;

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("profiles")
        .join("smoke.toml")
}

fn umbrella_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("mvp.toml")
}

fn scripts_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

fn require_str<'a>(table: &'a toml::value::Table, key: &str, id: &str) -> &'a str {
    table
        .get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| panic!("smoke command {id}: missing or empty required string `{key}`"))
}

fn require_int(table: &toml::value::Table, key: &str, id: &str) -> i64 {
    table
        .get(key)
        .and_then(|v| v.as_integer())
        .unwrap_or_else(|| panic!("smoke command {id}: missing required integer `{key}`"))
}

fn require_bool(table: &toml::value::Table, key: &str, id: &str) -> bool {
    table
        .get(key)
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| panic!("smoke command {id}: missing required bool `{key}`"))
}

#[test]
fn smoke_profile_manifest_has_required_commands() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("smoke"),
        "smoke.toml `profile` must be \"smoke\""
    );

    let listed = doc
        .get("required_commands")
        .and_then(|v| v.as_array())
        .expect("smoke.toml missing required `required_commands` array");
    let listed: BTreeSet<&str> = listed.iter().filter_map(|v| v.as_str()).collect();
    let expected: BTreeSet<&str> = REQUIRED_COMMANDS.iter().copied().collect();
    assert_eq!(
        listed, expected,
        "smoke.toml `required_commands` must list exactly {REQUIRED_COMMANDS:?}; \
         got {listed:?} — removing or renaming an entry violates #2819 acceptance"
    );

    let commands = doc
        .get("commands")
        .and_then(|v| v.as_table())
        .expect("smoke.toml missing `[commands]` table");

    for required in REQUIRED_COMMANDS {
        assert!(
            commands.contains_key(*required),
            "smoke.toml missing required `[commands.{required}]` entry"
        );
    }
}

#[test]
fn smoke_profile_manifest_entries_are_well_formed() {
    let doc = load_toml(&manifest_path());
    let commands = doc
        .get("commands")
        .and_then(|v| v.as_table())
        .expect("smoke.toml missing `[commands]` table");

    for (id, entry) in commands {
        let table = entry
            .as_table()
            .unwrap_or_else(|| panic!("smoke command {id}: entry must be a TOML table"));

        let entry_id = require_str(table, "id", id);
        assert_eq!(
            entry_id, id,
            "smoke command {id}: `id` field {entry_id:?} must match table key"
        );

        let kind = require_str(table, "kind", id);
        assert!(
            ALLOWED_KINDS.contains(&kind),
            "smoke command {id}: kind = {kind:?} not in {ALLOWED_KINDS:?}"
        );

        let command = require_str(table, "command", id);
        assert!(
            command.starts_with("python3 scripts/"),
            "smoke command {id}: command {command:?} must invoke a script under `scripts/` \
             (`python3 scripts/...`) so the worker contract is uniform"
        );

        let _ = require_bool(table, "blocking", id);
        let _ = require_str(table, "outcome_field", id);

        let max_seconds = require_int(table, "max_seconds", id);
        assert!(
            max_seconds > 0 && max_seconds <= SMOKE_WALL_CEILING_SECS,
            "smoke command {id}: max_seconds = {max_seconds} must be within \
             (0, {SMOKE_WALL_CEILING_SECS}] — smoke is the fast gate"
        );
    }
}

#[test]
fn smoke_profile_policy_excludes_long_running_benchmarks() {
    let doc = load_toml(&manifest_path());

    let policy = doc
        .get("policy")
        .and_then(|v| v.as_table())
        .expect("smoke.toml missing `[policy]` block");

    let offline = policy
        .get("offline")
        .and_then(|v| v.as_bool())
        .expect("smoke.toml `[policy].offline` must be a bool");
    assert!(
        offline,
        "smoke profile must be offline by default — smoke = fast + offline"
    );

    let wall = policy
        .get("max_wall_seconds")
        .and_then(|v| v.as_integer())
        .expect("smoke.toml `[policy].max_wall_seconds` must be set");
    assert!(
        wall > 0 && wall <= SMOKE_WALL_CEILING_SECS,
        "smoke profile `[policy].max_wall_seconds` = {wall} must be within \
         (0, {SMOKE_WALL_CEILING_SECS}] — acceptance: \
         \"The profile excludes long-running benchmarks.\""
    );

    let forbid = policy
        .get("forbid_kinds")
        .and_then(|v| v.as_array())
        .expect("smoke.toml `[policy].forbid_kinds` must be an array");
    let forbid: BTreeSet<&str> = forbid.iter().filter_map(|v| v.as_str()).collect();
    for fb in FORBIDDEN_KINDS {
        assert!(
            forbid.contains(*fb),
            "smoke.toml `[policy].forbid_kinds` must include {fb:?}"
        );
    }

    let commands = doc
        .get("commands")
        .and_then(|v| v.as_table())
        .expect("smoke.toml missing `[commands]` table");
    for (id, entry) in commands {
        let kind = entry
            .as_table()
            .and_then(|t| t.get("kind"))
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        assert!(
            !forbid.contains(kind),
            "smoke command {id}: kind = {kind:?} is in forbid_kinds \
             — bench / perf / ecosystem belong in other profiles"
        );
    }
}

#[test]
fn smoke_profile_commands_reference_scripts_directory() {
    let doc = load_toml(&manifest_path());
    let commands = doc
        .get("commands")
        .and_then(|v| v.as_table())
        .expect("smoke.toml missing `[commands]` table");

    let scripts = scripts_root();
    for (id, entry) in commands {
        let command = entry
            .as_table()
            .and_then(|t| t.get("command"))
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let rel = command
            .strip_prefix("python3 scripts/")
            .unwrap_or_else(|| panic!("smoke command {id}: command {command:?} malformed"));
        let script_name = rel.split_whitespace().next().unwrap_or_default();
        // `compile_list` and `inventory_summary` already ship; `skip_debt_summary`
        // is a forward reference whose script is delivered by the release-gate
        // runner issue (#2821). We only assert the existing-ship scripts here
        // so this smoke gate locks shape without pulling unwritten work in.
        if matches!(id.as_str(), "compile_list" | "inventory_summary") {
            let on_disk = scripts.join(script_name);
            assert!(
                on_disk.exists(),
                "smoke command {id}: script {script_name:?} must exist at {}",
                on_disk.display()
            );
        }
    }
}

#[test]
fn mvp_umbrella_links_to_smoke_manifest() {
    let doc = load_toml(&umbrella_path());
    let smoke = doc
        .get("profiles")
        .and_then(|v| v.get("smoke"))
        .and_then(|v| v.as_table())
        .expect("validation/mvp.toml missing `[profiles.smoke]`");

    let manifest = smoke
        .get("manifest")
        .and_then(|v| v.as_str())
        .expect("validation/mvp.toml `[profiles.smoke].manifest` must be set so workers can locate smoke.toml");
    assert_eq!(
        manifest, "profiles/smoke.toml",
        "umbrella must point at profiles/smoke.toml; got {manifest:?}"
    );

    let resolved = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join(manifest);
    assert!(
        resolved.exists(),
        "smoke manifest path {} does not exist",
        resolved.display()
    );

    let issue = smoke
        .get("issue")
        .and_then(|v| v.as_integer())
        .expect("validation/mvp.toml `[profiles.smoke].issue` must record the issue id");
    assert_eq!(issue, 2819, "smoke profile owner issue must be #2819");
}
