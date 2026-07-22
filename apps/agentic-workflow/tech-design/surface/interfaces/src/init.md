---
id: projects-score-src-init-rs
fill_sections: [overview, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Support CLI surfaces expose AW Core bootstrap, chat, hook, project, and workspace invariants."
---

# Standardized apps/agentic-workflow/src/cli/init.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/cli/init.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `NewArgs` | apps/agentic-workflow/src/cli/init.rs | struct | pub | 62 |  |
| `run_new` | apps/agentic-workflow/src/cli/init.rs | function | pub | 96 | run_new(args: NewArgs) -> Result<()> |
| `WorkspaceType` | apps/agentic-workflow/src/cli/init.rs | enum | pub(crate) | 649 |  |
| `detect_workspace_type` | apps/agentic-workflow/src/cli/init.rs | function | pub(crate) | 665 | detect_workspace_type(project_root: &Path) -> WorkspaceType |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=apps/agentic-workflow/src/cli/init.rs -->
```rust
// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/interfaces/src/init.md#source
// CODEGEN-BEGIN
use crate::cli::doc_mirror;
use crate::models::{SddConfig, SddInterface};
use crate::services::project_registry;
use crate::{Context, Result};
use anyhow::bail;
use clap::Args;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};
use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

// Current version for tracking upgrades
const SDD_VERSION: &str = env!("CARGO_PKG_VERSION");

// Claude Code Skills
const SKILL_WI: &str = include_str!("../../templates/cli/mainthread/skills/aw-wi/SKILL.md");
const SKILL_BUILD_DEBUG: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-build-debug/SKILL.md");
const SKILL_MAMBA_TEST_COVERAGE: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-mamba-test-coverage/SKILL.md");
const SKILL_BUILD_RELEASE: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-build-release/SKILL.md");
const SKILL_HEALTH: &str = include_str!("../../templates/cli/mainthread/skills/aw-health/SKILL.md");
const SKILL_GUARD: &str = include_str!("../../templates/cli/mainthread/skills/aw-guard/SKILL.md");
const SKILL_GOAL: &str = include_str!("../../templates/cli/mainthread/skills/aw-goal/SKILL.md");
const SCRIPT_BUILD_RELEASE: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-build-release/scripts/release.sh");
// @spec apps/agentic-workflow/tech-design/surface/specs/init-command.md#R15
const SCRIPT_BUILD_DEBUG: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-build-debug/scripts/build.sh");
const SCRIPT_MAMBA_TEST_COVERAGE: &str = include_str!(
    "../../templates/cli/mainthread/skills/aw-mamba-test-coverage/scripts/coverage.sh"
);

// Claude Code Agent (subagent) definitions (issue #1034: init-projector
// follow-up to #986 — `templates/cli/mainthread/agents/` is the sole source
// for the aw-* subagent fleet, same model as `aw_skill_entries`).
// @spec apps/agentic-workflow/tech-design/surface/specs/init-command.md#R18
const AGENT_AW_DEV: &str = include_str!("../../templates/cli/mainthread/agents/aw-dev.md");
const AGENT_AW_TD_WRITER: &str =
    include_str!("../../templates/cli/mainthread/agents/aw-td-writer.md");
const AGENT_AW_EC_WRITER: &str =
    include_str!("../../templates/cli/mainthread/agents/aw-ec-writer.md");
const AGENT_AW_EC_REVIEWER: &str =
    include_str!("../../templates/cli/mainthread/agents/aw-ec-reviewer.md");
const AGENT_AW_HW_FILLER: &str =
    include_str!("../../templates/cli/mainthread/agents/aw-hw-filler.md");
const AGENT_PROJECT_PLANNER: &str =
    include_str!("../../templates/cli/mainthread/agents/project-planner.md");
const AGENT_PROJECT_DEV: &str =
    include_str!("../../templates/cli/mainthread/agents/project-dev.md");
const AGENT_PROJECT_RESEARCH: &str =
    include_str!("../../templates/cli/mainthread/agents/project-research.md");

// Claude Code settings.json template
// @spec apps/agentic-workflow/tech-design/surface/specs/init-command.md#R9
const SETTINGS_JSON_TEMPLATE: &str = include_str!("../../templates/cli/mainthread/settings.json");

/// Arguments for `aw new`.
///
/// `aw new` creates the project directory first, then delegates to the same
/// project asset installer used for greenfield bootstrapping.
// @spec apps/agentic-workflow/tech-design/logic/manage-aw-init-templates-as-greenfield-ready-artifacts.md#CLI
#[derive(Debug, Args)]
pub struct NewArgs {
    /// Project directory name when --path is not supplied
    pub name: String,

    /// Explicit target directory. When omitted, target is ./<name>.
    #[arg(long, value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Allow reusing an existing non-empty directory and force-refresh managed assets.
    #[arg(short, long)]
    pub force: bool,

    /// Create the target directory without installing Agentic Workflow assets.
    #[arg(long)]
    pub no_assets: bool,

    /// Read-only: report aw-* agent fleet projection drift across
    /// `.claude/agents/`, `.codex/agents/`, and `.agents/agents/` for an
    /// existing target instead of installing (issue #1842). `name`/`--path`
    /// still resolve the target the same way they do for a normal install.
    #[arg(long)]
    pub check_agents: bool,

    /// Write-only: project the aw-* agent fleet to `.claude/agents/`,
    /// `.codex/agents/`, and `.agents/agents/` for an existing target,
    /// bypassing the full asset installer (no aw.toml/hooks/settings/
    /// skills/META-doc refresh) so it is safe to run against an
    /// already-initialized project (issue #1842). `name`/`--path` still
    /// resolve the target the same way they do for a normal install.
    #[arg(long)]
    pub sync_agents: bool,
}

// @spec apps/agentic-workflow/tech-design/logic/manage-aw-init-templates-as-greenfield-ready-artifacts.md#Logic
pub async fn run_new(args: NewArgs) -> Result<()> {
    let current_dir = env::current_dir()?;
    if args.check_agents {
        let target = resolve_new_target(&current_dir, &args.name, args.path.as_deref())?;
        return run_agent_fleet_check(&target);
    }
    if args.sync_agents {
        let target = resolve_new_target(&current_dir, &args.name, args.path.as_deref())?;
        if !target.is_dir() {
            anyhow::bail!(
                "target directory does not exist: {} (run `aw new` without --sync-agents first)",
                target.display()
            );
        }
        install_agent_fleet(&target, &target.join(".claude"))?;
        println!(
            "{}",
            format!(
                "✅ aw-* agent fleet synced at {} (.claude/agents, .codex/agents, .agents/agents)",
                target.display()
            )
            .green()
            .bold()
        );
        return Ok(());
    }
    let outcome = run_new_with_current_dir(args, &current_dir)?;

    println!();
    println!(
        "{}",
        format!("✅ Project ready at {}", outcome.target.display())
            .green()
            .bold()
    );
    println!();
    println!("{}", "⏭️  Next Steps:".yellow().bold());
    println!("   {}", format!("cd {}", outcome.target.display()).cyan());
    if !outcome.assets_installed {
        println!("   {}", "aw health --project <project>".cyan());
    }

    Ok(())
}

struct NewProjectOutcome {
    target: PathBuf,
    assets_installed: bool,
}

fn run_new_with_current_dir(args: NewArgs, current_dir: &Path) -> Result<NewProjectOutcome> {
    let target = resolve_new_target(current_dir, &args.name, args.path.as_deref())?;
    prepare_new_target(&target, args.force)?;

    if args.no_assets {
        println!(
            "{}",
            format!("📁 Created project directory {}", target.display()).cyan()
        );
        println!(
            "   ℹ Skipped Agentic Workflow asset installation because --no-assets was supplied"
        );
        return Ok(NewProjectOutcome {
            target,
            assets_installed: false,
        });
    }

    run_at_project_root(args.force, &target)?;

    Ok(NewProjectOutcome {
        target,
        assets_installed: true,
    })
}

fn resolve_new_target(current_dir: &Path, name: &str, path: Option<&Path>) -> Result<PathBuf> {
    if name.trim().is_empty() {
        anyhow::bail!("project name must not be empty");
    }

    let raw_target = path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(name));

    if raw_target.as_os_str().is_empty() {
        anyhow::bail!("target path must not be empty");
    }

    if raw_target.is_absolute() {
        Ok(raw_target)
    } else {
        Ok(current_dir.join(raw_target))
    }
}

fn prepare_new_target(target: &Path, force: bool) -> Result<()> {
    if target.exists() {
        if !target.is_dir() {
            anyhow::bail!(
                "target path exists and is not a directory: {}",
                target.display()
            );
        }
        if !force && !is_directory_empty(target)? {
            anyhow::bail!(
                "target directory is not empty: {} (rerun aw new with --force to install assets there)",
                target.display()
            );
        }
        return Ok(());
    }

    std::fs::create_dir_all(target)?;
    Ok(())
}

fn is_directory_empty(path: &Path) -> Result<bool> {
    Ok(std::fs::read_dir(path)?.next().is_none())
}

fn run_at_project_root(force: bool, project_root: &Path) -> Result<()> {
    let legacy_score_dir = project_root.join(concat!(".", "score"));
    let legacy_cclab_dir = project_root.join("cclab");
    let sdd_dir = crate::shared::workspace::workspace_runtime_path(project_root);
    let claude_dir = project_root.join(".claude");

    if legacy_score_dir.exists() {
        anyhow::bail!(
            "legacy Agentic Workflow state found at {}; active runtime state now lives under /tmp/aw and root config lives in aw.toml. Move or remove the old directory explicitly, then rerun the relevant producer command.",
            legacy_score_dir.display()
        );
    }

    // Auto-migrate: rename cclab/ → runtime workspace dir if the legacy dir exists
    if legacy_cclab_dir.exists() && !sdd_dir.exists() {
        println!(
            "{}",
            "🔄 Migrating cclab/ → /tmp/aw runtime workspace...".cyan()
        );
        std::fs::rename(&legacy_cclab_dir, &sdd_dir)?;
        println!("   ✓ Renamed cclab/ to {}", sdd_dir.display());
        println!();
    }

    // Check if already initialized
    let is_initialized = sdd_dir.exists();

    if is_initialized {
        // Update mode: overwrite system files, preserve project.md
        let old_version =
            read_version_from_config_or_file(&sdd_dir).unwrap_or_else(|| "unknown".to_string());
        let old_version_trimmed = old_version.trim();

        // Check for version downgrade (skip with --force)
        if !force && old_version_trimmed != "unknown" && old_version_trimmed != SDD_VERSION {
            if !crate::cli::update::is_newer(SDD_VERSION, old_version_trimmed) {
                println!(
                    "{}",
                    format!(
                        "⚠️  Cannot downgrade from {} to {}",
                        old_version_trimmed, SDD_VERSION
                    )
                    .yellow()
                    .bold()
                );
                println!();
                println!(
                    "   {} This would downgrade your SDD installation.",
                    "⚠️".yellow()
                );
                println!(
                    "   {} Current CLI version: {}",
                    "ℹ️".cyan(),
                    SDD_VERSION.yellow()
                );
                println!(
                    "   {} Installed version:  {}",
                    "ℹ️".cyan(),
                    old_version_trimmed.green()
                );
                println!();
                println!(
                    "{}",
                    "💡 To upgrade, install a newer version of the CLI first:".yellow()
                );
                println!("   curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/cclab/main/install.sh | bash");
                println!();
                return Ok(());
            }
        }

        println!(
            "{}",
            format!(
                "🔄 Updating Agentic Workflow {} → {}...",
                old_version_trimmed, SDD_VERSION
            )
            .cyan()
            .bold()
        );
        println!();
        run_update(&project_root, &sdd_dir, &claude_dir, force)?;
    } else {
        // Fresh install - CLI interface, determine platform
        let interface = SddInterface::Cli;
        let platform_toml = determine_platform(&project_root)?;

        println!(
            "{}",
            format!("🎭 Initializing Agentic Workflow v{}...", SDD_VERSION)
                .cyan()
                .bold()
        );
        println!("   Interface: {}", interface.name().green());
        println!();
        run_fresh_install(
            &project_root,
            &sdd_dir,
            &claude_dir,
            interface,
            platform_toml,
        )?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Platform selection
// ---------------------------------------------------------------------------

// Platform type selected during init
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Platform {
    GitHub,
    GitLab,
}

// Auth method selected during init
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthMethod {
    Cli,
    Token,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PlatformTomlUpdate {
    Preserve,
    Remove,
    Replace(String),
}

fn determine_platform_update(
    project_root: &Path,
    existing_config: Option<&str>,
) -> Result<PlatformTomlUpdate> {
    if !io::stdin().is_terminal() {
        if existing_config
            .map(|content| content.contains("[agentic_workflow.issue_platform]"))
            .unwrap_or(false)
        {
            println!("   ℹ Non-interactive init: preserving existing issue platform");
            return Ok(PlatformTomlUpdate::Preserve);
        }
        println!("   ℹ Non-interactive init: skipping issue platform selection");
        return Ok(PlatformTomlUpdate::Remove);
    }

    determine_platform(project_root).map(|platform_toml| match platform_toml {
        Some(toml) => PlatformTomlUpdate::Replace(toml),
        None => PlatformTomlUpdate::Remove,
    })
}

// Interactive platform selection for project asset installation.
///
// Returns the TOML text for the `[platform]` section, or None if user chose None.
fn determine_platform(project_root: &Path) -> Result<Option<String>> {
    if !io::stdin().is_terminal() {
        println!("   ℹ Non-interactive init: skipping issue platform selection");
        return Ok(None);
    }

    let items = &[
        "GitHub - CLI (gh)",
        "GitHub - API (token)",
        "GitLab - CLI (glab)",
        "GitLab - API (token)",
        "Jira",
        "None",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select issue platform")
        .items(items)
        .default(0)
        .interact()?;

    let (platform, auth_method) = match selection {
        0 => (Platform::GitHub, AuthMethod::Cli),
        1 => (Platform::GitHub, AuthMethod::Token),
        2 => (Platform::GitLab, AuthMethod::Cli),
        3 => (Platform::GitLab, AuthMethod::Token),
        4 => {
            // Jira setup
            let repo = detect_repo_from_git(project_root);
            let repo_str = repo.as_deref().unwrap_or("PROJECT");

            print!("   Jira project key (default: {}): ", repo_str);
            io::stdout().flush()?;
            let mut jira_input = String::new();
            io::stdin().read_line(&mut jira_input)?;
            let project_key = if jira_input.trim().is_empty() {
                repo_str.to_string()
            } else {
                jira_input.trim().to_string()
            };

            print!("   Jira base URL (e.g. https://yourorg.atlassian.net): ");
            io::stdout().flush()?;
            let mut url_input = String::new();
            io::stdin().read_line(&mut url_input)?;
            let base_url = url_input.trim().to_string();

            let toml = format!(
                "\n[agentic_workflow.issue_platform]\ntype = \"jira\"\nproject = \"{}\"\nbase_url = \"{}\"\nauth_method = \"token\"\n",
                project_key, base_url
            );
            return Ok(Some(toml));
        }
        5 => return Ok(None),
        _ => (Platform::GitHub, AuthMethod::Cli),
    };

    // Detect repo from git remote
    let repo = detect_repo_from_git(project_root);

    let cli_tool = match platform {
        Platform::GitHub => "gh",
        Platform::GitLab => "glab",
    };

    // Verify CLI tool if CLI auth selected
    if auth_method == AuthMethod::Cli {
        match Command::new(cli_tool).arg("--version").output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                println!(
                    "   {} {} {}",
                    "✓".green(),
                    cli_tool,
                    version.trim().dimmed()
                );
            }
            _ => {
                println!(
                    "   {} {} not found. Install it first: {}",
                    "⚠️".yellow(),
                    cli_tool,
                    match platform {
                        Platform::GitHub => "https://cli.github.com",
                        Platform::GitLab => "https://gitlab.com/gitlab-org/cli",
                    }
                );
            }
        }
        println!();
    }

    // Build TOML text
    let platform_type = match platform {
        Platform::GitHub => "github",
        Platform::GitLab => "gitlab",
    };

    let repo_str = repo.as_deref().unwrap_or("owner/repo");

    let mut toml = format!(
        "\n[agentic_workflow.issue_platform]\ntype = \"{}\"\nrepo = \"{}\"\n",
        platform_type, repo_str
    );

    match auth_method {
        AuthMethod::Cli => {
            toml.push_str("auth_method = \"cli\"\n");
        }
        AuthMethod::Token => {
            let envfield = match platform {
                Platform::GitHub => "GITHUB_TOKEN",
                Platform::GitLab => "GITLAB_TOKEN",
            };

            toml.push_str(&format!("auth_method = \"token\"\n"));

            // Prompt for token
            print!("   Enter {} (or press Enter to skip): ", envfield);
            io::stdout().flush()?;

            let mut token_input = String::new();
            io::stdin().read_line(&mut token_input)?;
            let token = token_input.trim();

            if !token.is_empty() {
                let env_path = project_root.join(".env");
                let mut env_content = if env_path.exists() {
                    std::fs::read_to_string(&env_path)?
                } else {
                    String::new()
                };
                if !env_content.ends_with('\n') && !env_content.is_empty() {
                    env_content.push('\n');
                }
                env_content.push_str(&format!("{}={}\n", envfield, token));
                std::fs::write(&env_path, &env_content)?;
                println!("   {} Wrote {} to .env", "✓".green(), envfield);
                ensure_gitignore_entry(project_root, ".env")?;
            }
            println!();
        }
    }

    Ok(Some(toml))
}

// Replace a TOML section (from header to next `[` header or EOF).
fn replace_toml_section(content: &str, header: &str, replacement: Option<&str>) -> String {
    let mut result = String::new();
    let mut in_section = false;

    for line in content.lines() {
        if line.trim() == header
            || line
                .trim()
                .starts_with(&format!("{}.", header.trim_start_matches('[')))
        {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with('[') {
            in_section = false;
            // Fall through to add this line (it's the start of the next section)
        }
        if !in_section {
            result.push_str(line);
            result.push('\n');
        }
    }

    // Append replacement at end
    if let Some(repl) = replacement {
        if !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(repl);
        if !repl.ends_with('\n') {
            result.push('\n');
        }
    }

    result
}

fn apply_platform_update(content: &str, platform_update: &PlatformTomlUpdate) -> String {
    match platform_update {
        PlatformTomlUpdate::Preserve => content.to_string(),
        PlatformTomlUpdate::Remove => {
            replace_toml_section(content, "[agentic_workflow.issue_platform]", None)
        }
        PlatformTomlUpdate::Replace(platform_toml) => replace_toml_section(
            content,
            "[agentic_workflow.issue_platform]",
            Some(platform_toml.as_str()),
        ),
    }
}

fn refresh_existing_config_content(
    content: &str,
    old_version: &str,
    platform_update: &PlatformTomlUpdate,
) -> (String, Vec<String>) {
    let (migrated, applied) = crate::cli::migrate::migrate_config(content, old_version.trim());
    let migrated = update_version_in_content(&migrated, SDD_VERSION);

    // Replace/remove the platform section only when the operator selected a
    // new platform. Non-interactive updates preserve the existing routing.
    let migrated = apply_platform_update(&migrated, platform_update);
    let migrated = replace_toml_section(&migrated, "[workflow.agents]", None);

    (migrated, applied)
}

// Detect repo (owner/repo) from git remote URL.
///
fn detect_repo_from_git(project_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(project_root)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if let Some(rest) = url.strip_prefix("git@") {
        let path = rest.split(':').nth(1)?;
        return Some(path.trim_end_matches(".git").to_string());
    }

    let parts: Vec<&str> = url.trim_end_matches(".git").rsplitn(3, '/').collect();
    if parts.len() >= 2 {
        return Some(format!("{}/{}", parts[1], parts[0]));
    }

    None
}

// Ensure an entry exists in .gitignore
fn ensure_gitignore_entry(project_root: &Path, entry: &str) -> Result<()> {
    let gitignore_path = project_root.join(".gitignore");
    let content = if gitignore_path.exists() {
        std::fs::read_to_string(&gitignore_path)?
    } else {
        String::new()
    };

    if content.lines().any(|line| line.trim() == entry) {
        return Ok(());
    }

    let mut new_content = content;
    if !new_content.ends_with('\n') && !new_content.is_empty() {
        new_content.push('\n');
    }
    new_content.push_str(entry);
    new_content.push('\n');
    std::fs::write(&gitignore_path, new_content)?;
    println!("   {} Added '{}' to .gitignore", "✓".green(), entry);

    Ok(())
}

// ---------------------------------------------------------------------------
// Workspace type detection (REQ-5)
// ---------------------------------------------------------------------------

// Workspace type detected from project root markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/init.md#source
pub(crate) enum WorkspaceType {
    /// `Cargo.toml` containing `[workspace]` section → Rust monorepo.
    RustCargo,
    /// `pyproject.toml` present → Python project.
    Python,
    /// `package.json` present → JS/TS project.
    NodeJs,
    /// No recognized workspace marker found.
    Unknown,
}

// Detect the workspace type from the project root directory.
///
// Priority order: Rust (Cargo.toml with `[workspace]`) > Python (pyproject.toml) >
// JS/TS (package.json) > Unknown.
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/init.md#source
pub(crate) fn detect_workspace_type(project_root: &Path) -> WorkspaceType {
    let cargo_path = project_root.join("Cargo.toml");
    if cargo_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&cargo_path) {
            if content.contains("[workspace]") {
                return WorkspaceType::RustCargo;
            }
        }
    }
    if project_root.join("pyproject.toml").exists() {
        return WorkspaceType::Python;
    }
    if project_root.join("package.json").exists() {
        return WorkspaceType::NodeJs;
    }
    WorkspaceType::Unknown
}

// Populate `config.specs.scopes` with default entries for a Rust Cargo workspace.
///
// Scans `crates/` and `projects/` directories for immediate subdirectories and
// registers each as a scope entry (`name → "crates"` or `name → "projects"`).
// Called during fresh install when `detect_workspace_type` returns `RustCargo`.
fn populate_rust_scopes(config: &mut SddConfig, project_root: &Path) {
    // Scan crates/ for package directories
    let crates_dir = project_root.join("crates");
    if let Ok(entries) = std::fs::read_dir(&crates_dir) {
        let mut names: Vec<String> = entries
            .flatten()
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| e.file_name().to_str().map(|n| n.to_string()))
            .collect();
        names.sort();
        for name in names {
            config.specs.scopes.insert(name, "crates".to_string());
        }
    }
    // Scan projects/ for project directories
    let projects_dir = project_root.join("projects");
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        let mut names: Vec<String> = entries
            .flatten()
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| e.file_name().to_str().map(|n| n.to_string()))
            .collect();
        names.sort();
        for name in names {
            config.specs.scopes.insert(name, "projects".to_string());
        }
    }
}

// Build a TOML comment hint for the spec scopes section based on workspace type.
///
// Returns a raw TOML string (with commented section header and example entries)
// to append to `config.toml` when no real scope entries could be auto-detected.
// Returns `None` for unknown workspace types where no sensible default exists.
fn build_scopes_comment_hint(workspace_type: WorkspaceType) -> Option<String> {
    match workspace_type {
        WorkspaceType::RustCargo => Some(
            "\n# Spec scope mapping — maps spec group name → parent subdir under .aw/tech-design/\n\
             # [specs.scopes]\n\
             # my-crate = \"crates\"\n\
             # my-project = \"projects\"\n"
                .to_string(),
        ),
        WorkspaceType::Python => Some(
            "\n# Spec scope mapping — maps spec group name → parent subdir under .aw/tech-design/\n\
             # [specs.scopes]\n\
             # my-package = \"src\"\n"
                .to_string(),
        ),
        WorkspaceType::NodeJs => Some(
            "\n# Spec scope mapping — maps spec group name → parent subdir under .aw/tech-design/\n\
             # [specs.scopes]\n\
             # my-package = \"packages\"\n"
                .to_string(),
        ),
        WorkspaceType::Unknown => None,
    }
}

// ---------------------------------------------------------------------------
// Fresh install / update
// ---------------------------------------------------------------------------

// Fresh install: create all directories and files
fn run_fresh_install(
    project_root: &Path,
    sdd_dir: &Path,
    claude_dir: &Path,
    interface: SddInterface,
    platform_toml: Option<String>,
) -> Result<()> {
    // Create directory structure. `tech-design/` is a durable project
    // artifact and lives at the project root (`workspace::tech_design_path`),
    // not inside the ephemeral `/tmp/aw` runtime workspace (`sdd_dir`) — see
    // CLAUDE.md "durable project artifacts live under their project
    // directories" (#1302 residue).
    println!("{}", "📁 Creating directory structure...".cyan());
    std::fs::create_dir_all(sdd_dir)?;
    std::fs::create_dir_all(crate::shared::workspace::tech_design_path(project_root))?;

    // Create Claude Code skills directory
    let skills_dir = claude_dir.join("skills");
    std::fs::create_dir_all(&skills_dir)?;

    // Create config with selected interface
    let mut config = SddConfig::with_interface(interface);
    config.set_version(SDD_VERSION);

    // Detect workspace type and pre-populate spec scopes for Rust monorepos.
    let workspace_type = detect_workspace_type(project_root);
    if workspace_type == WorkspaceType::RustCargo {
        populate_rust_scopes(&mut config, project_root);
    }
    config.save(project_root)?;

    // Append platform + workspace-specific scopes hint to config.toml
    let config_path = project_root.join("aw.toml");
    let mut content = std::fs::read_to_string(&config_path)?;
    if let Some(platform) = &platform_toml {
        content.push_str(platform);
    }
    // Append a commented scopes hint when no real entries were auto-detected
    if config.specs.scopes.is_empty() {
        if let Some(hint) = build_scopes_comment_hint(workspace_type) {
            content.push_str(&hint);
        }
    }
    std::fs::write(&config_path, content)?;

    println!("   ✓ aw.toml (interface: {})", interface.name());

    // Install system files
    install_system_files(project_root, sdd_dir, claude_dir)?;

    // The public `aw meta` registry is the sole META-doc producer.
    crate::cli::meta::sync_repository_product_docs(project_root)?;

    Ok(())
}

// Update mode: overwrite config.toml, update system files
fn run_update(project_root: &Path, sdd_dir: &Path, claude_dir: &Path, force: bool) -> Result<()> {
    println!("{}", "📦 User data:".cyan());
    println!("   ✓ project tech-design roots (untouched)");

    let config_path = project_root.join(crate::shared::workspace::CONFIG_FILE);
    let legacy_config_path = sdd_dir.join("config.toml");
    let existing_config = if config_path.exists() {
        Some(std::fs::read_to_string(&config_path)?)
    } else if legacy_config_path.exists() {
        Some(std::fs::read_to_string(&legacy_config_path)?)
    } else {
        None
    };
    let platform_update = determine_platform_update(project_root, existing_config.as_deref())?;

    if let Some(content) = existing_config {
        let old_version =
            read_version_from_config_or_file(sdd_dir).unwrap_or_else(|| "0.0.0".to_string());
        let (migrated, applied) =
            refresh_existing_config_content(&content, old_version.trim(), &platform_update);

        std::fs::write(&config_path, &migrated)?;
        if !applied.is_empty() {
            println!("   ✓ aw.toml (migrated: {})", applied.join(", "));
        } else if force {
            println!("   ✓ aw.toml (force refreshed)");
        } else {
            println!("   ✓ aw.toml (updated)");
        }
    } else {
        let mut config = SddConfig::with_interface(SddInterface::Cli);
        config.set_version(SDD_VERSION);
        config.save(project_root)?;

        let mut content = std::fs::read_to_string(&config_path)?;
        content = apply_platform_update(&content, &platform_update);
        std::fs::write(&config_path, content)?;
        println!("   ✓ aw.toml (created)");
    }
    println!();

    // Clean up legacy scripts directory (no longer used)
    let scripts_dir = sdd_dir.join("scripts");
    if scripts_dir.exists() {
        let _ = std::fs::remove_dir_all(&scripts_dir);
    }

    // Migrate specs/knowledge/ → specs/ (knowledge concept merged into specs)
    let old_knowledge = sdd_dir.join("specs/knowledge");
    if old_knowledge.exists() {
        let mut migrated = 0;
        for entry in std::fs::read_dir(&old_knowledge)? {
            let entry = entry?;
            let dest = sdd_dir.join("specs").join(entry.file_name());
            if !dest.exists() {
                std::fs::rename(entry.path(), &dest)?;
                migrated += 1;
            }
        }
        // Remove the knowledge directory (may fail if non-empty due to conflicts)
        let _ = std::fs::remove_dir_all(&old_knowledge);
        if migrated > 0 {
            println!(
                "   {} Migrated specs/knowledge/ → specs/ ({} items)",
                "✓".green(),
                migrated
            );
        }
    }

    // Install/update system files
    install_system_files(project_root, sdd_dir, claude_dir)?;

    // The public `aw meta` registry is the sole META-doc producer.
    crate::cli::meta::sync_repository_product_docs(project_root)?;

    // Clean up legacy .version file (version now lives in config.toml)
    let legacy_version_file = sdd_dir.join(".version");
    if legacy_version_file.exists() {
        let _ = std::fs::remove_file(&legacy_version_file);
    }

    println!();
    println!("{}", "✅ Update complete!".green().bold());

    Ok(())
}

// Install/update all system files (skills, retired-agent cleanup, hooks, settings)
fn install_system_files(project_root: &Path, _sdd_dir: &Path, claude_dir: &Path) -> Result<()> {
    let skills_dir = claude_dir.join("skills");
    std::fs::create_dir_all(&skills_dir)?;

    // Install Claude Code Skills
    println!("{}", "🤖 Updating Claude Code Skills...".cyan());
    install_claude_skills(&skills_dir)?;

    // Install the same aw-* skills into `.agents/skills/` (issue #986:
    // init-projector slice 3/3 — templates/ is the sole source, projected to
    // BOTH runtime trees so `.agents` is never a hand-maintained mirror).
    println!();
    println!("{}", "🤖 Updating Codex/.agents Skills...".cyan());
    let agents_skills_dir = project_root.join(".agents").join("skills");
    std::fs::create_dir_all(&agents_skills_dir)?;
    install_agents_skills(&agents_skills_dir)?;

    // Install the aw-* agent fleet on all three hosts (issue #1842:
    // generalized from Claude-only #1034) from the same
    // `templates/cli/mainthread/agents/` source as the skill installers
    // above.
    println!();
    println!("{}", "🧠 Updating Claude/Codex/AGY Agents...".cyan());
    install_agent_fleet(project_root, claude_dir)?;

    // Remove retired Claude Code hook scripts.
    println!();
    println!("{}", "🪝 Retiring Claude Code Hooks...".cyan());
    install_hooks(claude_dir)?;

    // Install/merge settings.json
    println!();
    println!("{}", "⚙️  Updating .claude/settings.json...".cyan());
    install_settings_json(claude_dir)?;

    // Install shell completions
    println!();
    println!("{}", "🐚 Installing shell completions...".cyan());
    install_shell_completions()?;

    Ok(())
}

// Read version from config.toml `version` key, with legacy `.version` file fallback.
fn read_version_from_config_or_file(sdd_dir: &Path) -> Option<String> {
    let config_path = sdd_dir.join("config.toml");
    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("version") {
                    if let Some(val) = trimmed.strip_prefix("version") {
                        let val = val.trim().trim_start_matches('=').trim();
                        let val = val.trim_matches('"').trim_matches('\'');
                        if !val.is_empty() {
                            return Some(val.to_string());
                        }
                    }
                }
                // Stop searching after [project] or [workflow] sections begin
                if trimmed.starts_with('[') {
                    break;
                }
            }
        }
    }
    // Legacy fallback for projects that haven't run init yet
    let version_file = sdd_dir.join(".version");
    std::fs::read_to_string(&version_file)
        .ok()
        .map(|v| v.trim().to_string())
}

// Update the `version = "..."` line in config.toml content, or prepend it if missing.
fn update_version_in_content(content: &str, new_version: &str) -> String {
    let version_line = format!("version = \"{}\"", new_version);
    let mut result = String::new();
    let mut found = false;
    for line in content.lines() {
        if !found && line.trim().starts_with("version") && line.contains('=') {
            result.push_str(&version_line);
            found = true;
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }
    if !found {
        format!("{}\n\n{}", version_line, content)
    } else {
        result
    }
}

// Every `aw-*` skill's directory name + templates-sourced `SKILL.md`
// content (issue #986: the single list consumed by BOTH the `.claude/skills`
// installer and the `.agents/skills` installer, so the two trees can never
// list a different skill set than each other or than `templates/`).
fn aw_skill_entries() -> Vec<(&'static str, &'static str)> {
    vec![
        ("aw-wi", SKILL_WI),
        ("aw-build-debug", SKILL_BUILD_DEBUG),
        ("aw-mamba-test-coverage", SKILL_MAMBA_TEST_COVERAGE),
        ("aw-build-release", SKILL_BUILD_RELEASE),
        ("aw-health", SKILL_HEALTH),
        ("aw-guard", SKILL_GUARD),
        ("aw-goal", SKILL_GOAL),
    ]
}

// Companion `scripts/<file>` payloads for the subset of `aw-*` skills that
// ship one (issue #986: shared by both skill-tree installers; scripts need
// no `.agents` transform — verified zero `.claude`/`CLAUDE` literal
// references in any of the 4 scripts).
fn skill_script_entries() -> &'static [(&'static str, &'static str, &'static str)] {
    &[
        ("aw-build-debug", "build.sh", SCRIPT_BUILD_DEBUG),
        (
            "aw-mamba-test-coverage",
            "coverage.sh",
            SCRIPT_MAMBA_TEST_COVERAGE,
        ),
        ("aw-build-release", "release.sh", SCRIPT_BUILD_RELEASE),
    ]
}

// Legacy/retired skill directory names pruned from a skills tree on every
// the project asset installer (issue #986: shared by both `.claude/skills` and `.agents/skills`
// so a deprecated skill can never survive in one tree only).
fn deprecated_skill_names() -> Vec<&'static str> {
    vec![
        "genesis-proposal",
        "genesis-challenge",
        "genesis-reproposal",
        "genesis-implement",
        "genesis-review",
        "genesis-resolve-reviews",
        "genesis-fix",
        "genesis-verify",
        // Old workflow skill names (renamed)
        "genesis-plan",
        "genesis-impl",
        "genesis-archive",
        "genesis-plan-change",
        "genesis-impl-change",
        "genesis-merge-change",
        // Old cc- prefix names
        "cc-gen-plan-change",
        "cc-gen-impl-change",
        "cc-gen-merge-change",
        // Deprecated individual workflow skills (replaced by run-change)
        "cclab-sdd-decide-change",
        "cclab-sdd-plan-change",
        "cclab-sdd-impl-change",
        "cclab-sdd-merge-change",
        // Replaced by explore-specs + explore-codebase
        "cclab-gemini-explore",
        // Renamed: llm → agent
        "cclab-sdd-llm",
        // Renamed: cclab-sdd-* -> aw-*.
        "cclab-sdd-run-change",
        "cclab-sdd-agent",
        "cclab-sdd-fillback-main-specs",
        "cclab-sdd-merge",
        "cclab-sdd-revise-artifact",
        // Removed: /aw:agent skill (subprocess orchestrator dispatch,
        // deleted when Score moved to client-dispatched executor model).
        "score-agent",
        // Legacy sdd-* and score-* prefixed skills (renamed to aw-* prefix).
        "sdd-run-change",
        "sdd-merge",
        "sdd-fillback-main-specs",
        "sdd-codex-review",
        "sdd-gemini-explore-specs",
        "sdd-gemini-explore-codebase",
        "sdd-revise-artifact",
        "sdd-issue",
        "sdd-issue-patrol",
        "score-run-change",
        "score-codex-review",
        "score-gemini-explore-specs",
        "score-gemini-explore-codebase",
        "score-merge",
        "score-revise-artifact",
        "score-wi",
        "score-wi-patrol",
        "score-handoff",
        "score-takeoff",
        "score-build-debug",
        "score-release-patch",
        "score-mamba-test-coverage",
        "score-td-create",
        "score-cb-fill",
        "score-cb-claim",
        "score-standardize-run",
        "score-standardize-managed-loop",
        "score-standardize-regenerable-loop",
        "aw-run-change",
        "aw-revise-artifact",
        "aw-handoff",
        "aw-takeoff",
        "aw-standardize-run",
        "aw-standardize-managed-loop",
        "aw-standardize-regenerable-loop",
        // Removed: cron-style issue patrol is superseded by
        // `aw capability run --project`.
        "aw-wi-patrol",
        "score-build-release",
        "score-chat-listen",
        "score-fillback-main-specs",
        // #1503: the cross-checkout chat transport and its listener skill are
        // retired without a compatibility alias or subagent replacement.
        "aw-chat-listen",
        // Removed: `aw td merge` no longer exists (LINEAR lifecycle;
        // `aw td code-check` is the terminal step).
        "aw-merge",
        // Removed: the `aw standardize` namespace no longer exists (#1278,
        // epic #1270 R7). `audit check` reporting folded into `aw health`'s
        // `takeover-audit` axis; `audit record` rehomed as `aw td
        // audit-record`.
        "aw-standardize",
        // #1858: eight stale skills retired (lifecycle-superseded +
        // external-model helpers no longer in use).
        "aw-release-patch",
        "aw-cb-claim",
        "aw-cb-fill",
        "aw-td-create",
        "aw-capability",
        "aw-codex-review",
        "aw-gemini-explore-codebase",
        "aw-gemini-explore-specs",
        // #1897: the generic Stop-hook goal-loop skill (never reliably
        // fired, see its own "Known gaps") is retired in favor of the
        // CLI-owned `aw goal` verifiable-condition loop + thin `aw-goal`
        // dispatcher skill.
        "goal-loop",
    ]
}

