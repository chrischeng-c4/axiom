// <HANDWRITE gap="standardize:claim-code" tracker="projects-jet-src-standard-cli-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
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
const TOPIC_HELP: &str = "Topic to print: outline (default), workflow, quickstart, stories, browser, build-publish, test-e2e, debug-oracle, recipes";

const TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "workflow",
        summary: "mental model: package, develop, stories, prove, publish",
        body: "\
# jet workflow

jet is a Rust-native JavaScript/TypeScript toolchain. Treat it as one binary
covering package management, dev serving, production builds, Storybook-like
stories, browser proof, tests, publish, and diagnostics.

1. `jet install` / `jet add` manage dependencies and `jet-lock.yaml`.
2. `jet dev` serves an app with HMR.
3. `jet stories` serves CSF stories with a Storybook-compatible manager/preview
   contract for component review.
4. `jet bb` and `jet browser` provide agent-first browser control for real
   screenshots, clicks, console errors, and visual proof.
5. `jet build` builds apps or libraries; `jet check` type-checks.
6. `jet test` and `jet e2e` prove unit/component/product flows.
7. `jet pack` / `jet publish` package libraries for npm-compatible registries.

Packages live in a global content-addressed store (`jet store`). The lockfile is
`jet-lock.yaml`; configuration is `jet.toml` (inspect with `jet config`).",
    },
    cli_std::llm::Topic {
        id: "quickstart",
        summary: "copy-paste app, library, and stories startup",
        body: "\
# jet quickstart

    jet init my-app
    cd my-app
    jet install
    jet dev            # serves with HMR
    jet stories        # component workbench / Storybook-compatible review
    # ...edit src...
    jet build          # production bundle in dist/
    jet test           # native test runner",
    },
    cli_std::llm::Topic {
        id: "stories",
        summary: "Storybook-compatible component workbench",
        body: "\
# jet stories

Use `jet stories` when the task is component review, CSF parity, Controls, docs,
or Storybook migration.

Core commands:

    jet stories --host 127.0.0.1 --port 6134
    jet build --stories

Agent workflow:

1. Start official Storybook and Jet on different ports when comparing parity.
2. Compare `/index.json` first; mismatched story IDs usually explain bad manager
   routing before pixels matter.
3. Validate iframe output with screenshots for representative stories.
4. Click real controls/components in the browser; do not rely on shell success.
5. Inspect console/page errors and network 404s before calling parity good.

Current stories mode aims to match the official Storybook manager/preview
contract while keeping Jet's fast native server path. It handles CSF stories,
Controls, docs canvas rendering, Storybook channel events, preview iframe
routes, and optimized heavy dependency loading for common component-library deps.",
    },
    cli_std::llm::Topic {
        id: "browser",
        summary: "Browser Bridge and real interaction proof",
        body: "\
# jet browser

Use browser proof for UI tasks, stories parity, visual regressions, and e2e
debugging. A passing build is not enough when the requested behavior is visual
or interactive.

Core commands:

    jet bb --help
    jet browser --help

Agent workflow:

1. Open the local target with Browser Bridge or Playwright.
2. Wait for the expected UI state, not just page load.
3. Capture screenshots for desktop/mobile or relevant component frames.
4. Click representative controls and assert visible text/state changed.
5. Record console errors, page exceptions, failed requests, and timing.

For Storybook parity, prefer an oracle that checks official Storybook and Jet
side by side: manager shell, iframe pixels, story contract, and real click
smoke. Use screenshots as the debugging baseline when shell output looks right
but the UI looks wrong.",
    },
    cli_std::llm::Topic {
        id: "build-publish",
        summary: "production builds, pack, publish, and issue #1240 guardrail",
        body: "\
# jet build-publish

Build apps with `jet build`; build libraries with `jet build --lib`. Use
`jet pack` to inspect the tarball locally before publishing. Use `jet publish`
only when the registry result is verified.

Core commands:

    jet build
    jet build --lib
    jet pack
    jet publish --dry-run
    jet publish

Publish guardrail:

#1240 is the regression case for npm installability: a real `jet publish` must
write npm version metadata under `versions[version].dist`, not only upload the
tarball attachment. Standard npm-compatible clients install from
`dist.tarball`, then verify `dist.shasum` and `dist.integrity`.

After any real publish to a registry, verify the published version metadata
explicitly:

    npm view <pkg>@<version> dist --registry <registry>
    curl -s <registry-url>/<encoded-pkg> | jq '.versions[\"<version>\"].dist'

Do not treat exit code 0 from `jet publish` as sufficient installability proof
for registry-facing releases. The registry manifest must contain `dist.tarball`,
`dist.shasum`, and `dist.integrity`.",
    },
    cli_std::llm::Topic {
        id: "test-e2e",
        summary: "native tests and product-flow e2e",
        body: "\
# jet test-e2e

Use `jet test` for native unit/component/integration-style tests and `jet e2e`
for product-flow validation.

Core commands:

    jet test
    jet test --help
    jet e2e run
    jet e2e --help
    jet report --help

Agent workflow:

1. Run the narrowest test that covers the changed behavior.
2. For UI behavior, pair automated tests with browser screenshots/click smoke.
3. Save or inspect HTML reports with `jet report` when debugging failures.
4. Use real services where the repo contract requires them; do not replace
   service-backed behavior with mocks unless the contract says SaaS-only mock.",
    },
    cli_std::llm::Topic {
        id: "debug-oracle",
        summary: "parity debugging with screenshots, index data, and issues",
        body: "\
# jet debug-oracle

When Jet is replacing an existing frontend tool path, debug against the external
oracle before changing assumptions.

Useful checks:

1. Compare official output and Jet output from the same project and story IDs.
2. Check route contracts (`/index.json`, iframe URLs, static assets, websocket
   events) before chasing CSS.
3. Use screenshots or perceptual diffs for visual claims.
4. Click a small but representative set of components.
5. Inspect console errors, page exceptions, and failed network requests.
6. Use `jet issue view <n>` or `jet issue search <query>` to pull live tracker
   context, then update issues with `jet issue comment <n> ...` when evidence
   changes.

For publish or registry bugs, reproduce against a disposable local registry
such as Verdaccio first, then prove the published package is installable with a
standard npm-compatible client.",
    },
    cli_std::llm::Topic {
        id: "recipes",
        summary: "task -> command cheat-sheet",
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
| pack a library             | `jet pack`                    |
| dry-run publish            | `jet publish --dry-run`       |
| real publish               | verify issue #1240 guardrail  |
| start the dev server       | `jet dev`                     |
| start stories workbench    | `jet stories`                 |
| compare UI in browser      | `jet bb --help`               |
| run e2e flows              | `jet e2e run`                 |
| inspect / lint config      | `jet config lint`             |
| update this tool           | `jet upgrade`                 |
| search known issues        | `jet issue search \"hmr\"`      |
| file a bug                 | `jet issue create \"...\"`      |
| reopen + comment follow-up | `jet issue comment 123 \"...\"` |",
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
                .help(TOPIC_HELP),
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

/// `jet issue <search|view|create|comment>` — search, read, file, and follow up
/// on jet issues.
pub fn issue_command() -> Command {
    Command::new("issue")
        .about("Search, view, file, and comment on jet issues on the axiom tracker")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("search")
                .about("Search jet's issues (app:jet); omit the query to list recent")
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
                .about("File a structured issue (auto-tagged app:jet)")
                .arg(
                    Arg::new("title")
                        .long("title")
                        .help("Issue title (default: derived from the message)"),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .action(ArgAction::SetTrue)
                        .help(
                            "Print the issue that would be filed (and its URL) without creating it",
                        ),
                )
                .arg(
                    Arg::new("message")
                        .num_args(0..)
                        .help("Free-text description of the problem"),
                ),
        )
        .subcommand(
            Command::new("comment")
                .about("Comment on an issue and ensure it is open")
                .arg(
                    Arg::new("number")
                        .required(true)
                        .value_parser(clap::value_parser!(u64))
                        .help("Issue number"),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .action(ArgAction::SetTrue)
                        .help("Print the reopen/comment request without changing GitHub state"),
                )
                .arg(
                    Arg::new("message")
                        .num_args(0..)
                        .help("Follow-up note to add after reopening"),
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
        matches
            .get_one::<String>("format")
            .map(String::as_str)
            .unwrap_or("md"),
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

/// `jet issue <verb>` — dispatch search/view/create/comment to cli-std.
/// `create` always tags `app:jet`; `search` defaults to jet's own issues.
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
                    label: vec!["app:jet".to_string()],
                    dry_run: m.get_flag("dry-run"),
                    yes: true,
                },
            )
            .await
        }
        Some(("comment", m)) => {
            let number = *m.get_one::<u64>("number").expect("number is required");
            let msg = m
                .get_many::<String>("message")
                .map(|v| v.cloned().collect::<Vec<_>>().join(" "))
                .filter(|s| !s.trim().is_empty());
            cli_std::issue::comment(
                &TOOL,
                cli_std::issue::CommentOptions {
                    number,
                    message: msg,
                    repo: None,
                    dry_run: m.get_flag("dry-run"),
                    yes: true,
                },
            )
            .await
        }
        _ => anyhow::bail!("unknown `jet issue` subcommand; try search / view / create / comment"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn llm_outline_lists_modern_topics() {
        let out = cli_std::llm::render(
            TOOL.project,
            TOOL.version,
            TOPICS,
            "outline",
            cli_std::llm::Format::Md,
        )
        .expect("render llm outline");
        for topic in [
            "stories",
            "browser",
            "build-publish",
            "test-e2e",
            "debug-oracle",
        ] {
            assert!(
                out.contains(topic),
                "outline should include {topic}, got:\n{out}"
            );
        }
    }

    #[test]
    fn llm_publish_topic_mentions_issue_1240_dist_metadata() {
        let out = cli_std::llm::render(
            TOOL.project,
            TOOL.version,
            TOPICS,
            "build-publish",
            cli_std::llm::Format::Md,
        )
        .expect("render build-publish topic");
        assert!(
            out.contains("#1240"),
            "publish topic should mention issue #1240"
        );
        assert!(
            out.contains("dist.tarball"),
            "publish topic should mention dist.tarball"
        );
        assert!(
            out.contains("dist.integrity"),
            "publish topic should mention dist.integrity"
        );
    }

    #[test]
    fn llm_help_lists_modern_topics() {
        let mut help = Vec::new();
        llm_command()
            .write_long_help(&mut help)
            .expect("render llm help");
        let help = String::from_utf8(help).expect("help is UTF-8");
        for topic in [
            "stories",
            "browser",
            "build-publish",
            "test-e2e",
            "debug-oracle",
        ] {
            assert!(
                help.contains(topic),
                "llm help should include {topic}, got:\n{help}"
            );
        }
    }

    #[test]
    fn issue_help_lists_comment() {
        let mut help = Vec::new();
        issue_command()
            .write_long_help(&mut help)
            .expect("render issue help");
        let help = String::from_utf8(help).expect("help is UTF-8");
        assert!(
            help.contains("comment"),
            "issue help should list comment subcommand, got:\n{help}"
        );
    }

    #[test]
    fn issue_comment_parses_number_message_and_dry_run() {
        let matches = issue_command()
            .try_get_matches_from(["issue", "comment", "123", "--dry-run", "still", "broken"])
            .expect("parse issue comment");
        let Some(("comment", comment)) = matches.subcommand() else {
            panic!("expected comment subcommand");
        };
        assert_eq!(comment.get_one::<u64>("number"), Some(&123));
        assert!(comment.get_flag("dry-run"));
        let message = comment
            .get_many::<String>("message")
            .map(|v| v.cloned().collect::<Vec<_>>().join(" "))
            .unwrap_or_default();
        assert_eq!(message, "still broken");
    }
}

// </HANDWRITE>
