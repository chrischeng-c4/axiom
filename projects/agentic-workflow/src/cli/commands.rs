// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/commands.md#source
// CODEGEN-BEGIN
use crate::Result;
use clap::Subcommand;

use crate::cli::capability;
use crate::cli::chat;
use crate::cli::ec;
use crate::cli::generator;
use crate::cli::init;
use crate::cli::issues;
use crate::cli::llm;
use crate::cli::project;
use crate::cli::run as run_root;
use crate::cli::standard_cli;
use crate::cli::standardize;
use crate::cli::sync;

/// Agentic Workflow CLI commands
#[derive(Subcommand)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/commands.md#source
pub enum Commands {
    // =====================================================================
    // Project initialization
    // =====================================================================

    // @spec projects/agentic-workflow/tech-design/surface/specs/init-command.md#R1
    /// Bootstrap .aw/ config and installed workflow skills/settings
    Init {
        /// Project name (deprecated, ignored)
        #[arg(short, long)]
        name: Option<String>,

        /// Override version downgrade protection and force-replace all assets
        #[arg(short, long)]
        force: bool,
    },

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

    /// Root-driven workflow runner for project, capability, epic, or change scopes.
    Run(run_root::RunArgs),

    /// Auto-discover projects and refresh the `.aw/config.toml` registry block.
    Sync(sync::SyncArgs),

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
        // =================================================================
        // Project initialization
        // =================================================================
        // @spec projects/agentic-workflow/tech-design/surface/specs/init-command.md#R2
        Commands::Init { name, force } => {
            init::run(name.as_deref(), force, None).await?;
        }
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
        Commands::Run(args) => {
            run_root::run(args).await?;
        }
        Commands::Sync(args) => {
            sync::run(args)?;
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