// Remove every [`deprecated_skill_names`] directory still present under
// `skills_dir` (issue #986: shared by both skill-tree installers).
fn prune_deprecated_skills(skills_dir: &Path) -> Result<()> {
    for deprecated in deprecated_skill_names() {
        let deprecated_dir = skills_dir.join(deprecated);
        if deprecated_dir.exists() {
            std::fs::remove_dir_all(&deprecated_dir)?;
            println!("   {} {} (removed)", "✗".red(), deprecated);
        }
    }
    Ok(())
}

// Write one skill's `SKILL.md` under `skills_dir/<name>/`.
fn write_skill_file(skills_dir: &Path, name: &str, content: &str) -> Result<()> {
    let skill_dir = skills_dir.join(name);
    std::fs::create_dir_all(&skill_dir)?;
    std::fs::write(skill_dir.join("SKILL.md"), content)?;
    println!("   ✓ {}", name);
    Ok(())
}

// Install every [`skill_script_entries`] companion script under `skills_dir`
// with executable permissions (issue #986: shared by both skill-tree
// installers; scripts are byte-identical in both trees, no transform).
fn install_skill_scripts(skills_dir: &Path) -> Result<()> {
    for (skill_name, script_name, content) in skill_script_entries() {
        let scripts_dir = skills_dir.join(skill_name).join("scripts");
        std::fs::create_dir_all(&scripts_dir)?;
        let script_path = scripts_dir.join(script_name);
        std::fs::write(&script_path, content)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
        }
    }
    Ok(())
}

