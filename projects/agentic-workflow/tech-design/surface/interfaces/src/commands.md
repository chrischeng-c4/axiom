---
id: projects-score-src-commands-rs
fill_sections: [overview, changes]
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: cli-workflow-chain
    claim: cli-workflow-chain
    coverage: full
    rationale: "CLI entrypoint and dispatch surfaces support root command parsing and workflow command routing."
---

# Standardized projects/agentic-workflow/src/cli/commands.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/commands.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Commands` | projects/agentic-workflow/src/cli/commands.rs | enum | pub | 23 |  |
| `run_command` | projects/agentic-workflow/src/cli/commands.rs | function | pub | 100 | run_command(cmd: Commands) -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/commands.rs -->
```rust
// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/commands.md#source
// CODEGEN-BEGIN
use crate::Result;
use clap::Subcommand;

use crate::cli::capability;
use crate::cli::chat;
use crate::cli::ec;
use crate::cli::generator;
use crate::cli::guard;
use crate::cli::init;
use crate::cli::issues;
use crate::cli::llm;
use crate::cli::project;
use crate::cli::standard_cli;
use crate::cli::standardize;
use crate::cli::sync;
use crate::cli::view;

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

        /// Read-only: report stale CLAUDE.md/AGENTS.md projections without writing (issue #984)
        #[arg(long)]
        check: bool,
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

    /// Agent-runtime direct edit/create guard for Codex and Claude Code.
    Guard(guard::GuardArgs),

    /// Read-only repo reader: projects/libs catalog, README capabilities, EC, TD, and native desktop app.
    View(view::ViewArgs),

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
        // =================================================================
        // Project initialization
        // =================================================================
        // @spec projects/agentic-workflow/tech-design/surface/specs/init-command.md#R2
        Commands::Init { name, force, check } => {
            if check {
                init::run_check()?;
            } else {
                init::run(name.as_deref(), force, None).await?;
            }
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
        Commands::Guard(args) => {
            guard::run(args)?;
        }
        Commands::View(args) => {
            view::run(args).await?;
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
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/commands.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
  - path: projects/agentic-workflow/src/cli/commands.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Issue #848: whole-file resync — adds the Guard/View/Llm/Upgrade/Issue/
      ReportIssue command_refs and their guard/view/llm/standard_cli module
      imports and run_command match arms that had drifted out of the mirror.
  - path: projects/agentic-workflow/src/cli/commands.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Issue #984 (init-projector slice 1/3): `Commands::Init` gains a
      read-only `--check` flag; `run_command` dispatches it to
      `init::run_check()` instead of the mutating `init::run(...)` path.
```
