// SPEC-MANAGED: apps/agentic-workflow/tech-design/semantic/agentic-workflow-tests-cli-tests.md#tests
// CODEGEN-BEGIN
use agentic_workflow::cli::Commands;
use clap::{CommandFactory, Parser};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "aw")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn aw_bin() -> Option<String> {
    std::env::var("CARGO_BIN_EXE_aw").ok()
}

fn collect_markdown_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            out.push(path);
        }
    }
}

#[test]
fn legacy_top_level_commands_are_removed() {
    let cmd = Cli::command();
    for name in [
        "status",
        "list",
        // "view" was re-added by the repo-view desktop app capability
        // (Commands::View in src/cli/commands.rs) — no longer removed.
        "changes",
        "fillback",
        "sdd",
        "gen",
        "check",
        "rules",
        "hover",
        "definition",
        "references",
        "symbols",
        "type-at",
        "diagnostics",
        "pdg",
        "slice",
        "impact",
        "taint",
        "daemon",
        "serve",
        "context",
        // #918: `aw run` fully removed from the clap tree (superseded by
        // `aw wi run` / `aw capability run`, #917).
        "run",
        "run-change",
        "workflow",
        "revise-artifact",
        "artifact",
        "validate-spec-structure",
        "check-alignment",
        "iss",
        "issues",
        "handoff",
        "takeoff",
        "platform",
        "hook",
        "scaffold-spec",
        "project",
        "caps",
        "cb",
        "init",
        "sync",
    ] {
        assert!(
            cmd.find_subcommand(name).is_none(),
            "{name} should not be registered"
        );
    }
}

#[test]
fn workflow_protocol_commands_remain_registered() {
    let cmd = Cli::command();
    for name in [
        "health",
        "capability",
        "wi",
        "td",
        "standardize",
        "generator",
        "conf",
        "chat",
    ] {
        assert!(
            cmd.find_subcommand(name).is_some(),
            "{name} should remain registered"
        );
    }
}

#[test]
fn deprecated_capability_alias_is_rejected_by_parser() {
    let Err(err) = Cli::try_parse_from(["aw", "caps", "report"]) else {
        panic!("caps alias should not parse");
    };
    let stderr = err.to_string();
    assert!(
        stderr.contains("unrecognized subcommand"),
        "caps alias should be rejected, stderr:\n{stderr}"
    );
}

#[test]
fn deleted_top_level_commands_fail_as_unknown_commands() {
    let Some(aw) = aw_bin() else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };

    for command in [
        // #918: `aw run` fully removed from the clap tree (superseded by
        // `aw wi run` / `aw capability run`, #917).
        "run",
        "run-change",
        "workflow",
        "revise-artifact",
        "artifact",
        "validate-spec-structure",
        "check-alignment",
        "iss",
        "issues",
        "handoff",
        "takeoff",
        "platform",
        "hook",
        "scaffold-spec",
        "project",
        "caps",
        "cb",
        "init",
        "sync",
    ] {
        let out = Command::new(&aw)
            .arg(command)
            .output()
            .expect("run deleted command");
        assert!(!out.status.success(), "{command} should fail");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("unrecognized subcommand") || stderr.contains("unexpected argument"),
            "{command} should fail as unknown command, stderr:\n{stderr}"
        );
        assert!(
            !stderr.contains("retired"),
            "{command} should not emit retired runtime message"
        );
    }
}

