// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/init.md#source
// CODEGEN-BEGIN
use crate::models::{SddConfig, SddInterface};
use crate::Result;
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
const SKILL_CODEX_REVIEW: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-codex-review/SKILL.md");
const SKILL_GEMINI_EXPLORE_SPECS: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-gemini-explore-specs/SKILL.md");
const SKILL_GEMINI_EXPLORE_CODEBASE: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-gemini-explore-codebase/SKILL.md");
const SKILL_MERGE: &str = include_str!("../../templates/cli/mainthread/skills/aw-merge/SKILL.md");
const SKILL_CAPABILITY: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-capability/SKILL.md");
const SKILL_WI: &str = include_str!("../../templates/cli/mainthread/skills/aw-wi/SKILL.md");
const SKILL_BUILD_DEBUG: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-build-debug/SKILL.md");
const SKILL_RELEASE_PATCH: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-release-patch/SKILL.md");
const SKILL_MAMBA_TEST_COVERAGE: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-mamba-test-coverage/SKILL.md");
const SKILL_TD_CREATE: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-td-create/SKILL.md");
const SKILL_CB_FILL: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-cb-fill/SKILL.md");
const SKILL_CB_CLAIM: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-cb-claim/SKILL.md");
const SKILL_STANDARDIZE: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-standardize/SKILL.md");
const SKILL_BUILD_RELEASE: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-build-release/SKILL.md");
const SKILL_CHAT_LISTEN: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-chat-listen/SKILL.md");
const SKILL_HEALTH: &str = include_str!("../../templates/cli/mainthread/skills/aw-health/SKILL.md");
const SCRIPT_BUILD_RELEASE: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-build-release/scripts/release.sh");
// @spec projects/agentic-workflow/tech-design/surface/specs/init-command.md#R15
const SCRIPT_BUILD_DEBUG: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-build-debug/scripts/build.sh");
const SCRIPT_RELEASE_PATCH: &str =
    include_str!("../../templates/cli/mainthread/skills/aw-release-patch/scripts/release.sh");
const SCRIPT_MAMBA_TEST_COVERAGE: &str = include_str!(
    "../../templates/cli/mainthread/skills/aw-mamba-test-coverage/scripts/coverage.sh"
);

// Claude Code settings.json template
// @spec projects/agentic-workflow/tech-design/surface/specs/init-command.md#R9
const SETTINGS_JSON_TEMPLATE: &str = include_str!("../../templates/cli/mainthread/settings.json");

// CLAUDE.md Template for target projects
const CLAUDE_TEMPLATE: &str = include_str!("../../templates/cli/mainthread/CLAUDE.md");

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/init.md#source
pub async fn run(name: Option<&str>, force: bool, _agent_mode: Option<&str>) -> Result<()> {
    // `_agent_mode` is retained in the signature for backward-compat with old
    // CLI invocations, but is ignored. Agentic Workflow uses a fixed executor mapping
    // (Claude Code subagent + mainthread hybrid) — there is no mode to select.
    let project_root = env::current_dir()?;
    run_at_project_root(name, force, &project_root, true)
}

/// Arguments for `aw new`.
///
/// `aw new` creates the project directory first, then delegates to the same
/// in-place installer used by `aw init`.
// @spec projects/agentic-workflow/tech-design/logic/manage-aw-init-templates-as-greenfield-ready-artifacts.md#CLI
#[derive(Debug, Args)]
pub struct NewArgs {
    /// Project directory name when --path is not supplied
    pub name: String,

    /// Explicit target directory. When omitted, target is ./<name>.
    #[arg(long, value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Allow reusing an existing non-empty directory and force-refresh init assets.
    #[arg(short, long)]
    pub force: bool,

    /// Create the target directory without running aw init.
    #[arg(long)]
    pub no_init: bool,
}

