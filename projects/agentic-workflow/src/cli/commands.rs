// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/commands.md#source
// CODEGEN-BEGIN
use crate::Result;
use clap::Subcommand;

use crate::cli::capability;
use crate::cli::chat;
use crate::cli::conf;
use crate::cli::ec;
use crate::cli::generator;
use crate::cli::guard;
use crate::cli::init;
use crate::cli::issues;
use crate::cli::llm;
use crate::cli::project;
use crate::cli::standard_cli;
use crate::cli::standardize;
use crate::cli::view;

/// Agentic Workflow CLI commands
#[derive(Subcommand)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/commands.md#source
pub enum Commands {
    /// Create a greenfield project directory and bootstrap Agentic Workflow.
    // @spec projects/agentic-workflow/tech-design/logic/manage-aw-init-templates-as-greenfield-ready-artifacts.md#CLI
    New(init::NewArgs),

    /// Aggregate project readiness, production gates, and blocker status.
    #[command(alias = "hc")]
    Health(project::ProjectHealthArgs),

    /// Product capability completion loop: report/next/run/check.
    Capability(capability::CapabilityArgs),

    /// Generator gap request surface after takeover readiness.
    Generator(generator::GeneratorArgs),

    /// Agent-runtime direct edit/create guard for Codex and Claude Code.
    Guard(guard::GuardArgs),

    /// Read-only repo reader: projects/libs catalog, README capabilities, EC, TD, and native desktop app.
    View(view::ViewArgs),

    /// Manage `.aw/config.toml` and Agentic Workflow configuration producers.
    Conf(conf::ConfArgs),

    /// Manage work-items — list/show/create/validate across local + GitHub backends.
    // @spec projects/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#cli
    #[command(name = "wi")]
    Issues(issues::IssuesArgs),

    /// Cross-checkout agent messaging via shared plain-text channel
    Chat(chat::ChatArgs),

    /// Offline agent orientation: outline + capability/td/ec pillars + loop.
    Llm(llm::LlmArgs),

    /// Self-update this binary from a published GitHub release.
    Upgrade(standard_cli::UpgradeArgs),

    /// Search, view, or create Agentic Workflow issues.
    Issue(standard_cli::IssueArgs),

    /// File a diagnostics-rich GitHub issue for aw.
    #[command(name = "report-issue")]
    ReportIssue(standard_cli::ReportIssueArgs),

    /// Tech-design and generated-code lifecycle
    Td(crate::cli::td::TdArgs),

    /// External-contract lifecycle: generate tests/tool configs and verify EC gates.
    Ec(ec::EcArgs),

    /// Existing-project workflow guidance and bounded remediation.
    Standardize(standardize::StandardizeArgs),
}

/// Run an Agentic Workflow CLI command
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/commands.md#source
pub async fn run_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::New(args) => {
            init::run_new(args).await?;
        }

        Commands::Health(args) => {
            project::run_health(args).await?;
        }
        Commands::Capability(args) => {
            capability::run(args).await?;
        }
        Commands::Generator(args) => {
            generator::run(args).await?;
        }
        Commands::Guard(args) => {
            guard::run(args)?;
        }
        Commands::View(args) => {
            view::run(args).await?;
        }
        Commands::Conf(args) => {
            conf::run(args)?;
        }
        Commands::Issues(args) => {
            issues::run(args).await?;
        }
        Commands::Chat(args) => {
            chat::run_chat(args)?;
        }
        Commands::Llm(args) => {
            llm::run(args)?;
        }
        Commands::Upgrade(args) => {
            standard_cli::run_upgrade(args).await?;
        }
        Commands::Issue(args) => {
            standard_cli::run_issue(args).await?;
        }
        Commands::ReportIssue(args) => {
            standard_cli::run_report_issue(args).await?;
        }
        Commands::Td(args) => {
            crate::cli::td::run(args).await?;
        }
        Commands::Ec(args) => {
            ec::run(args)?;
        }
        Commands::Standardize(args) => {
            standardize::run(args).await?;
        }
    }

    Ok(())
}

// CODEGEN-END