// Install/refresh every `aw-*` skill under `.claude/skills/` from
// `templates/cli/mainthread/skills/` verbatim (the `.claude` tree is the
// untransformed install source; see [`install_agents_skills`] for the
// sibling `.agents/skills/` projection).
// @spec apps/agentic-workflow/tech-design/surface/specs/init-command.md#R12
// @spec apps/agentic-workflow/tech-design/surface/specs/init-command.md#R13
fn install_claude_skills(skills_dir: &Path) -> Result<()> {
    prune_deprecated_skills(skills_dir)?;
    for (name, content) in aw_skill_entries() {
        write_skill_file(skills_dir, name, content)?;
    }
    install_skill_scripts(skills_dir)?;
    Ok(())
}

// Install/refresh every `aw-*` skill under `.agents/skills/` (issue #986:
// init-projector slice 3/3). Same skill set and deprecated-prune list as
// [`install_claude_skills`], with each `SKILL.md` body run through
// [`doc_mirror::agents_skill_body_from_claude_skill_body`] so the two trees
// can only ever differ by the declared transform, never by hand-editing.
fn install_agents_skills(skills_dir: &Path) -> Result<()> {
    prune_deprecated_skills(skills_dir)?;
    for (name, content) in aw_skill_entries() {
        let projected = doc_mirror::agents_skill_body_from_claude_skill_body(content);
        write_skill_file(skills_dir, name, &projected)?;
    }
    install_skill_scripts(skills_dir)?;
    Ok(())
}

// Every `aw-*` subagent's file name (without extension) + templates-sourced
// `<name>.md` content (issue #1034: init-projector follow-up to #986 —
// `templates/cli/mainthread/agents/` is the sole source, same pattern as
// [`aw_skill_entries`]).
fn aw_agent_entries() -> Vec<(&'static str, &'static str)> {
    vec![
        ("aw-dev", AGENT_AW_DEV),
        ("aw-td-writer", AGENT_AW_TD_WRITER),
        ("aw-ec-writer", AGENT_AW_EC_WRITER),
        ("aw-ec-reviewer", AGENT_AW_EC_REVIEWER),
        ("aw-hw-filler", AGENT_AW_HW_FILLER),
    ]
}

/// A fully rendered canonical agent entry. The five `aw-*` entries are
/// static, while project roles replace the project/name placeholders in one
/// of the three role templates below before projection.
#[derive(Debug, Clone)]
struct AgentFleetEntry {
    name: String,
    raw: String,
}

/// The three user-facing roles that every registered app and `projects/mamba`
/// receives. Their templates are host-neutral; the `model_tier` frontmatter
/// selects the one explicit three-host mapping in [`AGENT_MODEL_TIERS`].
const PROJECT_AGENT_ROLES: &[(&str, &str)] = &[
    ("planner", AGENT_PROJECT_PLANNER),
    ("dev", AGENT_PROJECT_DEV),
    ("research", AGENT_PROJECT_RESEARCH),
];

/// Render the project-specific role fleet from the repository's authoritative
/// `[[projects]]` registry. Scope is deliberately narrow: all direct
/// `apps/*` projects plus the top-level `projects/mamba`; libraries, Sift,
/// nested Mamba libraries, and legacy duplicate project roots are excluded.
///
/// A non-Axiom project has no matching registry rows and therefore continues
/// to receive only the reusable `aw-*` fleet.
fn project_role_entries(project_root: &Path) -> Result<Vec<AgentFleetEntry>> {
    let mut entries = Vec::new();
    for project in project_registry::load_project_config_rows(project_root)? {
        let project_path = project.path.replace('\\', "/");
        let is_direct_app = project_path.starts_with("apps/");
        let is_mamba = project.name == "mamba" && project_path == "projects/mamba";
        if !is_direct_app && !is_mamba {
            continue;
        }

        for (role, template) in PROJECT_AGENT_ROLES {
            let name = format!("{}-{role}", project.name);
            let raw = template
                .replace("{{agent_name}}", &name)
                .replace("{{project_name}}", &project.name)
                .replace("{{project_path}}", &project_path);
            entries.push(AgentFleetEntry { name, raw });
        }
    }
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(entries)
}

/// Every managed agent in the current target. `aw-*` remains available in
/// every repository; the project-role fleet appears only where the project
/// registry identifies Axiom's direct apps or Mamba.
fn agent_fleet_entries(project_root: &Path) -> Result<Vec<AgentFleetEntry>> {
    let mut entries: Vec<AgentFleetEntry> = aw_agent_entries()
        .into_iter()
        .map(|(name, raw)| AgentFleetEntry {
            name: name.to_string(),
            raw: raw.to_string(),
        })
        .collect();
    entries.extend(project_role_entries(project_root)?);
    Ok(entries)
}

// Legacy/retired subagent definition file names (without extension) pruned
// from `.claude/agents/` on every asset install (issue #1034: renamed from
// the previous inline `retired_agents` list in [`install_agents`] to match
// [`deprecated_skill_names`]'s shape; same entries, same behavior).
fn deprecated_agent_names() -> Vec<&'static str> {
    vec![
        "sdd-change-implementation",
        "sdd-change-spec",
        "sdd-reference-context",
        "sdd-review",
        "sdd-issue-author",
        // score-reference-context replaced by score-issue-author
        "score-reference-context",
        // score-change-* retired — `score workflow` is gone, see aw td
        "score-change-implementation",
        "score-change-spec",
        "score-review",
        "score-issue-author",
        "score-issue-reviewer",
        "score-issue-reviser",
        "score-td-author",
        "score-td-reviewer",
        "score-td-reviser",
        "score-cb-handwriter",
    ]
}

// Remove every [`deprecated_agent_names`] file still present under
// `agents_dir`, at the given file extension (issue #1842: generalized from a
// hardcoded `.md` so the same prune list covers Claude/AGY `.md` and Codex
// `.toml` projections — extends the mechanism named in R4/AC4 rather than
// duplicating it per host).
fn prune_deprecated_fleet_files(agents_dir: &Path, ext: &str) -> Result<()> {
    for deprecated in deprecated_agent_names() {
        let deprecated_path = agents_dir.join(format!("{deprecated}.{ext}"));
        if deprecated_path.exists() {
            std::fs::remove_file(&deprecated_path)?;
            println!("   {} {} (removed retired)", "✗".red(), deprecated);
        }
    }
    Ok(())
}

// Remove every [`deprecated_agent_names`] `.md` file still present under
// `agents_dir` (the historical Claude-only entry point; kept so existing
// callers/tests are unaffected by the issue #1842 multi-host generalization).
fn prune_deprecated_agents(agents_dir: &Path) -> Result<()> {
    prune_deprecated_fleet_files(agents_dir, "md")
}

// Write one subagent's `<name>.md` under `agents_dir`.
fn write_agent_file(agents_dir: &Path, name: &str, content: &str) -> Result<()> {
    std::fs::write(agents_dir.join(format!("{name}.md")), content)?;
    println!("   ✓ {}", name);
    Ok(())
}

// Install/refresh every `aw-*` subagent definition under `.claude/agents/`
// from `templates/cli/mainthread/agents/` verbatim (issue #1034: same
// single-source-of-truth model as [`install_claude_skills`]). Claude is one
// of three host projections since issue #1842 — see [`install_agent_fleet`]
// for the Codex/`.codex/agents/` and AGY/`.agents/agents/` siblings, all
// still sourced from `templates/cli/mainthread/agents/` alone.
// @spec apps/agentic-workflow/tech-design/surface/specs/init-command.md#R5
// @spec apps/agentic-workflow/tech-design/surface/specs/init-command.md#R6
// @spec apps/agentic-workflow/tech-design/surface/specs/init-command.md#R18
fn install_agents(claude_dir: &Path) -> Result<()> {
    let agents_dir = claude_dir.join("agents");
    std::fs::create_dir_all(&agents_dir)?;

    prune_deprecated_agents(&agents_dir)?;
    for (name, content) in aw_agent_entries() {
        write_agent_file(&agents_dir, name, content)?;
    }

    Ok(())
}

