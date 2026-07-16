// SPEC-MANAGED: apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#source
// CODEGEN-BEGIN
// generator-gap: aw-guard-agent-hooks-v1
// reason: Agent-runtime hook installation and pre-tool policy are not yet covered by deterministic CLI codegen primitives.
use anyhow::{Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use globset::{GlobSet, GlobSetBuilder};
use serde_json::{json, Map, Value};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cli::guard_sanction::{self, SanctionReason};
use crate::services::path_scope::{self, AllowedScope};
use crate::services::project_registry::{self, ProjectConfigRow};

const CODEX_HOOKS_REL: &str = ".codex/hooks.json";
const CLAUDE_SETTINGS_REL: &str = ".claude/settings.json";
const CODEX_MATCHER: &str = "Edit|Write|apply_patch";
const CLAUDE_MATCHER: &str = "Edit|Write|MultiEdit|NotebookEdit";
const AGY_CONFIG_KEY: &str = "aw-project-guard";
const AGY_MATCHER: &str = "run_command";
const LOCAL_BYPASS_GIT_REL: &str = "aw/guard-bypass";

#[derive(Debug, Args)]
/// Install or run AW agent-write guard hooks.
/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#source
pub struct GuardArgs {
    #[command(subcommand)]
    pub command: GuardCommand,
}

#[derive(Debug, Subcommand)]
/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#source
pub enum GuardCommand {
    /// Enable direct edit/create tool guards for a project scope and commit the
    /// repo-tracked Codex/Claude policy files when they change.
    On(GuardToggleArgs),
    /// Disable AW-managed direct edit/create tool guards for a project scope
    /// and commit the repo-tracked Codex/Claude policy files when they change.
    /// Already-loaded hook callbacks observe the disabled state on their next
    /// invocation, so Codex does not need a session reload.
    Off(GuardToggleArgs),
    /// Temporarily allow direct edits in this worktree without changing the
    /// committed guard policy. The bypass expires automatically.
    Bypass(GuardBypassArgs),
    /// Remove a worktree-local temporary bypass before it expires.
    Resume(GuardBypassArgs),
    /// Hook entrypoint: read PreToolUse JSON from stdin and allow/deny.
    Pretool(GuardPretoolArgs),
}

#[derive(Debug, Args)]
/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#source
pub struct GuardToggleArgs {
    /// Project name or alias from AW project config.
    #[arg(long)]
    pub project: String,
    /// Which agent hook config to manage.
    #[arg(long, value_enum, default_value_t = GuardAgent::All)]
    pub agent: GuardAgent,
    /// Emit JSON summary.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#source
pub struct GuardPretoolArgs {
    /// Project name or alias from AW project config.
    #[arg(long)]
    pub project: String,
    /// Agent adapter that produced the hook payload.
    #[arg(long, value_enum, default_value_t = GuardAgent::All)]
    pub agent: GuardAgent,
}

#[derive(Debug, Args)]
/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#source
pub struct GuardBypassArgs {
    /// Project name or alias from AW project config.
    #[arg(long)]
    pub project: String,
    /// Lifetime for `bypass`; ignored by `resume`.
    #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..=1_440))]
    pub minutes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#source
pub enum GuardAgent {
    All,
    Codex,
    Claude,
    Agy,
}

impl GuardAgent {
    fn includes_codex(self) -> bool {
        matches!(self, GuardAgent::All | GuardAgent::Codex)
    }

    fn includes_claude(self) -> bool {
        matches!(self, GuardAgent::All | GuardAgent::Claude)
    }

    fn includes_agy(self) -> bool {
        matches!(self, GuardAgent::All | GuardAgent::Agy)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GuardHookChange {
    agent: &'static str,
    path: PathBuf,
    changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GuardDecision {
    Allow,
    /// A target that falls inside the guarded scope but is currently
    /// sanctioned for direct edit by a TD `impl_mode: hand-written` declaration
    /// at an eligible WI phase (#1428/#1429). `path` is the repo-root-relative
    /// target that was sanctioned; `reason` names the sanctioning WI/TD/phase
    /// for the allow envelope.
    AllowSanctioned {
        path: String,
        reason: SanctionReason,
    },
    Deny {
        reason: String,
    },
}

/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#source
pub async fn run(args: GuardArgs) -> Result<()> {
    match args.command {
        GuardCommand::On(args) => run_on(args),
        GuardCommand::Off(args) => run_off(args),
        GuardCommand::Bypass(args) => run_bypass(args),
        GuardCommand::Resume(args) => run_resume(args),
        GuardCommand::Pretool(args) => run_pretool(args).await,
    }
}

fn run_on(args: GuardToggleArgs) -> Result<()> {
    let root = crate::find_project_root()?;
    let row = project_registry::resolve_project_config_row(&root, &args.project)?;
    ensure_tracked_guard_paths_clean(&root, args.agent)?;
    let changes = install_guard_hooks(&root, &row.name, args.agent)?;
    let commit = commit_guard_policy(&root, "enable", &row.name, &changes)?;
    emit_toggle_summary("enabled", &row.name, &changes, commit.as_deref(), args.json);
    Ok(())
}

fn run_off(args: GuardToggleArgs) -> Result<()> {
    let root = crate::find_project_root()?;
    let row = project_registry::resolve_project_config_row(&root, &args.project)?;
    ensure_tracked_guard_paths_clean(&root, args.agent)?;
    let changes = remove_guard_hooks(&root, &row.name, args.agent)?;
    let commit = commit_guard_policy(&root, "disable", &row.name, &changes)?;
    emit_toggle_summary(
        "disabled",
        &row.name,
        &changes,
        commit.as_deref(),
        args.json,
    );
    Ok(())
}

fn run_bypass(args: GuardBypassArgs) -> Result<()> {
    let root = crate::find_project_root()?;
    let row = project_registry::resolve_project_config_row(&root, &args.project)?;
    let now = unix_now()?;
    let expires_at = now.saturating_add(args.minutes.saturating_mul(60));
    let path = local_bypass_path(&root, &row.name)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating local guard bypass dir {}", parent.display()))?;
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "schema_version": 1,
            "project": row.name,
            "expires_at_unix": expires_at,
        }))? + "\n",
    )
    .with_context(|| format!("writing local guard bypass {}", path.display()))?;
    println!(
        "aw guard bypass active for project `{}` until unix timestamp {}; next: aw guard resume --project {}",
        row.name, expires_at, row.name
    );
    Ok(())
}

fn run_resume(args: GuardBypassArgs) -> Result<()> {
    let root = crate::find_project_root()?;
    let row = project_registry::resolve_project_config_row(&root, &args.project)?;
    let path = local_bypass_path(&root, &row.name)?;
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("removing local guard bypass {}", path.display()))?;
        println!("aw guard bypass cleared for project `{}`", row.name);
    } else {
        println!(
            "aw guard bypass already inactive for project `{}`",
            row.name
        );
    }
    Ok(())
}

async fn run_pretool(args: GuardPretoolArgs) -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("reading guard hook stdin")?;
    let payload: Value = match serde_json::from_str(&input) {
        Ok(payload) => payload,
        Err(err) => {
            eprintln!("aw guard: fail-open: invalid PreToolUse JSON: {err}");
            emit_agy_allow_if_needed(args.agent);
            return Ok(());
        }
    };

    let root = match crate::find_project_root() {
        Ok(root) => root,
        Err(err) => {
            eprintln!("aw guard: fail-open: cannot resolve project root: {err:#}");
            emit_agy_allow_if_needed(args.agent);
            return Ok(());
        }
    };
    match decide_active_pretool_payload(&root, &args.project, args.agent, &payload).await {
        Ok(GuardDecision::Allow) => emit_agy_allow_if_needed(args.agent),
        Ok(GuardDecision::AllowSanctioned { path, reason }) => {
            let reason = format!(
                "AW guard allows direct hand-written edit at `{}` — sanctioned by WI #{} (TD `{}`, phase `{}`).",
                path, reason.wi_id, reason.td_path, reason.phase
            );
            emit_pretool_allow(args.agent, Some(reason));
        }
        Ok(GuardDecision::Deny { reason }) => {
            emit_pretool_deny(args.agent, &reason);
        }
        Err(err) => {
            eprintln!("aw guard: fail-open: {err:#}");
            emit_agy_allow_if_needed(args.agent);
        }
    }
    Ok(())
}

