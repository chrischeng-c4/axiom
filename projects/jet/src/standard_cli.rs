//! jet's adapter over the shared `cli-std` crate for the three standard
//! agent-facing commands (`llm` / `upgrade` / `issue`), per the CLI
//! convention in `CONTRIBUTING.md`.
//!
//! `cli-std` is clap-agnostic and owns the *logic* (offline docs render, release
//! self-update, GitHub issue submit). This module owns jet's *surface*: the clap
//! builders (so jet keeps the convention's flag shape — `--topic`, not a
//! positional), the [`cli_std::ToolInfo`] identity, jet's topic content, and the
//! dispatch that wires the two together.

use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};

/// jet's identity + build provenance for the standard CLI ops. The `JET_*`
/// values are stamped by `build.rs`.
const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "jet",
    repo: "chrischeng-c4/axiom",
    target: env!("JET_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("JET_GIT_SHA"),
    built_at: env!("JET_BUILT_AT"),
};

/// jet's agent-facing `llm` topics — the single in-code source of truth. The
/// `outline` topic + standard-command footer are rendered by `cli-std`.
const TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "workflow",
        summary: "mental model: install → dev → build → test",
        body: "\
# jet workflow

jet is a Rust-native build tool + package manager for JavaScript/TypeScript
(bun/vite/npm in one binary). The mental model:

1. `jet init`              scaffold a new project
2. `jet install`          resolve + install deps from package.json → jet-lock.yaml
3. `jet add <pkg>`        add a dependency (`-D` for devDependencies)
4. `jet dev`              dev server with hot module reload (HMR)
5. `jet build`            production build (app, or a library with `--lib`)
6. `jet test` / `jet e2e` native unit/component tests and product-flow E2E
7. `jet check`            TypeScript type-check

Packages live in a global content-addressed store (`jet store`). The lockfile is
`jet-lock.yaml`; configuration is `jet.toml` (inspect with `jet config`).",
    },
    cli_std::llm::Topic {
        id: "quickstart",
        summary: "copy-paste create → dev → build",
        body: "\
# jet quickstart

    jet init my-app
    cd my-app
    jet install
    jet dev            # serves with HMR
    # ...edit src...
    jet build          # production bundle in dist/
    jet test           # run the native test runner",
    },
    cli_std::llm::Topic {
        id: "recipes",
        summary: "task → command cheat-sheet",
        body: "\
# jet recipes

| task                       | command                       |
|----------------------------|-------------------------------|
| add a dependency           | `jet add lodash`              |
| add a dev dependency       | `jet add -D vitest`           |
| remove a dependency        | `jet remove lodash`           |
| run a package.json script  | `jet run build`               |
| run a one-off binary (npx) | `jet jtx cowsay hi`           |
| type-check                 | `jet check`                   |
| build a library            | `jet build --lib`             |
| start the dev server       | `jet dev`                     |
| run e2e flows              | `jet e2e run`                 |
| inspect / lint config      | `jet config lint`             |
| update this tool           | `jet upgrade`                 |
| search known issues        | `jet issue search \"hmr\"`      |
| file a bug                 | `jet issue create \"...\"`      |",
    },
];

// ---------------------------------------------------------------------------
// clap registration — called from `cli::command()`. Positional slots are
// reserved for subcommands, so every parameter here is a flag.
// ---------------------------------------------------------------------------

