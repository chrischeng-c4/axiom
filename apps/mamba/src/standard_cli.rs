//! mamba's adapter over `cli-std` for the three standard agent-facing commands:
//! `llm`, `upgrade`, and `report-issue`.
//!
//! `cli-std` owns the shared logic. This module owns mamba's clap surface,
//! tool identity, topic text, and dispatch glue.

use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::future::Future;

const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "mamba",
    repo: "chrischeng-c4/axiom",
    target: env!("MAMBA_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("MAMBA_GIT_SHA"),
    built_at: env!("MAMBA_BUILT_AT"),
};

const TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "workflow",
        summary: "compile, check, run, and test Python-compatible Mamba programs",
        body: "\
# mamba workflow

mamba is a Python-compatible runtime and package workflow with native code
generation.

Common agent paths:

- `mamba run <file.py>` executes a Python/Mamba source file directly.
- `mamba -c \"print(1)\"` runs inline code with CPython-style semantics.
- `mamba build <file.py>` compiles a source file or project entry point.
- `mamba check <file.tp>` type-checks without code generation.
- `mamba test <path>` runs Python-perspective tests under the mamba runner.
- `mamba test --conformance --category <name>` runs focused CPython fixtures.",
    },
    cli_std::llm::Topic {
        id: "package-management",
        summary: "project env management: add, lock, sync, venv, python, pip, tools",
        body: "\
# mamba package management

mamba's package workflow is local-first and lockfile-driven:

- `mamba init [path]` scaffolds a project.
- `mamba add <spec>` adds a dependency and updates `mamba.lock`.
- `mamba lock --check` verifies the lockfile without rewriting it.
- `mamba sync` converges `.venv`/site-packages to `mamba.lock`.
- `mamba venv ...`, `mamba python ...`, and `mamba pip ...` manage the project
  Python environment.
- `mamba tool run <name> -- <args...>` runs package-provided tools.
- `mamba tool upgrade <name>` upgrades an installed package tool from a frozen
  index. Top-level `mamba upgrade` is reserved for the mamba binary self-update.",
    },
    cli_std::llm::Topic {
        id: "runtime-validation",
        summary: "focused runtime validation using CPython fixtures and debug builds",
        body: "\
# mamba runtime validation

Use focused validation before broad sweeps:

- `cargo build -p mamba --bin mamba` builds the debug binary.
- `MAMBA_BIN=target/debug/mamba cargo test -p mamba --test conformance <fixture>`
  runs one CPython fixture through the conformance harness.
- `target/debug/mamba <fixture.py>` directly runs a fixture during triage.
- `cargo test -p mamba --lib <module>` is useful for Rust-side stdlib modules.

Prefer debug builds for iteration. Release builds are reserved for final
release/performance gates.",
    },
];

pub fn llm_command() -> Command {
    Command::new("llm")
        .about("Print agent-facing docs for driving mamba; offline, no network")
        .arg(
            Arg::new("topic")
                .long("topic")
                .value_name("TOPIC")
                .default_value("outline")
                .help("Topic to print: outline, workflow, package-management, runtime-validation"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .value_parser(["md", "json"])
                .default_value("md")
                .help("Output format"),
        )
}

pub fn upgrade_command() -> Command {
    Command::new("upgrade")
        .about("Update mamba to the latest mamba@* GitHub release")
        .arg(
            Arg::new("version")
                .long("version")
                .value_name("TAG")
                .help("Install a specific release tag, e.g. mamba@0.4.2 or 0.4.2"),
        )
        .arg(
            Arg::new("check")
                .long("check")
                .action(ArgAction::SetTrue)
                .help("Only report whether a newer release exists; do not install"),
        )
        .arg(
            Arg::new("force")
                .long("force")
                .action(ArgAction::SetTrue)
                .help("Reinstall even when the selected version is already installed"),
        )
        .arg(
            Arg::new("yes")
                .short('y')
                .long("yes")
                .action(ArgAction::SetTrue)
                .help("Skip the confirmation prompt"),
        )
}

pub fn report_issue_command() -> Command {
    Command::new("report-issue")
        .about("File a structured mamba issue report against the axiom tracker")
        .arg(
            Arg::new("title")
                .long("title")
                .short('t')
                .value_name("TITLE")
                .help("Issue title; defaults to a title derived from the message"),
        )
        .arg(
            Arg::new("url")
                .long("url")
                .value_name("URL")
                .help("Include a running node's /version and /healthz in the report"),
        )
        .arg(
            Arg::new("repo")
                .long("repo")
                .value_name("OWNER/REPO")
                .help("Target repository; defaults to mamba's release repo"),
        )
        .arg(
            Arg::new("label")
                .long("label")
                .value_name("LABEL")
                .action(ArgAction::Append)
                .help("Add a label; repeatable"),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .action(ArgAction::SetTrue)
                .help("Print the issue that would be filed without creating it"),
        )
        .arg(
            Arg::new("yes")
                .short('y')
                .long("yes")
                .action(ArgAction::SetTrue)
                .help("Skip the confirmation prompt"),
        )
        .arg(
            Arg::new("message")
                .value_name("MSG")
                .num_args(0..)
                .trailing_var_arg(true)
                .allow_hyphen_values(true)
                .help("Free-text description of the problem"),
        )
}

pub fn run_llm(matches: &ArgMatches) -> Result<()> {
    let topic = matches
        .get_one::<String>("topic")
        .map(String::as_str)
        .unwrap_or("outline");
    let format = cli_std::llm::Format::parse(
        matches
            .get_one::<String>("format")
            .map(String::as_str)
            .unwrap_or("md"),
    );
    let out = cli_std::llm::render(TOOL.project, TOOL.version, TOPICS, topic, format)?;
    println!("{out}");
    Ok(())
}

pub fn run_upgrade(matches: &ArgMatches) -> Result<()> {
    block_on(cli_std::upgrade::run(
        &TOOL,
        cli_std::upgrade::Options {
            check: matches.get_flag("check"),
            tag: matches.get_one::<String>("version").cloned(),
            force: matches.get_flag("force"),
            yes: matches.get_flag("yes"),
        },
    ))
}

pub fn run_report_issue(matches: &ArgMatches) -> Result<()> {
    let msg = matches
        .get_many::<String>("message")
        .map(|values| values.cloned().collect::<Vec<_>>().join(" "))
        .unwrap_or_default();
    let title = matches
        .get_one::<String>("title")
        .cloned()
        .unwrap_or_else(|| {
            if msg.trim().is_empty() {
                "mamba: issue report".to_string()
            } else {
                let head: String = msg.lines().next().unwrap_or("").chars().take(72).collect();
                format!("mamba: {head}")
            }
        });
    let message = (!msg.trim().is_empty()).then_some(msg);
    let user_labels = matches
        .get_many::<String>("label")
        .map(|values| values.cloned().collect::<Vec<_>>())
        .unwrap_or_default();

    block_on(cli_std::report_issue::run(
        &TOOL,
        cli_std::report_issue::Options {
            title,
            message,
            url: matches.get_one::<String>("url").cloned(),
            repo: matches.get_one::<String>("repo").cloned(),
            label: std::iter::once("project:mamba".to_string())
                .chain(user_labels)
                .collect(),
            dry_run: matches.get_flag("dry-run"),
            yes: matches.get_flag("yes"),
        },
    ))
}

fn block_on<F>(future: F) -> Result<()>
where
    F: Future<Output = Result<()>>,
{
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(future)
}
