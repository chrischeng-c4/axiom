//! Stale installed-binary drift warning (#1309).
//!
//! Loud, cheap detection only — never auto-upgrades, never fails a command.
//! When `aw` runs inside the axiom checkout (detected by
//! `apps/agentic-workflow/Cargo.toml` existing at or above CWD) and the
//! running binary's embedded build version is behind the checkout's
//! declared source version, print a single one-line warning to stderr
//! (never stdout — stdout is the protocol) naming both versions and the
//! remediation. Throttled to at most once per hour per binary version via a
//! stamp file under `/tmp/aw`, so `aw wi run` / `aw capability run` loops
//! aren't spammed on every envelope hop.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Parsed `major.minor.patch`, ignoring any `-pre` suffix (debug builds
/// append `-dev.<sha>` — see `build.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SemverCore(u32, u32, u32);

fn parse_semver_core(raw: &str) -> Option<SemverCore> {
    let base = raw.split('-').next().unwrap_or(raw);
    let mut parts = base.trim().splitn(3, '.');
    let major = parts.next()?.trim().parse().ok()?;
    let minor = parts.next()?.trim().parse().ok()?;
    let patch = parts.next()?.trim().parse().ok()?;
    Some(SemverCore(major, minor, patch))
}

/// `true` only when both versions parse AND the binary is strictly behind
/// the source. Equal, ahead, or unparseable versions never warn.
fn is_behind(binary_version: &str, source_version: &str) -> bool {
    match (
        parse_semver_core(binary_version),
        parse_semver_core(source_version),
    ) {
        (Some(b), Some(s)) => b < s,
        _ => false,
    }
}

/// Walk up from `start` looking for a directory containing
/// `apps/agentic-workflow/Cargo.toml` — the axiom-repo signal from #1309.
fn find_axiom_repo_root(start: &Path) -> Option<PathBuf> {
    let mut dir = start;
    loop {
        if dir.join("apps/agentic-workflow/Cargo.toml").is_file() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

/// Extract `key = "value"` from the given TOML section (`[section]`
/// heading match is exact). Comments after the value are stripped.
fn extract_section_value(text: &str, section: &str, key: &str) -> Option<String> {
    let mut in_section = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == section;
            continue;
        }
        if !in_section {
            continue;
        }
        let Some((lhs, rhs)) = trimmed.split_once('=') else {
            continue;
        };
        if lhs.trim() != key {
            continue;
        }
        let rhs = rhs.split('#').next().unwrap_or(rhs).trim();
        let unquoted = rhs.strip_prefix('"').and_then(|s| s.strip_suffix('"'))?;
        return Some(unquoted.to_string());
    }
    None
}

/// Resolve the checkout's declared source version: `apps/agentic-workflow/Cargo.toml`'s
/// own `[package] version` when explicitly set, else the workspace root's
/// `[workspace.package] version` (the `version.workspace = true` case, which
/// is how `apps/agentic-workflow/Cargo.toml` is authored today).
fn resolve_source_version(repo_root: &Path) -> Option<String> {
    let pkg_toml = repo_root.join("apps/agentic-workflow/Cargo.toml");
    if let Ok(text) = std::fs::read_to_string(&pkg_toml) {
        if let Some(v) = extract_section_value(&text, "[package]", "version") {
            return Some(v);
        }
    }
    let workspace_toml = repo_root.join("Cargo.toml");
    let text = std::fs::read_to_string(&workspace_toml).ok()?;
    extract_section_value(&text, "[workspace.package]", "version")
}

const THROTTLE: Duration = Duration::from_secs(3600);