fn emit_toggle_summary(
    action: &str,
    project: &str,
    changes: &[GuardHookChange],
    commit: Option<&str>,
    json_output: bool,
) {
    if json_output {
        println!(
            "{}",
            json!({
                "action": action,
                "project": project,
                "commit": commit,
                "changes": changes.iter().map(|change| {
                    json!({
                        "agent": change.agent,
                        "path": change.path.to_string_lossy(),
                        "changed": change.changed,
                    })
                }).collect::<Vec<_>>(),
            })
        );
        return;
    }

    println!("aw guard {action} for project `{project}`");
    for change in changes {
        let marker = if change.changed {
            "updated"
        } else {
            "unchanged"
        };
        println!(
            "{}: {} ({marker})",
            change.agent,
            change.path.to_string_lossy()
        );
    }
    if let Some(commit) = commit {
        println!("commit: {commit}");
    }
}

fn install_guard_hooks(
    root: &Path,
    project: &str,
    agent: GuardAgent,
) -> Result<Vec<GuardHookChange>> {
    let mut changes = Vec::new();
    if agent.includes_codex() {
        changes.push(upsert_hook_file(
            root,
            CODEX_HOOKS_REL,
            "codex",
            CODEX_MATCHER,
            &guard_command("codex", project),
        )?);
    }
    if agent.includes_claude() {
        changes.push(upsert_hook_file(
            root,
            CLAUDE_SETTINGS_REL,
            "claude",
            CLAUDE_MATCHER,
            &guard_command("claude", project),
        )?);
    }
    if agent.includes_agy() {
        changes.push(install_agy_guard_hook(project)?);
    }
    Ok(changes)
}

fn remove_guard_hooks(
    root: &Path,
    project: &str,
    agent: GuardAgent,
) -> Result<Vec<GuardHookChange>> {
    let mut changes = Vec::new();
    if agent.includes_codex() {
        changes.push(remove_hook_from_file(
            root,
            CODEX_HOOKS_REL,
            "codex",
            Some("codex"),
            project,
        )?);
    }
    if agent.includes_claude() {
        changes.push(remove_hook_from_file(
            root,
            CLAUDE_SETTINGS_REL,
            "claude",
            Some("claude"),
            project,
        )?);
    }
    if agent.includes_agy() {
        changes.push(remove_agy_guard_hook(project)?);
    }
    Ok(changes)
}

fn guard_command(agent: &str, project: &str) -> String {
    format!("aw guard pretool --agent {agent} --project {project}")
}

fn agy_guard_command(project: &str) -> String {
    let executable = std::env::current_exe()
        .ok()
        .and_then(|path| path.to_str().map(str::to_owned))
        .unwrap_or_else(|| "aw".to_string());
    format!("{executable} guard pretool --agent agy --project {project}")
}

fn agy_hooks_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("resolving $HOME for AGY hook config")?;
    let current = home.join(".gemini").join("config").join("hooks.json");
    let legacy = home
        .join(".gemini")
        .join("antigravity-cli")
        .join("hooks.json");
    if current.exists() || !legacy.exists() {
        Ok(current)
    } else {
        Ok(legacy)
    }
}

fn install_agy_guard_hook(project: &str) -> Result<GuardHookChange> {
    let path = agy_hooks_path()?;
    let command = agy_guard_command(project);
    let changed = upsert_agy_guard_hook_at(&path, &command)?;
    Ok(GuardHookChange {
        agent: "agy",
        path,
        changed,
    })
}

fn remove_agy_guard_hook(project: &str) -> Result<GuardHookChange> {
    let path = agy_hooks_path()?;
    let changed = remove_agy_guard_hook_at(&path, project)?;
    Ok(GuardHookChange {
        agent: "agy",
        path,
        changed,
    })
}

fn upsert_agy_guard_hook_at(path: &Path, command: &str) -> Result<bool> {
    let mut doc = read_json_or_empty_object(path)?;
    let before = pretty_json(&doc)?;
    let root = ensure_object(&mut doc)?;
    let config = ensure_child_object(root, AGY_CONFIG_KEY)?;
    let pretool = ensure_child_array(config, "PreToolUse")?;
    let already_present = pretool.iter().any(|entry| {
        entry
            .get("hooks")
            .and_then(Value::as_array)
            .is_some_and(|hooks| {
                hooks.iter().any(|hook| {
                    hook.get("command")
                        .and_then(Value::as_str)
                        .is_some_and(|existing| existing == command)
                })
            })
    });
    if !already_present {
        pretool.push(json!({
            "matcher": AGY_MATCHER,
            "hooks": [{ "type": "command", "command": command, "timeout": 10 }],
        }));
    }
    let after = pretty_json(&doc)?;
    write_json_if_changed(path, &before, &after)?;
    Ok(before != after)
}

fn remove_agy_guard_hook_at(path: &Path, project: &str) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let mut doc = read_json_or_empty_object(path)?;
    let before = pretty_json(&doc)?;
    let Some(configs) = doc.as_object_mut() else {
        anyhow::bail!("AGY hook config root must be a JSON object");
    };
    for config in configs.values_mut() {
        let Some(config) = config.as_object_mut() else {
            continue;
        };
        let Some(entries) = config.get_mut("PreToolUse").and_then(Value::as_array_mut) else {
            continue;
        };
        let mut retained = Vec::with_capacity(entries.len());
        for mut entry in std::mem::take(entries) {
            let keep = entry
                .get_mut("hooks")
                .and_then(Value::as_array_mut)
                .map(|hooks| {
                    hooks.retain(|hook| !is_aw_guard_handler(hook, Some("agy"), Some(project)));
                    !hooks.is_empty()
                })
                .unwrap_or(true);
            if keep {
                retained.push(entry);
            }
        }
        *entries = retained;
    }
    let after = pretty_json(&doc)?;
    write_json_if_changed(path, &before, &after)?;
    Ok(before != after)
}

fn upsert_hook_file(
    root: &Path,
    rel: &str,
    agent_name: &'static str,
    matcher: &str,
    command: &str,
) -> Result<GuardHookChange> {
    let path = root.join(rel);
    let mut doc = read_json_or_empty_object(&path)?;
    let before = pretty_json(&doc)?;
    remove_aw_guard_handlers(&mut doc, Some(agent_name), command_project(command));
    append_pretool_handler(&mut doc, matcher, aw_guard_handler(command))?;
    let after = pretty_json(&doc)?;
    write_json_if_changed(&path, &before, &after)?;
    Ok(GuardHookChange {
        agent: agent_name,
        path,
        changed: before != after,
    })
}

fn remove_hook_from_file(
    root: &Path,
    rel: &str,
    agent_name: &'static str,
    hook_agent: Option<&str>,
    project: &str,
) -> Result<GuardHookChange> {
    let path = root.join(rel);
    if !path.exists() {
        return Ok(GuardHookChange {
            agent: agent_name,
            path,
            changed: false,
        });
    }
    let mut doc = read_json_or_empty_object(&path)?;
    let before = pretty_json(&doc)?;
    remove_aw_guard_handlers(&mut doc, hook_agent, Some(project));
    let after = pretty_json(&doc)?;
    write_json_if_changed(&path, &before, &after)?;
    Ok(GuardHookChange {
        agent: agent_name,
        path,
        changed: before != after,
    })
}

