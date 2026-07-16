// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/interfaces/src/commands.md#source
// CODEGEN-BEGIN
use crate::Result;
use clap::Subcommand;

use crate::cli::capability;
use crate::cli::conf;
use crate::cli::drift;
use crate::cli::ec;
use crate::cli::generator;
use crate::cli::guard;
use crate::cli::init;
use crate::cli::issues;
use crate::cli::llm;
use crate::cli::meta;
use crate::cli::project;
use crate::cli::standard_cli;

/// Agentic Workflow CLI commands
#[derive(Subcommand)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/commands.md#source
pub enum Commands {
    /// Create a greenfield project directory and bootstrap Agentic Workflow.
    // @spec apps/agentic-workflow/tech-design/logic/manage-aw-init-templates-as-greenfield-ready-artifacts.md#CLI
    New(init::NewArgs),

    /// Aggregate project readiness, production gates, and blocker status.
    #[command(alias = "hc")]
    Health(project::ProjectHealthArgs),

    /// Product capability completion loop: report/next/run/check.
    Capability(capability::CapabilityArgs),

    /// Generator gap request surface after takeover readiness.
    Generator(generator::GeneratorArgs),

    /// Agent-runtime direct edit/create guard for Codex, Claude Code, and AGY.
    Guard(guard::GuardArgs),

    /// Manage `aw.toml` and Agentic Workflow configuration producers.
    Conf(conf::ConfArgs),

    /// Initialize, synchronize, and check repository/project META-docs.
    Meta(meta::MetaArgs),

    /// Manage work-items — list/show/create/validate across local + GitHub backends.
    // @spec apps/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#cli
    #[command(name = "wi")]
    Issues(issues::IssuesArgs),

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

    /// External-contract lifecycle: draft/fill, independent semantic review, generate, and verify.
    Ec(ec::EcArgs),
}

/// Run an Agentic Workflow CLI command
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/commands.md#source
pub async fn run_command(cmd: Commands) -> Result<()> {
    drift::check_once(env!("AW_BUILD_VERSION"), env!("AW_GIT_SHA"));
    drift::enforce_mutating_verb_gate(env!("AW_BUILD_VERSION"), env!("AW_GIT_SHA"));

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
            guard::run(args).await?;
        }
        Commands::Conf(args) => {
            conf::run(args)?;
        }
        Commands::Meta(args) => {
            meta::run(args)?;
        }
        Commands::Issues(args) => {
            issues::run(args).await?;
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
    }

    Ok(())
}

// CODEGEN-END