/// Stamp-file throttle: returns `true` (should warn) at most once per
/// [`THROTTLE`] window per binary version, touching the stamp whenever it
/// returns `true`. `aw` only calls this once per process, so within-process
/// spam is trivially avoided; this covers repeated invocations across an
/// `aw wi run` / `aw capability run` loop.
fn should_warn_throttled(marker_dir: &Path, binary_version: &str, now: SystemTime) -> bool {
    if std::fs::create_dir_all(marker_dir).is_err() {
        // Can't persist a throttle marker — warn conservatively rather than
        // silently swallowing the signal.
        return true;
    }
    let marker = marker_dir.join(format!("drift-warn-{binary_version}.stamp"));
    let stale = match std::fs::metadata(&marker).and_then(|m| m.modified()) {
        Ok(modified) => now
            .duration_since(modified)
            .map(|elapsed| elapsed >= THROTTLE)
            .unwrap_or(true),
        Err(_) => true,
    };
    if stale {
        let _ = std::fs::write(&marker, b"");
    }
    stale
}

/// Build the one-line warning text (pure, independently testable). The only
/// caller that prints it writes to stderr via `eprintln!` — never `println!`
/// — because stdout is the aw.cli.v1 protocol channel.
fn format_warning(binary_version: &str, binary_sha: &str, source_version: &str) -> String {
    format!(
        "aw: warning: installed binary v{binary_version} (sha {binary_sha}) is behind this checkout's source v{source_version} (apps/agentic-workflow) — run `cargo install --path apps/agentic-workflow` or `aw upgrade` to resync the protocol."
    )
}

fn warn_stale_binary(binary_version: &str, binary_sha: &str, source_version: &str) {
    eprintln!(
        "{}",
        format_warning(binary_version, binary_sha, source_version)
    );
}

fn check_once_at(
    cwd: &Path,
    binary_version: &str,
    binary_sha: &str,
    marker_dir: &Path,
    now: SystemTime,
) {
    let Some(repo_root) = find_axiom_repo_root(cwd) else {
        return;
    };
    let Some(source_version) = resolve_source_version(&repo_root) else {
        return;
    };
    if !is_behind(binary_version, &source_version) {
        return;
    }
    if !should_warn_throttled(marker_dir, binary_version, now) {
        return;
    }
    warn_stale_binary(binary_version, binary_sha, &source_version);
}

/// Entry point: detect the axiom-repo + version skew from the process CWD
/// and warn on stderr at most once per hour per binary version. Never
/// fails or blocks the command — all I/O errors are swallowed and outside
/// the axiom repo this is a no-op.
pub fn check_once(binary_version: &str, binary_sha: &str) {
    let Ok(cwd) = std::env::current_dir() else {
        return;
    };
    check_once_at(
        &cwd,
        binary_version,
        binary_sha,
        Path::new("/tmp/aw"),
        SystemTime::now(),
    );
}

// ---------------------------------------------------------------------------
// #1417: hard-gate lifecycle-mutating verbs on stale-binary skew.
//
// [`check_once`] above is (and remains) warn-only for every verb — that
// covers the general "you're about to talk to a stale protocol" signal.
// This layer adds a second, narrower gate: when the installed binary is
// strictly behind the checkout's source version AND the invoked verb WRITES
// tracked lifecycle/config state (per `chain::VERB_LIFECYCLE_REGISTRY`'s
// `mutates_lifecycle` bit — #1417), a stale binary could write artifacts in
// a retired protocol shape, so the command hard-refuses instead of merely
// warning. Read-only verbs (list/show/report/check/verify/health/...) are
// unaffected — they keep going through `check_once`'s warn only. The escape
// hatch is the `AW_ALLOW_STALE_BINARY=1` environment variable: an allowed
// run proceeds, but the override is still logged (stderr) so it's visible.
// ---------------------------------------------------------------------------

/// Outcome of the stale-binary lifecycle-mutation gate for one invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StaleBinaryGate {
    /// Not behind (equal/ahead/unparseable/dev-suffix binary), or not
    /// running inside an axiom checkout at all: never refuse.
    Proceed,
    /// Behind, but the invoked verb doesn't mutate lifecycle state (or the
    /// verb couldn't be resolved at all): [`check_once`]'s warn already
    /// covers this — proceed without any additional gate action.
    WarnOnly,
    /// Behind, the invoked verb mutates lifecycle state, but
    /// `AW_ALLOW_STALE_BINARY=1` overrode the refusal (AC2): proceed, but
    /// the caller must log the override.
    Overridden {
        source_version: String,
        verb_path: String,
    },
    /// Behind, the invoked verb mutates lifecycle state, no escape hatch:
    /// hard refuse.
    Refuse {
        source_version: String,
        verb_path: String,
    },
}

