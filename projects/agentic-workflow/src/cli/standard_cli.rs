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
    #[arg(value_name = "message", trailing_var_arg = true)]
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
    }
}

fn report_issue_labels(mut extra: Vec<String>) -> Vec<String> {
    let mut labels = vec!["project:agentic-workflow".to_string()];
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

    #[test]
    fn report_issue_labels_include_project_once() {
        let labels = report_issue_labels(vec![
            "bug".to_string(),
            "project:agentic-workflow".to_string(),
        ]);

        assert_eq!(
            labels,
            vec!["project:agentic-workflow".to_string(), "bug".to_string()]
        );
    }
}