fn command_project(command: &str) -> Option<&str> {
    command.split("--project ").nth(1).and_then(|tail| {
        tail.split_whitespace()
            .next()
            .map(|value| value.trim_matches('"'))
    })
}

fn read_json_or_empty_object(path: &Path) -> Result<Value> {
    if !path.exists() {
        return Ok(json!({}));
    }
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing JSON at {}", path.display()))
}

fn pretty_json(value: &Value) -> Result<String> {
    let mut text = serde_json::to_string_pretty(value).context("serializing hook JSON")?;
    text.push('\n');
    Ok(text)
}

fn write_json_if_changed(path: &Path, before: &str, after: &str) -> Result<()> {
    if before == after {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    fs::write(path, after).with_context(|| format!("writing {}", path.display()))
}

fn aw_guard_handler(command: &str) -> Value {
    json!({
        "type": "command",
        "command": command,
        "timeout": 30,
        "statusMessage": "Checking AW guard",
    })
}

fn append_pretool_handler(doc: &mut Value, matcher: &str, handler: Value) -> Result<()> {
    let root = ensure_object(doc)?;
    let hooks = ensure_child_object(root, "hooks")?;
    let pretool = ensure_child_array(hooks, "PreToolUse")?;

    if let Some(group) = pretool.iter_mut().find(|group| {
        group
            .get("matcher")
            .and_then(Value::as_str)
            .map(|value| value == matcher)
            .unwrap_or(false)
    }) {
        let group_obj = group
            .as_object_mut()
            .context("PreToolUse matcher group must be an object")?;
        ensure_child_array(group_obj, "hooks")?.push(handler);
        return Ok(());
    }

    pretool.push(json!({
        "matcher": matcher,
        "hooks": [handler],
    }));
    Ok(())
}

fn ensure_object(value: &mut Value) -> Result<&mut Map<String, Value>> {
    if value.is_null() {
        *value = json!({});
    }
    value
        .as_object_mut()
        .context("hook config root must be a JSON object")
}

fn ensure_child_object<'a>(
    object: &'a mut Map<String, Value>,
    key: &str,
) -> Result<&'a mut Map<String, Value>> {
    if !object.contains_key(key) || object.get(key).is_some_and(Value::is_null) {
        object.insert(key.to_string(), json!({}));
    }
    object
        .get_mut(key)
        .and_then(Value::as_object_mut)
        .with_context(|| format!("`{key}` must be a JSON object"))
}

fn ensure_child_array<'a>(
    object: &'a mut Map<String, Value>,
    key: &str,
) -> Result<&'a mut Vec<Value>> {
    if !object.contains_key(key) || object.get(key).is_some_and(Value::is_null) {
        object.insert(key.to_string(), json!([]));
    }
    object
        .get_mut(key)
        .and_then(Value::as_array_mut)
        .with_context(|| format!("`{key}` must be a JSON array"))
}

fn remove_aw_guard_handlers(doc: &mut Value, agent: Option<&str>, project: Option<&str>) {
    let Some(groups) = doc
        .get_mut("hooks")
        .and_then(|hooks| hooks.get_mut("PreToolUse"))
        .and_then(Value::as_array_mut)
    else {
        return;
    };

    for group in groups {
        let Some(hooks) = group.get_mut("hooks").and_then(Value::as_array_mut) else {
            continue;
        };
        hooks.retain(|hook| !is_aw_guard_handler(hook, agent, project));
    }
}

fn is_aw_guard_handler(hook: &Value, agent: Option<&str>, project: Option<&str>) -> bool {
    let Some(command) = hook.get("command").and_then(Value::as_str) else {
        return false;
    };
    if !command.contains("aw guard pretool") {
        return false;
    }
    if let Some(agent) = agent {
        if !command.contains(&format!("--agent {agent}")) {
            return false;
        }
    }
    if let Some(project) = project {
        if !command.contains(&format!("--project {project}")) {
            return false;
        }
    }
    true
}

fn emit_agy_allow_if_needed(agent: GuardAgent) {
    if matches!(agent, GuardAgent::Agy) {
        println!("{}", agy_allow_output(None));
    }
}

fn emit_pretool_allow(agent: GuardAgent, reason: Option<String>) {
    if matches!(agent, GuardAgent::Agy) {
        println!("{}", agy_allow_output(reason));
        return;
    }
    if let Some(reason) = reason {
        println!(
            "{}",
            json!({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "allow",
                    "permissionDecisionReason": reason,
                }
            })
        );
    }
}

fn emit_pretool_deny(agent: GuardAgent, reason: &str) {
    if matches!(agent, GuardAgent::Agy) {
        println!("{}", agy_deny_output(reason));
        return;
    }
    println!(
        "{}",
        json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": reason,
            }
        })
    );
}

fn agy_allow_output(reason: Option<String>) -> Value {
    let mut output = json!({ "decision": "allow" });
    if let Some(reason) = reason {
        output["reason"] = Value::String(reason);
    }
    output
}

fn agy_deny_output(reason: &str) -> Value {
    json!({ "decision": "deny", "reason": reason })
}

fn tracked_guard_paths(agent: GuardAgent) -> Vec<&'static str> {
    let mut paths = Vec::new();
    if agent.includes_codex() {
        paths.push(CODEX_HOOKS_REL);
    }
    if agent.includes_claude() {
        paths.push(CLAUDE_SETTINGS_REL);
    }
    paths
}