/// Project the already-rendered fleet into Claude Code. Kept separate from
/// [`install_agents`] so the historical reusable-agent installer remains a
/// compact unit-test surface while the full producer can include per-project
/// entries.
fn install_claude_agent_entries(claude_dir: &Path, entries: &[AgentFleetEntry]) -> Result<()> {
    let agents_dir = claude_dir.join("agents");
    std::fs::create_dir_all(&agents_dir)?;
    prune_deprecated_agents(&agents_dir)?;
    for entry in entries {
        write_agent_file(&agents_dir, &entry.name, &entry.raw)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Three-host aw-* agent fleet projection (issue #1842)
// ---------------------------------------------------------------------------
//
// `templates/cli/mainthread/agents/<name>.md` is the ONE canonical source for
// every aw-* subagent (R1): YAML-ish frontmatter (`name`/`description`/
// `model`/`model_tier`/`tools`) plus a host-neutral Markdown body. Claude
// reads that file verbatim ([`install_agents`], passthrough). Codex
// (`.codex/agents/<name>.toml`) and AGY (`.agents/agents/<name>.md`) are
// rendered from the same parsed frontmatter+body via [`render_codex_agent`]
// and [`render_agy_agent`] (R2), with per-host models resolved from the
// declared `model_tier` through [`AGENT_MODEL_TIERS`] (R3). All three
// installers share [`aw_agent_entries`] and [`prune_deprecated_fleet_files`],
// so no host can silently diverge from the templates source or keep a
// retired name (R4).

/// Default AGY per-agent turn/timeout budget (issue #1842 R3 scope note:
/// "AGY/Codex budget fields with defaults"). No canonical template currently
/// needs to override these; a future one could add `agy_max_turns`/
/// `agy_timeout_mins` frontmatter fields if that changes.
const AGENT_FLEET_AGY_DEFAULT_MAX_TURNS: u32 = 30;
const AGENT_FLEET_AGY_DEFAULT_TIMEOUT_MINS: u32 = 20;

/// One canonical agent's parsed frontmatter (issue #1842). Borrows from the
/// `'static` `templates/` source string, so it never allocates beyond the
/// `tools` split.
struct CanonicalAgentFrontmatter<'a> {
    name: &'a str,
    description: &'a str,
    /// The literal Claude-facing `model:` field — kept alongside
    /// `model_tier` (not derived from it) because Claude's projection is a
    /// verbatim passthrough of the raw template file. Cross-checked against
    /// `model_tier`'s resolved claude mapping by
    /// [`validate_agent_fleet_frontmatter`] so the two fields can never
    /// silently disagree.
    model: &'a str,
    model_tier: &'a str,
    tools: Vec<&'a str>,
}

/// Split one `templates/cli/mainthread/agents/<name>.md` file into its parsed
/// frontmatter and its host-neutral Markdown body (everything after the
/// closing `---`, including the leading blank line — AGY's body is this
/// slice verbatim, see [`render_agy_agent`]).
///
/// # Errors
///
/// Returns an error if the frontmatter delimiters are missing/malformed, or
/// if `name`/`description`/`model_tier` are absent — all template authoring
/// defects, not runtime input failures (R1/R3).
fn parse_agent_frontmatter(raw: &str) -> Result<(CanonicalAgentFrontmatter<'_>, &str)> {
    let after_open = raw
        .strip_prefix("---\n")
        .context("canonical agent template must open with `---` frontmatter")?;
    let close_at = after_open
        .find("\n---\n")
        .context("canonical agent template frontmatter must be closed with `---`")?;
    let frontmatter = &after_open[..close_at];
    let body = &after_open[close_at + "\n---\n".len()..];

    let mut name = None;
    let mut description = None;
    let mut model = None;
    let mut model_tier = None;
    let mut tools = Vec::new();
    for line in frontmatter.lines() {
        let (key, value) = line
            .split_once(':')
            .with_context(|| format!("malformed agent frontmatter line `{line}`"))?;
        match key.trim() {
            "name" => name = Some(value.trim()),
            "description" => description = Some(value.trim()),
            "model" => model = Some(value.trim()),
            "model_tier" => model_tier = Some(value.trim()),
            "tools" => {
                tools = value
                    .split(',')
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .collect();
            }
            // Any other/future field is passthrough-only for Claude and
            // unused by the Codex/AGY renderers.
            _ => {}
        }
    }

    Ok((
        CanonicalAgentFrontmatter {
            name: name.context("agent frontmatter missing required `name`")?,
            description: description.context("agent frontmatter missing required `description`")?,
            model: model.context("agent frontmatter missing required `model`")?,
            model_tier: model_tier.context(
                "agent frontmatter missing required `model_tier` (declare model_tier: top | standard | cheap)",
            )?,
            tools,
        },
        body,
    ))
}

/// One model tier's resolved per-host model id(s). `codex` carries
/// `(model, model_reasoning_effort)` since Codex needs both. `None` means
/// "this tier declares no mapping for this host" (issue #1842 AC5).
#[derive(Debug, Clone, Copy, Default)]
struct TierHostModels {
    claude: Option<&'static str>,
    codex: Option<(&'static str, &'static str)>,
    agy: Option<&'static str>,
}

/// The ONE per-tier -> per-host model mapping every aw-* agent's
/// `model_tier` frontmatter field resolves through (R3). Current fleet:
/// `aw-ec-reviewer` = top, `aw-dev`/`aw-td-writer`/`aw-ec-writer` = standard,
/// `aw-hw-filler` = cheap.
const AGENT_MODEL_TIERS: &[(&str, TierHostModels)] = &[
    (
        "top",
        TierHostModels {
            claude: Some("opus"),
            codex: Some(("gpt-5.6-sol", "high")),
            agy: Some("Gemini 3.1 Pro (High)"),
        },
    ),
    (
        "standard",
        TierHostModels {
            claude: Some("sonnet"),
            codex: Some(("gpt-5.6-terra", "high")),
            agy: Some("Gemini 3.6 Flash (High)"),
        },
    ),
    (
        "cheap",
        TierHostModels {
            claude: Some("haiku"),
            codex: Some(("gpt-5.6-luna", "medium")),
            agy: Some("Gemini 3.6 Flash (Medium)"),
        },
    ),
    (
        "planner",
        TierHostModels {
            claude: Some("sonnet"),
            codex: Some(("gpt-5.6-terra", "xhigh")),
            agy: Some("Gemini 3.6 Flash (High)"),
        },
    ),
    (
        "dev",
        TierHostModels {
            claude: Some("haiku"),
            codex: Some(("gpt-5.6-luna", "medium")),
            agy: Some("Gemini 3.6 Flash (Medium)"),
        },
    ),
    (
        "research",
        TierHostModels {
            claude: Some("opus"),
            codex: Some(("gpt-5.6-sol", "max")),
            agy: Some("Gemini 3.1 Pro (High)"),
        },
    ),
];

/// Resolve `tier`'s [`TierHostModels`] from `table`. Errors loudly on an
/// unknown tier (issue #1842 AC5) instead of silently defaulting.
fn tier_host_models(table: &[(&str, TierHostModels)], tier: &str) -> Result<TierHostModels> {
    table
        .iter()
        .find(|(t, _)| *t == tier)
        .map(|(_, models)| *models)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "unknown model_tier `{tier}`; declare a mapped profile in the canonical agent frontmatter"
            )
        })
}

/// Resolve one host's model mapping out of `tier_host_models(table, tier)`,
/// naming `agent_name`/`host` in the error so a tier missing a host mapping
/// fails loudly (issue #1842 AC5) instead of silently rendering nothing.
fn resolve_host_model<T>(
    table: &[(&str, TierHostModels)],
    agent_name: &str,
    tier: &str,
    host: &str,
    field: impl FnOnce(TierHostModels) -> Option<T>,
) -> Result<T> {
    let models = tier_host_models(table, tier)?;
    field(models).ok_or_else(|| {
        anyhow::anyhow!("agent `{agent_name}` model_tier `{tier}` has no `{host}` host mapping")
    })
}

/// Escape `value` for embedding in a TOML basic (single-line, double-quoted)
/// string.
fn toml_escape_basic_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Render one canonical agent body into Codex's `developer_instructions`
/// prose (issue #1842 R2): strip inline Markdown emphasis (`**bold**` and
/// `` `code` ``, both of which always occur as matched pairs in the
/// canonical templates) and turn `## Heading` lines into `Heading:` — the
/// exact shape Codex's `developer_instructions` convention already used by
/// `lumen-dev.toml`/`mamba-dev.toml` expects. Every other line (prose,
/// bullets, numbered steps) passes through unchanged but for the same
/// emphasis strip.
fn codex_developer_instructions(body: &str) -> String {
    let trimmed = body.trim_matches('\n');
    let mut out = String::new();
    for line in trimmed.split('\n') {
        let plain = line.trim_end().replace("**", "").replace('`', "");
        if let Some(heading) = plain.strip_prefix("## ") {
            out.push_str(heading.trim());
            out.push(':');
        } else {
            out.push_str(&plain);
        }
        out.push('\n');
    }
    out
}

/// Render one canonical agent as a Codex agent TOML file (issue #1842 R2):
/// `name`/`description`/`model`/`model_reasoning_effort`/`sandbox_mode` plus
/// deterministic `nickname_candidates`, and the body embedded as
/// `developer_instructions` via [`codex_developer_instructions`].
///
/// # Errors
///
/// Returns an error if the rendered body would contain a literal `"""`,
/// which would break out of the TOML triple-quoted string — a template
/// authoring defect, not a runtime input failure.
fn render_codex_agent(
    fm: &CanonicalAgentFrontmatter<'_>,
    body: &str,
    codex_model: &str,
    codex_effort: &str,
) -> Result<String> {
    let instructions = codex_developer_instructions(body);
    if instructions.contains("\"\"\"") {
        bail!(
            "agent `{}` body contains a TOML triple-quote sequence; cannot embed as developer_instructions",
            fm.name
        );
    }
    let name = toml_escape_basic_string(fm.name);
    let description = toml_escape_basic_string(fm.description);
    let nickname_underscored = toml_escape_basic_string(&fm.name.replace('-', "_"));
    let sandbox_mode = if fm
        .tools
        .iter()
        .any(|tool| *tool == "Write" || *tool == "Edit")
    {
        "workspace-write"
    } else {
        "read-only"
    };
    Ok(format!(
        "name = \"{name}\"\n\
         description = \"{description}\"\n\
         model = \"{codex_model}\"\n\
         model_reasoning_effort = \"{codex_effort}\"\n\
         sandbox_mode = \"{sandbox_mode}\"\n\
         nickname_candidates = [\"{name}\", \"{nickname_underscored}\"]\n\
         \n\
         developer_instructions = \"\"\"\n{instructions}\"\"\"\n"
    ))
}

/// Render one canonical agent as an AGY workspace subagent file (issue #1842
/// R2): `kind: local`/`model`/`max_turns`/`timeout_mins`/
/// `enable_write_tools`/`enable_mcp_tools` frontmatter, with the canonical
/// body preserved byte-for-byte (no Markdown transform — AGY renders
/// Markdown natively, same as Claude).
///
/// `enable_write_tools` is derived from the canonical `tools:` list (true iff
/// it declares `Write` or `Edit`) rather than a separate frontmatter field,
/// so the two can never drift.
fn render_agy_agent(fm: &CanonicalAgentFrontmatter<'_>, body: &str, agy_model: &str) -> String {
    let enable_write_tools = fm
        .tools
        .iter()
        .any(|tool| *tool == "Write" || *tool == "Edit");
    format!(
        "---\n\
         name: {name}\n\
         description: {description}\n\
         kind: local\n\
         model: {agy_model}\n\
         max_turns: {max_turns}\n\
         timeout_mins: {timeout_mins}\n\
         enable_write_tools: {enable_write_tools}\n\
         enable_mcp_tools: false\n\
         ---\n\
         {body}",
        name = fm.name,
        description = fm.description,
        max_turns = AGENT_FLEET_AGY_DEFAULT_MAX_TURNS,
        timeout_mins = AGENT_FLEET_AGY_DEFAULT_TIMEOUT_MINS,
    )
}

/// Install/refresh every `aw-*` subagent's Codex projection under
/// `codex_dir/agents/*.toml` (issue #1842).
fn install_codex_agents(codex_dir: &Path, entries: &[AgentFleetEntry]) -> Result<()> {
    let agents_dir = codex_dir.join("agents");
    std::fs::create_dir_all(&agents_dir)?;
    prune_deprecated_fleet_files(&agents_dir, "toml")?;
    for entry in entries {
        let name = entry.name.as_str();
        let raw = entry.raw.as_str();
        let (fm, body) =
            parse_agent_frontmatter(raw).with_context(|| format!("agent `{name}` template"))?;
        let (model, effort) =
            resolve_host_model(AGENT_MODEL_TIERS, name, fm.model_tier, "codex", |models| {
                models.codex
            })?;
        let rendered = render_codex_agent(&fm, body, model, effort)?;
        std::fs::write(agents_dir.join(format!("{name}.toml")), rendered)?;
        println!("   ✓ {}", name);
    }
    Ok(())
}

/// Install/refresh every `aw-*` subagent's AGY projection under
/// `agy_agents_dir/*.md` (issue #1842). `agy_agents_dir` is the
/// `.agents/agents/` directory itself (unlike `install_agents`/
/// `install_codex_agents`, which take the parent runtime dir and join
/// `agents/`) because `.agents/skills/` is this tree's only existing sibling
/// and already follows that same "pass the leaf dir" shape.
fn install_agy_agents(agy_agents_dir: &Path, entries: &[AgentFleetEntry]) -> Result<()> {
    std::fs::create_dir_all(agy_agents_dir)?;
    prune_deprecated_fleet_files(agy_agents_dir, "md")?;
    for entry in entries {
        let name = entry.name.as_str();
        let raw = entry.raw.as_str();
        let (fm, body) =
            parse_agent_frontmatter(raw).with_context(|| format!("agent `{name}` template"))?;
        let model = resolve_host_model(AGENT_MODEL_TIERS, name, fm.model_tier, "agy", |models| {
            models.agy
        })?;
        let rendered = render_agy_agent(&fm, body, model);
        std::fs::write(agy_agents_dir.join(format!("{name}.md")), rendered)?;
        println!("   ✓ {}", name);
    }
    Ok(())
}

/// Cross-check every canonical agent's literal `model:` field against its
/// `model_tier`'s resolved claude mapping in [`AGENT_MODEL_TIERS`] (issue
/// #1842 AC5 sibling check: the two fields describe the same thing two
/// different ways — passthrough source vs. tier-resolved — and must never
/// silently disagree). Also a cheap way to fail loudly on an unknown tier or
/// a tier missing its claude mapping before any host projection is written.
fn validate_agent_fleet_frontmatter(entries: &[AgentFleetEntry]) -> Result<()> {
    for entry in entries {
        let name = entry.name.as_str();
        let raw = entry.raw.as_str();
        let (fm, _body) =
            parse_agent_frontmatter(raw).with_context(|| format!("agent `{name}` template"))?;
        let expected_claude_model =
            resolve_host_model(AGENT_MODEL_TIERS, name, fm.model_tier, "claude", |models| {
                models.claude
            })?;
        if expected_claude_model != fm.model {
            bail!(
                "agent `{name}` frontmatter `model: {actual}` does not match its `model_tier: {tier}` claude mapping `{expected_claude_model}` — keep them in sync",
                actual = fm.model,
                tier = fm.model_tier,
            );
        }
    }
    Ok(())
}

/// Install/refresh the full aw-* agent fleet on all three hosts from
/// `templates/cli/mainthread/agents/` (issue #1842 R4): Claude
/// (`.claude/agents/`, passthrough), Codex (`.codex/agents/`, TOML), and AGY
/// (`.agents/agents/`, frontmatter-mapped Markdown).
fn install_agent_fleet(project_root: &Path, claude_dir: &Path) -> Result<()> {
    let entries = agent_fleet_entries(project_root)?;
    validate_agent_fleet_frontmatter(&entries)?;
    install_claude_agent_entries(claude_dir, &entries)?;
    install_codex_agents(&project_root.join(".codex"), &entries)?;
    install_agy_agents(&project_root.join(".agents").join("agents"), &entries)?;
    Ok(())
}