/// `jet llm [--topic <topic>] [--format md|json]`
pub fn llm_command() -> Command {
    Command::new("llm")
        .about("Print agent-facing docs for driving jet — offline, no network")
        .arg(
            Arg::new("topic")
                .long("topic")
                .value_name("topic")
                .default_value("outline")
                .help("Topic to print: outline (default), workflow, quickstart, recipes"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .value_parser(["md", "json"])
                .default_value("md")
                .help("Output format"),
        )
}

/// `jet upgrade [--version <tag>] [--check]`
pub fn upgrade_command() -> Command {
    Command::new("upgrade")
        .about("Update jet to the latest jet@* GitHub release")
        .arg(
            Arg::new("version")
                .long("version")
                .help("Install a specific release tag (e.g. jet@0.4.2 or 0.4.2)"),
        )
        .arg(
            Arg::new("check")
                .long("check")
                .action(ArgAction::SetTrue)
                .help("Only report whether a newer release exists; do not install"),
        )
}

/// `jet issue <search|view|create>` — search, read, and file jet issues.
pub fn issue_command() -> Command {
    Command::new("issue")
        .about("Search, view, and file jet issues on the axiom tracker")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("search")
                .about("Search jet's issues (project:jet); omit the query to list recent")
                .arg(
                    Arg::new("query")
                        .num_args(0..)
                        .help("Search text (omit to list recent issues)"),
                )
                .arg(
                    Arg::new("state")
                        .long("state")
                        .value_parser(["open", "closed", "all"])
                        .default_value("open")
                        .help("Issue state filter"),
                )
                .arg(
                    Arg::new("limit")
                        .long("limit")
                        .value_parser(clap::value_parser!(u32))
                        .default_value("20")
                        .help("Max results"),
                ),
        )
        .subcommand(
            Command::new("view")
                .about("Print a single issue by number")
                .arg(
                    Arg::new("number")
                        .required(true)
                        .value_parser(clap::value_parser!(u64))
                        .help("Issue number"),
                ),
        )
        .subcommand(
            Command::new("create")
                .about("File a structured issue (auto-tagged project:jet)")
                .arg(
                    Arg::new("title")
                        .long("title")
                        .help("Issue title (default: derived from the message)"),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .action(ArgAction::SetTrue)
                        .help("Print the issue that would be filed (and its URL) without creating it"),
                )
                .arg(
                    Arg::new("message")
                        .num_args(0..)
                        .help("Free-text description of the problem"),
                ),
        )
}

// ---------------------------------------------------------------------------
// dispatch — wire jet's parsed args into cli-std's logic.
// ---------------------------------------------------------------------------

/// `jet llm` — render the requested topic offline via cli-std.
pub fn run_llm(matches: &ArgMatches) -> Result<()> {
    let topic = matches
        .get_one::<String>("topic")
        .map(String::as_str)
        .unwrap_or("outline");
    let format = cli_std::llm::Format::parse(
        matches.get_one::<String>("format").map(String::as_str).unwrap_or("md"),
    );
    let out = cli_std::llm::render(TOOL.project, TOOL.version, TOPICS, topic, format)?;
    println!("{out}");
    Ok(())
}

/// `jet upgrade` — self-update via cli-std (non-interactive).
pub async fn run_upgrade(matches: &ArgMatches) -> Result<()> {
    cli_std::upgrade::run(
        &TOOL,
        cli_std::upgrade::Options {
            check: matches.get_flag("check"),
            tag: matches.get_one::<String>("version").cloned(),
            force: false,
            yes: true,
        },
    )
    .await
}

/// `jet issue <verb>` — dispatch search/view/create to cli-std. `create` always
/// tags `project:jet`; `search` defaults to jet's own issues.
pub async fn run_issue(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("search", m)) => {
            let query = m
                .get_many::<String>("query")
                .map(|v| v.cloned().collect::<Vec<_>>().join(" "))
                .filter(|s| !s.trim().is_empty());
            cli_std::issue::search(
                &TOOL,
                cli_std::issue::SearchOptions {
                    query,
                    state: m
                        .get_one::<String>("state")
                        .cloned()
                        .unwrap_or_else(|| "open".to_string()),
                    limit: *m.get_one::<u32>("limit").unwrap_or(&20),
                },
            )
            .await
        }
        Some(("view", m)) => {
            let number = *m.get_one::<u64>("number").expect("number is required");
            cli_std::issue::view(&TOOL, number).await
        }
        Some(("create", m)) => {
            let msg = m
                .get_many::<String>("message")
                .map(|v| v.cloned().collect::<Vec<_>>().join(" "))
                .unwrap_or_default();
            let title = m.get_one::<String>("title").cloned().unwrap_or_else(|| {
                if msg.trim().is_empty() {
                    "jet: issue report".to_string()
                } else {
                    let head: String = msg.lines().next().unwrap_or("").chars().take(72).collect();
                    format!("jet: {head}")
                }
            });
            let message = (!msg.trim().is_empty()).then_some(msg);
            cli_std::issue::create(
                &TOOL,
                cli_std::issue::CreateOptions {
                    title,
                    message,
                    url: None,
                    repo: None,
                    label: vec!["project:jet".to_string()],
                    dry_run: m.get_flag("dry-run"),
                    yes: true,
                },
            )
            .await
        }
        _ => anyhow::bail!("unknown `jet issue` subcommand; try search / view / create"),
    }
}