// @spec projects/agentic-workflow/tech-design/logic/manage-aw-init-templates-as-greenfield-ready-artifacts.md#Logic
pub async fn run_new(args: NewArgs) -> Result<()> {
    let current_dir = env::current_dir()?;
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
    if !outcome.init_ran {
        println!("   {}", "aw init".cyan());
    }

    Ok(())
}

struct NewProjectOutcome {
    target: PathBuf,
    init_ran: bool,
}

fn run_new_with_current_dir(args: NewArgs, current_dir: &Path) -> Result<NewProjectOutcome> {
    let target = resolve_new_target(current_dir, &args.name, args.path.as_deref())?;
    prepare_new_target(&target, args.force)?;

    if args.no_init {
        println!(
            "{}",
            format!("📁 Created project directory {}", target.display()).cyan()
        );
        println!("   ℹ Skipped aw init because --no-init was supplied");
        return Ok(NewProjectOutcome {
            target,
            init_ran: false,
        });
    }

    run_at_project_root(Some(&args.name), args.force, &target, false)?;

    Ok(NewProjectOutcome {
        target,
        init_ran: true,
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
                "target directory is not empty: {} (rerun with --force to run aw init there)",
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

fn run_at_project_root(
    name: Option<&str>,
    force: bool,
    project_root: &Path,
    print_fresh_success: bool,
) -> Result<()> {
    let legacy_score_dir = project_root.join(concat!(".", "score"));
    let legacy_cclab_dir = project_root.join("cclab");
    let sdd_dir = project_root.join(crate::shared::workspace::WORKSPACE_DIR);
    let claude_dir = project_root.join(".claude");

    if legacy_score_dir.exists() {
        anyhow::bail!(
            "legacy Agentic Workflow state found at {}; active state now lives under .aw. Move or remove the old directory explicitly, then rerun aw init.",
            legacy_score_dir.display()
        );
    }

    // Auto-migrate: rename cclab/ → .aw/ if the legacy dir exists
    if legacy_cclab_dir.exists() && !sdd_dir.exists() {
        println!("{}", "🔄 Migrating cclab/ → .aw/...".cyan());
        std::fs::rename(&legacy_cclab_dir, &sdd_dir)?;
        println!("   ✓ Renamed cclab/ to .aw/");
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
        run_update(name, &project_root, &sdd_dir, &claude_dir, force)?;
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
            name,
            &project_root,
            &sdd_dir,
            &claude_dir,
            interface,
            platform_toml,
            print_fresh_success,
        )?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Platform selection
// ---------------------------------------------------------------------------

// Platform type selected during init
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum Platform {
    GitHub,
    GitLab,
    None,
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

// Interactive platform selection for aw init.
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
        Platform::None => unreachable!(),
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
                        Platform::None => unreachable!(),
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
        Platform::None => unreachable!(),
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
                Platform::None => unreachable!(),
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
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/init.md#source
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
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/init.md#source
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
    name: Option<&str>,
    project_root: &Path,
    sdd_dir: &Path,
    claude_dir: &Path,
    interface: SddInterface,
    platform_toml: Option<String>,
    print_success_message: bool,
) -> Result<()> {
    // Create directory structure
    println!("{}", "📁 Creating directory structure...".cyan());
    std::fs::create_dir_all(sdd_dir)?;
    std::fs::create_dir_all(sdd_dir.join("tech-design"))?;

    // Create Claude Code skills directory
    let skills_dir = claude_dir.join("skills");
    std::fs::create_dir_all(&skills_dir)?;

    // Create config with selected interface
    let _ = name; // name parameter is deprecated and ignored
    let mut config = SddConfig::with_interface(interface);
    config.set_version(SDD_VERSION);

    // Detect workspace type and pre-populate spec scopes for Rust monorepos.
    let workspace_type = detect_workspace_type(project_root);
    if workspace_type == WorkspaceType::RustCargo {
        populate_rust_scopes(&mut config, project_root);
    }
    config.save(project_root)?;

    // Append platform + workspace-specific scopes hint to config.toml
    let config_path = project_root.join(".aw/config.toml");
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

    println!("   ✓ .aw/config.toml (interface: {})", interface.name());

    // Install system files
    install_system_files(project_root, sdd_dir, claude_dir)?;

    // Generate CLAUDE.md with project context
    generate_claude_md(project_root, sdd_dir)?;

    if print_success_message {
        print_init_success();
    }

    Ok(())
}

// Update mode: overwrite config.toml, update system files
fn run_update(
    name: Option<&str>,
    project_root: &Path,
    sdd_dir: &Path,
    claude_dir: &Path,
    force: bool,
) -> Result<()> {
    // name parameter is deprecated and ignored
    let _ = name;

    println!("{}", "📦 User data:".cyan());
    println!("   ✓ .aw/tech-design/     (untouched)");

    let config_path = sdd_dir.join("config.toml");
    let existing_config = if config_path.exists() {
        Some(std::fs::read_to_string(&config_path)?)
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
            println!("   ✓ .aw/config.toml (migrated: {})", applied.join(", "));
        } else if force {
            println!("   ✓ .aw/config.toml (force refreshed)");
        } else {
            println!("   ✓ .aw/config.toml (updated)");
        }
    } else {
        let mut config = SddConfig::with_interface(SddInterface::Cli);
        config.set_version(SDD_VERSION);
        config.save(project_root)?;

        let mut content = std::fs::read_to_string(&config_path)?;
        content = apply_platform_update(&content, &platform_update);
        std::fs::write(&config_path, content)?;
        println!("   ✓ .aw/config.toml (created)");
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

    // Regenerate CLAUDE.md with project context
    generate_claude_md(project_root, sdd_dir)?;

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
fn install_system_files(_project_root: &Path, _sdd_dir: &Path, claude_dir: &Path) -> Result<()> {
    let skills_dir = claude_dir.join("skills");
    std::fs::create_dir_all(&skills_dir)?;

    // Install Claude Code Skills
    println!("{}", "🤖 Updating Claude Code Skills...".cyan());
    install_claude_skills(&skills_dir)?;

    // Install Claude Code Agent definitions
    println!();
    println!("{}", "🧠 Retiring legacy Claude Code Agents...".cyan());
    install_agents(claude_dir)?;

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

// Print success message for fresh install
fn print_init_success() {
    println!();
    println!(
        "{}",
        "✅ Agentic Workflow initialized successfully!"
            .green()
            .bold()
    );
    println!();
    println!("{}", "📁 Structure:".cyan());
    println!("   .aw/                  - Agentic Workflow workspace (hidden)");
    println!("   .aw/config.toml       - Configuration");
    println!("   .aw/tech-design/      - Tech design docs");
    println!();
    println!("{}", "🤖 Claude Code assets installed:".cyan());
    println!("   .claude/skills/          - Agentic Workflow skills");
    println!("   .claude/hooks/           - retired hook cleanup area");
    println!("   .claude/settings.json    - permissions + status line settings");
    println!();

    println!("{}", "🎯 Primary Workflow (use skills):".cyan().bold());
    println!(
        "   {} - Tech-design lifecycle",
        "/aw:td:create".green().bold()
    );
    println!(
        "   {} - Existing-project takeover",
        "/aw:standardize".green().bold()
    );
    println!();

    println!("{}", "⏭️  Next Steps:".yellow().bold());
    println!("   Start your first change:");
    println!("      {}", "/aw:td:create my-feature".cyan());
}

// Score section markers (must match templates/mainthread/CLAUDE.md)
const GENESIS_START_MARKER: &str = "<!-- aw:start -->";
const GENESIS_END_MARKER: &str = "<!-- aw:end -->";

// Extract the SDD section from template (between markers)
fn get_sdd_section() -> &'static str {
    let start = CLAUDE_TEMPLATE.find(GENESIS_START_MARKER).unwrap_or(0);
    let end = CLAUDE_TEMPLATE
        .find(GENESIS_END_MARKER)
        .map(|i| i + GENESIS_END_MARKER.len())
        .unwrap_or(CLAUDE_TEMPLATE.len());
    &CLAUDE_TEMPLATE[start..end]
}

// Remove old SDD sections (without markers) from content
fn remove_old_sdd_sections(content: &str) -> String {
    let mut result = content.to_string();

    // Pattern 1: "## SDD Workflow" section (old format)
    if let Some(start) = result.find("## SDD Workflow") {
        // Find the next ## heading or end of file
        let after_start = &result[start + 18..]; // skip "## SDD Workflow"
        if let Some(next_heading) = after_start.find("\n## ") {
            let end = start + 18 + next_heading + 1; // +1 to keep the newline before next heading
            result = format!("{}{}", &result[..start], &result[end..]);
        } else {
            // No next heading - remove to end
            result = result[..start].trim_end().to_string();
        }
    }

    // Pattern 2: "## File Structure" section with genesis paths (old format)
    if let Some(start) = result.find("## File Structure") {
        let section_content = &result[start..];
        // Only remove if it contains genesis-specific content
        if section_content.contains("cclab/project.md")
            || section_content.contains(".aw/tech-design/")
        {
            let after_start = &result[start + 17..]; // skip "## File Structure"
            if let Some(next_heading) = after_start.find("\n## ") {
                let end = start + 17 + next_heading + 1;
                result = format!("{}{}", &result[..start], &result[end..]);
            } else if let Some(next_heading) = after_start.find("\n# ") {
                let end = start + 17 + next_heading + 1;
                result = format!("{}{}", &result[..start], &result[end..]);
            } else {
                result = result[..start].trim_end().to_string();
            }
        }
    }

    // Clean up multiple consecutive newlines
    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n");
    }

    result
}

// Generate or update CLAUDE.md with SDD section (upsert mode)
fn generate_claude_md(project_root: &Path, _sdd_dir: &Path) -> Result<()> {
    let claude_md_path = project_root.join("CLAUDE.md");

    let sdd_section = get_sdd_section();

    if claude_md_path.exists() {
        // CLAUDE.md exists - upsert the SDD section
        let existing_content = std::fs::read_to_string(&claude_md_path)?;

        // First, remove old format sections (without markers)
        let cleaned_content = remove_old_sdd_sections(&existing_content);

        let new_content = if let (Some(start), Some(end)) = (
            cleaned_content.find(GENESIS_START_MARKER),
            cleaned_content.find(GENESIS_END_MARKER),
        ) {
            // Markers exist - replace content between them
            let before = &cleaned_content[..start];
            let after = &cleaned_content[end + GENESIS_END_MARKER.len()..];
            format!("{}{}{}", before, sdd_section, after)
        } else {
            // No markers - prepend SDD section after first heading or at top
            if let Some(first_newline) = cleaned_content.find('\n') {
                let first_line = &cleaned_content[..first_newline];
                if first_line.starts_with('#') {
                    // Insert after the first heading
                    let after_heading = &cleaned_content[first_newline..];
                    format!("{}\n\n{}{}", first_line, sdd_section, after_heading)
                } else {
                    // Prepend at top
                    format!("{}\n\n{}", sdd_section, cleaned_content)
                }
            } else {
                format!("{}\n\n{}", sdd_section, cleaned_content)
            }
        };

        // Check if content changed
        if new_content.trim() == existing_content.trim() {
            println!("   {} CLAUDE.md (up to date)", "✓".green());
        } else {
            std::fs::write(&claude_md_path, new_content)?;
            println!("   {} CLAUDE.md (updated)", "✓".green());
        }
    } else {
        // CLAUDE.md doesn't exist - create with full template
        std::fs::write(&claude_md_path, CLAUDE_TEMPLATE)?;
        println!("   {} CLAUDE.md (created)", "✓".green());
    }

    Ok(())
}

// Check if upgrade is available and optionally auto-upgrade
// Returns true if auto-upgrade was performed
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/init.md#source
pub fn check_and_auto_upgrade(auto_upgrade: bool) -> bool {
    let project_root = match env::current_dir() {
        Ok(p) => p,
        Err(_) => return false,
    };

    let sdd_dir = project_root.join(crate::shared::workspace::WORKSPACE_DIR);

    // Not initialized, nothing to upgrade
    if !sdd_dir.exists() {
        return false;
    }

    let installed_version_owned =
        read_version_from_config_or_file(&sdd_dir).unwrap_or_else(|| "0.0.0".to_string());
    let installed_version = installed_version_owned.trim();

    // Compare versions - only upgrade if CLI version is newer than installed
    if installed_version == SDD_VERSION {
        return false; // Already up to date
    }

    // Check if CLI version is actually newer (not older)
    if !crate::cli::update::is_newer(SDD_VERSION, installed_version) {
        // CLI is older than installed - don't downgrade
        return false;
    }

    // CLI version is newer - upgrade
    if auto_upgrade {
        println!(
            "{}",
            format!(
                "🔄 Auto-upgrading SDD: {} → {}",
                installed_version, SDD_VERSION
            )
            .cyan()
        );

        let sdd_dir = project_root.join(crate::shared::workspace::WORKSPACE_DIR);
        let claude_dir = project_root.join(".claude");

        if let Err(e) = run_update(None, &project_root, &sdd_dir, &claude_dir, false) {
            eprintln!("{}", format!("⚠️  Auto-upgrade failed: {}", e).yellow());
            return false;
        }

        println!();
        return true;
    } else {
        // Just notify
        println!(
            "{}",
            format!(
                "💡 SDD update available: {} → {} (run {} to upgrade)",
                installed_version,
                SDD_VERSION,
                "aw init --force".cyan()
            )
            .yellow()
        );
        println!();
        return false;
    }
}

// Get installed version (for display purposes)
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/init.md#source
pub fn get_installed_version() -> Option<String> {
    let project_root = env::current_dir().ok()?;
    let sdd_dir = project_root.join(crate::shared::workspace::WORKSPACE_DIR);
    read_version_from_config_or_file(&sdd_dir)
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

// Get current CLI version
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/init.md#source
pub fn get_current_version() -> &'static str {
    SDD_VERSION
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

fn install_claude_skills(skills_dir: &Path) -> Result<()> {
    // Remove deprecated skills
    let deprecated_skills = vec![
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
        // Removed: cron-style issue patrol is superseded by aw run --project.
        "aw-wi-patrol",
        "score-build-release",
        "score-chat-listen",
        "score-fillback-main-specs",
    ];

    for deprecated in &deprecated_skills {
        let deprecated_dir = skills_dir.join(deprecated);
        if deprecated_dir.exists() {
            std::fs::remove_dir_all(&deprecated_dir)?;
            println!("   {} {} (removed)", "✗".red(), deprecated);
        }
    }

    // Install current skills
    // @spec projects/agentic-workflow/tech-design/surface/specs/init-command.md#R12
    // @spec projects/agentic-workflow/tech-design/surface/specs/init-command.md#R13
    // @spec projects/agentic-workflow/tech-design/surface/specs/init-command.md#R12
    // @spec projects/agentic-workflow/tech-design/surface/specs/init-command.md#R13
    let skills = vec![
        ("aw-codex-review", SKILL_CODEX_REVIEW),
        ("aw-gemini-explore-specs", SKILL_GEMINI_EXPLORE_SPECS),
        ("aw-gemini-explore-codebase", SKILL_GEMINI_EXPLORE_CODEBASE),
        ("aw-merge", SKILL_MERGE),
        ("aw-capability", SKILL_CAPABILITY),
        ("aw-wi", SKILL_WI),
        ("aw-build-debug", SKILL_BUILD_DEBUG),
        ("aw-release-patch", SKILL_RELEASE_PATCH),
        ("aw-mamba-test-coverage", SKILL_MAMBA_TEST_COVERAGE),
        ("aw-td-create", SKILL_TD_CREATE),
        ("aw-cb-fill", SKILL_CB_FILL),
        ("aw-cb-claim", SKILL_CB_CLAIM),
        ("aw-standardize", SKILL_STANDARDIZE),
        ("aw-build-release", SKILL_BUILD_RELEASE),
        ("aw-chat-listen", SKILL_CHAT_LISTEN),
        ("aw-health", SKILL_HEALTH),
    ];

    for (name, content) in skills {
        let skill_dir = skills_dir.join(name);
        std::fs::create_dir_all(&skill_dir)?;
        std::fs::write(skill_dir.join("SKILL.md"), content)?;
        println!("   ✓ {}", name);
    }

    // Install scripts for skills that have them
    // @spec projects/agentic-workflow/tech-design/surface/specs/init-command.md#R16
    let skill_scripts: &[(&str, &str, &str)] = &[
        ("aw-build-debug", "build.sh", SCRIPT_BUILD_DEBUG),
        ("aw-release-patch", "release.sh", SCRIPT_RELEASE_PATCH),
        (
            "aw-mamba-test-coverage",
            "coverage.sh",
            SCRIPT_MAMBA_TEST_COVERAGE,
        ),
        ("aw-build-release", "release.sh", SCRIPT_BUILD_RELEASE),
    ];

    for (skill_name, script_name, content) in skill_scripts {
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

// Remove retired Claude Code subagent definition files from `.claude/agents/`.
// @spec projects/agentic-workflow/tech-design/surface/specs/init-command.md#R5
// @spec projects/agentic-workflow/tech-design/surface/specs/init-command.md#R6
fn install_agents(claude_dir: &Path) -> Result<()> {
    let agents_dir = claude_dir.join("agents");
    std::fs::create_dir_all(&agents_dir)?;

    // Remove legacy/deprecated agent stubs. The current Score workflow is
    // mainthread-only, so there are no current score-* agent files to install.
    let retired_agents = &[
        "sdd-change-implementation.md",
        "sdd-change-spec.md",
        "sdd-reference-context.md",
        "sdd-review.md",
        "sdd-issue-author.md",
        // score-reference-context replaced by score-issue-author
        "score-reference-context.md",
        // score-change-* retired — `score workflow` is gone, see aw td
        "score-change-implementation.md",
        "score-change-spec.md",
        "score-review.md",
        "score-issue-author.md",
        "score-issue-reviewer.md",
        "score-issue-reviser.md",
        "score-td-author.md",
        "score-td-reviewer.md",
        "score-td-reviser.md",
        "score-cb-handwriter.md",
    ];
    for legacy in retired_agents {
        let legacy_path = agents_dir.join(legacy);
        if legacy_path.exists() {
            std::fs::remove_file(&legacy_path)?;
            println!("   {} {} (removed retired)", "✗".red(), legacy);
        }
    }

    Ok(())
}

// Delete legacy hook scripts from `.claude/hooks/`.
//
// Legacy flat-layout and subagent hook scripts from earlier score versions are
// removed so deployments do not register autonomous Claude hook callbacks.
// @spec projects/agentic-workflow/tech-design/surface/specs/init-command.md#R8
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
// @spec projects/agentic-workflow/tech-design/surface/specs/init-command.md#R10
// @spec projects/agentic-workflow/tech-design/surface/specs/init-command.md#R11
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
        // R13: deny rules for `.aw/tech-design/**` are installed.
        let deny = content["permissions"]["deny"]
            .as_array()
            .expect("permissions.deny should be present after fresh install");
        let rules: Vec<&str> = deny.iter().filter_map(|e| e.as_str()).collect();
        assert!(
            rules.contains(&"Edit(.aw/tech-design/**)"),
            "deny list missing Edit rule, got {:?}",
            rules
        );
        assert!(
            rules.contains(&"Write(.aw/tech-design/**)"),
            "deny list missing Write rule, got {:?}",
            rules
        );
        assert!(
            rules.contains(&"MultiEdit(.aw/tech-design/**)"),
            "deny list missing MultiEdit rule, got {:?}",
            rules
        );
    }

    // R13: re-running `aw init` against an existing settings.json that
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
        assert!(rules.contains(&"Edit(.aw/tech-design/**)"), "{:?}", rules);
        assert!(rules.contains(&"Write(.aw/tech-design/**)"), "{:?}", rules);
        assert!(
            rules.contains(&"MultiEdit(.aw/tech-design/**)"),
            "{:?}",
            rules
        );
        // No duplicates after second run.
        let edit_count = rules
            .iter()
            .filter(|r| **r == "Edit(.aw/tech-design/**)")
            .count();
        assert_eq!(
            edit_count, 1,
            "Edit rule duplicated after second install: {:?}",
            rules
        );
    }

    // REQ: R11 — install_settings_json removes existing retired score-* hook
    // matchers when re-running `aw init`.
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

    // REQ: aw-greenfield-project-bootstrap UT3 — --no-init creates only the target directory.
    #[test]
    fn test_new_no_init_creates_target_directory_only() {
        let tmp = TempDir::new().unwrap();
        let args = NewArgs {
            name: "ai-studio".to_string(),
            path: None,
            force: false,
            no_init: true,
        };

        let outcome = run_new_with_current_dir(args, tmp.path()).unwrap();

        assert_eq!(outcome.target, tmp.path().join("ai-studio"));
        assert!(!outcome.init_ran);
        assert!(outcome.target.is_dir());
        assert!(!outcome.target.join(".aw").exists());
    }

    // REQ: aw-greenfield-project-bootstrap UT5 — aw new delegates to the shared init installer.
    #[test]
    fn test_new_runs_shared_init_installer() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("ai-studio");
        let args = NewArgs {
            name: "ai-studio".to_string(),
            path: Some(target.clone()),
            force: false,
            no_init: false,
        };

        let outcome = run_new_with_current_dir(args, tmp.path()).unwrap();

        assert_eq!(outcome.target, target);
        assert!(outcome.init_ran);
        assert!(target.join(".aw/config.toml").exists());
        assert!(target.join(".aw/tech-design").is_dir());
        assert!(target.join("CLAUDE.md").exists());
        assert!(
            target.join(".claude/skills/aw-health/SKILL.md").exists(),
            "aw new should install the same current skills as aw init"
        );
    }

    #[test]
    fn test_refresh_existing_config_preserves_projects_on_force_refresh() {
        let existing = r#"
version = "0.1.0"

[[projects]]
name = "agentic-workflow"
path = "projects/agentic-workflow"
td_path = "projects/agentic-workflow/tech-design"
label = "project:agentic-workflow"

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
        assert!(updated.contains("td_path = \"projects/agentic-workflow/tech-design\""));
        assert!(updated.contains("label = \"project:agentic-workflow\""));
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
path = "projects/agentic-workflow"
td_path = "projects/agentic-workflow/tech-design"
label = "project:agentic-workflow"

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
        assert!(updated.contains("label = \"project:agentic-workflow\""));
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
            "aw-codex-review",
            "aw-gemini-explore-specs",
            "aw-gemini-explore-codebase",
            "aw-merge",
            "aw-capability",
            "aw-wi",
            "aw-standardize",
            "aw-health",
            // REQ: R12 — active support skills
            "aw-build-debug",
            "aw-release-patch",
            "aw-mamba-test-coverage",
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
            ("aw-release-patch", "release.sh"),
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
            "aw-release-patch",
            "aw-mamba-test-coverage",
        ] {
            assert!(
                skills_dir.join(skill).join("SKILL.md").exists(),
                "SKILL.md for '{}' should survive re-installation",
                skill
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
