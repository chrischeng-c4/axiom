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

use crate::cli::guard_sanction::{self, SanctionReason};
use crate::services::path_scope::{self, AllowedScope};
use crate::services::project_registry::{self, ProjectConfigRow};

const CODEX_HOOKS_REL: &str = ".codex/hooks.json";
const CLAUDE_SETTINGS_REL: &str = ".claude/settings.json";
const CODEX_MATCHER: &str = "Edit|Write|apply_patch";
const CLAUDE_MATCHER: &str = "Edit|Write|MultiEdit|NotebookEdit";

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
    /// Enable Codex/Claude direct edit/create tool guards for a project scope.
    On(GuardToggleArgs),
    /// Disable AW-managed direct edit/create tool guards for a project scope.
    Off(GuardToggleArgs),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#source
pub enum GuardAgent {
    All,
    Codex,
    Claude,
}

impl GuardAgent {
    fn includes_codex(self) -> bool {
        matches!(self, GuardAgent::All | GuardAgent::Codex)
    }

    fn includes_claude(self) -> bool {
        matches!(self, GuardAgent::All | GuardAgent::Claude)
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
        GuardCommand::Pretool(args) => run_pretool(args).await,
    }
}

fn run_on(args: GuardToggleArgs) -> Result<()> {
    let root = crate::find_project_root()?;
    let row = project_registry::resolve_project_config_row(&root, &args.project)?;
    let changes = install_guard_hooks(&root, &row.name, args.agent)?;
    emit_toggle_summary("enabled", &row.name, &changes, args.json);
    Ok(())
}

fn run_off(args: GuardToggleArgs) -> Result<()> {
    let root = crate::find_project_root()?;
    let row = project_registry::resolve_project_config_row(&root, &args.project)?;
    let changes = remove_guard_hooks(&root, &row.name, args.agent)?;
    emit_toggle_summary("disabled", &row.name, &changes, args.json);
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
            return Ok(());
        }
    };

    let root = match crate::find_project_root() {
        Ok(root) => root,
        Err(err) => {
            eprintln!("aw guard: fail-open: cannot resolve project root: {err:#}");
            return Ok(());
        }
    };
    match decide_pretool_payload(&root, &args.project, args.agent, &payload).await {
        Ok(GuardDecision::Allow) => {}
        Ok(GuardDecision::AllowSanctioned { path, reason }) => {
            println!(
                "{}",
                json!({
                    "hookSpecificOutput": {
                        "hookEventName": "PreToolUse",
                        "permissionDecision": "allow",
                        "permissionDecisionReason": format!(
                            "AW guard allows direct hand-written edit at `{}` — sanctioned by WI #{} (TD `{}`, phase `{}`).",
                            path, reason.wi_id, reason.td_path, reason.phase
                        ),
                    }
                })
            );
        }
        Ok(GuardDecision::Deny { reason }) => {
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
        Err(err) => {
            eprintln!("aw guard: fail-open: {err:#}");
        }
    }
    Ok(())
}

fn emit_toggle_summary(
    action: &str,
    project: &str,
    changes: &[GuardHookChange],
    json_output: bool,
) {
    if json_output {
        println!(
            "{}",
            json!({
                "action": action,
                "project": project,
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
    Ok(changes)
}

fn guard_command(agent: &str, project: &str) -> String {
    format!("aw guard pretool --agent {agent} --project {project}")
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
                            "AW guard blocks direct edit/create for project `{}` at `{}`. Use the AW CLI lifecycle, or explicitly run `aw guard off --project {}` before a manual bypass.",
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

    targets
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

        let first = install_guard_hooks(tmp.path(), "demo", GuardAgent::All).unwrap();
        let second = install_guard_hooks(tmp.path(), "demo", GuardAgent::All).unwrap();
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
        install_guard_hooks(tmp.path(), "demo", GuardAgent::All).unwrap();

        let changes = remove_guard_hooks(tmp.path(), "demo", GuardAgent::All).unwrap();
        assert!(changes.iter().all(|change| change.changed));
        let codex = fs::read_to_string(tmp.path().join(CODEX_HOOKS_REL)).unwrap();
        assert!(!codex.contains("aw guard pretool"));
        let claude = fs::read_to_string(tmp.path().join(CLAUDE_SETTINGS_REL)).unwrap();
        assert!(!claude.contains("aw guard pretool"));
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