/// One drifted/missing/stale agent-fleet projection file (issue #1842 AC3).
#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentFleetFinding {
    host: &'static str,
    path: PathBuf,
    status: &'static str,
}

/// Compare every host's on-disk agent-fleet projection against what
/// [`install_agent_fleet`] would render, without writing anything (issue
/// #1842 AC3). Returns one [`AgentFleetFinding`] per missing render, per
/// byte-mismatched render, and per still-present deprecated fleet file on
/// any of the three hosts.
fn check_agent_fleet(project_root: &Path) -> Result<Vec<AgentFleetFinding>> {
    let entries = agent_fleet_entries(project_root)?;
    validate_agent_fleet_frontmatter(&entries)?;
    let claude_agents = project_root.join(".claude").join("agents");
    let codex_agents = project_root.join(".codex").join("agents");
    let agy_agents = project_root.join(".agents").join("agents");

    let mut findings = Vec::new();
    for entry in &entries {
        let name = entry.name.as_str();
        let raw = entry.raw.as_str();
        let (fm, body) =
            parse_agent_frontmatter(raw).with_context(|| format!("agent `{name}` template"))?;

        check_one_projection(
            &claude_agents.join(format!("{name}.md")),
            raw,
            "claude",
            &mut findings,
        )?;

        let (codex_model, codex_effort) =
            resolve_host_model(AGENT_MODEL_TIERS, name, fm.model_tier, "codex", |models| {
                models.codex
            })?;
        let codex_rendered = render_codex_agent(&fm, body, codex_model, codex_effort)?;
        check_one_projection(
            &codex_agents.join(format!("{name}.toml")),
            &codex_rendered,
            "codex",
            &mut findings,
        )?;

        let agy_model =
            resolve_host_model(AGENT_MODEL_TIERS, name, fm.model_tier, "agy", |models| {
                models.agy
            })?;
        let agy_rendered = render_agy_agent(&fm, body, agy_model);
        check_one_projection(
            &agy_agents.join(format!("{name}.md")),
            &agy_rendered,
            "agy",
            &mut findings,
        )?;
    }

    for deprecated in deprecated_agent_names() {
        for (dir, ext, host) in [
            (&claude_agents, "md", "claude"),
            (&codex_agents, "toml", "codex"),
            (&agy_agents, "md", "agy"),
        ] {
            let stale_path = dir.join(format!("{deprecated}.{ext}"));
            if stale_path.exists() {
                findings.push(AgentFleetFinding {
                    host,
                    path: stale_path,
                    status: "stale",
                });
            }
        }
    }

    Ok(findings)
}

/// Record whether `path` matches `expected`, pushing a `missing` or
/// `drifted` [`AgentFleetFinding`] onto `findings` when it does not.
fn check_one_projection(
    path: &Path,
    expected: &str,
    host: &'static str,
    findings: &mut Vec<AgentFleetFinding>,
) -> Result<()> {
    if !path.exists() {
        findings.push(AgentFleetFinding {
            host,
            path: path.to_path_buf(),
            status: "missing",
        });
        return Ok(());
    }
    let actual =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    if actual != expected {
        findings.push(AgentFleetFinding {
            host,
            path: path.to_path_buf(),
            status: "drifted",
        });
    }
    Ok(())
}

/// `aw new --check-agents` entry point (issue #1842 AC3): read-only,
/// names every drifted/missing/stale projection with the exact remediation
/// command, and exits non-zero when any finding exists.
fn run_agent_fleet_check(target: &Path) -> Result<()> {
    if !target.is_dir() {
        bail!(
            "--check-agents target does not exist: {} (run without --check-agents to install first)",
            target.display()
        );
    }
    let findings = check_agent_fleet(target)?;
    for finding in &findings {
        println!(
            "   {} [{}] {} ({})",
            "✗".red(),
            finding.host,
            finding.path.display(),
            finding.status
        );
    }
    if findings.is_empty() {
        println!(
            "{}",
            "✅ aw-* agent fleet projection is clean on all three hosts (claude/codex/agy)".green()
        );
        return Ok(());
    }
    bail!(
        "aw-* agent fleet projection drift detected in {} file(s); run `aw new {} --path {} --sync-agents` to remediate",
        findings.len(),
        target
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "<project>".to_string()),
        target.display(),
    );
}

// Delete legacy hook scripts from `.claude/hooks/`.
//
// Legacy flat-layout and subagent hook scripts from earlier score versions are
// removed so deployments do not register autonomous Claude hook callbacks.
// @spec apps/agentic-workflow/tech-design/surface/specs/init-command.md#R8
fn install_hooks(claude_dir: &Path) -> Result<()> {
    let hooks_dir = claude_dir.join("hooks");
    let global_dir = hooks_dir.join("global");
    let shared_dir = hooks_dir.join("agents").join("_shared");
    std::fs::create_dir_all(&global_dir)?;
    std::fs::create_dir_all(&shared_dir)?;

    // Clean up any legacy flat-layout hook scripts from prior installs.
    let legacy_names: &[&str] = &[
        "score-safe-bash.sh",
        "score-readonly-bash.sh",
        "score-next-step.sh",
        "score-subagent-start.sh",
        "score-artifact-guard.sh",
        "score-validate-advance.sh",
        "sdd-safe-bash.sh",
        "sdd-readonly-bash.sh",
    ];
    for name in legacy_names {
        let legacy_path = hooks_dir.join(name);
        if legacy_path.exists() {
            let _ = std::fs::remove_file(&legacy_path);
            println!("   ✓ removed legacy hook {}", name);
        }
    }
    let retired_nested_hooks: &[&str] = &[
        "global/subagentstart-setup.sh",
        "global/subagentstop-validate.sh",
        "global/pretooluse-artifact-guard.sh",
        "agents/_shared/pretooluse-safe-bash.sh",
        "agents/_shared/pretooluse-readonly-bash.sh",
        "agents/issue-author/pretooluse-write-guard.sh",
        "agents/issue-author/subagentstop-apply.sh",
        "agents/issue-author/subagentstart-brief.sh",
        "agents/issue-reviewer/pretooluse-write-guard.sh",
        "agents/issue-reviewer/subagentstop-apply.sh",
        "agents/issue-reviewer/subagentstart-brief.sh",
        "agents/issue-reviser/pretooluse-write-guard.sh",
        "hook1-post-apply-validate.sh",
        "hook2-pre-apply-guard.sh",
        "hook5-session-start-idle.sh",
    ];
    for rel in retired_nested_hooks {
        let retired_path = hooks_dir.join(rel);
        if retired_path.exists() {
            let _ = std::fs::remove_file(&retired_path);
            println!("   ✓ removed retired hook {}", rel);
        }
    }

    Ok(())
}

// Install or merge `.claude/settings.json` with the current mainthread template.
///
// Strategy:
// - If no settings.json exists: write the template directly.
// - If settings.json exists: merge permissions and remove retired Score hook entries.
// @spec apps/agentic-workflow/tech-design/surface/specs/init-command.md#R10
// @spec apps/agentic-workflow/tech-design/surface/specs/init-command.md#R11
fn install_settings_json(claude_dir: &Path) -> Result<()> {
    let settings_path = claude_dir.join("settings.json");

    if !settings_path.exists() {
        std::fs::write(&settings_path, SETTINGS_JSON_TEMPLATE)?;
        println!("   ✓ .claude/settings.json (created)");
        return Ok(());
    }

    let existing_content = std::fs::read_to_string(&settings_path)?;
    let mut existing: serde_json::Value =
        serde_json::from_str(&existing_content).unwrap_or_else(|_| serde_json::json!({}));

    let template: serde_json::Value = serde_json::from_str(SETTINGS_JSON_TEMPLATE)?;

    let existing_obj = existing
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("settings.json root is not an object"))?;

    // Merge `permissions.deny` (R13: protect `.aw/tech-design/**` from
    // direct Edit/Write/MultiEdit; spec writes go through `aw td`).
    if let Some(tmpl_perms) = template.get("permissions").and_then(|p| p.as_object()) {
        if let Some(tmpl_deny) = tmpl_perms.get("deny").and_then(|d| d.as_array()) {
            let perms = existing_obj
                .entry("permissions")
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
                .ok_or_else(|| {
                    anyhow::anyhow!("permissions is not an object in existing settings.json")
                })?;
            let deny_arr = perms
                .entry("deny")
                .or_insert_with(|| serde_json::json!([]))
                .as_array_mut()
                .ok_or_else(|| {
                    anyhow::anyhow!("permissions.deny is not an array in existing settings.json")
                })?;
            for rule in tmpl_deny {
                if !deny_arr.iter().any(|existing_rule| existing_rule == rule) {
                    deny_arr.push(rule.clone());
                }
            }
        }
    }

    if let Some(hooks_value) = existing_obj.get_mut("hooks") {
        let hooks = hooks_value
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("hooks is not an object in existing settings.json"))?;
        prune_retired_score_hooks(hooks);
    }

    if let Some(tmpl_hooks) = template.get("hooks").and_then(|h| h.as_object()) {
        let hooks = existing_obj
            .entry("hooks")
            .or_insert_with(|| serde_json::json!({}))
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("hooks is not an object in existing settings.json"))?;

        for (event, entries) in tmpl_hooks {
            let Some(new_entries) = entries.as_array() else {
                continue;
            };
            let event_arr = hooks
                .entry(event.clone())
                .or_insert_with(|| serde_json::json!([]))
                .as_array_mut()
                .ok_or_else(|| anyhow::anyhow!("hooks.{} is not an array", event))?;

            for new_entry in new_entries {
                let new_matcher = new_entry
                    .get("matcher")
                    .and_then(|m| m.as_str())
                    .unwrap_or("");

                if let Some(pos) = event_arr.iter().position(|e| {
                    e.get("matcher")
                        .and_then(|m| m.as_str())
                        .map(|m| m == new_matcher)
                        .unwrap_or(false)
                }) {
                    // Replace so existing deployments get the current hook paths
                    // (e.g. `global/subagentstop-validate.sh`) in place of older flat
                    // `score-*.sh` layouts.
                    event_arr[pos] = new_entry.clone();
                } else {
                    event_arr.push(new_entry.clone());
                }
            }
        }
    }

    let updated = serde_json::to_string_pretty(&existing)?;
    std::fs::write(&settings_path, format!("{updated}\n"))?;
    println!("   ✓ .claude/settings.json (merged permissions.deny + retired legacy hooks removed)");

    Ok(())
}

fn prune_retired_score_hooks(hooks: &mut serde_json::Map<String, serde_json::Value>) {
    let events: Vec<String> = hooks.keys().cloned().collect();
    for event in events {
        let Some(entries) = hooks.get_mut(&event).and_then(|value| value.as_array_mut()) else {
            continue;
        };
        entries.retain(|entry| !is_retired_score_hook_entry(entry));
    }
    hooks.retain(|_, value| match value.as_array() {
        Some(entries) => !entries.is_empty(),
        None => true,
    });
}

fn is_retired_score_hook_entry(entry: &serde_json::Value) -> bool {
    let matcher = entry
        .get("matcher")
        .and_then(|matcher| matcher.as_str())
        .unwrap_or_default();
    if matcher == "score-*" || matcher.starts_with("score-") {
        return true;
    }
    entry
        .get("hooks")
        .and_then(|hooks| hooks.as_array())
        .into_iter()
        .flatten()
        .filter_map(|hook| hook.get("command").and_then(|command| command.as_str()))
        .any(|command| {
            command.contains(".claude/hooks/score-")
                || command.contains(".claude/hooks/global/")
                || command.contains(".claude/hooks/agents/")
        })
}

