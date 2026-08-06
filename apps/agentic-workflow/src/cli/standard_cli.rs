use crate::Result;
use clap::{Args, Subcommand};

const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "aw",
    repo: "chrischeng-c4/axiom",
    target: env!("AW_TARGET"),
    version: env!("AW_BUILD_VERSION"),
    git_sha: env!("AW_GIT_SHA"),
    built_at: env!("AW_BUILT_AT"),
};

const ISSUE_TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "agentic-workflow",
    repo: "chrischeng-c4/axiom",
    target: env!("AW_TARGET"),
    version: env!("AW_BUILD_VERSION"),
    git_sha: env!("AW_GIT_SHA"),
    built_at: env!("AW_BUILT_AT"),
};

/// Self-update this binary from a published GitHub release.
#[derive(Debug, Args, Clone)]
pub struct UpgradeArgs {
    /// Report the current and latest version without modifying the binary.
    #[arg(long)]
    pub check: bool,

    /// Install this exact version (`0.1.0` or `aw@0.1.0`) instead of latest.
    #[arg(long)]
    pub version: Option<String>,

    /// Reinstall even when already on the selected version.
    #[arg(long)]
    pub force: bool,

    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    pub yes: bool,
}

/// File a diagnostics-rich GitHub issue for aw.
#[derive(Debug, Args, Clone)]
pub struct ReportIssueArgs {
    /// Issue title.
    #[arg(short = 't', long)]
    pub title: Option<String>,

    /// Target repository (`owner/name`); defaults to aw's release repo.
    #[arg(long)]
    pub repo: Option<String>,

    /// Add a label (repeatable).
    #[arg(long)]
    pub label: Vec<String>,

    /// Assemble and print the report without submitting anything.
    #[arg(long)]
    pub dry_run: bool,

    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Free-text description placed above the diagnostics block.
    #[arg(value_name = "message", trailing_var_arg = true)]
    pub message: Vec<String>,
}

/// Search, view, or create Agentic Workflow issues via the shared CLI convention.
#[derive(Debug, Args, Clone)]
pub struct IssueArgs {
    #[command(subcommand)]
    pub command: IssueCommand,
}

#[derive(Debug, Subcommand, Clone)]
pub enum IssueCommand {
    /// Search Agentic Workflow issues.
    Search(IssueSearchArgs),
    /// View one Agentic Workflow issue by number.
    View(IssueViewArgs),
    /// Create a diagnostics-rich Agentic Workflow issue.
    Create(IssueCreateArgs),
    /// Reopen (if needed) and add a diagnostics-rich comment to an issue.
    Comment(IssueCommentArgs),
}

#[derive(Debug, Args, Clone)]
pub struct IssueSearchArgs {
    /// Free-text query. Omit to list recent issues.
    pub query: Option<String>,

    /// Issue state: open, closed, or all.
    #[arg(long, default_value = "open")]
    pub state: String,

    /// Maximum number of issues to return.
    #[arg(long, default_value_t = 20)]
    pub limit: u32,
}

#[derive(Debug, Args, Clone)]
pub struct IssueViewArgs {
    /// Issue number.
    pub number: u64,
}

#[derive(Debug, Args, Clone)]
pub struct IssueCreateArgs {
    /// Issue title.
    #[arg(short = 't', long)]
    pub title: Option<String>,

    /// Target repository (`owner/name`); defaults to aw's repository.
    #[arg(long)]
    pub repo: Option<String>,

    /// Add a label (repeatable).
    #[arg(long)]
    pub label: Vec<String>,

    /// Assemble and print the report without submitting anything.
    #[arg(long)]
    pub dry_run: bool,

    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Free-text description placed above the diagnostics block.
    //
    // Deliberately NOT `trailing_var_arg`: that setting makes clap stop
    // recognizing flags (including `--dry-run`) once the first message word
    // is consumed, so `issue create <title> "<msg words>" --dry-run` (flag
    // after the message, the natural invocation order) silently swallows
    // `--dry-run` as a literal message word and falls through to the real
    // submission call instead of previewing. See #3335.
    #[arg(value_name = "message")]
    pub message: Vec<String>,
}

#[derive(Debug, Args, Clone)]
pub struct IssueCommentArgs {
    /// Issue number to comment on.
    pub number: u64,

    /// Target repository (`owner/name`); defaults to aw's repository.
    #[arg(long)]
    pub repo: Option<String>,

    /// Assemble and print the comment without submitting anything.
    #[arg(long)]
    pub dry_run: bool,

    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Optional verification note placed above the diagnostics block.
    //
    // Deliberately NOT `trailing_var_arg`: that setting makes clap stop
    // recognizing flags (including `--dry-run`) once the first message word
    // is consumed, so `issue comment <n> "<msg words>" --dry-run` (flag
    // after the message, the natural invocation order) silently swallows
    // `--dry-run` as a literal message word and falls through to the real
    // submission call instead of previewing. See #3335.
    #[arg(value_name = "message")]
    pub message: Vec<String>,
}

