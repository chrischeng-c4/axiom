// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests-cli-tests.md#tests
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
        "view",
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
        "init",
        "health",
        "wi",
        "td",
        "cb",
        "standardize",
        "generator",
        "sync",
        "chat",
    ] {
        assert!(
            cmd.find_subcommand(name).is_some(),
            "{name} should remain registered"
        );
    }
}

#[test]
fn deleted_top_level_commands_fail_as_unknown_commands() {
    let Some(aw) = aw_bin() else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };

    for command in [
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
        "aw run-change",
        "aw workflow",
        "aw revise-artifact",
        "aw artifact",
        "aw validate-spec-structure",
        "aw check-alignment",
        "aw iss",
        "aw issues",
        "aw chat agents",
        "aw handoff",
        "aw takeoff",
        "aw platform",
        "aw hook",
        "aw scaffold-spec",
        "aw project health",
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
}

#[test]
fn canonical_cb_commands_remain_registered() {
    let cmd = Cli::command();
    let cb = cmd.find_subcommand("cb").expect("cb namespace registered");
    for name in [
        "gen",
        "check",
        "claim",
        "fill",
        "review",
        "revise",
        "arbitrate",
    ] {
        assert!(
            cb.find_subcommand(name).is_some(),
            "cb {name} should remain registered"
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
    assert!(standardize.find_subcommand("semantic").is_some());

    let generator = cmd
        .find_subcommand("generator")
        .expect("generator namespace registered");
    assert!(generator.find_subcommand("check").is_some());
    assert!(generator.find_subcommand("request").is_some());

    let cb = cmd.find_subcommand("cb").expect("cb namespace registered");
    assert!(cb.find_subcommand("check").is_some());
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests-cli-tests.md#schema
// CODEGEN-BEGIN
// SPEC-REF: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests-cli-tests.md#schema
// TODO: Existing source behavior is covered by this feature/domain semantic TD.

// CODEGEN-END
