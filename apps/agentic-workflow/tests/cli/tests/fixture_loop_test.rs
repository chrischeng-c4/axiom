// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/validate/tests/fixture_loop_test.md#source
// CODEGEN-BEGIN
//! Fixture-loop e2e proof (#1279, epic #1270 R8a): drive a self-contained
//! fixture project through the real `aw` binary as a generic *envelope
//! follower* — start one command, parse the emitted `aw.cli.v1`-shaped JSON
//! envelope, execute whatever `next.command` (or `invoke.command` +
//! `invoke.args.slug`) it names, supply canned payload content keyed by
//! marker id when the envelope hands back a `payload_path`, and keep going
//! until a terminal envelope (`action:"done"`), a HITL stop, or an error is
//! reached — bounded by [`MAX_HOPS`] (a livelock is a hard failure, not a
//! slow pass).
//!
//! Scope note on `completion.workflow_complete` / `aw wi run`: #1279's own
//! "Out of Scope" note excludes GitHub-backend behavior — "fixture uses the
//! local backend". `completion.workflow_complete` is a field on the
//! `aw wi run` / `aw capability run` root-driven-runner envelope
//! (`cli/run.rs`'s `resolve_issue`), which resolves its backend via
//! `issues::resolve_default_backend` and rejects `type = "local"` there *by
//! design* (see that module's `resolve_tests::invalid_type_errors`) — a
//! fully offline sandbox cannot drive `aw wi run` to
//! `workflow_complete=true` without a live GitHub/GitLab fixture, exactly
//! the boundary `chain_liveness_test.rs`'s module doc already documents for
//! the same reason. This suite therefore follows the envelope chain through
//! the internal `LocalBackend`-driven segment instead — `aw td fill` (brief
//! + apply, marker by marker) through terminal `aw td code-check` — and
//! treats that terminal envelope's `"action":"done"` as the practical
//! completion signal, matching `chain_liveness_test.rs`'s own convention.
//! Wiring a literal `completion.workflow_complete=true` proof would need
//! either a `gh`-CLI PATH-shadow mock or a local/fixture-only
//! `resolve_default_backend` override — both out of scope here.

use std::path::Path;
use std::process::Command;

use agentic_workflow::issues::types::{td_phase, IssueType};
use agentic_workflow::issues::{Issue, IssueBackend, IssueState, LocalBackend};

/// Bounded hop budget: exhausting this many hops without reaching a
/// terminal envelope is a livelock failure (AC2's "bounded tick count"),
/// not a slow pass. Generous relative to this fixture's real (4-hop) trace
/// so unrelated future envelope-shape growth doesn't make the test flaky.
const MAX_HOPS: usize = 40;

fn skip_unless_binaries() -> Option<(std::path::PathBuf, String)> {
    let git = agentic_workflow::git::find_git_bin()?;
    let aw_bin = std::env::var("CARGO_BIN_EXE_aw").ok()?;
    Some((git, aw_bin))
}

/// Seed a from-scratch repo on a non-"main" branch: TD/CB verbs only
/// require a provisioned `td-<slug>` branch when launched from `main`
/// (`should_use_td_branch` in td.rs); a real project branch runs the
/// internal lifecycle verbs in place instead (matches
/// `cb_fill_test.rs::test_apply_marker_replaces_block`'s real-binary
/// round trip, the closest existing precedent for driving `aw td fill` for
/// real).
fn init_seed_repo(git: &Path, root: &Path) {
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["init", "-b", "project-test"])
        .status()
        .expect("git init");
    for (k, v) in [
        ("user.email", "test@test"),
        ("user.name", "test"),
        ("commit.gpgsign", "false"),
    ] {
        Command::new(git)
            .arg("-C")
            .arg(root)
            .args(["config", k, v])
            .status()
            .unwrap();
    }
    std::fs::write(root.join("README.md"), "seed\n").unwrap();
    std::fs::create_dir_all(root.join(".aw")).unwrap();
    std::fs::write(root.join("aw.toml"), "").unwrap();
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "seed"])
        .status()
        .unwrap();
}