// No longer used - shell scripts are no longer generated during init.
// Orchestrators now call CLI tools directly instead of using shell scripts.
// The cclab/scripts/ directory is cleaned up during init.

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // REQ: R5 — install_agents retires Score subagent files.
    #[test]
    fn test_install_agents_retires_score_agents() {
        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        let agents_dir = claude_dir.join("agents");
        fs::create_dir_all(&agents_dir).unwrap();

        let retired = [
            "score-review.md",
            "score-issue-author.md",
            "score-issue-reviewer.md",
            "score-issue-reviser.md",
            "score-td-author.md",
            "score-td-reviewer.md",
            "score-td-reviser.md",
            "score-cb-handwriter.md",
        ];
        for name in &retired {
            fs::write(agents_dir.join(name), "retired").unwrap();
        }

        install_agents(&claude_dir).unwrap();
        assert!(agents_dir.exists(), ".claude/agents/ should exist");

        for name in &retired {
            assert!(
                !agents_dir.join(name).exists(),
                "Retired agent {} should be removed",
                name
            );
        }
    }

    // REQ: R6 — install_agents removes legacy sdd-*.md files
    #[test]
    fn test_install_agents_removes_legacy() {
        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        let agents_dir = claude_dir.join("agents");
        fs::create_dir_all(&agents_dir).unwrap();

        // Place a legacy file
        let legacy = agents_dir.join("sdd-change-implementation.md");
        fs::write(&legacy, "legacy content").unwrap();
        assert!(legacy.exists());

        install_agents(&claude_dir).unwrap();
        assert!(!legacy.exists(), "Legacy sdd-* agent should be removed");
    }

    // Issue #1842 R6/AC2 — install_agent_fleet projects all five aw-* agents
    // to all three hosts with per-tier models resolved, and a second run is
    // byte-idempotent (AC1).
    #[test]
    fn test_install_agent_fleet_projects_all_hosts_and_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let project_root = tmp.path();
        let claude_dir = project_root.join(".claude");

        install_agent_fleet(project_root, &claude_dir).unwrap();

        let expected_models = [
            (
                "aw-dev",
                "sonnet",
                "gpt-5.6-terra",
                "Gemini 3.6 Flash (High)",
            ),
            (
                "aw-td-writer",
                "sonnet",
                "gpt-5.6-terra",
                "Gemini 3.6 Flash (High)",
            ),
            (
                "aw-ec-writer",
                "sonnet",
                "gpt-5.6-terra",
                "Gemini 3.6 Flash (High)",
            ),
            (
                "aw-ec-reviewer",
                "opus",
                "gpt-5.6-sol",
                "Gemini 3.1 Pro (High)",
            ),
            (
                "aw-hw-filler",
                "haiku",
                "gpt-5.6-luna",
                "Gemini 3.6 Flash (Medium)",
            ),
        ];
        for (name, claude_model, codex_model, agy_model) in expected_models {
            let claude_path = claude_dir.join("agents").join(format!("{name}.md"));
            assert!(claude_path.exists(), "missing claude projection {name}");
            let claude_body = fs::read_to_string(&claude_path).unwrap();
            assert!(
                claude_body.contains(&format!("model: {claude_model}")),
                "{name} claude projection should keep model: {claude_model}"
            );

            let codex_path = project_root
                .join(".codex")
                .join("agents")
                .join(format!("{name}.toml"));
            assert!(codex_path.exists(), "missing codex projection {name}");
            let codex_body = fs::read_to_string(&codex_path).unwrap();
            assert!(
                codex_body.contains(&format!("model = \"{codex_model}\"")),
                "{name} codex projection should resolve model {codex_model}, got:\n{codex_body}"
            );
            assert!(
                codex_body.contains("developer_instructions = \"\"\""),
                "{name} codex projection should embed developer_instructions"
            );

            let agy_path = project_root
                .join(".agents")
                .join("agents")
                .join(format!("{name}.md"));
            assert!(agy_path.exists(), "missing agy projection {name}");
            let agy_body = fs::read_to_string(&agy_path).unwrap();
            assert!(
                agy_body.contains(&format!("model: {agy_model}")),
                "{name} agy projection should resolve model {agy_model}, got:\n{agy_body}"
            );
            assert!(
                agy_body.contains("kind: local"),
                "{name} agy projection should declare kind: local"
            );
        }

        // AC1: snapshot every projected file, run again, and assert nothing
        // changed byte-for-byte (idempotency).
        let mut before = std::collections::BTreeMap::new();
        for dir in [
            claude_dir.join("agents"),
            project_root.join(".codex").join("agents"),
            project_root.join(".agents").join("agents"),
        ] {
            for entry in fs::read_dir(&dir).unwrap() {
                let entry = entry.unwrap();
                before.insert(entry.path(), fs::read(entry.path()).unwrap());
            }
        }

        install_agent_fleet(project_root, &claude_dir).unwrap();

        for (path, contents) in &before {
            let after = fs::read(path).unwrap();
            assert_eq!(
                &after,
                contents,
                "{} should be byte-identical on re-run",
                path.display()
            );
        }
    }

    fn write_project_role_test_registry(root: &Path) {
        fs::write(
            root.join("aw.toml"),
            r#"
[[projects]]
name = "cap"
path = "apps/cap"
[[projects.workspaces]]
paths = ["apps/cap/**"]
target = "rust"

[[projects]]
name = "mamba"
path = "projects/mamba"
[[projects.workspaces]]
paths = ["projects/mamba/**"]
target = "rust"

[[projects]]
name = "meter"
path = "apps/meter"
[[projects.workspaces]]
paths = ["apps/meter/**"]
target = "rust"

[[projects]]
name = "meter"
path = "projects/meter"
[[projects.workspaces]]
paths = ["projects/meter/**"]
target = "rust"

[[projects]]
name = "sift"
path = "projects/sift"
[[projects.workspaces]]
paths = ["projects/sift/**"]
target = "rust"

[[projects]]
name = "pg"
path = "projects/mamba/mambalibs/pgkit"
[[projects.workspaces]]
paths = ["projects/mamba/mambalibs/pgkit/**"]
target = "rust"
"#,
        )
        .unwrap();
    }

    // Issue #2400 — direct apps and the top-level Mamba project each get
    // planner/dev/research roles from the registry; excluded project classes
    // cannot leak a role into any host projection.
    #[test]
    fn test_project_role_fleet_uses_registry_scope_and_model_matrix() {
        let tmp = TempDir::new().unwrap();
        let project_root = tmp.path();
        write_project_role_test_registry(project_root);

        let names: Vec<String> = project_role_entries(project_root)
            .unwrap()
            .into_iter()
            .map(|entry| entry.name)
            .collect();
        assert_eq!(
            names,
            vec![
                "cap-dev",
                "cap-planner",
                "cap-research",
                "mamba-dev",
                "mamba-planner",
                "mamba-research",
                "meter-dev",
                "meter-planner",
                "meter-research",
            ]
        );

        let claude_dir = project_root.join(".claude");
        install_agent_fleet(project_root, &claude_dir).unwrap();

        let planner = fs::read_to_string(claude_dir.join("agents/cap-planner.md")).unwrap();
        assert!(planner.contains("model: sonnet"));
        assert!(planner.contains("effort: xhigh"));
        assert!(planner.contains("never implement product source"));

        let dev = fs::read_to_string(claude_dir.join("agents/cap-dev.md")).unwrap();
        assert!(dev.contains("model: haiku"));
        assert!(dev.contains("effort: medium"));

        let research = fs::read_to_string(claude_dir.join("agents/cap-research.md")).unwrap();
        assert!(research.contains("model: opus"));
        assert!(research.contains("effort: max"));
        assert!(!research.contains("tools: Read, Edit"));

        let planner_codex =
            fs::read_to_string(project_root.join(".codex/agents/cap-planner.toml")).unwrap();
        assert!(planner_codex.contains("model = \"gpt-5.6-terra\""));
        assert!(planner_codex.contains("model_reasoning_effort = \"xhigh\""));
        assert!(planner_codex.contains("sandbox_mode = \"workspace-write\""));

        let dev_codex =
            fs::read_to_string(project_root.join(".codex/agents/cap-dev.toml")).unwrap();
        assert!(dev_codex.contains("model = \"gpt-5.6-luna\""));
        assert!(dev_codex.contains("model_reasoning_effort = \"medium\""));

        let research_codex =
            fs::read_to_string(project_root.join(".codex/agents/cap-research.toml")).unwrap();
        assert!(research_codex.contains("model = \"gpt-5.6-sol\""));
        assert!(research_codex.contains("model_reasoning_effort = \"max\""));
        assert!(research_codex.contains("sandbox_mode = \"read-only\""));

        let planner_agy =
            fs::read_to_string(project_root.join(".agents/agents/cap-planner.md")).unwrap();
        assert!(planner_agy.contains("model: Gemini 3.6 Flash (High)"));
        assert!(planner_agy.contains("enable_write_tools: true"));

        let dev_agy = fs::read_to_string(project_root.join(".agents/agents/cap-dev.md")).unwrap();
        assert!(dev_agy.contains("model: Gemini 3.6 Flash (Medium)"));

        let research_agy =
            fs::read_to_string(project_root.join(".agents/agents/cap-research.md")).unwrap();
        assert!(research_agy.contains("model: Gemini 3.1 Pro (High)"));
        assert!(research_agy.contains("enable_write_tools: false"));

        assert!(check_agent_fleet(project_root).unwrap().is_empty());
    }

    // Issue #1842 AC1 — deleting the aw-ec-reviewer Codex/AGY projections and
    // re-running the producer regenerates them byte-identically.
    #[test]
    fn test_install_agent_fleet_regenerates_deleted_reviewer_projections() {
        let tmp = TempDir::new().unwrap();
        let project_root = tmp.path();
        let claude_dir = project_root.join(".claude");
        install_agent_fleet(project_root, &claude_dir).unwrap();

        let codex_reviewer = project_root
            .join(".codex")
            .join("agents")
            .join("aw-ec-reviewer.toml");
        let agy_reviewer = project_root
            .join(".agents")
            .join("agents")
            .join("aw-ec-reviewer.md");
        let before_codex = fs::read(&codex_reviewer).unwrap();
        let before_agy = fs::read(&agy_reviewer).unwrap();

        fs::remove_file(&codex_reviewer).unwrap();
        fs::remove_file(&agy_reviewer).unwrap();
        assert!(!codex_reviewer.exists());
        assert!(!agy_reviewer.exists());

        install_agent_fleet(project_root, &claude_dir).unwrap();

        assert_eq!(fs::read(&codex_reviewer).unwrap(), before_codex);
        assert_eq!(fs::read(&agy_reviewer).unwrap(), before_agy);
    }

    // Issue #1842 R5 — non-fleet files in every host agents dir are never
    // touched by install_agent_fleet (hand-authored dev agents / user files).
    #[test]
    fn test_install_agent_fleet_preserves_non_fleet_files() {
        let tmp = TempDir::new().unwrap();
        let project_root = tmp.path();
        let claude_dir = project_root.join(".claude");

        let claude_agents = claude_dir.join("agents");
        let codex_agents = project_root.join(".codex").join("agents");
        let agy_agents = project_root.join(".agents").join("agents");
        fs::create_dir_all(&claude_agents).unwrap();
        fs::create_dir_all(&codex_agents).unwrap();
        fs::create_dir_all(&agy_agents).unwrap();

        fs::write(claude_agents.join("jet-dev.md"), "jet-dev hand-authored").unwrap();
        fs::write(
            claude_agents.join("lumen-dev.md"),
            "lumen-dev hand-authored",
        )
        .unwrap();
        fs::write(
            claude_agents.join("mamba-dev.md"),
            "mamba-dev hand-authored",
        )
        .unwrap();
        fs::write(codex_agents.join("lumen-dev.toml"), "lumen-dev toml").unwrap();
        fs::write(codex_agents.join("mamba-dev.toml"), "mamba-dev toml").unwrap();
        fs::write(agy_agents.join("some-user-agent.md"), "user agent").unwrap();

        install_agent_fleet(project_root, &claude_dir).unwrap();

        assert_eq!(
            fs::read_to_string(claude_agents.join("jet-dev.md")).unwrap(),
            "jet-dev hand-authored"
        );
        assert_eq!(
            fs::read_to_string(claude_agents.join("lumen-dev.md")).unwrap(),
            "lumen-dev hand-authored"
        );
        assert_eq!(
            fs::read_to_string(claude_agents.join("mamba-dev.md")).unwrap(),
            "mamba-dev hand-authored"
        );
        assert_eq!(
            fs::read_to_string(codex_agents.join("lumen-dev.toml")).unwrap(),
            "lumen-dev toml"
        );
        assert_eq!(
            fs::read_to_string(codex_agents.join("mamba-dev.toml")).unwrap(),
            "mamba-dev toml"
        );
        assert_eq!(
            fs::read_to_string(agy_agents.join("some-user-agent.md")).unwrap(),
            "user agent"
        );
    }

    // Issue #1842 AC4 — retiring a fleet name prunes its projection from all
    // three host dirs on the next producer run.
    #[test]
    fn test_install_agent_fleet_prunes_retired_name_on_all_hosts() {
        let tmp = TempDir::new().unwrap();
        let project_root = tmp.path();
        let claude_dir = project_root.join(".claude");
        let claude_agents = claude_dir.join("agents");
        let codex_agents = project_root.join(".codex").join("agents");
        let agy_agents = project_root.join(".agents").join("agents");
        fs::create_dir_all(&claude_agents).unwrap();
        fs::create_dir_all(&codex_agents).unwrap();
        fs::create_dir_all(&agy_agents).unwrap();

        // Simulate a retired fleet member already present on all three hosts
        // (`deprecated_agent_names()` already covers the retired score-*
        // agents this repo actually shipped).
        let retired = deprecated_agent_names()[0];
        fs::write(claude_agents.join(format!("{retired}.md")), "retired").unwrap();
        fs::write(codex_agents.join(format!("{retired}.toml")), "retired").unwrap();
        fs::write(agy_agents.join(format!("{retired}.md")), "retired").unwrap();

        install_agent_fleet(project_root, &claude_dir).unwrap();

        assert!(!claude_agents.join(format!("{retired}.md")).exists());
        assert!(!codex_agents.join(format!("{retired}.toml")).exists());
        assert!(!agy_agents.join(format!("{retired}.md")).exists());
    }

    // Issue #1842 AC3 — check_agent_fleet reports clean on a freshly
    // installed tree and reports drifted/missing/stale findings otherwise.
    #[test]
    fn test_check_agent_fleet_clean_then_reports_drift_and_stale() {
        let tmp = TempDir::new().unwrap();
        let project_root = tmp.path();
        let claude_dir = project_root.join(".claude");
        install_agent_fleet(project_root, &claude_dir).unwrap();

        assert!(
            check_agent_fleet(project_root).unwrap().is_empty(),
            "freshly installed fleet should report no drift"
        );

        // Missing: delete one host's projection.
        let missing_path = project_root
            .join(".agents")
            .join("agents")
            .join("aw-dev.md");
        fs::remove_file(&missing_path).unwrap();

        // Drifted: tamper with another host's projection.
        let drifted_path = project_root
            .join(".codex")
            .join("agents")
            .join("aw-td-writer.toml");
        fs::write(&drifted_path, "tampered").unwrap();

        // Stale: drop a retired name's file onto the claude host.
        let retired = deprecated_agent_names()[0];
        let stale_path = claude_dir.join("agents").join(format!("{retired}.md"));
        fs::write(&stale_path, "retired").unwrap();

        let findings = check_agent_fleet(project_root).unwrap();
        assert!(findings
            .iter()
            .any(|f| f.path == missing_path && f.status == "missing"));
        assert!(findings
            .iter()
            .any(|f| f.path == drifted_path && f.status == "drifted"));
        assert!(findings
            .iter()
            .any(|f| f.path == stale_path && f.status == "stale"));

        // Repairing via the producer clears every finding.
        install_agent_fleet(project_root, &claude_dir).unwrap();
        assert!(check_agent_fleet(project_root).unwrap().is_empty());
    }

    // Issue #1842 AC5 — an unknown model_tier fails loudly instead of
    // silently defaulting.
    #[test]
    fn test_tier_host_models_rejects_unknown_tier() {
        let err = tier_host_models(AGENT_MODEL_TIERS, "legendary").unwrap_err();
        assert!(
            err.to_string().contains("unknown model_tier"),
            "unexpected error: {err}"
        );
    }

    // Issue #1842 AC5 — a tier that declares no mapping for a given host
    // fails loudly instead of silently rendering an empty/default model.
    #[test]
    fn test_resolve_host_model_rejects_missing_host_mapping() {
        let synthetic: &[(&str, TierHostModels)] = &[(
            "codex-blind",
            TierHostModels {
                claude: Some("sonnet"),
                codex: None,
                agy: Some("Gemini 3.5 Pro (High)"),
            },
        )];
        let err = resolve_host_model(synthetic, "test-agent", "codex-blind", "codex", |m| m.codex)
            .unwrap_err();
        assert!(
            err.to_string().contains("no `codex` host mapping"),
            "unexpected error: {err}"
        );
    }

    // Issue #1842 R2 — the Codex render strips inline Markdown emphasis and
    // turns `## Heading` into `Heading:`.
    #[test]
    fn test_codex_developer_instructions_strips_markdown() {
        let body = "\n## Scope\nDo **not** touch `capability.rs`.\n";
        let out = codex_developer_instructions(body);
        assert_eq!(out, "Scope:\nDo not touch capability.rs.\n");
    }

    // Issue #1842 R2 — the AGY render preserves the canonical body verbatim
    // (no Markdown transform) and derives enable_write_tools from tools:.
    #[test]
    fn test_render_agy_agent_preserves_body_and_derives_write_tools() {
        let fm = CanonicalAgentFrontmatter {
            name: "aw-example",
            description: "Example agent",
            model: "sonnet",
            model_tier: "standard",
            tools: vec!["Read", "Write", "Bash"],
        };
        let body = "\n## Scope\nDo the thing.\n";
        let rendered = render_agy_agent(&fm, body, "Gemini 3.5 Pro (High)");
        assert!(
            rendered.ends_with(body),
            "body should be preserved verbatim"
        );
        assert!(rendered.contains("enable_write_tools: true"));
        assert!(rendered.contains("kind: local"));
        assert!(rendered.contains("model: Gemini 3.5 Pro (High)"));
    }

    // REQ: R8 — install_hooks retires stale hook scripts
    #[test]
    fn test_install_hooks_removes_mainthread_hooks() {
        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        let hooks_dir = claude_dir.join("hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        for rel in [
            "hook1-post-apply-validate.sh",
            "hook2-pre-apply-guard.sh",
            "hook5-session-start-idle.sh",
        ] {
            fs::write(hooks_dir.join(rel), "# stale").unwrap();
        }

        install_hooks(&claude_dir).unwrap();

        assert!(hooks_dir.exists(), ".claude/hooks/ should exist");
        for rel in [
            "hook1-post-apply-validate.sh",
            "hook2-pre-apply-guard.sh",
            "hook5-session-start-idle.sh",
        ] {
            assert!(
                !hooks_dir.join(rel).exists(),
                "Hook {} should be retired",
                rel
            );
        }
    }

    // REQ: R8 — install_hooks removes legacy flat-layout hook files
    #[test]
    fn test_install_hooks_removes_legacy_flat_layout() {
        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        let hooks_dir = claude_dir.join("hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        // Seed legacy flat-layout files from earlier score versions.
        let legacy = [
            "score-next-step.sh",
            "score-safe-bash.sh",
            "score-readonly-bash.sh",
            "score-subagent-start.sh",
            "score-artifact-guard.sh",
            "score-validate-advance.sh",
            "sdd-safe-bash.sh",
            "sdd-readonly-bash.sh",
        ];
        for name in &legacy {
            fs::write(hooks_dir.join(name), "# legacy").unwrap();
        }

        install_hooks(&claude_dir).unwrap();

        for name in &legacy {
            assert!(
                !hooks_dir.join(name).exists(),
                "Legacy hook {} should be removed after re-install",
                name
            );
        }
    }

    // REQ: R9 — settings.json template does not register Claude hooks.
    #[test]
    fn test_settings_json_template_has_no_hooks() {
        let template: serde_json::Value = serde_json::from_str(SETTINGS_JSON_TEMPLATE).unwrap();
        assert!(
            template.get("hooks").is_none(),
            "settings template should not install Claude hooks: {template:?}"
        );
    }

    // REQ: R10 — install_settings_json creates fresh settings.json if not present
    #[test]
    fn test_install_settings_json_fresh() {
        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();

        install_settings_json(&claude_dir).unwrap();

        let settings_path = claude_dir.join("settings.json");
        assert!(settings_path.exists(), "settings.json should be created");

        let content: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
        // R13: deny rules for `tech-design/**` are installed.
        let deny = content["permissions"]["deny"]
            .as_array()
            .expect("permissions.deny should be present after fresh install");
        let rules: Vec<&str> = deny.iter().filter_map(|e| e.as_str()).collect();
        assert!(
            rules.contains(&"Edit(tech-design/**)"),
            "deny list missing Edit rule, got {:?}",
            rules
        );
        assert!(
            rules.contains(&"Write(tech-design/**)"),
            "deny list missing Write rule, got {:?}",
            rules
        );
        assert!(
            rules.contains(&"MultiEdit(tech-design/**)"),
            "deny list missing MultiEdit rule, got {:?}",
            rules
        );
    }

    // R13: re-running the project asset installer against an existing settings.json that
    // already has `permissions.deny` rules MUST merge — preserve user
    // additions, add the spec-protection rules without duplicating.
    #[test]
    fn test_install_settings_json_merges_deny_rules_idempotent() {
        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();

        let existing = serde_json::json!({
            "permissions": {
                "deny": ["Bash(rm -rf /*)"]
            }
        });
        fs::write(
            claude_dir.join("settings.json"),
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .unwrap();

        install_settings_json(&claude_dir).unwrap();
        // Second run must be idempotent (no duplicates).
        install_settings_json(&claude_dir).unwrap();

        let raw = fs::read_to_string(claude_dir.join("settings.json")).unwrap();
        assert!(
            raw.ends_with('\n'),
            "merged settings.json should end with a newline"
        );
        let content: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let deny = content["permissions"]["deny"].as_array().unwrap();
        let rules: Vec<&str> = deny.iter().filter_map(|e| e.as_str()).collect();
        // Pre-existing rule preserved.
        assert!(
            rules.contains(&"Bash(rm -rf /*)"),
            "user rule lost: {:?}",
            rules
        );
        // Template rules merged.
        assert!(rules.contains(&"Edit(tech-design/**)"), "{:?}", rules);
        assert!(rules.contains(&"Write(tech-design/**)"), "{:?}", rules);
        assert!(rules.contains(&"MultiEdit(tech-design/**)"), "{:?}", rules);
        // No duplicates after second run.
        let edit_count = rules
            .iter()
            .filter(|r| **r == "Edit(tech-design/**)")
            .count();
        assert_eq!(
            edit_count, 1,
            "Edit rule duplicated after second install: {:?}",
            rules
        );
    }

    // REQ: R11 — install_settings_json removes existing retired score-* hook
    // matchers when re-running the project asset installer.
    #[test]
    fn test_install_settings_json_removes_existing_score_hook_matcher() {
        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();

        let existing = serde_json::json!({
            "hooks": {
                "SubagentStop": [
                    {"matcher": "score-*", "hooks": [{"type": "command", "command": ".claude/hooks/score-next-step.sh"}]}
                ]
            }
        });
        fs::write(
            claude_dir.join("settings.json"),
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .unwrap();

        install_settings_json(&claude_dir).unwrap();

        let content: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(claude_dir.join("settings.json")).unwrap())
                .unwrap();
        let hooks = content["hooks"]
            .as_object()
            .expect("hooks should be object");
        assert!(
            !hooks.contains_key("SubagentStop"),
            "retired public hook command should be removed: {hooks:?}"
        );
    }

    // REQ: aw-greenfield-project-bootstrap UT1, UT2 — target resolution.
    #[test]
    fn test_new_resolves_default_and_explicit_targets() {
        let tmp = TempDir::new().unwrap();

        let default_target = resolve_new_target(tmp.path(), "ai-studio", None).unwrap();
        assert_eq!(default_target, tmp.path().join("ai-studio"));

        let relative_target =
            resolve_new_target(tmp.path(), "ignored", Some(Path::new("custom/path"))).unwrap();
        assert_eq!(relative_target, tmp.path().join("custom/path"));

        let absolute = tmp.path().join("explicit");
        let absolute_target = resolve_new_target(tmp.path(), "ignored", Some(&absolute)).unwrap();
        assert_eq!(absolute_target, absolute);
    }

    // REQ: aw-greenfield-project-bootstrap UT3, UT4 — safe target preparation.
    #[test]
    fn test_new_prepares_targets_and_rejects_unsafe_paths() {
        let tmp = TempDir::new().unwrap();

        let missing = tmp.path().join("missing");
        prepare_new_target(&missing, false).unwrap();
        assert!(missing.is_dir());

        let empty = tmp.path().join("empty");
        fs::create_dir_all(&empty).unwrap();
        prepare_new_target(&empty, false).unwrap();

        let non_empty = tmp.path().join("non-empty");
        fs::create_dir_all(&non_empty).unwrap();
        fs::write(non_empty.join("README.md"), "# existing").unwrap();
        assert!(prepare_new_target(&non_empty, false).is_err());
        prepare_new_target(&non_empty, true).unwrap();

        let file_target = tmp.path().join("file");
        fs::write(&file_target, "not a dir").unwrap();
        assert!(prepare_new_target(&file_target, true).is_err());
    }

    // REQ: aw-greenfield-project-bootstrap UT3 — --no-assets creates only the target directory.
    #[test]
    fn test_new_no_assets_creates_target_directory_only() {
        let tmp = TempDir::new().unwrap();
        let args = NewArgs {
            name: "ai-studio".to_string(),
            path: None,
            force: false,
            no_assets: true,
            check_agents: false,
            sync_agents: false,
        };

        let outcome = run_new_with_current_dir(args, tmp.path()).unwrap();

        assert_eq!(outcome.target, tmp.path().join("ai-studio"));
        assert!(!outcome.assets_installed);
        assert!(outcome.target.is_dir());
        assert!(!outcome.target.join(".aw").exists());
    }

    // REQ: aw-greenfield-project-bootstrap UT5 — aw new delegates to the shared asset installer.
    #[test]
    fn test_new_runs_shared_asset_installer() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("ai-studio");
        let args = NewArgs {
            name: "ai-studio".to_string(),
            path: Some(target.clone()),
            force: false,
            no_assets: false,
            check_agents: false,
            sync_agents: false,
        };

        let outcome = run_new_with_current_dir(args, tmp.path()).unwrap();

        assert_eq!(outcome.target, target);
        assert!(outcome.assets_installed);
        assert!(target.join("aw.toml").exists());
        assert!(target.join("tech-design").is_dir());
        assert!(target.join("AGENTS.md").exists());
        assert!(target.join("CLAUDE.md").exists());
        assert!(target.join("README.md").exists());
        assert!(target.join("CONTRIBUTING.md").exists());
        assert!(target.join("CAPABILITIES.md").exists());
        assert!(
            target.join(".claude/skills/aw-health/SKILL.md").exists(),
            "aw new should install current projected skills"
        );
        assert!(
            target.join(".claude/agents/aw-dev.md").exists(),
            "aw new should install current projected agents"
        );
    }

    #[test]
    fn test_refresh_existing_config_preserves_projects_on_force_refresh() {
        let existing = r#"
version = "0.1.0"

[[projects]]
name = "agentic-workflow"
path = "apps/agentic-workflow"
td_path = "apps/agentic-workflow/tech-design"
label = "app:agentic-workflow"

[agentic_workflow.issue_platform]
type = "github"
repo = "chrischeng-c4/cclab"
auth_method = "cli"

[workflow.agents]
mode = "legacy"
"#;

        let (updated, _applied) =
            refresh_existing_config_content(existing, "0.1.0", &PlatformTomlUpdate::Preserve);

        assert!(updated.contains(&format!("version = \"{}\"", SDD_VERSION)));
        assert!(updated.contains("[[projects]]"), "{updated}");
        assert!(updated.contains("td_path = \"apps/agentic-workflow/tech-design\""));
        assert!(updated.contains("label = \"app:agentic-workflow\""));
        assert!(updated.contains("[agentic_workflow.issue_platform]"));
        assert!(updated.contains("repo = \"chrischeng-c4/cclab\""));
        assert!(!updated.contains("[workflow.agents]"), "{updated}");
    }

    #[test]
    fn test_refresh_existing_config_replaces_platform_without_dropping_projects() {
        let existing = r#"
version = "0.1.0"

[[projects]]
name = "agentic-workflow"
path = "apps/agentic-workflow"
td_path = "apps/agentic-workflow/tech-design"
label = "app:agentic-workflow"

[agentic_workflow.issue_platform]
type = "github"
repo = "old/repo"
auth_method = "cli"
"#;
        let new_platform = "\n[agentic_workflow.issue_platform]\ntype = \"gitlab\"\nrepo = \"new/repo\"\nauth_method = \"cli\"\n";

        let (updated, _applied) = refresh_existing_config_content(
            existing,
            "0.1.0",
            &PlatformTomlUpdate::Replace(new_platform.to_string()),
        );

        assert!(updated.contains("[[projects]]"), "{updated}");
        assert!(updated.contains("label = \"app:agentic-workflow\""));
        assert!(updated.contains("type = \"gitlab\""));
        assert!(updated.contains("repo = \"new/repo\""));
        assert!(!updated.contains("repo = \"old/repo\""), "{updated}");
    }

    // REQ: R12, R14 — install_claude_skills installs all current aw-* skills.
    #[test]
    fn test_install_claude_skills_installs_current_skills() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        install_claude_skills(&skills_dir).unwrap();

        let expected_skills = [
            "aw-wi",
            "aw-health",
            // REQ: R12 — active support skills
            "aw-build-debug",
            "aw-build-release",
            "aw-mamba-test-coverage",
            "aw-guard",
        ];

        for skill in &expected_skills {
            let skill_path = skills_dir.join(skill).join("SKILL.md");
            assert!(
                skill_path.exists(),
                "SKILL.md for '{}' should be installed",
                skill
            );
            let content = fs::read_to_string(&skill_path).unwrap();
            assert!(
                !content.is_empty(),
                "SKILL.md for '{}' should not be empty",
                skill
            );
        }
    }

    #[test]
    fn test_install_claude_skills_prunes_legacy_standardize_loops() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        for skill in [
            "score-standardize-run",
            "score-standardize-managed-loop",
            "score-standardize-regenerable-loop",
            "aw-standardize-run",
            "aw-standardize-managed-loop",
            "aw-standardize-regenerable-loop",
            "aw-wi-patrol",
        ] {
            let legacy_dir = skills_dir.join(skill);
            fs::create_dir_all(&legacy_dir).unwrap();
            fs::write(legacy_dir.join("SKILL.md"), "# legacy").unwrap();
        }

        install_claude_skills(&skills_dir).unwrap();

        for skill in [
            "score-standardize-run",
            "score-standardize-managed-loop",
            "score-standardize-regenerable-loop",
            "aw-standardize-run",
            "aw-standardize-managed-loop",
            "aw-standardize-regenerable-loop",
            "aw-wi-patrol",
        ] {
            assert!(
                !skills_dir.join(skill).exists(),
                "legacy skill {} should be pruned",
                skill
            );
        }
    }

    // REQ: R14 — install_claude_skills prunes the removed aw-merge skill
    // (`aw td merge` no longer exists; the terminal step is code-check).
    #[test]
    fn test_install_claude_skills_prunes_aw_merge() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let aw_merge_dir = skills_dir.join("aw-merge");
        fs::create_dir_all(&aw_merge_dir).unwrap();
        fs::write(aw_merge_dir.join("SKILL.md"), "# aw-merge").unwrap();

        install_claude_skills(&skills_dir).unwrap();

        assert!(
            !skills_dir.join("aw-merge").exists(),
            "removed aw-merge skill should be pruned"
        );
    }

    // #1281: install_claude_skills prunes the removed aw-standardize skill
    // (`aw standardize` no longer exists; folded into `aw health`'s
    // `takeover-audit` axis and `aw td audit-record`, #1278).
    #[test]
    fn test_install_claude_skills_prunes_aw_standardize() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let aw_standardize_dir = skills_dir.join("aw-standardize");
        fs::create_dir_all(&aw_standardize_dir).unwrap();
        fs::write(aw_standardize_dir.join("SKILL.md"), "# aw-standardize").unwrap();

        install_claude_skills(&skills_dir).unwrap();

        assert!(
            !skills_dir.join("aw-standardize").exists(),
            "removed aw-standardize skill should be pruned"
        );
    }

    // #1503: both skill-tree installers share the same deprecated-skill
    // pruning path, so a previous aw-chat-listen install cannot survive.
    #[test]
    fn test_install_skills_prunes_aw_chat_listen() {
        for install in [
            install_claude_skills as fn(&Path) -> Result<()>,
            install_agents_skills as fn(&Path) -> Result<()>,
        ] {
            let tmp = TempDir::new().unwrap();
            let skills_dir = tmp.path().join("skills");
            let retired = skills_dir.join("aw-chat-listen");
            fs::create_dir_all(&retired).unwrap();
            fs::write(retired.join("SKILL.md"), "# retired chat listener").unwrap();

            install(&skills_dir).unwrap();

            assert!(
                !retired.exists(),
                "removed aw-chat-listen skill should be pruned"
            );
        }
    }

    // #1858: eight stale skills (lifecycle-superseded + external-model
    // helpers) are retired; both skill-tree installers must prune them
    // if found on disk from a previous install.
    #[test]
    fn test_install_skills_prunes_1858_retired_skills() {
        let retired = [
            "aw-release-patch",
            "aw-cb-claim",
            "aw-cb-fill",
            "aw-td-create",
            "aw-capability",
            "aw-codex-review",
            "aw-gemini-explore-codebase",
            "aw-gemini-explore-specs",
        ];

        for install in [
            install_claude_skills as fn(&Path) -> Result<()>,
            install_agents_skills as fn(&Path) -> Result<()>,
        ] {
            let tmp = TempDir::new().unwrap();
            let skills_dir = tmp.path().join("skills");
            fs::create_dir_all(&skills_dir).unwrap();

            for skill in &retired {
                let legacy_dir = skills_dir.join(skill);
                fs::create_dir_all(&legacy_dir).unwrap();
                fs::write(legacy_dir.join("SKILL.md"), "# legacy").unwrap();
            }

            install(&skills_dir).unwrap();

            for skill in &retired {
                assert!(
                    !skills_dir.join(skill).exists(),
                    "retired skill {} should be pruned",
                    skill
                );
            }
        }
    }

    // #1897: the generic Stop-hook `goal-loop` skill is retired in favor of
    // the CLI-owned `aw goal` verifiable-condition loop; both skill-tree
    // installers must prune it if found on disk from a previous install.
    #[test]
    fn test_install_skills_prunes_goal_loop() {
        for install in [
            install_claude_skills as fn(&Path) -> Result<()>,
            install_agents_skills as fn(&Path) -> Result<()>,
        ] {
            let tmp = TempDir::new().unwrap();
            let skills_dir = tmp.path().join("skills");
            let retired = skills_dir.join("goal-loop");
            fs::create_dir_all(&retired).unwrap();
            fs::write(retired.join("SKILL.md"), "# retired goal-loop").unwrap();

            install(&skills_dir).unwrap();

            assert!(
                !retired.exists(),
                "removed goal-loop skill should be pruned"
            );
        }
    }

    // #1897: the new `aw-goal` skill projects into both skill trees.
    #[test]
    fn test_install_skills_projects_aw_goal() {
        for install in [
            install_claude_skills as fn(&Path) -> Result<()>,
            install_agents_skills as fn(&Path) -> Result<()>,
        ] {
            let tmp = TempDir::new().unwrap();
            let skills_dir = tmp.path().join("skills");
            fs::create_dir_all(&skills_dir).unwrap();

            install(&skills_dir).unwrap();

            let skill_path = skills_dir.join("aw-goal").join("SKILL.md");
            assert!(skill_path.exists(), "aw-goal skill should be installed");
            let content = fs::read_to_string(&skill_path).unwrap();
            assert!(content.contains("aw goal set"));
        }
    }

    #[test]
    fn test_install_claude_skills_preserves_unrelated_codex_review_skill() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let codex_review_dir = skills_dir.join("codex-review");
        fs::create_dir_all(&codex_review_dir).unwrap();
        fs::write(codex_review_dir.join("SKILL.md"), "# generic codex review").unwrap();

        install_claude_skills(&skills_dir).unwrap();

        let preserved = fs::read_to_string(codex_review_dir.join("SKILL.md")).unwrap();
        assert_eq!(preserved, "# generic codex review");
    }

    // REQ: R16, R17 — install_claude_skills writes scripts/ subdirectory with executable permissions
    #[test]
    fn test_install_claude_skills_installs_scripts_with_exec_perms() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        install_claude_skills(&skills_dir).unwrap();

        // REQ: R15, R17 — three skills have companion script files
        let expected_scripts: &[(&str, &str)] = &[
            ("aw-build-debug", "build.sh"),
            ("aw-build-release", "release.sh"),
            ("aw-mamba-test-coverage", "coverage.sh"),
        ];

        for (skill, script) in expected_scripts {
            let script_path = skills_dir.join(skill).join("scripts").join(script);
            assert!(
                script_path.exists(),
                "Script {}/{} should be installed",
                skill,
                script
            );
            let content = fs::read_to_string(&script_path).unwrap();
            assert!(
                content.starts_with("#!/"),
                "Script {}/{} should have shebang line",
                skill,
                script
            );

            // REQ: R16 — scripts must be executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = fs::metadata(&script_path).unwrap().permissions().mode();
                assert!(
                    mode & 0o111 != 0,
                    "Script {}/{} should be executable (mode={:o})",
                    skill,
                    script,
                    mode
                );
            }
        }
    }

    // REQ: R13 — install_claude_skills is idempotent.
    #[test]
    fn test_install_claude_skills_idempotent() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // First install
        install_claude_skills(&skills_dir).unwrap();
        // Second install (re-run / update)
        install_claude_skills(&skills_dir).unwrap();

        // Core skills should still be present
        for skill in &[
            "aw-build-debug",
            "aw-build-release",
            "aw-mamba-test-coverage",
        ] {
            assert!(
                skills_dir.join(skill).join("SKILL.md").exists(),
                "SKILL.md for '{}' should survive re-installation",
                skill
            );
        }
    }

    // Issue #1034: install_agents installs the current aw-* subagent fleet
    // from templates/cli/mainthread/agents/.
    #[test]
    fn test_install_agents_installs_current_agents() {
        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();

        install_agents(&claude_dir).unwrap();

        let agents_dir = claude_dir.join("agents");
        for agent in ["aw-dev", "aw-td-writer", "aw-ec-writer", "aw-hw-filler"] {
            let agent_path = agents_dir.join(format!("{agent}.md"));
            assert!(agent_path.exists(), "{}.md should be installed", agent);
            let content = fs::read_to_string(&agent_path).unwrap();
            assert!(!content.is_empty(), "{}.md should not be empty", agent);
        }
    }

    // Issue #1034: install_agents prunes deprecated agent stubs.
    #[test]
    fn test_install_agents_prunes_deprecated_agents() {
        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        let agents_dir = claude_dir.join("agents");
        fs::create_dir_all(&agents_dir).unwrap();

        for legacy in ["sdd-review", "score-td-author", "score-cb-handwriter"] {
            fs::write(agents_dir.join(format!("{legacy}.md")), "# legacy").unwrap();
        }

        install_agents(&claude_dir).unwrap();

        for legacy in ["sdd-review", "score-td-author", "score-cb-handwriter"] {
            assert!(
                !agents_dir.join(format!("{legacy}.md")).exists(),
                "legacy agent {} should be pruned",
                legacy
            );
        }
    }

    // Issue #1034: install_agents is idempotent — repeat installs leave the
    // templates-sourced content byte-identical.
    #[test]
    fn test_install_agents_idempotent() {
        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();

        install_agents(&claude_dir).unwrap();
        let agents_dir = claude_dir.join("agents");
        let first_pass: Vec<String> = aw_agent_entries()
            .into_iter()
            .map(|(name, _)| fs::read_to_string(agents_dir.join(format!("{name}.md"))).unwrap())
            .collect();

        install_agents(&claude_dir).unwrap();
        let second_pass: Vec<String> = aw_agent_entries()
            .into_iter()
            .map(|(name, _)| fs::read_to_string(agents_dir.join(format!("{name}.md"))).unwrap())
            .collect();

        assert_eq!(
            first_pass, second_pass,
            "repeat install_agents should be a no-op"
        );
    }

    // Issue #1034: AC2 — templates/ carries every aw-* agent; no agent
    // content exists only in .claude/agents (byte-for-byte match).
    #[test]
    fn test_aw_agent_entries_match_templates_source() {
        for (name, content) in aw_agent_entries() {
            assert!(
                content.starts_with("---\nname: "),
                "{} should carry Claude Code agent frontmatter",
                name
            );
            assert!(
                content.contains(&format!("name: {name}")),
                "{} frontmatter name field should match its file name",
                name
            );
        }
    }

    // REQ: R10 — install_settings_json merges hook into existing settings
    #[test]
    fn test_install_settings_json_merges() {
        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();

        // Existing settings with PreToolUse hook but no SubagentStop
        let existing = serde_json::json!({
            "hooks": {
                "PreToolUse": [{"matcher": "Bash", "hooks": []}]
            },
            "permissions": {"allow": ["Bash"]}
        });
        fs::write(
            claude_dir.join("settings.json"),
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .unwrap();

        install_settings_json(&claude_dir).unwrap();

        let content: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(claude_dir.join("settings.json")).unwrap())
                .unwrap();

        // Original PreToolUse preserved
        assert!(
            content["hooks"]["PreToolUse"].is_array(),
            "PreToolUse should be preserved"
        );
        // Existing user hook is preserved and AW does not install its own hooks.
        let hooks = content["hooks"]
            .as_object()
            .expect("hooks should be object");
        assert!(hooks.contains_key("PreToolUse"));
        assert!(
            !hooks.contains_key("SubagentStop"),
            "SubagentStop should not be added: {hooks:?}"
        );
        let rendered = serde_json::to_string(hooks).unwrap();
        assert!(
            !rendered.contains("hook1-post-apply-validate.sh"),
            "{rendered}"
        );
        assert!(!rendered.contains("hook2-pre-apply-guard.sh"), "{rendered}");
        // permissions preserved
        assert!(content["permissions"]["allow"].is_array());
    }
}

