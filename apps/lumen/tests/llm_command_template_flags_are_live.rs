//! DX contract `command_template` fact-freshness gate (#2494).
//!
//! `apps/lumen/src/dx.rs` renders the `lumen llm` task-navigation protocol
//! from the hand-authored `dx-contract` YAML in
//! `apps/lumen/tech-design/interfaces/dx/lumen-dx-contract.md`. Fully-bound
//! `command` steps are already executed against the live binary by
//! `apps/lumen/tests/cli_convention.rs`'s
//! `llm_outline_advertised_topic_commands_parse` /
//! `llm_v2_executes_only_fully_bound_advertised_commands` — but templated
//! `command_template` steps (the ones with `{placeholder}` inputs, e.g.
//! `lumen backup --url {url} --dest {destination}`) are never dispatched:
//! they need real argument values a task-navigation smoke test can't
//! fabricate. Nothing has stopped their literal tokens — the subcommand
//! path and flag names — from drifting out of sync with the actual clap
//! surface if a flag gets renamed elsewhere.
//!
//! This test extracts every `command_template` from the live
//! `lumen llm --topic <id> --format json` output (not the raw YAML file,
//! so the check also exercises the dx.rs rendering path), strips the
//! `{placeholder}` argument values, and confirms the remaining subcommand
//! path resolves (`lumen <path> --help` succeeds) and every literal
//! `--flag` token it names is advertised in that subcommand's own
//! `--help` text.

use serde_json::Value;
use std::process::Command;

fn run_lumen(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("run lumen {args:?}: {err}"));
    assert!(
        output.status.success(),
        "lumen {args:?} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("lumen stdout is utf8")
}

/// Every `command_template` string advertised by the typed task manifest,
/// read from the live binary (id -> template).
fn outline_command_templates() -> Vec<(String, String)> {
    let outline: Value = serde_json::from_str(&run_lumen(&[
        "llm", "--topic", "outline", "--format", "json",
    ]))
    .expect("outline JSON parses");
    let mut out = Vec::new();
    for task in outline["tasks"].as_array().expect("outline has tasks") {
        let topic = task["topic"].as_str().expect("task topic is a string");
        let detail: Value =
            serde_json::from_str(&run_lumen(&["llm", "--topic", topic, "--format", "json"]))
                .unwrap_or_else(|e| panic!("topic {topic} detail JSON parses: {e}"));
        for step in detail["runbook"]["steps"]
            .as_array()
            .expect("runbook has steps")
        {
            if let Some(template) = step.get("command_template").and_then(Value::as_str) {
                out.push((topic.to_string(), template.to_string()));
            }
        }
    }
    out
}

/// Split a `command_template` into (subcommand path, literal `--flag`
/// tokens), dropping `{placeholder}` argument values and anything after a
/// bare `--` wrapped-command separator (`connect-kubernetes`'s trailing
/// `-- {command}`).
fn path_and_flags(template: &str) -> (Vec<&str>, Vec<&str>) {
    let tokens: Vec<&str> = template.split_whitespace().collect();
    assert_eq!(
        tokens.first(),
        Some(&"lumen"),
        "command_template must start with `lumen`: {template}"
    );

    let mut path = Vec::new();
    let mut flags = Vec::new();
    let mut in_flags = false;
    for &tok in &tokens[1..] {
        if tok == "--" {
            // Wrapped-command separator: nothing after this is ours to check.
            break;
        }
        if tok.starts_with("--") {
            in_flags = true;
            flags.push(tok);
        } else if tok.starts_with('{') {
            // A placeholder argument value for the preceding flag/positional.
            continue;
        } else if !in_flags {
            path.push(tok);
        }
    }
    (path, flags)
}

/// #2494: templated command steps' literal subcommand path and flag names
/// must still be advertised by the live CLI surface.
#[test]
fn command_template_literal_tokens_are_live() {
    let templates = outline_command_templates();
    assert!(
        !templates.is_empty(),
        "expected at least one command_template in the DX contract"
    );

    for (topic, template) in templates {
        let (path, flags) = path_and_flags(&template);
        assert!(
            !path.is_empty(),
            "{topic}: command_template has no subcommand path: {template}"
        );

        let mut help_args = path.clone();
        help_args.push("--help");
        let help = run_lumen(&help_args);

        for flag in flags {
            assert!(
                help.contains(flag),
                "{topic}: `{template}` names `{flag}`, but `lumen {} --help` doesn't advertise it:\n{help}",
                path.join(" ")
            );
        }
    }
}