/// Commit every current working-tree change. Matches
/// `chain_liveness_test.rs`/`td_no_merge_test.rs`'s `commit_all`: real
/// `aw td gen`/`aw td fill` already commit generated/filled files before
/// terminal `aw td code-check` runs, so a fixture that hand-writes a WI's
/// touched-scope files directly must do the same to satisfy the #807/#1275
/// clean-touched-scope precondition.
fn commit_all(git: &Path, root: &Path) {
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["add", "-A"])
        .status()
        .unwrap();
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "wip: touched-scope fixture"])
        .status()
        .unwrap();
}

/// aw.toml with one resolvable project row and deliberately NO
/// `[aw.ec.generated]` table — the "no EC inventory configured" advisory
/// path (matches `td_no_merge_test.rs::write_858_ec_configured_aw_toml`'s
/// sibling shape minus the `[aw.ec.generated]` block), so the fixture loop
/// completes without any external EC runner.
fn write_fixture_aw_toml(root: &Path, project: &str) {
    std::fs::write(
        root.join("aw.toml"),
        format!(
            "[[projects]]\nname = \"{project}\"\npath = \".\"\n\n\
             [[projects.workspaces]]\nname = \"{project}\"\npaths = [\"**\"]\ntarget = \"rust\"\n"
        ),
    )
    .unwrap();
}

const DEMO_SPEC_REL: &str = ".aw/tech-design/specs/demo.md";
const MARKER_A_PATH: &str = "src/fixture_loop_demo_a.rs";
const MARKER_B_PATH: &str = "src/fixture_loop_demo_b.rs";
const MARKER_A_ID: &str = "fixture-loop-marker-a";
const MARKER_B_ID: &str = "fixture-loop-marker-b";

/// Write a minimal TD spec whose `## Changes` section lists the given
/// `(path, action)` entries, each `impl_mode: hand-written` (matches
/// `chain_liveness_test.rs::write_demo_changes_spec`).
fn write_fixture_changes_spec(root: &Path, entries: &[(&str, &str)]) {
    let mut yaml = String::from("changes:\n");
    for (path, action) in entries {
        yaml.push_str(&format!(
            "  - path: {path}\n    action: {action}\n    impl_mode: hand-written\n"
        ));
    }
    let content = format!(
        "---\nid: demo\nfill_sections: [changes]\n---\n\n# Demo\n\n## Changes\n\
         <!-- type: changes lang: yaml -->\n\n```yaml\n{yaml}```\n"
    );
    let spec_dir = root.join(".aw/tech-design/specs");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(spec_dir.join("demo.md"), content).unwrap();
}

/// Write an unfilled HANDWRITE marker source file (a "trivially fillable
/// target" — one gap, one stub line).
fn write_handwrite_marker_file(root: &Path, path_rel: &str, gap_id: &str) {
    let abs = root.join(path_rel);
    std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
    std::fs::write(
        &abs,
        format!(
            "// HANDWRITE-BEGIN gap=\"{gap_id}\" tracker=\"none\" reason=\"unfilled\"\n\
             // TODO: hand-write content for `{path_rel}`.\n\
             // HANDWRITE-END\n"
        ),
    )
    .unwrap();
}

/// Seed an open issue at `phase`, scoped to `spec_rel` (`Issue.implements`)
/// and `project` (an `app:<project>` label so the terminal EC-advisory and
/// touched-scope standardization gates resolve to a real project row —
/// matches `td_no_merge_test.rs::seed_858_open_issue_with_project`).
async fn seed_open_issue_at_phase_with_project(
    root: &Path,
    slug: &str,
    phase: &str,
    spec_rel: &str,
    project: &str,
) {
    let backend = LocalBackend::from_project_root(root);
    let issue = Issue {
        issue_type: IssueType::Enhancement,
        title: format!("{slug} WI"),
        state: IssueState::Open,
        id: None,
        github_id: None,
        gitlab_id: None,
        url: None,
        author: None,
        labels: vec![format!("phase:{}", phase), format!("app:{project}")],
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        slug: slug.to_string(),
        body: format!("# {slug} WI\n"),
        related: Vec::new(),
        implements: vec![spec_rel.to_string()],
        phase: Some(phase.to_string()),
        branch: None,
        target_branch: None,
        git_workflow: None,
        change_id: None,
        iteration: None,
        current_task_id: None,
        impl_spec_phase: None,
        task_revisions: None,
        revision_counts: None,
        last_action: None,
        session_id: None,
        validation_errors: Vec::new(),
        review_count: None,
        flagged_sections: None,
        fill_retry_count: None,
        ship_status: None,
        ship_commit: None,
        regen_verified_at: None,
    };
    backend.create(&issue).await.expect("seed open issue");
}