pub async fn run_upgrade(args: UpgradeArgs) -> Result<()> {
    cli_std::upgrade::run(
        &TOOL,
        cli_std::upgrade::Options {
            check: args.check,
            tag: args.version,
            force: args.force,
            yes: args.yes,
        },
    )
    .await
}

pub async fn run_report_issue(args: ReportIssueArgs) -> Result<()> {
    let message = (!args.message.is_empty()).then(|| args.message.join(" "));
    cli_std::report_issue::run(
        &TOOL,
        cli_std::report_issue::Options {
            title: args.title.unwrap_or_else(|| "aw issue report".to_string()),
            message,
            url: None,
            repo: args.repo,
            label: report_issue_labels(args.label),
            dry_run: args.dry_run,
            yes: args.yes,
        },
    )
    .await
}

pub async fn run_issue(args: IssueArgs) -> Result<()> {
    match args.command {
        IssueCommand::Search(args) => {
            cli_std::issue::search(
                &ISSUE_TOOL,
                cli_std::issue::SearchOptions {
                    query: args.query,
                    state: args.state,
                    limit: args.limit,
                },
            )
            .await
        }
        IssueCommand::View(args) => cli_std::issue::view(&ISSUE_TOOL, args.number).await,
        IssueCommand::Create(args) => {
            let message = (!args.message.is_empty()).then(|| args.message.join(" "));
            cli_std::issue::create(
                &ISSUE_TOOL,
                cli_std::issue::CreateOptions {
                    title: args.title.unwrap_or_else(|| "aw issue report".to_string()),
                    message,
                    url: None,
                    repo: args.repo,
                    label: report_issue_labels(args.label),
                    dry_run: args.dry_run,
                    yes: args.yes,
                },
            )
            .await
        }
        IssueCommand::Comment(args) => {
            let message = (!args.message.is_empty()).then(|| args.message.join(" "));
            cli_std::issue::comment(
                &ISSUE_TOOL,
                cli_std::issue::CommentOptions {
                    number: args.number,
                    message,
                    repo: args.repo,
                    dry_run: args.dry_run,
                    yes: args.yes,
                },
            )
            .await
        }
    }
}