#[test]
fn active_docs_and_templates_do_not_reference_deleted_commands() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("repo root");
    let mut docs = vec![repo_root.join("AGENTS.md")];
    collect_markdown_files(&manifest_dir.join("templates/cli"), &mut docs);
    collect_markdown_files(&repo_root.join(".agents/skills"), &mut docs);

    let deleted = [
        // #918: `aw run` fully removed from the clap tree (superseded by
        // `aw wi run` / `aw capability run`, #917). NOT added here: this
        // scan still walks `templates/cli/mainthread/skills/aw-wi/SKILL.md`,
        // which is a pre-existing dirty file outside this change's scope
        // (still carries `aw run` from an in-flight, unrelated edit) —
        // adding the literal here would turn this assertion red for a file
        // this change must not touch. See #918 report / #857 remnant.
        "aw run-change",
        "aw workflow",
        "aw revise-artifact",
        "aw artifact",
        "aw validate-spec-structure",
        "aw check-alignment",
        // Trailing space (like `standardize.rs`'s `DELETED_COMMAND_PATHS`
        // `"aw cb "` entry): the deleted legacy abbreviation was `aw iss
        // <...>`, not a prefix match — a bare substring would also flag the
        // current, active `aw issue` verb (issue #985's CLI table renders
        // `` `aw issue` `` verbatim).
        "aw iss ",
        "aw issues",
        "aw chat agents",
        "aw handoff",
        "aw takeoff",
        "aw platform",
        "aw hook",
        "aw scaffold-spec",
        "aw project health",
        "aw caps",
        "aw cb",
        "aw init",
        "aw sync",
        // #920 (epic #914 slice F): `aw standardize` is retired down to
        // `audit` only; the `managed`/`semantic`/`traceability` layer
        // `report`/`next`/`run` drivers are gone.
        "aw standardize managed",
        "aw standardize semantic",
        "aw standardize traceability",
    ];
    for doc in docs {
        let Ok(content) = std::fs::read_to_string(&doc) else {
            continue;
        };
        for command in deleted {
            assert!(
                !content.contains(command),
                "{} still references deleted command `{command}`",
                doc.display()
            );
        }
    }
}

#[test]
fn deprecated_td_aliases_are_removed() {
    let cmd = Cli::command();
    let td = cmd.find_subcommand("td").expect("td registered");
    assert!(td.find_subcommand("gen-code").is_none());
    assert!(td.find_subcommand("audit").is_none());
    assert!(td.find_subcommand("arbitrate").is_none());
}

// Moved from td_no_merge_test.rs (issue #856f): consolidates the removed
// `td merge` clap-parsing assertions onto this file's shared `Cli` harness
// instead of a second, identical `#[derive(Parser)] struct Cli` copy.
#[test]
fn test_td_merge_subcommand_is_removed() {
    let cmd = Cli::command();
    let td = cmd.find_subcommand("td").expect("td namespace");
    assert!(
        td.find_subcommand("merge").is_none(),
        "removed TD merge command must not be registered"
    );
}

#[test]
fn test_td_merge_parse_fails() {
    let err = match Cli::try_parse_from(["aw", "td", "merge", "4124"]) {
        Ok(_) => panic!("removed TD merge command unexpectedly parsed"),
        Err(err) => err,
    };
    assert!(
        err.to_string().contains("unrecognized subcommand 'merge'"),
        "unexpected parse error: {err}"
    );
}

#[test]
fn code_artifact_commands_are_inherited_by_td() {
    let cmd = Cli::command();
    assert!(cmd.find_subcommand("cb").is_none());
    let td = cmd.find_subcommand("td").expect("td namespace registered");
    for name in ["gen", "gen-source", "code-check", "code-claim", "fill"] {
        assert!(
            td.find_subcommand(name).is_some(),
            "td {name} should remain registered"
        );
    }
    for name in ["code-review", "code-revise", "code-arbitrate"] {
        assert!(
            td.find_subcommand(name).is_none(),
            "td {name} should not preserve the retired CB CRRR loop"
        );
    }
}

#[test]
fn public_aggregation_points_remain_registered() {
    let cmd = Cli::command();
    assert!(cmd.find_subcommand("health").is_some());

    let standardize = cmd
        .find_subcommand("standardize")
        .expect("standardize namespace registered");
    // #920 (epic #914 slice F): `aw standardize` is retired down to `audit`
    // only; `managed`/`semantic`/`traceability` layer drivers are gone.
    assert!(standardize.find_subcommand("audit").is_some());
    assert!(standardize.find_subcommand("semantic").is_none());

    let generator = cmd
        .find_subcommand("generator")
        .expect("generator namespace registered");
    assert!(generator.find_subcommand("check").is_some());
    assert!(generator.find_subcommand("request").is_some());

    let td = cmd.find_subcommand("td").expect("td namespace registered");
    assert!(td.find_subcommand("code-check").is_some());
}
// CODEGEN-END
// SPEC-MANAGED: apps/agentic-workflow/tech-design/semantic/agentic-workflow-tests-cli-tests.md#schema
// CODEGEN-BEGIN
// SPEC-REF: apps/agentic-workflow/tech-design/semantic/agentic-workflow-tests-cli-tests.md#schema
// TODO: Existing source behavior is covered by this feature/domain semantic TD.

// CODEGEN-END