/// One executed hop: the command run and the envelope it produced.
#[derive(Debug)]
struct Hop {
    index: usize,
    command: Vec<String>,
    envelope: serde_json::Value,
}

/// AC2: on any failed hop, the failure must name the hop index, the
/// command executed, and the envelope JSON that produced it (when one was
/// parseable).
#[derive(Debug)]
struct FollowFailure {
    hop_index: usize,
    command: Vec<String>,
    envelope: Option<serde_json::Value>,
    reason: String,
}

impl std::fmt::Display for FollowFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let envelope_json = self
            .envelope
            .as_ref()
            .map(|e| serde_json::to_string_pretty(e).unwrap_or_else(|_| e.to_string()))
            .unwrap_or_else(|| "<no parseable envelope>".to_string());
        write!(
            f,
            "fixture-loop follower broke at hop {}: command `aw {}` — {}\nenvelope:\n{}",
            self.hop_index,
            self.command.join(" "),
            self.reason,
            envelope_json,
        )
    }
}

/// Canned fill content keyed by marker id (not by call sequence) — the
/// follower discovers which marker to fill from the envelope itself.
fn canned_content_for_marker(marker_id: &str) -> String {
    format!(
        "// filled by fixture_loop_test (marker `{marker_id}`)\npub fn {}() {{}}\n",
        marker_id.replace('-', "_")
    )
}

/// Prefer `next.command` (always a full runnable string on the `cb_fill`/
/// `cb` envelopes this suite follows); fall back to the `invoke.command` +
/// `invoke.args.slug` (or top-level `slug`) reconstruction rule documented
/// by `chain.rs`'s `EMIT_REGISTRY` for envelope shapes without a `next`
/// field (e.g. `td.rs`'s `TdEnvelope::Dispatch`).
fn extract_next_command(envelope: &serde_json::Value) -> Option<String> {
    if let Some(cmd) = envelope
        .get("next")
        .and_then(|n| n.get("command"))
        .and_then(|c| c.as_str())
    {
        return Some(cmd.to_string());
    }
    let invoke_command = envelope.get("invoke")?.get("command")?.as_str()?;
    let slug = envelope
        .get("invoke")?
        .get("args")
        .and_then(|a| a.get("slug"))
        .and_then(|s| s.as_str())
        .or_else(|| envelope.get("slug").and_then(|s| s.as_str()))?;
    Some(format!("{invoke_command} {slug}"))
}

fn extract_payload_path(envelope: &serde_json::Value) -> Option<String> {
    envelope
        .get("next")?
        .get("payload_path")?
        .as_str()
        .map(|s| s.to_string())
}

/// The marker id a fill envelope names, whichever shape it's in: brief
/// mode's `invoke.args.marker_list[0].id`, or apply-continuation's
/// `invoke.args.marker`.
fn extract_marker_id(envelope: &serde_json::Value) -> Option<String> {
    let args = envelope.get("invoke")?.get("args")?;
    if let Some(id) = args.get("marker").and_then(|m| m.as_str()) {
        return Some(id.to_string());
    }
    args.get("marker_list")?
        .get(0)?
        .get("id")?
        .as_str()
        .map(|s| s.to_string())
}