/// Pure decision function (cwd/source-version resolution reads disk, but no
/// process env / stdio / exit) — the testable core of the gate, mirroring
/// [`check_once_at`]'s injected-inputs shape. `verb_path` is the already
/// -resolved dot-joined leaf verb (see
/// [`super::chain::resolve_invoked_verb_path`]), and `allow_stale_binary_env`
/// is the raw `AW_ALLOW_STALE_BINARY` value (`None` if unset) — both passed
/// in rather than read from ambient state so tests can inject them cheaply.
fn gate_decision_at(
    cwd: &Path,
    binary_version: &str,
    verb_path: Option<&str>,
    allow_stale_binary_env: Option<&str>,
) -> StaleBinaryGate {
    let Some(repo_root) = find_axiom_repo_root(cwd) else {
        return StaleBinaryGate::Proceed;
    };
    let Some(source_version) = resolve_source_version(&repo_root) else {
        return StaleBinaryGate::Proceed;
    };
    if !is_behind(binary_version, &source_version) {
        return StaleBinaryGate::Proceed;
    }
    let mutates = verb_path
        .and_then(super::chain::verb_mutates_lifecycle)
        .unwrap_or(false);
    if !mutates {
        return StaleBinaryGate::WarnOnly;
    }
    let verb_path = verb_path.unwrap_or("<unknown>").to_string();
    if allow_stale_binary_env == Some("1") {
        return StaleBinaryGate::Overridden {
            source_version,
            verb_path,
        };
    }
    StaleBinaryGate::Refuse {
        source_version,
        verb_path,
    }
}

/// Build the `aw.cli.v1` error envelope for a hard refusal
/// ([`StaleBinaryGate::Refuse`]) — pure, independently testable. The only
/// caller that prints it, [`print_refusal_envelope`], writes it to stdout.
fn build_refusal_envelope(
    binary_version: &str,
    binary_sha: &str,
    source_version: &str,
    verb_path: &str,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "aw.cli.v1",
        "action": "error",
        "message": format!(
            "aw: refused — installed binary v{binary_version} (sha {binary_sha}) is behind this \
             checkout's source v{source_version} (apps/agentic-workflow) and `aw {verb_path}` \
             mutates tracked lifecycle state; a stale binary must not write artifacts in a \
             retired protocol shape."
        ),
        "next": {
            "kind": "remediation",
            "command": "cargo install --path apps/agentic-workflow",
            "reason": format!(
                "resync the installed binary to source v{source_version} (or run `aw upgrade`), \
                 then re-run `aw {verb_path}`"
            ),
            "requires_hitl": false,
            "payload_path": serde_json::Value::Null,
        },
        "escape_hatch": {
            "env": "AW_ALLOW_STALE_BINARY=1",
            "note": "intentional old-binary use only; the override is logged to stderr when used",
        },
    })
}

/// Print [`build_refusal_envelope`]'s `aw.cli.v1` error envelope to stdout.
/// This is the ONE place in this module that prints to stdout (stdout is
/// the protocol channel; see
/// `stdout_output_is_scoped_to_the_stale_binary_refusal_envelope` below).
fn print_refusal_envelope(
    binary_version: &str,
    binary_sha: &str,
    source_version: &str,
    verb_path: &str,
) {
    let envelope = build_refusal_envelope(binary_version, binary_sha, source_version, verb_path);
    println!(
        "{}",
        serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| envelope.to_string())
    );
}