fn report_issue_labels(mut extra: Vec<String>) -> Vec<String> {
    let mut labels = vec!["app:agentic-workflow".to_string()];
    for label in extra.drain(..) {
        if !labels.contains(&label) {
            labels.push(label);
        }
    }
    labels
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn report_issue_labels_include_project_once() {
        let labels =
            report_issue_labels(vec!["bug".to_string(), "app:agentic-workflow".to_string()]);

        assert_eq!(
            labels,
            vec!["app:agentic-workflow".to_string(), "bug".to_string()]
        );
    }

    #[derive(Debug, clap::Parser)]
    struct IssueCommandCli {
        #[command(subcommand)]
        command: IssueCommand,
    }

    #[test]
    fn issue_comment_parses_number_dry_run_and_message() {
        let cli = IssueCommandCli::try_parse_from([
            "aw",
            "comment",
            "123",
            "--dry-run",
            "still",
            "failing",
        ])
        .expect("issue comment should parse");

        let IssueCommand::Comment(args) = cli.command else {
            panic!("expected IssueCommand::Comment");
        };
        assert_eq!(args.number, 123);
        assert!(args.dry_run);
        assert!(!args.yes);
        assert_eq!(
            args.message,
            vec!["still".to_string(), "failing".to_string()]
        );
    }

    #[test]
    fn issue_comment_requires_a_number() {
        let err = IssueCommandCli::try_parse_from(["aw", "comment"])
            .expect_err("issue comment without a number should fail to parse");
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    // --- #3335 regression: `--dry-run` after the free-text message ---------
    //
    // Root cause was not a missing read of `dry_run` (it's read correctly in
    // `cli_std::issue::{comment,create}`'s early-return branch) but
    // `trailing_var_arg = true` on `message`: once clap starts consuming the
    // free-text positional it stops recognizing *any* subsequent token as a
    // flag, even a defined one like `--dry-run`, and swallows it as a literal
    // message word instead. `aw issue comment <n> "<msg>" --dry-run` — the
    // exact order from the live-reproduced incident (#3332) — hit this. The
    // fix drops `trailing_var_arg` from `IssueCommentArgs`/`IssueCreateArgs`
    // so flags are recognized regardless of where they sit relative to the
    // message.

    #[test]
    fn issue_comment_dry_run_after_message_still_registers() {
        let cli = IssueCommandCli::try_parse_from([
            "aw",
            "comment",
            "123",
            "still",
            "failing",
            "--dry-run",
        ])
        .expect("issue comment should parse");

        let IssueCommand::Comment(args) = cli.command else {
            panic!("expected IssueCommand::Comment");
        };
        assert!(
            args.dry_run,
            "--dry-run after the message must still register: {args:?}"
        );
        assert_eq!(
            args.message,
            vec!["still".to_string(), "failing".to_string()],
            "the flag must not be swallowed into the message text"
        );
    }

    #[test]
    fn issue_create_dry_run_after_message_still_registers() {
        let cli = IssueCommandCli::try_parse_from([
            "aw",
            "create",
            "--title",
            "t",
            "still",
            "failing",
            "--dry-run",
        ])
        .expect("issue create should parse");

        let IssueCommand::Create(args) = cli.command else {
            panic!("expected IssueCommand::Create");
        };
        assert!(
            args.dry_run,
            "--dry-run after the message must still register: {args:?}"
        );
        assert_eq!(
            args.message,
            vec!["still".to_string(), "failing".to_string()],
            "the flag must not be swallowed into the message text"
        );
    }

    #[tokio::test]
    async fn issue_comment_dry_run_after_message_makes_no_submission_call() {
        // Exercises the real parse -> `run_issue` dispatch ->
        // `cli_std::issue::comment` path end to end. That function's
        // `dry_run` branch returns before constructing an HTTP client or
        // resolving any credential/courier URL, so a fast `Ok(())` here is
        // proof no network/mutating call was attempted — not just that the
        // flag parsed correctly.
        let cli = IssueCommandCli::try_parse_from([
            "aw",
            "comment",
            "919191",
            "still",
            "failing",
            "--dry-run",
        ])
        .expect("issue comment should parse");
        let IssueCommand::Comment(ref args) = cli.command else {
            panic!("expected IssueCommand::Comment");
        };
        assert!(args.dry_run);

        let result = run_issue(IssueArgs {
            command: cli.command,
        })
        .await;
        assert!(
            result.is_ok(),
            "dry-run comment must return Ok without attempting a submission: {result:?}"
        );
    }

    #[tokio::test]
    async fn issue_create_dry_run_after_message_makes_no_submission_call() {
        // Same end-to-end proof as the comment case above, for `issue
        // create`'s identical opts-threading/parsing pattern.
        let cli = IssueCommandCli::try_parse_from([
            "aw",
            "create",
            "--title",
            "t",
            "still",
            "failing",
            "--dry-run",
        ])
        .expect("issue create should parse");
        let IssueCommand::Create(ref args) = cli.command else {
            panic!("expected IssueCommand::Create");
        };
        assert!(args.dry_run);

        let result = run_issue(IssueArgs {
            command: cli.command,
        })
        .await;
        assert!(
            result.is_ok(),
            "dry-run create must return Ok without attempting a submission: {result:?}"
        );
    }

    #[test]
    fn issue_comment_dispatches_via_cli_std_comment_options() {
        // Argument-shape parity with `cli_std::issue::CommentOptions` (the
        // dispatch this test's sibling above proves the parser produces)
        // rather than a live network round trip — `run_issue` forwards
        // `IssueCommentArgs` fields 1:1 into `CommentOptions`.
        let args = IssueCommentArgs {
            number: 927,
            repo: Some("o/n".to_string()),
            dry_run: true,
            yes: false,
            message: vec!["hello".to_string()],
        };
        let message = (!args.message.is_empty()).then(|| args.message.join(" "));
        let opts = cli_std::issue::CommentOptions {
            number: args.number,
            message,
            repo: args.repo.clone(),
            dry_run: args.dry_run,
            yes: args.yes,
        };
        assert_eq!(opts.number, 927);
        assert_eq!(opts.message.as_deref(), Some("hello"));
        assert_eq!(opts.repo.as_deref(), Some("o/n"));
        assert!(opts.dry_run);
        assert!(!opts.yes);
    }

    #[test]
    fn issue_create_dispatches_via_cli_std_create_options() {
        // `dry_run` parity for `create`, mirroring the `comment` dispatch
        // test above — `run_issue` forwards `IssueCreateArgs.dry_run` 1:1
        // into `CreateOptions.dry_run`.
        let args = IssueCreateArgs {
            title: Some("t".to_string()),
            repo: Some("o/n".to_string()),
            label: vec![],
            dry_run: true,
            yes: false,
            message: vec!["hello".to_string()],
        };
        let message = (!args.message.is_empty()).then(|| args.message.join(" "));
        let opts = cli_std::issue::CreateOptions {
            title: args.title.clone().unwrap_or_default(),
            message,
            url: None,
            repo: args.repo.clone(),
            label: report_issue_labels(args.label.clone()),
            dry_run: args.dry_run,
            yes: args.yes,
        };
        assert_eq!(opts.title, "t");
        assert_eq!(opts.message.as_deref(), Some("hello"));
        assert_eq!(opts.repo.as_deref(), Some("o/n"));
        assert!(opts.dry_run);
        assert!(!opts.yes);
    }
}