/// Generic `aw.cli.v1` envelope follower: run `start`, then keep executing
/// whatever command each envelope names — filling in canned marker payload
/// content when asked — until a terminal (`action:"done"`) envelope, a
/// HITL stop (`next.requires_hitl == true`), or an error is reached.
/// Whitelist-guards every discovered command: it must start with `"aw "`.
fn follow_envelopes(aw_bin: &str, root: &Path, start: &[&str]) -> Result<Vec<Hop>, FollowFailure> {
    let mut hops = Vec::new();
    let mut command: Vec<String> = start.iter().map(|s| s.to_string()).collect();

    for index in 0..MAX_HOPS {
        let output = Command::new(aw_bin)
            .args(&command)
            .current_dir(root)
            .output()
            .map_err(|e| FollowFailure {
                hop_index: index,
                command: command.clone(),
                envelope: None,
                reason: format!("failed to spawn `aw {}`: {e}", command.join(" ")),
            })?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let parsed: Option<serde_json::Value> = serde_json::from_str(stdout.trim()).ok();

        if !output.status.success() {
            return Err(FollowFailure {
                hop_index: index,
                command: command.clone(),
                envelope: parsed,
                reason: format!(
                    "exited non-zero ({:?})\nstdout:\n{stdout}\nstderr:\n{stderr}",
                    output.status.code()
                ),
            });
        }

        let envelope = parsed.ok_or_else(|| FollowFailure {
            hop_index: index,
            command: command.clone(),
            envelope: None,
            reason: format!("stdout is not a single JSON envelope:\nstdout:\n{stdout}"),
        })?;

        if envelope.get("action").and_then(|a| a.as_str()) == Some("error") {
            return Err(FollowFailure {
                hop_index: index,
                command: command.clone(),
                envelope: Some(envelope),
                reason: "envelope reported action=\"error\"".to_string(),
            });
        }

        let requires_hitl = envelope
            .get("next")
            .and_then(|n| n.get("requires_hitl"))
            .and_then(|h| h.as_bool())
            == Some(true);
        let done = envelope.get("action").and_then(|a| a.as_str()) == Some("done");

        if !done && !requires_hitl {
            // Supply canned content BEFORE following the next command: the
            // marker's payload template is already initialized on disk by
            // the CLI itself, keyed by the marker id the envelope names.
            if let Some(payload_path) = extract_payload_path(&envelope) {
                if let Some(marker_id) = extract_marker_id(&envelope) {
                    std::fs::write(&payload_path, canned_content_for_marker(&marker_id)).map_err(
                        |e| FollowFailure {
                            hop_index: index,
                            command: command.clone(),
                            envelope: Some(envelope.clone()),
                            reason: format!(
                                "failed to write canned payload at {payload_path}: {e}"
                            ),
                        },
                    )?;
                }
            }
        }

        hops.push(Hop {
            index,
            command: command.clone(),
            envelope: envelope.clone(),
        });

        if done || requires_hitl {
            return Ok(hops);
        }

        let next_command = extract_next_command(&envelope).ok_or_else(|| FollowFailure {
            hop_index: index,
            command: command.clone(),
            envelope: Some(envelope.clone()),
            reason: "no runnable next command in envelope (no next.command, no invoke.command \
                      + slug fallback)"
                .to_string(),
        })?;
        if !next_command.starts_with("aw ") {
            return Err(FollowFailure {
                hop_index: index,
                command: command.clone(),
                envelope: Some(envelope.clone()),
                reason: format!(
                    "whitelist guard: next command must start with `aw `, got `{next_command}`"
                ),
            });
        }
        command = next_command
            .split_whitespace()
            .skip(1)
            .map(|s| s.to_string())
            .collect();
    }

    Err(FollowFailure {
        hop_index: MAX_HOPS,
        command,
        envelope: None,
        reason: format!("hop budget ({MAX_HOPS}) exhausted without reaching a terminal envelope"),
    })
}