/// Entry point: hard-gate lifecycle-mutating verbs on stale-binary skew
/// (#1417). Resolves the invoked verb path from `std::env::args()`, the
/// axiom-repo + source version from the process CWD (same detection as
/// [`check_once`]), and the escape hatch from the `AW_ALLOW_STALE_BINARY`
/// environment variable, then acts on [`gate_decision_at`]'s outcome:
/// `Refuse` prints the remediation envelope to stdout and exits nonzero
/// (this call does not return); `Overridden` logs the override to stderr;
/// `WarnOnly`/`Proceed` do nothing (the general warn is `check_once`'s job).
pub fn enforce_mutating_verb_gate(binary_version: &str, binary_sha: &str) {
    let Ok(cwd) = std::env::current_dir() else {
        return;
    };
    let args: Vec<String> = std::env::args().collect();
    let verb_path = super::chain::resolve_invoked_verb_path(&args);
    let allow_env = std::env::var("AW_ALLOW_STALE_BINARY").ok();
    match gate_decision_at(
        &cwd,
        binary_version,
        verb_path.as_deref(),
        allow_env.as_deref(),
    ) {
        StaleBinaryGate::Refuse {
            source_version,
            verb_path,
        } => {
            print_refusal_envelope(binary_version, binary_sha, &source_version, &verb_path);
            std::process::exit(2);
        }
        StaleBinaryGate::Overridden {
            source_version,
            verb_path,
        } => {
            eprintln!(
                "aw: warning: AW_ALLOW_STALE_BINARY=1 overrode the stale-binary refusal for \
                 `aw {verb_path}` (installed v{binary_version} behind checkout source \
                 v{source_version}) — proceeding anyway."
            );
        }
        StaleBinaryGate::WarnOnly | StaleBinaryGate::Proceed => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- version compare: behind / equal / ahead / unparseable ----------

    #[test]
    fn is_behind_true_when_binary_patch_behind_source() {
        assert!(is_behind("0.4.3", "0.4.4"));
    }

    #[test]
    fn is_behind_true_when_binary_minor_behind_source() {
        assert!(is_behind("0.3.60", "0.4.0"));
    }

    #[test]
    fn is_behind_false_when_versions_equal() {
        assert!(!is_behind("0.4.4", "0.4.4"));
    }

    #[test]
    fn is_behind_false_when_binary_ahead_of_source() {
        assert!(!is_behind("0.4.5", "0.4.4"));
    }

    #[test]
    fn is_behind_false_when_binary_dev_suffix_ahead_of_source() {
        // Debug builds bump the next patch and append -dev.<sha>; a freshly
        // built debug binary must never warn against its own checkout.
        assert!(!is_behind("0.4.5-dev.abcd1234", "0.4.4"));
    }

    #[test]
    fn is_behind_false_when_binary_version_unparseable() {
        assert!(!is_behind("not-a-version", "0.4.4"));
    }

    #[test]
    fn is_behind_false_when_source_version_unparseable() {
        assert!(!is_behind("0.4.3", "not-a-version"));
    }

    #[test]
    fn is_behind_false_when_both_unparseable() {
        assert!(!is_behind("nope", "also-nope"));
    }

    // -- repo detection ---------------------------------------------------

    #[test]
    fn find_axiom_repo_root_detects_apps_agentic_workflow_cargo_toml() {
        let tmp = tempfile::TempDir::new().unwrap();
        let repo = tmp.path().join("repo");
        let aw_dir = repo.join("apps/agentic-workflow");
        std::fs::create_dir_all(&aw_dir).unwrap();
        std::fs::write(aw_dir.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        let subdir = repo.join("some/nested/cwd");
        std::fs::create_dir_all(&subdir).unwrap();

        let found = find_axiom_repo_root(&subdir).unwrap();
        assert_eq!(
            std::fs::canonicalize(found).unwrap(),
            std::fs::canonicalize(&repo).unwrap()
        );
    }

    #[test]
    fn find_axiom_repo_root_none_outside_axiom_checkout() {
        let tmp = tempfile::TempDir::new().unwrap();
        let other = tmp.path().join("some/other/repo");
        std::fs::create_dir_all(&other).unwrap();

        assert!(find_axiom_repo_root(&other).is_none());
    }

    // -- source version resolution ----------------------------------------

    #[test]
    fn resolve_source_version_prefers_explicit_package_version() {
        let tmp = tempfile::TempDir::new().unwrap();
        let repo = tmp.path().join("repo");
        let aw_dir = repo.join("apps/agentic-workflow");
        std::fs::create_dir_all(&aw_dir).unwrap();
        std::fs::write(
            aw_dir.join("Cargo.toml"),
            "[package]\nname = \"agentic-workflow\"\nversion = \"9.9.9\"\n",
        )
        .unwrap();

        assert_eq!(resolve_source_version(&repo), Some("9.9.9".to_string()));
    }

    #[test]
    fn resolve_source_version_falls_back_to_workspace_package() {
        let tmp = tempfile::TempDir::new().unwrap();
        let repo = tmp.path().join("repo");
        let aw_dir = repo.join("apps/agentic-workflow");
        std::fs::create_dir_all(&aw_dir).unwrap();
        std::fs::write(
            aw_dir.join("Cargo.toml"),
            "[package]\nname = \"agentic-workflow\"\nversion.workspace = true\n",
        )
        .unwrap();
        std::fs::write(
            repo.join("Cargo.toml"),
            "[workspace]\nmembers = [\"apps/agentic-workflow\"]\n\n[workspace.package]\nversion = \"0.4.4\"\n",
        )
        .unwrap();

        assert_eq!(resolve_source_version(&repo), Some("0.4.4".to_string()));
    }

    // -- throttle -----------------------------------------------------------

    #[test]
    fn should_warn_throttled_true_on_first_call() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert!(should_warn_throttled(
            tmp.path(),
            "0.4.3",
            SystemTime::now()
        ));
    }

    #[test]
    fn should_warn_throttled_false_within_window() {
        let tmp = tempfile::TempDir::new().unwrap();
        let now = SystemTime::now();
        assert!(should_warn_throttled(tmp.path(), "0.4.3", now));
        // Same version, one second later: still inside the 1h window.
        assert!(!should_warn_throttled(
            tmp.path(),
            "0.4.3",
            now + Duration::from_secs(1)
        ));
    }

    #[test]
    fn should_warn_throttled_true_again_after_window_elapses() {
        let tmp = tempfile::TempDir::new().unwrap();
        let now = SystemTime::now();
        assert!(should_warn_throttled(tmp.path(), "0.4.3", now));
        assert!(should_warn_throttled(
            tmp.path(),
            "0.4.3",
            now + Duration::from_secs(3601)
        ));
    }

    #[test]
    fn should_warn_throttled_independent_per_binary_version() {
        let tmp = tempfile::TempDir::new().unwrap();
        let now = SystemTime::now();
        assert!(should_warn_throttled(tmp.path(), "0.4.3", now));
        // A different binary version gets its own throttle window.
        assert!(should_warn_throttled(tmp.path(), "0.4.2", now));
    }

    // -- end-to-end check_once_at -------------------------------------------

    #[test]
    fn check_once_at_no_repo_signal_outside_axiom_checkout() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cwd = tmp.path().join("elsewhere");
        std::fs::create_dir_all(&cwd).unwrap();
        let marker_dir = tmp.path().join("marker");

        // Must not panic and must not create a throttle marker (no-op path).
        check_once_at(&cwd, "0.1.0", "deadbeef", &marker_dir, SystemTime::now());
        assert!(!marker_dir.exists());
    }

    #[test]
    fn check_once_at_no_marker_when_binary_not_behind() {
        let tmp = tempfile::TempDir::new().unwrap();
        let repo = tmp.path().join("repo");
        let aw_dir = repo.join("apps/agentic-workflow");
        std::fs::create_dir_all(&aw_dir).unwrap();
        std::fs::write(
            aw_dir.join("Cargo.toml"),
            "[package]\nname = \"agentic-workflow\"\nversion = \"0.4.4\"\n",
        )
        .unwrap();
        let marker_dir = tmp.path().join("marker");

        check_once_at(&repo, "0.4.4", "deadbeef", &marker_dir, SystemTime::now());
        assert!(
            !marker_dir.join("drift-warn-0.4.4.stamp").exists(),
            "equal versions must not throttle-mark (no warning path taken)"
        );
    }

    #[test]
    fn check_once_at_marks_throttle_when_binary_behind() {
        let tmp = tempfile::TempDir::new().unwrap();
        let repo = tmp.path().join("repo");
        let aw_dir = repo.join("apps/agentic-workflow");
        std::fs::create_dir_all(&aw_dir).unwrap();
        std::fs::write(
            aw_dir.join("Cargo.toml"),
            "[package]\nname = \"agentic-workflow\"\nversion = \"9.9.9\"\n",
        )
        .unwrap();
        let marker_dir = tmp.path().join("marker");

        check_once_at(&repo, "0.4.4", "deadbeef", &marker_dir, SystemTime::now());
        assert!(marker_dir.join("drift-warn-0.4.4.stamp").exists());
    }

    // -- stale-binary mutating-verb gate (#1417) -----------------------------

    fn write_fixture_repo(tmp: &Path, source_version: &str) -> PathBuf {
        let repo = tmp.join("repo");
        let aw_dir = repo.join("apps/agentic-workflow");
        std::fs::create_dir_all(&aw_dir).unwrap();
        std::fs::write(
            aw_dir.join("Cargo.toml"),
            format!("[package]\nname = \"agentic-workflow\"\nversion = \"{source_version}\"\n"),
        )
        .unwrap();
        repo
    }

    #[test]
    fn gate_decision_refuses_when_behind_and_verb_mutates() {
        let tmp = tempfile::TempDir::new().unwrap();
        let repo = write_fixture_repo(tmp.path(), "9.9.9");
        let decision = gate_decision_at(&repo, "0.4.4", Some("td.fill"), None);
        assert_eq!(
            decision,
            StaleBinaryGate::Refuse {
                source_version: "9.9.9".to_string(),
                verb_path: "td.fill".to_string(),
            }
        );
    }

    #[test]
    fn gate_decision_warn_only_when_behind_and_verb_read_only() {
        let tmp = tempfile::TempDir::new().unwrap();
        let repo = write_fixture_repo(tmp.path(), "9.9.9");
        let decision = gate_decision_at(&repo, "0.4.4", Some("wi.list"), None);
        assert_eq!(decision, StaleBinaryGate::WarnOnly);
    }

    #[test]
    fn gate_decision_warn_only_when_behind_and_verb_unresolved() {
        let tmp = tempfile::TempDir::new().unwrap();
        let repo = write_fixture_repo(tmp.path(), "9.9.9");
        let decision = gate_decision_at(&repo, "0.4.4", None, None);
        assert_eq!(decision, StaleBinaryGate::WarnOnly);
    }

    #[test]
    fn gate_decision_proceeds_when_versions_equal() {
        let tmp = tempfile::TempDir::new().unwrap();
        let repo = write_fixture_repo(tmp.path(), "0.4.4");
        let decision = gate_decision_at(&repo, "0.4.4", Some("td.fill"), None);
        assert_eq!(decision, StaleBinaryGate::Proceed);
    }

    #[test]
    fn gate_decision_proceeds_when_binary_ahead() {
        let tmp = tempfile::TempDir::new().unwrap();
        let repo = write_fixture_repo(tmp.path(), "0.4.4");
        let decision = gate_decision_at(&repo, "0.4.5", Some("td.fill"), None);
        assert_eq!(decision, StaleBinaryGate::Proceed);
    }

    #[test]
    fn gate_decision_never_refuses_dev_suffix_binary() {
        // AC3: a fresh in-checkout debug build (next patch + `-dev.<sha>`)
        // must never refuse, even for a mutating verb.
        let tmp = tempfile::TempDir::new().unwrap();
        let repo = write_fixture_repo(tmp.path(), "0.4.4");
        let decision = gate_decision_at(&repo, "0.4.5-dev.abcd1234", Some("td.fill"), None);
        assert_eq!(decision, StaleBinaryGate::Proceed);
    }

    #[test]
    fn gate_decision_proceeds_outside_axiom_checkout() {
        let tmp = tempfile::TempDir::new().unwrap();
        let elsewhere = tmp.path().join("elsewhere");
        std::fs::create_dir_all(&elsewhere).unwrap();
        let decision = gate_decision_at(&elsewhere, "0.1.0", Some("td.fill"), None);
        assert_eq!(decision, StaleBinaryGate::Proceed);
    }

    #[test]
    fn gate_decision_escape_hatch_overrides_refusal() {
        let tmp = tempfile::TempDir::new().unwrap();
        let repo = write_fixture_repo(tmp.path(), "9.9.9");
        let decision = gate_decision_at(&repo, "0.4.4", Some("td.fill"), Some("1"));
        assert_eq!(
            decision,
            StaleBinaryGate::Overridden {
                source_version: "9.9.9".to_string(),
                verb_path: "td.fill".to_string(),
            }
        );
    }

    #[test]
    fn gate_decision_ignores_non_1_escape_hatch_values() {
        let tmp = tempfile::TempDir::new().unwrap();
        let repo = write_fixture_repo(tmp.path(), "9.9.9");
        let decision = gate_decision_at(&repo, "0.4.4", Some("td.fill"), Some("true"));
        assert_eq!(
            decision,
            StaleBinaryGate::Refuse {
                source_version: "9.9.9".to_string(),
                verb_path: "td.fill".to_string(),
            }
        );
    }

    #[test]
    fn refusal_envelope_names_versions_verb_and_remediation() {
        let envelope = build_refusal_envelope("0.4.3", "deadbeef", "0.4.4", "td.fill");
        assert_eq!(envelope["schema_version"], "aw.cli.v1");
        assert_eq!(envelope["action"], "error");
        let message = envelope["message"].as_str().unwrap();
        assert!(message.contains("0.4.3"), "must name the installed version");
        assert!(message.contains("0.4.4"), "must name the checkout version");
        assert!(message.contains("td.fill"), "must name the invoked verb");
        assert_eq!(
            envelope["next"]["command"],
            "cargo install --path apps/agentic-workflow"
        );
        assert!(
            envelope["next"]["reason"]
                .as_str()
                .unwrap()
                .contains("aw upgrade"),
            "remediation must also name `aw upgrade`"
        );
        assert_eq!(envelope["escape_hatch"]["env"], "AW_ALLOW_STALE_BINARY=1");
    }

    // -- warning text + stderr-only invariant --------------------------------

    #[test]
    fn format_warning_names_both_versions_and_remediation() {
        let msg = format_warning("0.4.3", "deadbeef", "0.4.4");
        assert!(msg.contains("0.4.3"), "must name the installed version");
        assert!(msg.contains("0.4.4"), "must name the checkout version");
        assert!(
            msg.contains("cargo install --path apps/agentic-workflow"),
            "must name the build remediation"
        );
        assert!(
            msg.contains("aw upgrade"),
            "must name the upgrade remediation"
        );
        assert_eq!(
            msg.lines().count(),
            1,
            "warning must be a single line, not multi-line"
        );
    }

    #[test]
    fn stdout_output_is_scoped_to_the_stale_binary_refusal_envelope() {
        // Guard the stdout-is-protocol invariant directly on source text.
        // The warn-only path (`warn_stale_binary`) must remain stderr-only
        // (`eprintln!`) — but #1417's mutating-verb hard-refusal path is a
        // genuine protocol-channel (stdout) envelope by design: stdout
        // carries `next.command` for the agent. This asserts exactly one
        // `println!` call exists in non-test code (the refusal envelope
        // print in `print_refusal_envelope`) so stdout usage here stays
        // scoped and doesn't silently sprawl to other paths.
        let source = include_str!("drift.rs");
        let non_test = source.split("#[cfg(test)]").next().unwrap();
        // `"eprintln!("` contains `"println!("` as a substring, so strip
        // `eprintln!(` occurrences first — otherwise every `eprintln!` call
        // would double-count as a `println!` match too.
        let without_eprintln = non_test.replace("eprintln!(", "");
        let println_count = without_eprintln.matches("println!(").count();
        assert_eq!(
            println_count, 1,
            "drift.rs must have exactly one println! call — the stale-binary refusal envelope \
             (stdout is the aw.cli.v1 protocol channel); any other output must go through \
             eprintln!"
        );
        assert!(non_test.contains("eprintln!("));
    }
}