fn ensure_tracked_guard_paths_clean(root: &Path, agent: GuardAgent) -> Result<()> {
    let paths = tracked_guard_paths(agent);
    if paths.is_empty() {
        return Ok(());
    }
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain", "--"])
        .args(&paths);
    let output = command
        .output()
        .context("checking guard policy git state")?;
    if !output.status.success() {
        anyhow::bail!(
            "git status failed before guard toggle: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let dirty = String::from_utf8_lossy(&output.stdout);
    if !dirty.trim().is_empty() {
        anyhow::bail!(
            "refusing to toggle AW guard because its tracked policy files already have uncommitted changes:\n{}",
            dirty.trim()
        );
    }
    Ok(())
}

fn commit_guard_policy(
    root: &Path,
    action: &str,
    project: &str,
    changes: &[GuardHookChange],
) -> Result<Option<String>> {
    let paths: Vec<&str> = changes
        .iter()
        .filter(|change| change.changed)
        .filter_map(|change| {
            change
                .path
                .strip_prefix(root)
                .ok()
                .and_then(|path| path.to_str())
        })
        .filter(|path| *path == CODEX_HOOKS_REL || *path == CLAUDE_SETTINGS_REL)
        .collect();
    if paths.is_empty() {
        return Ok(None);
    }

    let mut add = Command::new("git");
    add.arg("-C").arg(root).args(["add", "--"]).args(&paths);
    let output = add.output().context("staging AW guard policy")?;
    if !output.status.success() {
        anyhow::bail!(
            "git add for AW guard policy failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let message = format!("chore(aw-guard): {action} {project}");
    let mut commit = Command::new("git");
    commit
        .arg("-C")
        .arg(root)
        .args(["commit", "--only", "-m", &message, "--"])
        .args(&paths);
    let output = commit.output().context("committing AW guard policy")?;
    if !output.status.success() {
        anyhow::bail!(
            "git commit for AW guard policy failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .context("reading AW guard policy commit")?;
    if !output.status.success() {
        anyhow::bail!(
            "git rev-parse after AW guard commit failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(Some(
        String::from_utf8_lossy(&output.stdout).trim().to_string(),
    ))
}

fn local_bypass_path(root: &Path, project: &str) -> Result<PathBuf> {
    let git_rel = format!("{LOCAL_BYPASS_GIT_REL}/{project}.json");
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--git-path", &git_rel])
        .output()
        .context("resolving worktree-local AW guard bypass path")?;
    if !output.status.success() {
        anyhow::bail!(
            "git rev-parse --git-path failed for AW guard bypass: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let path = PathBuf::from(value);
    Ok(if path.is_absolute() {
        path
    } else {
        root.join(path)
    })
}

fn unix_now() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("reading system time for AW guard bypass")?
        .as_secs())
}

fn local_bypass_is_active(root: &Path, project: &str) -> Result<bool> {
    let path = local_bypass_path(root, project)?;
    if !path.exists() {
        return Ok(false);
    }
    let value: Value = serde_json::from_str(
        &fs::read_to_string(&path)
            .with_context(|| format!("reading local guard bypass {}", path.display()))?,
    )
    .with_context(|| format!("parsing local guard bypass {}", path.display()))?;
    let expires_at = value
        .get("expires_at_unix")
        .and_then(Value::as_u64)
        .context("local guard bypass is missing expires_at_unix")?;
    if unix_now()? < expires_at {
        return Ok(true);
    }
    fs::remove_file(&path)
        .with_context(|| format!("clearing expired local guard bypass {}", path.display()))?;
    Ok(false)
}

/// Check the live hook configuration before enforcing the policy. Codex can
/// retain a callback that was loaded before `aw guard off`; that callback still
/// invokes this command, so the command itself must honour the current config
/// instead of treating callback invocation as proof that the guard is enabled.
fn guard_handler_is_active(root: &Path, project: &str, agent: GuardAgent) -> Result<bool> {
    let mut active = false;
    if agent.includes_codex() {
        active |= hook_file_has_guard_handler(root, CODEX_HOOKS_REL, "codex", project)?;
    }
    if agent.includes_claude() {
        active |= hook_file_has_guard_handler(root, CLAUDE_SETTINGS_REL, "claude", project)?;
    }
    if agent.includes_agy() {
        active |= agy_hook_has_guard_handler(project)?;
    }
    Ok(active)
}

fn agy_hook_has_guard_handler(project: &str) -> Result<bool> {
    let path = agy_hooks_path()?;
    if !path.exists() {
        return Ok(false);
    }
    let doc = read_json_or_empty_object(&path)?;
    Ok(doc.as_object().is_some_and(|configs| {
        configs.values().any(|config| {
            config
                .get("PreToolUse")
                .and_then(Value::as_array)
                .is_some_and(|entries| {
                    entries.iter().any(|entry| {
                        entry
                            .get("hooks")
                            .and_then(Value::as_array)
                            .is_some_and(|hooks| {
                                hooks.iter().any(|hook| {
                                    is_aw_guard_handler(hook, Some("agy"), Some(project))
                                })
                            })
                    })
                })
        })
    }))
}

fn hook_file_has_guard_handler(root: &Path, rel: &str, agent: &str, project: &str) -> Result<bool> {
    let path = root.join(rel);
    if !path.exists() {
        return Ok(false);
    }
    let doc = read_json_or_empty_object(&path)?;
    Ok(doc
        .get("hooks")
        .and_then(|hooks| hooks.get("PreToolUse"))
        .and_then(Value::as_array)
        .is_some_and(|groups| {
            groups.iter().any(|group| {
                group
                    .get("hooks")
                    .and_then(Value::as_array)
                    .is_some_and(|hooks| {
                        hooks
                            .iter()
                            .any(|hook| is_aw_guard_handler(hook, Some(agent), Some(project)))
                    })
            })
        }))
}

async fn decide_active_pretool_payload(
    root: &Path,
    requested_project: &str,
    agent: GuardAgent,
    payload: &Value,
) -> Result<GuardDecision> {
    if !guard_handler_is_active(root, requested_project, agent)? {
        return Ok(GuardDecision::Allow);
    }
    if local_bypass_is_active(root, requested_project)? {
        return Ok(GuardDecision::Allow);
    }
    decide_pretool_payload(root, requested_project, agent, payload).await
}

async fn decide_pretool_payload(
    root: &Path,
    requested_project: &str,
    agent: GuardAgent,
    payload: &Value,
) -> Result<GuardDecision> {
    let scope = GuardScope::for_project(root, requested_project)?;
    let targets = extract_target_paths(payload, agent);
    let mut sanctioned_allow: Option<(String, SanctionReason)> = None;
    for target in targets {
        let Some(rel) = target_to_repo_rel(root, &target) else {
            continue;
        };
        if scope.contains(&rel) {
            match sanction_reason_for(root, &scope, &rel).await {
                Some(reason) => {
                    sanctioned_allow.get_or_insert((rel, reason));
                    continue;
                }
                None => {
                    return Ok(GuardDecision::Deny {
                        reason: format!(
                            "AW guard blocks direct edit/create for project `{}` at `{}`. Use the AW CLI lifecycle, or explicitly run `aw guard bypass --project {}` for a temporary local bypass.",
                            scope.project, rel, scope.project
                        ),
                    });
                }
            }
        }
    }
    if let Some((path, reason)) = sanctioned_allow {
        return Ok(GuardDecision::AllowSanctioned { path, reason });
    }
    Ok(GuardDecision::Allow)
}

/// Consult the #1428 sanctioned-path resolver for a target already known to
/// fall inside `scope`'s guarded prefixes: `repo_rel` is repo-root-relative
/// (matches `target_to_repo_rel`'s output); the resolver keys sanctioned
/// paths project-root-relative (see `guard_sanction`'s module doc), so this
/// strips `scope`'s project-root prefix first. `None` on any resolver miss
/// (unsanctioned path, resolver error, or a target outside the project root
/// proper such as a `td_path`/`cap_path`/workspace-glob match) — callers
/// fail closed on `None`, matching #1428 AC3 (deterministic, no panics).
async fn sanction_reason_for(
    root: &Path,
    scope: &GuardScope,
    repo_rel: &str,
) -> Option<SanctionReason> {
    let project_rel = scope.strip_project_prefix(repo_rel)?;
    guard_sanction::is_sanctioned(root, &scope.project, Path::new(&project_rel))
        .await
        .ok()
        .flatten()
}

fn extract_target_paths(payload: &Value, agent: GuardAgent) -> Vec<PathBuf> {
    let mut targets = Vec::new();
    let tool_name = payload
        .get("tool_name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let tool_input = payload.get("tool_input").unwrap_or(&Value::Null);

    if let Some(path) = tool_input.get("file_path").and_then(Value::as_str) {
        if !path.is_empty() {
            targets.push(PathBuf::from(path));
        }
    }

    if matches!(agent, GuardAgent::All | GuardAgent::Codex)
        || tool_name == "apply_patch"
        || tool_name == "Edit"
        || tool_name == "Write"
    {
        if let Some(command) = tool_input.get("command").and_then(Value::as_str) {
            targets.extend(
                parse_apply_patch_targets(command)
                    .into_iter()
                    .map(PathBuf::from),
            );
        }
    }

    if agent.includes_agy() {
        if let Some(command) = agy_command_line(payload) {
            targets.extend(
                parse_agy_direct_mutation_targets(command)
                    .into_iter()
                    .map(PathBuf::from),
            );
        }
    }

    targets
}

fn agy_command_line(payload: &Value) -> Option<&str> {
    payload
        .get("toolCall")
        .and_then(|call| {
            call.get("name")
                .and_then(Value::as_str)
                .map(|name| (call, name))
        })
        .filter(|(_, name)| *name == "run_command")
        .and_then(|(call, _)| call.get("args"))
        .and_then(|args| args.get("CommandLine"))
        .and_then(Value::as_str)
}

/// AGY exposes shell-like `run_command`, not structured Write/Edit events.
/// Deliberately recognize only direct, explicit mutations with a target path;
/// unknown commands remain allowed rather than turning AW guard into a broad
/// shell-command policy (that role remains with `cap`).
fn parse_agy_direct_mutation_targets(command: &str) -> Vec<String> {
    let mut targets = Vec::new();
    for segment in command.split(|ch| matches!(ch, ';' | '|' | '&')) {
        let tokens: Vec<&str> = segment
            .split_whitespace()
            .filter(|token| !token.is_empty())
            .collect();
        for (index, token) in tokens.iter().enumerate() {
            if matches!(*token, ">" | ">>") {
                if let Some(path) = tokens.get(index + 1) {
                    targets.push(clean_shell_path(path));
                }
            }
            if let Some(path) = token.strip_prefix(">>") {
                if !path.is_empty() {
                    targets.push(clean_shell_path(path));
                }
            } else if let Some(path) = token.strip_prefix(">") {
                if !path.is_empty() {
                    targets.push(clean_shell_path(path));
                }
            }
        }

        for (index, token) in tokens.iter().enumerate() {
            match *token {
                "touch" | "tee" => {
                    targets.extend(shell_path_operands(&tokens[index + 1..]));
                }
                "sed" => {
                    if tokens
                        .get(index + 1)
                        .is_some_and(|value| value.starts_with("-i"))
                    {
                        let mut operands = shell_path_operands(&tokens[index + 2..]).into_iter();
                        let _script = operands.next();
                        targets.extend(operands);
                    }
                }
                "rm" => {
                    targets.extend(shell_path_operands(&tokens[index + 1..]));
                }
                "cp" | "mv" | "install" => {
                    if let Some(path) = tokens[index + 1..]
                        .iter()
                        .rev()
                        .find(|value| !value.starts_with('-'))
                    {
                        targets.push(clean_shell_path(path));
                    }
                }
                _ => {}
            }
        }
    }

    targets.retain(|path| !path.is_empty());
    targets.sort();
    targets.dedup();
    targets
}

fn shell_path_operands(tokens: &[&str]) -> Vec<String> {
    tokens
        .iter()
        .filter(|value| !value.starts_with('-'))
        .map(|value| clean_shell_path(value))
        .filter(|value| !value.is_empty())
        .collect()
}

fn clean_shell_path(value: &str) -> String {
    value
        .trim_matches(|ch| matches!(ch, '\'' | '"' | '`' | '(' | ')' | '{' | '}' | ','))
        .to_string()
}

fn parse_apply_patch_targets(command: &str) -> Vec<String> {
    let mut targets = Vec::new();
    for line in command.lines().map(str::trim) {
        for marker in [
            "*** Add File: ",
            "*** Update File: ",
            "*** Delete File: ",
            "*** Move to: ",
            "*** Rename to: ",
        ] {
            if let Some(path) = line.strip_prefix(marker) {
                let trimmed = path.trim();
                if !trimmed.is_empty() {
                    targets.push(trimmed.to_string());
                }
            }
        }
    }
    targets.sort();
    targets.dedup();
    targets
}

#[derive(Debug)]
struct GuardScope {
    project: String,
    prefixes: Vec<String>,
    globset: GlobSet,
    legacy_scope: Option<AllowedScope>,
    /// The project's repo-root-relative source directory (`row.path`,
    /// trailing slash trimmed) — the base the #1428 resolver's sanctioned
    /// paths are relative to. See `strip_project_prefix`.
    project_root_rel: String,
}

impl GuardScope {
    fn for_project(root: &Path, requested_project: &str) -> Result<Self> {
        let row = project_registry::resolve_project_config_row(root, requested_project)?;
        let mut prefixes = guard_prefixes_from_row(&row);
        prefixes.sort();
        prefixes.dedup();
        let project_root_rel = row.path.trim_end_matches('/').to_string();

        let legacy_scope = path_scope::load_scope(root)?
            .and_then(|cfg| path_scope::project_by_name(&cfg, &row.name).cloned())
            .map(|project| AllowedScope::for_project(&project))
            .transpose()?;

        let globset = GlobSetBuilder::new()
            .build()
            .context("building empty guard globset")?;
        Ok(Self {
            project: row.name,
            prefixes,
            globset,
            legacy_scope,
            project_root_rel,
        })
    }

    fn contains(&self, rel: &str) -> bool {
        if self.prefixes.iter().any(|prefix| {
            rel == prefix || rel.starts_with(&format!("{}/", prefix.trim_end_matches('/')))
        }) {
            return true;
        }
        if self.globset.is_match(rel) {
            return true;
        }
        self.legacy_scope
            .as_ref()
            .map(|scope| scope.contains(rel))
            .unwrap_or(false)
    }

    /// Strip this project's root prefix from a repo-root-relative path,
    /// returning the project-root-relative remainder the #1428 resolver
    /// keys sanctioned paths by. `None` when `repo_rel` is not strictly
    /// inside the project root (e.g. it matched via `td_path`, `cap_path`,
    /// or a legacy workspace glob instead) — those are never TD-sanctioned
    /// hand-write targets.
    fn strip_project_prefix(&self, repo_rel: &str) -> Option<String> {
        let prefix = format!("{}/", self.project_root_rel);
        repo_rel.strip_prefix(&prefix).map(|s| s.to_string())
    }
}

fn guard_prefixes_from_row(row: &ProjectConfigRow) -> Vec<String> {
    [
        Some(row.path.as_str()),
        row.td_path.as_deref(),
        row.cap_path.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(|value| value.trim_end_matches('/').to_string())
    .collect()
}

fn target_to_repo_rel(root: &Path, target: &Path) -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let abs_target = if target.is_absolute() {
        target.to_path_buf()
    } else {
        cwd.join(target)
    };
    let root_abs = root
        .canonicalize()
        .unwrap_or_else(|_| lexical_normalize(root));
    let target_abs = resolve_existing_prefix(&abs_target);
    target_abs
        .strip_prefix(&root_abs)
        .ok()
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
}

fn resolve_existing_prefix(path: &Path) -> PathBuf {
    if path.exists() {
        return path
            .canonicalize()
            .unwrap_or_else(|_| lexical_normalize(path));
    }

    let mut suffix = PathBuf::new();
    let mut probe = path;
    while !probe.exists() {
        if let Some(name) = probe.file_name() {
            suffix = Path::new(name).join(suffix);
        }
        let Some(parent) = probe.parent() else {
            return lexical_normalize(path);
        };
        probe = parent;
    }

    let base = probe
        .canonicalize()
        .unwrap_or_else(|_| lexical_normalize(probe));
    lexical_normalize(&base.join(suffix))
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // Serialize against the crate-wide `shell_env::CWD_LOCK` (issue #1401),
    // not a module-local mutex: cwd is process-global, so a local lock here
    // does not stop these tests from racing a concurrently-running
    // cwd-mutating test in another module (e.g. `cli/mod.rs`, `cli/cb.rs`).
    use crate::cli::shell_env::CWD_LOCK;
    use crate::issues::types::td_phase;

    fn write_project_config(root: &Path) {
        fs::create_dir_all(root.join(".aw")).unwrap();
        fs::write(
            root.join("aw.toml"),
            r#"
[[projects]]
name = "demo"
path = "projects/demo"
td_path = "projects/demo/tech-design"
cap_path = "projects/demo/CAPABILITIES.md"

[[projects.workspaces]]
paths = ["libs/demo/**"]
"#,
        )
        .unwrap();
    }

    #[test]
    fn guard_on_installs_codex_and_claude_handlers_idempotently() {
        let tmp = TempDir::new().unwrap();
        write_project_config(tmp.path());
        fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        fs::write(
            tmp.path().join(".claude/settings.json"),
            r#"{"statusLine":{"type":"command","command":"status.sh"},"hooks":{"PreToolUse":[{"matcher":"Edit|Write|MultiEdit|NotebookEdit","hooks":[]}]}}"#,
        )
        .unwrap();

        let mut first = install_guard_hooks(tmp.path(), "demo", GuardAgent::Codex).unwrap();
        first.extend(install_guard_hooks(tmp.path(), "demo", GuardAgent::Claude).unwrap());
        let mut second = install_guard_hooks(tmp.path(), "demo", GuardAgent::Codex).unwrap();
        second.extend(install_guard_hooks(tmp.path(), "demo", GuardAgent::Claude).unwrap());
        assert!(first.iter().all(|change| change.changed));
        assert!(second.iter().all(|change| !change.changed));

        let codex = fs::read_to_string(tmp.path().join(CODEX_HOOKS_REL)).unwrap();
        assert!(codex.contains("aw guard pretool --agent codex --project demo"));
        let claude = fs::read_to_string(tmp.path().join(CLAUDE_SETTINGS_REL)).unwrap();
        assert!(claude.contains("status.sh"));
        assert!(claude.contains("aw guard pretool --agent claude --project demo"));
    }

    #[test]
    fn guard_off_removes_only_aw_guard_handlers() {
        let tmp = TempDir::new().unwrap();
        write_project_config(tmp.path());
        install_guard_hooks(tmp.path(), "demo", GuardAgent::Codex).unwrap();
        install_guard_hooks(tmp.path(), "demo", GuardAgent::Claude).unwrap();

        let mut changes = remove_guard_hooks(tmp.path(), "demo", GuardAgent::Codex).unwrap();
        changes.extend(remove_guard_hooks(tmp.path(), "demo", GuardAgent::Claude).unwrap());
        assert!(changes.iter().all(|change| change.changed));
        let codex = fs::read_to_string(tmp.path().join(CODEX_HOOKS_REL)).unwrap();
        assert!(!codex.contains("aw guard pretool"));
        let claude = fs::read_to_string(tmp.path().join(CLAUDE_SETTINGS_REL)).unwrap();
        assert!(!claude.contains("aw guard pretool"));
    }

    #[test]
    fn agy_hook_lifecycle_preserves_unrelated_global_hooks() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("hooks.json");
        fs::write(
            &path,
            r#"{"cap-agent-guard":{"PreToolUse":[{"matcher":"run_command","hooks":[{"command":"cap hook agy"}]}]}}"#,
        )
        .unwrap();
        let command = "aw guard pretool --agent agy --project demo";

        assert!(upsert_agy_guard_hook_at(&path, command).unwrap());
        assert!(!upsert_agy_guard_hook_at(&path, command).unwrap());
        let installed = fs::read_to_string(&path).unwrap();
        assert!(installed.contains("cap hook agy"));
        assert!(installed.contains(command));

        assert!(remove_agy_guard_hook_at(&path, "demo").unwrap());
        let removed = fs::read_to_string(&path).unwrap();
        assert!(removed.contains("cap hook agy"));
        assert!(!removed.contains("--agent agy --project demo"));
    }

    #[test]
    fn agy_direct_mutation_parser_is_explicit_and_conservative() {
        assert_eq!(
            parse_agy_direct_mutation_targets(
                "printf x > projects/demo/src/lib.rs && touch projects/demo/src/new.rs projects/demo/src/also-new.rs && printf y >>projects/demo/src/append.rs && tee projects/demo/src/tee-a.rs projects/demo/src/tee-b.rs && rm projects/demo/src/remove-a.rs projects/demo/src/remove-b.rs && rg needle ."
            ),
            vec![
                "projects/demo/src/also-new.rs".to_string(),
                "projects/demo/src/append.rs".to_string(),
                "projects/demo/src/lib.rs".to_string(),
                "projects/demo/src/new.rs".to_string(),
                "projects/demo/src/remove-a.rs".to_string(),
                "projects/demo/src/remove-b.rs".to_string(),
                "projects/demo/src/tee-a.rs".to_string(),
                "projects/demo/src/tee-b.rs".to_string(),
            ]
        );
        assert!(parse_agy_direct_mutation_targets("cargo test -p demo").is_empty());
    }

    #[test]
    fn agy_hook_output_is_always_a_decision_envelope() {
        assert_eq!(agy_allow_output(None), json!({ "decision": "allow" }));
        assert_eq!(
            agy_allow_output(Some("sanctioned".to_string())),
            json!({ "decision": "allow", "reason": "sanctioned" })
        );
        assert_eq!(
            agy_deny_output("direct edit denied"),
            json!({ "decision": "deny", "reason": "direct edit denied" })
        );
    }

    fn init_git_repo(root: &Path) {
        for args in [
            ["init", "--initial-branch=main"].as_slice(),
            ["config", "user.email", "test@example.com"].as_slice(),
            ["config", "user.name", "Test"].as_slice(),
            ["commit", "--allow-empty", "-m", "init", "-q"].as_slice(),
        ] {
            let output = Command::new("git")
                .args(args)
                .current_dir(root)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "git {:?}: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[test]
    fn persistent_guard_policy_commit_is_scoped_to_hook_files() {
        let tmp = TempDir::new().unwrap();
        init_git_repo(tmp.path());
        fs::write(tmp.path().join("unrelated.txt"), "leave me dirty\n").unwrap();
        let changes = install_guard_hooks(tmp.path(), "demo", GuardAgent::Codex).unwrap();

        let commit = commit_guard_policy(tmp.path(), "enable", "demo", &changes)
            .unwrap()
            .expect("guard change should commit");
        assert!(!commit.is_empty());
        let files = Command::new("git")
            .args(["show", "--format=", "--name-only", "HEAD"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        assert!(files.status.success());
        assert_eq!(
            String::from_utf8_lossy(&files.stdout).trim(),
            CODEX_HOOKS_REL
        );
        let status = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        assert!(String::from_utf8_lossy(&status.stdout).contains("unrelated.txt"));
    }

    #[test]
    fn persistent_guard_toggle_refuses_preexisting_policy_drift() {
        let tmp = TempDir::new().unwrap();
        init_git_repo(tmp.path());
        fs::create_dir_all(tmp.path().join(".codex")).unwrap();
        fs::write(tmp.path().join(CODEX_HOOKS_REL), "{}\n").unwrap();

        let error = ensure_tracked_guard_paths_clean(tmp.path(), GuardAgent::Codex)
            .expect_err("dirty policy config must be preserved instead of committed");
        assert!(error.to_string().contains("tracked policy files"));
        assert!(error.to_string().contains(CODEX_HOOKS_REL));
    }

    #[test]
    fn local_bypass_is_worktree_local_and_expires() {
        let tmp = TempDir::new().unwrap();
        init_git_repo(tmp.path());
        let path = local_bypass_path(tmp.path(), "demo").unwrap();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            serde_json::to_string(&json!({
                "schema_version": 1,
                "project": "demo",
                "expires_at_unix": unix_now().unwrap() + 60,
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(local_bypass_is_active(tmp.path(), "demo").unwrap());

        fs::write(
            &path,
            serde_json::to_string(&json!({
                "schema_version": 1,
                "project": "demo",
                "expires_at_unix": 0,
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(!local_bypass_is_active(tmp.path(), "demo").unwrap());
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn pretool_allows_current_session_callback_after_guard_off() {
        let _guard = CWD_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tmp = TempDir::new().unwrap();
        init_git_repo(tmp.path());
        write_project_config(tmp.path());
        fs::create_dir_all(tmp.path().join("projects/demo/src")).unwrap();
        install_guard_hooks(tmp.path(), "demo", GuardAgent::Codex).unwrap();
        assert!(
            guard_handler_is_active(tmp.path(), "demo", GuardAgent::Codex).unwrap(),
            "{}",
            fs::read_to_string(tmp.path().join(CODEX_HOOKS_REL)).unwrap()
        );

        let payload = json!({
            "tool_name": "apply_patch",
            "tool_input": {
                "command": "*** Begin Patch\n*** Add File: projects/demo/src/new.rs\n+pub fn demo() {}\n*** End Patch\n",
            },
        });
        let previous = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        let enabled =
            decide_active_pretool_payload(tmp.path(), "demo", GuardAgent::Codex, &payload)
                .await
                .unwrap();
        assert!(
            matches!(enabled, GuardDecision::Deny { .. }),
            "expected the live handler to deny before off, got {enabled:?}"
        );

        remove_guard_hooks(tmp.path(), "demo", GuardAgent::Codex).unwrap();
        let disabled =
            decide_active_pretool_payload(tmp.path(), "demo", GuardAgent::Codex, &payload)
                .await
                .unwrap();
        std::env::set_current_dir(previous).unwrap();
        assert_eq!(disabled, GuardDecision::Allow);
    }

    #[tokio::test]
    async fn pretool_denies_claude_write_inside_project_path() {
        let _guard = CWD_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tmp = TempDir::new().unwrap();
        write_project_config(tmp.path());
        fs::create_dir_all(tmp.path().join("projects/demo/src")).unwrap();
        let previous = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();

        let payload = json!({
            "tool_name": "Write",
            "tool_input": {
                "file_path": tmp.path().join("projects/demo/src/lib.rs").to_string_lossy(),
                "content": "fn main() {}",
            },
        });
        let decision = decide_pretool_payload(tmp.path(), "demo", GuardAgent::Claude, &payload)
            .await
            .unwrap();
        std::env::set_current_dir(previous).unwrap();

        assert!(matches!(decision, GuardDecision::Deny { .. }));
    }

    #[tokio::test]
    async fn pretool_allows_claude_write_outside_project_path() {
        let _guard = CWD_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tmp = TempDir::new().unwrap();
        write_project_config(tmp.path());
        fs::create_dir_all(tmp.path().join("projects/other/src")).unwrap();
        let previous = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();

        let payload = json!({
            "tool_name": "Write",
            "tool_input": {
                "file_path": tmp.path().join("projects/other/src/lib.rs").to_string_lossy(),
                "content": "fn main() {}",
            },
        });
        let decision = decide_pretool_payload(tmp.path(), "demo", GuardAgent::Claude, &payload)
            .await
            .unwrap();
        std::env::set_current_dir(previous).unwrap();

        assert_eq!(decision, GuardDecision::Allow);
    }

    #[tokio::test]
    async fn pretool_denies_codex_apply_patch_inside_project_path() {
        let _guard = CWD_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tmp = TempDir::new().unwrap();
        write_project_config(tmp.path());
        fs::create_dir_all(tmp.path().join("projects/demo/src")).unwrap();
        let previous = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();

        let payload = json!({
            "tool_name": "apply_patch",
            "tool_input": {
                "command": "*** Begin Patch\n*** Add File: projects/demo/src/new.rs\n+pub fn demo() {}\n*** End Patch\n",
            },
        });
        let decision = decide_pretool_payload(tmp.path(), "demo", GuardAgent::Codex, &payload)
            .await
            .unwrap();
        std::env::set_current_dir(previous).unwrap();

        assert!(matches!(decision, GuardDecision::Deny { .. }));
    }

    #[tokio::test]
    async fn pretool_denies_agy_explicit_write_and_allows_non_mutating_command() {
        let _guard = CWD_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tmp = TempDir::new().unwrap();
        write_project_config(tmp.path());
        fs::create_dir_all(tmp.path().join("projects/demo/src")).unwrap();
        let previous = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();

        let write_payload = json!({
            "toolCall": {
                "name": "run_command",
                "args": { "CommandLine": "printf x > projects/demo/src/lib.rs" },
            },
        });
        let denied = decide_pretool_payload(tmp.path(), "demo", GuardAgent::Agy, &write_payload)
            .await
            .unwrap();
        assert!(matches!(denied, GuardDecision::Deny { .. }));

        let inspect_payload = json!({
            "toolCall": {
                "name": "run_command",
                "args": { "CommandLine": "cargo test -p demo" },
            },
        });
        let allowed = decide_pretool_payload(tmp.path(), "demo", GuardAgent::Agy, &inspect_payload)
            .await
            .unwrap();
        std::env::set_current_dir(previous).unwrap();

        assert_eq!(allowed, GuardDecision::Allow);
    }

    #[tokio::test]
    async fn pretool_ignores_bash_payload_without_direct_edit_target() {
        let tmp = TempDir::new().unwrap();
        write_project_config(tmp.path());
        let payload = json!({
            "tool_name": "Bash",
            "tool_input": {
                "command": "sed -i '' 's/a/b/' projects/demo/src/lib.rs",
            },
        });
        let decision = decide_pretool_payload(tmp.path(), "demo", GuardAgent::All, &payload)
            .await
            .unwrap();

        assert_eq!(decision, GuardDecision::Allow);
    }

    #[test]
    fn parse_apply_patch_targets_collects_file_markers() {
        let targets = parse_apply_patch_targets(
            "*** Begin Patch\n*** Update File: projects/demo/src/lib.rs\n*** Move to: projects/demo/src/main.rs\n*** End Patch\n",
        );
        assert_eq!(
            targets,
            vec![
                "projects/demo/src/lib.rs".to_string(),
                "projects/demo/src/main.rs".to_string()
            ]
        );
    }

    // -----------------------------------------------------------------
    // #1429: pretool consults the #1428 sanctioned-path resolver.
    //
    // Uses the `github` read-through cache (`crate::issues::
    // remote_read_cache_backend`) rather than the `local`-fixture escape
    // hatch (`AW_FIXTURE_LOCAL_BACKEND`): that flag is a process-global env
    // var guarded elsewhere (`issues::mod::resolve_tests`) and touching it
    // here would race concurrently-running tests in that module under
    // `cargo test --lib`. The read-through cache dir is scoped by a unique
    // per-test repo name instead, so no shared mutable state is touched.
    // -----------------------------------------------------------------

    const HAND_WRITTEN_TD: &str = "\
# TD

## Changes
```yaml
changes:
  - path: src/bundler/dts.rs
    section: source
    impl_mode: hand-written
```
";

    /// Unique-per-test repo name for the read-through cache, so parallel
    /// test runs (and other concurrently-running `aw` processes on this
    /// machine) never collide on `/tmp/aw/issues/<host>-<repo>/<kind>`.
    fn unique_test_repo(case: &str) -> String {
        format!("aw-guard-pretool-test/{case}-{}", uuid::Uuid::new_v4())
    }

    fn write_project_config_with_github_backend(root: &Path, repo: &str) {
        fs::create_dir_all(root.join(".aw")).unwrap();
        fs::write(
            root.join("aw.toml"),
            format!(
                "[[projects]]\n\
                 name = \"demo\"\n\
                 path = \"projects/demo\"\n\
                 td_path = \"projects/demo/tech-design\"\n\
                 cap_path = \"projects/demo/CAPABILITIES.md\"\n\n\
                 [agentic_workflow.issue_platform]\n\
                 type = \"github\"\n\
                 repo = \"{repo}\"\n"
            ),
        )
        .unwrap();
    }

    fn write_project_td(root: &Path, rel: &str, content: &str) {
        let path = root.join("projects/demo").join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn sanctioned_wi_issue(
        github_id: u64,
        phase: &str,
        state: crate::issues::IssueState,
    ) -> crate::issues::Issue {
        crate::issues::Issue {
            issue_type: crate::issues::IssueType::Enhancement,
            title: format!("wi {github_id}"),
            state,
            id: None,
            github_id: Some(github_id),
            gitlab_id: None,
            url: None,
            author: None,
            labels: Vec::new(),
            created_at: None,
            updated_at: None,
            slug: github_id.to_string(),
            body: String::new(),
            related: Vec::new(),
            implements: vec!["tech-design/dts.md".to_string()],
            phase: Some(phase.to_string()),
            branch: None,
            target_branch: None,
            git_workflow: None,
            change_id: None,
            iteration: None,
            current_task_id: None,
            impl_spec_phase: None,
            task_revisions: None,
            revision_counts: None,
            last_action: None,
            session_id: None,
            validation_errors: Vec::new(),
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        }
    }

    /// Cleans up the shared `/tmp/aw/issues/<host>-<repo>` cache dir on
    /// drop, so these tests don't leak fixture state onto the real machine.
    struct CacheCleanup(PathBuf);
    impl Drop for CacheCleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(self.0.parent().unwrap_or(&self.0));
        }
    }

    async fn seed_cached_issue(repo: &str, issue: &crate::issues::Issue) -> CacheCleanup {
        use crate::issues::IssueBackend;
        let cache_dir = crate::issues::remote_read_cache_dir("github", Some(repo), None);
        let cleanup = CacheCleanup(cache_dir);
        let cache = crate::issues::remote_read_cache_backend("github", Some(repo), None);
        cache.write(issue).await.unwrap();
        cleanup
    }

    /// (a) The #1269 repro shape: a declared hand-written path, WI at
    /// `cb_genned` → allowed, with the envelope-worthy decision naming the
    /// sanctioning WI id, TD path, and phase.
    #[tokio::test]
    async fn pretool_allows_sanctioned_handwrite_path_and_names_wi_td_phase() {
        let _guard = CWD_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tmp = TempDir::new().unwrap();
        let repo = unique_test_repo("allow");
        write_project_config_with_github_backend(tmp.path(), &repo);
        write_project_td(tmp.path(), "tech-design/dts.md", HAND_WRITTEN_TD);
        fs::create_dir_all(tmp.path().join("projects/demo/src/bundler")).unwrap();
        let _cleanup = seed_cached_issue(
            &repo,
            &sanctioned_wi_issue(937, td_phase::CB_GENNED, crate::issues::IssueState::Open),
        )
        .await;

        let previous = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        let payload = json!({
            "tool_name": "Write",
            "tool_input": {
                "file_path": tmp.path().join("projects/demo/src/bundler/dts.rs").to_string_lossy(),
                "content": "fn demo() {}",
            },
        });
        let decision = decide_pretool_payload(tmp.path(), "demo", GuardAgent::Claude, &payload)
            .await
            .unwrap();
        std::env::set_current_dir(previous).unwrap();

        match decision {
            GuardDecision::AllowSanctioned { path, reason } => {
                assert_eq!(path, "projects/demo/src/bundler/dts.rs");
                assert_eq!(reason.wi_id, "937");
                assert_eq!(reason.td_path, "tech-design/dts.md");
                assert_eq!(reason.phase, td_phase::CB_GENNED);
            }
            other => panic!("expected AllowSanctioned, got {other:?}"),
        }
    }

    /// (b) An undeclared sibling path in the same project, same session, is
    /// still denied byte-for-byte (same reason-string shape as the
    /// pre-#1429 unconditional deny).
    #[tokio::test]
    async fn pretool_denies_undeclared_sibling_path_in_same_session() {
        let _guard = CWD_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tmp = TempDir::new().unwrap();
        let repo = unique_test_repo("sibling-deny");
        write_project_config_with_github_backend(tmp.path(), &repo);
        write_project_td(tmp.path(), "tech-design/dts.md", HAND_WRITTEN_TD);
        fs::create_dir_all(tmp.path().join("projects/demo/src/bundler")).unwrap();
        let _cleanup = seed_cached_issue(
            &repo,
            &sanctioned_wi_issue(937, td_phase::CB_GENNED, crate::issues::IssueState::Open),
        )
        .await;

        let previous = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        // Sibling file in the same project, not declared hand-written in
        // the TD's `## Changes` block.
        let payload = json!({
            "tool_name": "Write",
            "tool_input": {
                "file_path": tmp.path().join("projects/demo/src/bundler/sibling.rs").to_string_lossy(),
                "content": "fn sibling() {}",
            },
        });
        let decision = decide_pretool_payload(tmp.path(), "demo", GuardAgent::Claude, &payload)
            .await
            .unwrap();
        std::env::set_current_dir(previous).unwrap();

        assert!(matches!(decision, GuardDecision::Deny { .. }));
    }

    /// (c) Once the WI has advanced past code-check (closed / terminal),
    /// the previously sanctioned path is denied again.
    #[tokio::test]
    async fn pretool_denies_sanctioned_path_after_wi_closes_past_code_check() {
        let _guard = CWD_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tmp = TempDir::new().unwrap();
        let repo = unique_test_repo("post-terminal-deny");
        write_project_config_with_github_backend(tmp.path(), &repo);
        write_project_td(tmp.path(), "tech-design/dts.md", HAND_WRITTEN_TD);
        fs::create_dir_all(tmp.path().join("projects/demo/src/bundler")).unwrap();
        // WI closed (terminal, past code-check) — no longer eligible.
        let _cleanup = seed_cached_issue(
            &repo,
            &sanctioned_wi_issue(937, td_phase::TD_MERGED, crate::issues::IssueState::Closed),
        )
        .await;

        let previous = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        let payload = json!({
            "tool_name": "Write",
            "tool_input": {
                "file_path": tmp.path().join("projects/demo/src/bundler/dts.rs").to_string_lossy(),
                "content": "fn demo() {}",
            },
        });
        let decision = decide_pretool_payload(tmp.path(), "demo", GuardAgent::Claude, &payload)
            .await
            .unwrap();
        std::env::set_current_dir(previous).unwrap();

        assert!(matches!(decision, GuardDecision::Deny { .. }));
    }

    /// (d) Resolver error / empty cache (no `aw wi`/`aw td` traffic ever
    /// touched this project's read-through cache) → deny, fail-closed, no
    /// panic.
    #[tokio::test]
    async fn pretool_denies_on_empty_resolver_cache_fail_closed_no_panic() {
        let _guard = CWD_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let tmp = TempDir::new().unwrap();
        let repo = unique_test_repo("empty-cache-deny");
        write_project_config_with_github_backend(tmp.path(), &repo);
        write_project_td(tmp.path(), "tech-design/dts.md", HAND_WRITTEN_TD);
        fs::create_dir_all(tmp.path().join("projects/demo/src/bundler")).unwrap();
        // No issue ever written to the read-through cache for `repo`.

        let previous = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        let payload = json!({
            "tool_name": "Write",
            "tool_input": {
                "file_path": tmp.path().join("projects/demo/src/bundler/dts.rs").to_string_lossy(),
                "content": "fn demo() {}",
            },
        });
        let decision = decide_pretool_payload(tmp.path(), "demo", GuardAgent::Claude, &payload)
            .await
            .unwrap();
        std::env::set_current_dir(previous).unwrap();

        assert!(matches!(decision, GuardDecision::Deny { .. }));
    }
}
// CODEGEN-END