/// AC1 (internal-lifecycle-layer scope, see module doc): a bounded-tick,
/// self-contained fixture (tmp git repo, local issue backend, a `cb_genned`
/// WI whose TD spec names two trivially fillable HANDWRITE targets) driven
/// purely by following emitted envelopes — starting at `aw td fill <slug>`
/// — reaches terminal `"action":"done"` with the EC-advisory marker
/// (no EC inventory configured), lands both canned fills, and advances the
/// WI to `td_merged`.
#[tokio::test]
async fn fixture_loop_drives_cb_genned_wi_to_terminal_done() {
    let Some((git, aw_bin)) = skip_unless_binaries() else {
        eprintln!("skipping: git binary or CARGO_BIN_EXE_aw not available");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    init_seed_repo(&git, root);
    write_fixture_aw_toml(root, "demo");
    write_fixture_changes_spec(
        root,
        &[(MARKER_A_PATH, "create"), (MARKER_B_PATH, "create")],
    );
    write_handwrite_marker_file(root, MARKER_A_PATH, MARKER_A_ID);
    write_handwrite_marker_file(root, MARKER_B_PATH, MARKER_B_ID);
    commit_all(&git, root);

    let slug = "fixture-loop-demo";
    seed_open_issue_at_phase_with_project(root, slug, td_phase::CB_GENNED, DEMO_SPEC_REL, "demo")
        .await;

    let hops = follow_envelopes(&aw_bin, root, &["td", "fill", slug])
        .unwrap_or_else(|failure| panic!("{failure}"));

    // Real trace: fill(brief) -> fill --apply marker-a -> fill --apply
    // marker-b -> code-check. Assert the shape generically (>=4 hops,
    // terminal done) rather than pinning the exact count so unrelated
    // envelope-shape growth doesn't make this brittle.
    assert!(
        hops.len() >= 4,
        "expected at least 4 hops (brief + 2 marker applies + terminal code-check), got {}: {:#?}",
        hops.len(),
        hops
    );
    let last = hops.last().expect("at least one hop");
    assert_eq!(
        last.envelope["action"], "done",
        "terminal hop must report action=\"done\", got: {:#?}",
        last.envelope
    );
    assert_eq!(
        last.envelope["ec_gate"], "advisory (no inventory configured)",
        "no configured EC inventory must carry the explicit advisory marker, got: {:#?}",
        last.envelope
    );

    let backend = LocalBackend::from_project_root(root);
    let after = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert_eq!(after.phase.as_deref(), Some(td_phase::TD_MERGED));
    assert_eq!(after.state, IssueState::Closed);

    for (path, marker_id) in [(MARKER_A_PATH, MARKER_A_ID), (MARKER_B_PATH, MARKER_B_ID)] {
        let content = std::fs::read_to_string(root.join(path)).expect("read updated marker file");
        assert!(
            content.contains(&canned_content_for_marker(marker_id)),
            "{path} must contain the canned fill content, got:\n{content}"
        );
        assert!(
            !content.contains("TODO: hand-write content"),
            "{path} must no longer contain the unfilled stub, got:\n{content}"
        );
        assert!(
            content.contains("HANDWRITE-BEGIN") && content.contains("HANDWRITE-END"),
            "{path} must keep its HANDWRITE scaffold after fill (issue #932 marker gate), got:\n{content}"
        );
    }
}

/// AC2 companion: an induced breakage (a WI parked at a phase terminal
/// `aw td code-check` cannot complete from) must name the first broken hop
/// — hop index, the exact command executed, and the envelope JSON that
/// produced it — not fail generically or hang to the hop budget.
#[tokio::test]
async fn fixture_loop_reports_first_broken_hop_on_induced_phase_breakage() {
    let Some((git, aw_bin)) = skip_unless_binaries() else {
        eprintln!("skipping: git binary or CARGO_BIN_EXE_aw not available");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    init_seed_repo(&git, root);
    write_fixture_aw_toml(root, "demo");
    commit_all(&git, root);

    // `td_created` is not one of the terminal-code-checkable phases
    // (`cb_filled`, `cb_genned`, legacy `td_gen_coded`) and not the
    // `td_merged` retry phase either — `aw td code-check` must refuse with
    // an explicit `action:"error"` envelope instead of silently completing.
    let slug = "fixture-loop-broken-phase";
    seed_open_issue_at_phase_with_project(root, slug, td_phase::TD_CREATED, DEMO_SPEC_REL, "demo")
        .await;

    let err = follow_envelopes(&aw_bin, root, &["td", "code-check", slug])
        .expect_err("an unresolvable start phase must break the very first hop, not succeed");

    assert_eq!(
        err.hop_index, 0,
        "the first (and only) hop must be the one reported broken, got: {err}"
    );
    assert_eq!(
        err.command,
        vec!["td".to_string(), "code-check".to_string(), slug.to_string()],
        "the failure must name the exact command executed, got: {err}"
    );
    let envelope = err
        .envelope
        .as_ref()
        .expect("failure must carry the envelope JSON that produced it");
    assert_eq!(
        envelope["action"], "error",
        "expected an explicit error envelope, got: {:#?}",
        envelope
    );
    assert!(
        envelope["message"]
            .as_str()
            .unwrap_or_default()
            .contains("cannot complete code-check"),
        "error message should explain the phase mismatch, got: {:#?}",
        envelope
    );

    let rendered = err.to_string();
    assert!(
        rendered.contains("hop 0"),
        "Display impl must name the hop index, got:\n{rendered}"
    );
    assert!(
        rendered.contains("td code-check"),
        "Display impl must name the command, got:\n{rendered}"
    );
    assert!(
        rendered.contains("\"action\": \"error\"") || rendered.contains("\"action\":\"error\""),
        "Display impl must include the envelope JSON, got:\n{rendered}"
    );
}
// CODEGEN-END