// Install shell completions for supported shells
fn install_shell_completions() -> Result<()> {
    println!("   ℹ Shell completions are not installed by this binary");
    Ok(())
}

// CODEGEN-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/cli/init.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
      Issue #984 (init-projector slice 1/3): the asset installer projected BOTH root
      docs from the same `aw:start` template section. Refactored
      `generate_claude_md`'s upsert logic into shared
      `compute_upserted_doc`/`upsert_managed_section` helpers; added
      `generate_agents_md`/`get_agents_sdd_section` (AGENTS.md = CLAUDE.md's
      section plus `doc_mirror::agents_block_from_claude_block`'s Codex-only
      insertions), wired into both `run_fresh_install` and `run_update`.
      The retired top-level `aw init` status/check helpers and chainable
      `next:` emitters were removed; `aw new` now owns greenfield bootstrap
      user output, while existing-project guidance is routed through
      `aw health`.

      Issue #985 (init-projector slice 2/3): `get_sdd_section` now returns
      the template's `aw:start` section run through
      `doc_mirror::render_cli_tables`, so the document upsert path receives
      the rendered fine-grained
      `<!-- aw:cli-table:{workflow,support}:start/end -->` CLI-table
      markers directly. Added
      `update_readme_projects_table`
      (wired into `run_fresh_install` and `run_update`): the
      repo-root README's `<!-- aw:projects-table:start/end -->` Projects
      table (rendered by `doc_mirror::upsert_projects_table` from
      `.aw/config.toml`) is opt-in per document — managed asset installation only touches
      README.md when it already exists AND already carries the markers, so
      a README without them (or no README at all, e.g. a fresh `aw new`
      scaffold) is left untouched.

      Fresh-install fix (still #985): once `get_sdd_section` performs real
      rendering, `generate_claude_md` could no longer pass the raw
      `CLAUDE_TEMPLATE` constant as `upsert_managed_section`'s
      `full_doc_if_missing` fallback — a brand-new CLAUDE.md would be
      seeded with the template's unrendered markers/seed rows, which
      `agents_block_from_claude_block` would also project
      forward into a fresh AGENTS.md, corrupting it too). Extracted
      `split_claude_template` (shared before/section/after split at the
      `aw:start`/`aw:end` marker offsets) and added `rendered_claude_doc`
      (the same split, with the section rendered) so `generate_claude_md`'s
      fresh-install fallback is now content-equivalent to the update path, mirroring
      how `generate_agents_md` already built its fallback from the rendered
      `sdd_section` rather than a raw constant.

      Issue #986 (init-projector slice 3/3): `templates/cli/mainthread/
      skills/` is now the sole source for every `aw-*` skill, installed into
      BOTH `.claude/skills/` and `.agents/skills/`. Extracted the previously
      inline skill list/prune/write logic out of `install_claude_skills` into
      shared `aw_skill_entries`/`skill_script_entries`/
      `deprecated_skill_names`/`prune_deprecated_skills`/`write_skill_file`/
      `install_skill_scripts` helpers, added a new `SKILL_GUARD` embed
      (`aw-guard`, missing from the installed trees before this issue), and
      added a sibling `install_agents_skills` that projects each skill body
      through `doc_mirror::agents_skill_body_from_claude_skill_body` before
      writing it — same prune list, same script installer, only the body
      differs. `install_system_files` now also creates and populates
      `.agents/skills/`.

      Issue #1077 (archetype-as-traits slice 1/3): added
      `update_contributing_trait_table`, wired into `run_fresh_install` and
      `run_update` immediately after `update_readme_projects_table`.
      Same opt-in-per-document
      contract as the README Projects table (issue #985): only touches
      CONTRIBUTING.md when it exists AND already carries the
      `<!-- aw:trait-table:start/end -->` markers, rendering the enclosed
      table from `doc_mirror::upsert_trait_table`/`doc_mirror::TRAITS`.

      Issue #1034 (init-projector follow-up to #986): the aw-* Claude Code
      subagent fleet (`aw-dev`, `aw-td-writer`, `aw-ec-writer`,
      `aw-hw-filler`) is now projected from `templates/cli/mainthread/agents/`
      instead of living only as hand-maintained `.claude/agents/*.md` files.
      Added `AGENT_AW_DEV`/`AGENT_AW_TD_WRITER`/`AGENT_AW_EC_WRITER`/
      `AGENT_AW_HW_FILLER` embeds and an `aw_agent_entries` list (same shape
      as `aw_skill_entries`); extracted the previously inline retired-agent
      array out of `install_agents` into `deprecated_agent_names`/
      `prune_deprecated_agents` (same list, same behavior, renamed to match
      `deprecated_skill_names`/`prune_deprecated_skills`); added
      `write_agent_file`. `install_agents` now prunes deprecated agents and
      then installs every `aw_agent_entries` file, so a fresh `aw new` and a
      repeat `aw new`/update both leave `.claude/agents/` byte-identical to
      `templates/`. Unlike skills, agent definitions are Claude
      Code-runtime-only: there is no `.agents/agents/` counterpart because
      Codex has no matching subagent mechanism yet — `templates/` is still
      the sole source, so a future Codex-side projection is additive.

      Issue #1842 (three-host agent fleet projection): the Codex-side
      projection anticipated by #1034 now exists, plus a third host (AGY).
      Canonical `templates/cli/mainthread/agents/*.md` frontmatter gains a
      `model_tier: top | standard | cheap` field
      (`CanonicalAgentFrontmatter`/`parse_agent_frontmatter`) resolved
      through one per-tier, per-host model table (`AGENT_MODEL_TIERS`/
      `tier_host_models`/`resolve_host_model`) — unknown tier or a tier
      missing a host mapping fails loudly instead of silently defaulting.
      `render_codex_agent` embeds the canonical body as a Codex
      `developer_instructions` TOML string via `codex_developer_instructions`
      (Markdown emphasis stripped, `## Heading` -> `Heading:`);
      `render_agy_agent` preserves the canonical body verbatim under AGY
      workspace-subagent frontmatter (`kind: local`/`model`/`max_turns`/
      `timeout_mins`/`enable_write_tools`/`enable_mcp_tools`), deriving
      `enable_write_tools` from the canonical `tools:` list. `install_agents`
      itself is unchanged (still Claude-only, still used directly by
      existing callers/tests); the three-host producer is the new
      `install_agent_fleet`, which validates every agent's literal `model:`
      field against its `model_tier`'s resolved claude mapping
      (`validate_agent_fleet_frontmatter`) and then calls `install_agents`
      plus the new `install_codex_agents`/`install_agy_agents`
      (`.codex/agents/*.toml`, `.agents/agents/*.md`), all sharing the
      renamed `prune_deprecated_fleet_files` helper so a retired fleet name
      is pruned on all three hosts. `install_system_files` now calls
      `install_agent_fleet` instead of `install_agents` directly. Added a
      read-only `check_agent_fleet`/`run_agent_fleet_check` pair wired to a
      new `aw new --check-agents` flag (reports missing/drifted/stale
      projections per host with the exact remediation command, exits
      non-zero on drift) and a narrow write-only `aw new --sync-agents` flag
      (`install_agent_fleet` only, bypassing the rest of the asset installer
      so it is safe to run against an already-initialized project without
      also force-refreshing aw.toml/hooks/settings/skills/META-docs).

      Issue #1858: retired eight stale `aw-*` skills (`aw-release-patch`,
      `aw-cb-claim`, `aw-cb-fill`, `aw-td-create`, `aw-capability`,
      `aw-codex-review`, `aw-gemini-explore-codebase`,
      `aw-gemini-explore-specs`) — lifecycle-superseded (folded into `aw td
      create --from-source`, `aw td fill` envelopes + the aw-hw-filler
      agent, `aw wi run` resumption + the aw-td-writer agent, and the
      one-way WI-reference / CAPABILITIES.md cap_path shape from #1847/
      #1848) plus unused external-model helpers. Removed their
      `SKILL_*`/`SCRIPT_*` `include_str!` consts and `aw_skill_entries`/
      `skill_script_entries` rows, and added all eight names to
      `deprecated_skill_names` so both `install_claude_skills` and
      `install_agents_skills` prune them from `.claude/skills/` and
      `.agents/skills/` on every install. Kept skills' bodies (`aw-wi`)
      no longer reference the retired `/aw:capability` skill.
  - path: apps/agentic-workflow/src/cli/init.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Issue #1897: adds the `SKILL_GOAL` `include_str!` const (templates
      source `templates/cli/mainthread/skills/aw-goal/SKILL.md`) and its
      `aw_skill_entries` row so the new thin `/aw:goal` dispatcher skill
      projects into both `.claude/skills/` and `.agents/skills/`. Adds
      `"goal-loop"` to `deprecated_skill_names` so the retired generic
      Stop-hook goal-loop skill is pruned from both trees on every install
      (it never reliably fired its `SubagentStop` hook — see its own
      "Known gaps"; the CLI-owned `aw goal` verifiable-condition loop is
      the enforcement now). Adds `test_install_skills_prunes_goal_loop`
      and `test_install_skills_projects_aw_goal` fixtures proving both
      directions on both installers.
```
