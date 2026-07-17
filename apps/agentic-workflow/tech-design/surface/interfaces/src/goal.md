---
id: projects-score-src-goal-rs
fill_sections: [overview, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: aw-goal-verifiable-condition-loop
    claim: aw-goal-verifiable-condition-loop
    coverage: full
    rationale: "CLI-owned verifiable-condition loop (`aw goal`) for bounded work outside the WI/TD/EC tracked lifecycle."
---

# Standardized apps/agentic-workflow/src/cli/goal.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/cli/goal.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GoalArgs` | apps/agentic-workflow/src/cli/goal.rs | struct | pub | 54 |  |
| `GoalCommand` | apps/agentic-workflow/src/cli/goal.rs | enum | pub | 60 |  |
| `GoalCapabilityArgs` | apps/agentic-workflow/src/cli/goal.rs | struct | pub | 86 |  |
| `GoalSetArgs` | apps/agentic-workflow/src/cli/goal.rs | struct | pub | 127 |  |
| `GoalIdArgs` | apps/agentic-workflow/src/cli/goal.rs | struct | pub | 147 |  |
| `GoalState` | apps/agentic-workflow/src/cli/goal.rs | struct | pub | 154 |  |
| `run` | apps/agentic-workflow/src/cli/goal.rs | function | pub | 198 | run(args: GoalArgs) -> Result<()> |

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=apps/agentic-workflow/src/cli/goal.rs -->
```rust
// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/interfaces/src/goal.md#source
// CODEGEN-BEGIN
//! `aw goal` — CLI-owned verifiable-condition loop for ad-hoc,
//! WI-lifecycle-external work (issue #1897).
//!
//! State is workspace-scoped ephemeral JSON under
//! `/tmp/aw/workspaces/<workspace>/goals/{goal_id}.json` (never a repo-root
//! file) recording a prose intent plus one or more machine-runnable gate
//! commands. `aw goal check` executes every gate with a bounded
//! per-command timeout and reports deterministically:
//!
//!   - all gates green: clears the state file and emits a terminal `done`
//!     envelope (`completion.workflow_complete = true`).
//!   - any gate red: keeps the state file, reports the failing gate's
//!     output tail, and emits `next.command = aw goal check <id>`.
//!   - budget (`--budget-checks`/`--budget-minutes`) or the hard 24h
//!     expiry ceiling since `created_at` is exhausted: clears the state
//!     file and emits a terminal `gave_up` status without dropping the
//!     recorded intent from the report.
//!
//! This is explicitly not a daemon and does not replace `aw wi run`/`aw
//! capability run` completion semantics: gates are executable commands
//! only (no LLM-judged/prose-only conditions), and goal state never
//! crosses sessions/workspaces.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::cli::shell_env::protected_shell_command;
use crate::shared::workspace::{goal_state_path, goals_path};

/// Hard expiry ceiling since `created_at`, regardless of any configured
/// budget: a stale goal self-terminates the next time it is checked.
const MAX_GOAL_AGE_HOURS: i64 = 24;

/// Default per-gate subprocess timeout. Gates are expected to be bounded
/// commands (a single `cargo test`/`cargo build`/lint invocation); a gate
/// that runs longer than this is killed and reported as a failing gate
/// rather than left to hang the check loop.
const DEFAULT_GATE_TIMEOUT_SECS: u64 = 600;

/// Number of trailing output lines kept in a gate's report tail.
const OUTPUT_TAIL_LINES: usize = 40;

#[derive(Debug, Args, Clone)]
pub struct GoalArgs {
    #[command(subcommand)]
    pub command: GoalCommand,
}

#[derive(Debug, Subcommand, Clone)]
pub enum GoalCommand {
    /// Record a prose intent plus one or more machine-runnable gate commands.
    Set(GoalSetArgs),
    /// Execute a goal's gates and report deterministically.
    Check(GoalIdArgs),
    /// Show one goal's recorded intent, gates, and status.
    Show(GoalIdArgs),
    /// List every goal recorded for the current workspace.
    List,
    /// Discard a goal's recorded state.
    Clear(GoalIdArgs),
    /// Drive one work item to terminal (#1899: unified re-home of the
    /// retired `aw wi run <id>`).
    Wi(GoalWiArgs),
    /// Drive a capability work-root, or the whole project end to end when
    /// no capability id is given (#1899: unified re-home of the retired
    /// `aw capability run [<cap-id>] --project <p>`).
    Capability(GoalCapabilityArgs),
    /// Tracker-driven drain of every open work item for a project, one WI
    /// per tick via the same shared engine `aw goal wi <id>` uses; blocked
    /// WIs are parked (not surfaced) so the drain continues (#1899 R7).
    Backlog(GoalBacklogArgs),
}

/// `aw goal wi <id>` args (#1899). Deliberately a standalone struct (not a
/// re-export of `crate::cli::issues::WiRunArgs`): `GoalCommand` needs its
/// variant payloads to be `Clone`, and keeping the two structs distinct
/// means the now-retired `aw wi run <id>` clap leaf (`src/cli/issues.rs`)
/// can keep parsing for its `emit_retired_verb_redirect` envelope without
/// this canonical form depending on it. Field shape mirrors the old
/// `WiRunArgs` exactly so both verbs' argument parsing lines up.
#[derive(Debug, Args, Clone)]
pub struct GoalWiArgs {
    /// Work-item identifier (slug for local, numeric for github).
    pub id: String,

    /// Emit human-readable text instead of the default agent JSON envelope.
    #[arg(long)]
    pub human: bool,

    /// Pretty-print the default JSON envelope for debugging.
    #[arg(long)]
    pub pretty: bool,

    /// Generate a /goal-ready prompt for this work item instead of the normal run envelope.
    #[arg(long)]
    pub goal: bool,
}

/// `aw goal capability [<capability-id>] --project <project>` args (#1899).
/// Mirrors the field subset of `crate::cli::capability::CapabilityRunArgs`
/// that the project-wide rollup engine consumes, plus a mandatory
/// `--project` since `aw goal` has no project-scoped parent command to
/// inherit it from.
#[derive(Debug, Args, Clone)]
pub struct GoalCapabilityArgs {
    /// Capability id to drive via the shared root-driven workflow runner.
    /// Omit to run the project-wide capability completion loop end to end.
    pub capability_id: Option<String>,

    /// Project to drive.
    #[arg(long)]
    pub project: String,

    /// Capability map path override.
    #[arg(long = "cap-path")]
    pub cap_path: Option<PathBuf>,

    /// Require bounded, non-interactive execution. Used by the
    /// project-wide rollup form (no capability id); ignored for a single
    /// capability id, which is always one bounded tick.
    #[arg(long)]
    pub non_interactive: bool,

    /// Maximum bounded ticks to run (project-wide rollup form only).
    #[arg(long, default_value_t = 1)]
    pub max_ticks: usize,

    /// Include issue inventory when computing next action routing.
    #[arg(long = "include-issue-inventory")]
    pub include_issue_inventory: bool,

    /// Skip issue inventory for README/TD-only bounded ticks.
    #[arg(long = "skip-issue-inventory")]
    pub skip_issue_inventory: bool,

    /// Emit human-readable text instead of the default agent JSON envelope.
    #[arg(long)]
    pub human: bool,

    /// Pretty-print the default JSON envelope for debugging.
    #[arg(long)]
    pub pretty: bool,
}

/// `aw goal backlog --project <project>` args (#1899 R7).
#[derive(Debug, Args, Clone)]
pub struct GoalBacklogArgs {
    /// Project whose open work-item backlog to drain.
    #[arg(long)]
    pub project: String,

    /// Emit human-readable text instead of the default agent JSON envelope.
    #[arg(long)]
    pub human: bool,

    /// Pretty-print the default JSON envelope for debugging.
    #[arg(long)]
    pub pretty: bool,
}

#[derive(Debug, Args, Clone)]
pub struct GoalSetArgs {
    /// Prose description of the intent (what "done" means to a human).
    pub intent: Vec<String>,

    /// A machine-runnable shell gate command; repeat for multiple gates.
    /// At least one is required — a prose-only intent has no
    /// machine-verifiable condition.
    #[arg(long = "gate", value_name = "COMMAND")]
    pub gates: Vec<String>,

    /// Give up after this many `aw goal check` attempts.
    #[arg(long = "budget-checks", value_name = "N")]
    pub budget_checks: Option<u32>,

    /// Give up after this many minutes since the goal was recorded.
    #[arg(long = "budget-minutes", value_name = "N")]
    pub budget_minutes: Option<u32>,
}

#[derive(Debug, Args, Clone)]
pub struct GoalIdArgs {
    /// Goal id. Omit when the workspace has exactly one recorded goal.
    pub id: Option<String>,
}

/// Workspace-scoped `aw goal` state (issue #1897).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalState {
    pub id: String,
    pub intent: String,
    pub gates: Vec<String>,
    pub created_at: String,
    #[serde(default)]
    pub budget_checks: Option<u32>,
    #[serde(default)]
    pub budget_minutes: Option<u32>,
    #[serde(default)]
    pub checks_run: u32,
}

/// Outcome of running one gate command.
#[derive(Debug, Clone)]
struct GateOutcome {
    command: String,
    success: bool,
    output_tail: String,
    timed_out: bool,
}

/// Outcome of one `aw goal check` invocation.
#[derive(Debug, Clone)]
struct CheckOutcome {
    state: GoalState,
    status: CheckStatus,
    gates: Vec<GateOutcome>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckStatus {
    Done,
    Blocked,
    GaveUp,
}

/// Run `aw goal <command>`.
///
/// The lifecycle root types (`Wi`, `Capability`) don't touch the ad-hoc
/// goal-state engine below at all -- they delegate straight to the shared
/// root-driven workflow runner (`crate::cli::run` / `crate::cli::capability`)
/// that the now-retired `aw wi run` / `aw capability run` verbs used to
/// reach directly (#1899). Async only because those two delegate targets
/// are.
pub async fn run(args: GoalArgs) -> Result<()> {
    match args.command {
        GoalCommand::Set(set_args) => {
            let project_root = crate::find_project_root()?;
            run_set(&project_root, set_args)
        }
        GoalCommand::Check(id_args) => {
            let project_root = crate::find_project_root()?;
            run_check(&project_root, id_args)
        }
        GoalCommand::Show(id_args) => {
            let project_root = crate::find_project_root()?;
            run_show(&project_root, id_args)
        }
        GoalCommand::List => {
            let project_root = crate::find_project_root()?;
            run_list(&project_root)
        }
        GoalCommand::Clear(id_args) => {
            let project_root = crate::find_project_root()?;
            run_clear(&project_root, id_args)
        }
        GoalCommand::Wi(wi_args) => run_goal_wi(wi_args).await,
        GoalCommand::Capability(cap_args) => run_goal_capability(cap_args).await,
        GoalCommand::Backlog(backlog_args) => run_goal_backlog(backlog_args).await,
    }
}

/// `aw goal wi <id>` -- thin re-home of `aw wi run <id>` (#1899 R1):
/// identical envelope semantics, same shared root loop.
async fn run_goal_wi(args: GoalWiArgs) -> Result<()> {
    crate::cli::run::run_wi_root(
        &args.id,
        crate::cli::run::RunPrintOptions {
            human: args.human,
            pretty: args.pretty,
            goal: args.goal,
        },
    )
    .await
}

/// `aw goal capability [<capability-id>] --project <project>` -- thin
/// re-home of the retired `aw capability run [<cap-id>] --project <p>`
/// (#1899 R1/R3): a capability id drives that one work-root through the
/// shared root loop; no id drives the project-wide bounded-tick rollup
/// (the same engine `aw capability run --project <p> --non-interactive`
/// used).
async fn run_goal_capability(args: GoalCapabilityArgs) -> Result<()> {
    let print = crate::cli::run::RunPrintOptions {
        human: args.human,
        pretty: args.pretty,
        goal: false,
    };
    match args.capability_id.as_deref() {
        Some(capability_id) => {
            crate::cli::run::run_capability_root(&args.project, capability_id, print).await
        }
        None => {
            let run_args = crate::cli::capability::CapabilityRunArgs {
                capability_id: None,
                cap_path: args.cap_path.clone(),
                non_interactive: args.non_interactive,
                max_ticks: args.max_ticks,
                include_issue_inventory: args.include_issue_inventory,
                skip_issue_inventory: args.skip_issue_inventory,
                json: false,
                human: args.human,
                pretty: args.pretty,
            };
            crate::cli::capability::run_capability_tick(&args.project, run_args).await
        }
    }
}

/// `aw goal backlog --project <project>` -- thin re-home of the
/// tracker-driven drain engine (#1899 R7).
async fn run_goal_backlog(args: GoalBacklogArgs) -> Result<()> {
    crate::cli::run::run_backlog_root(
        &args.project,
        crate::cli::run::RunPrintOptions {
            human: args.human,
            pretty: args.pretty,
            goal: false,
        },
    )
    .await
}

fn run_set(project_root: &Path, args: GoalSetArgs) -> Result<()> {
    let intent = args.intent.join(" ").trim().to_string();
    if intent.is_empty() {
        return emit_error(
            "aw goal set requires prose intent text describing what \"done\" means, \
             e.g. `aw goal set --gate \"cargo test -p agentic-workflow --lib chain\" \
             all chain tests pass`",
        );
    }
    let gates: Vec<String> = args
        .gates
        .into_iter()
        .map(|gate| gate.trim().to_string())
        .filter(|gate| !gate.is_empty())
        .collect();
    if gates.is_empty() {
        return emit_error(
            "aw goal set requires at least one --gate <command>: a prose-only intent \
             has no machine-verifiable condition. Pass the narrowest command that \
             proves the intent, e.g. --gate \"cargo test -p agentic-workflow --lib chain\"",
        );
    }

    let state = set_goal(
        project_root,
        intent,
        gates,
        args.budget_checks,
        args.budget_minutes,
    )?;

    let env = json!({
        "schema_version": "aw.cli.v1",
        "status": "recorded",
        "action": "goal_set",
        "message": format!(
            "goal {} recorded with {} gate(s)",
            state.id,
            state.gates.len()
        ),
        "goal": goal_summary_json(&state),
        "requires_hitl": false,
        "next": {
            "kind": "run_command",
            "command": format!("aw goal check {}", state.id),
            "reason": "run the recorded gates and report status",
            "requires_hitl": false,
            "payload_path": Value::Null,
        },
        "completion": {
            "root_complete": false,
            "workflow_complete": false,
            "requires_hitl": false,
            "criteria": Vec::<&str>::new(),
            "missing": ["gate verification"],
        },
    });
    println!("{}", serde_json::to_string(&env)?);
    Ok(())
}

fn run_check(project_root: &Path, id_args: GoalIdArgs) -> Result<()> {
    let id = resolve_goal_id(project_root, id_args.id.as_deref())?;
    let outcome = check_goal(project_root, &id)?;
    print_check_outcome(&outcome)
}

fn run_show(project_root: &Path, id_args: GoalIdArgs) -> Result<()> {
    let id = resolve_goal_id(project_root, id_args.id.as_deref())?;
    let state = load_goal(project_root, &id)?;

    let env = json!({
        "schema_version": "aw.cli.v1",
        "status": "recorded",
        "action": "goal_show",
        "message": format!("goal {} has {} gate(s)", state.id, state.gates.len()),
        "goal": goal_summary_json(&state),
        "requires_hitl": false,
        "next": {
            "kind": "run_command",
            "command": format!("aw goal check {}", state.id),
            "reason": "run the recorded gates and report status",
            "requires_hitl": false,
            "payload_path": Value::Null,
        },
        "completion": {
            "root_complete": false,
            "workflow_complete": false,
            "requires_hitl": false,
            "criteria": Vec::<&str>::new(),
            "missing": ["gate verification"],
        },
    });
    println!("{}", serde_json::to_string(&env)?);
    Ok(())
}

fn run_list(project_root: &Path) -> Result<()> {
    let ids = list_goal_ids(project_root)?;
    let mut goals = Vec::new();
    for id in &ids {
        if let Ok(state) = load_goal(project_root, id) {
            goals.push(goal_summary_json(&state));
        }
    }

    let env = json!({
        "schema_version": "aw.cli.v1",
        "status": "recorded",
        "action": "goal_list",
        "message": format!("{} goal(s) recorded for this workspace", goals.len()),
        "goals": goals,
        "requires_hitl": false,
        "next": {
            "kind": if goals.is_empty() { "run_command" } else { "done" },
            "command": if goals.is_empty() { Some("aw goal set --help".to_string()) } else { None },
            "reason": if goals.is_empty() {
                "no goals recorded yet"
            } else {
                "goals recorded; run `aw goal check <id>` on any of them"
            },
            "requires_hitl": false,
            "payload_path": Value::Null,
        },
        "completion": {
            "root_complete": true,
            "workflow_complete": true,
            "requires_hitl": false,
            "criteria": ["goal inventory listed"],
            "missing": Vec::<&str>::new(),
        },
    });
    println!("{}", serde_json::to_string(&env)?);
    Ok(())
}

fn run_clear(project_root: &Path, id_args: GoalIdArgs) -> Result<()> {
    let id = resolve_goal_id(project_root, id_args.id.as_deref())?;
    clear_goal(project_root, &id)?;

    let env = json!({
        "schema_version": "aw.cli.v1",
        "status": "done",
        "action": "goal_clear",
        "message": format!("goal {id} cleared"),
        "requires_hitl": false,
        "next": {
            "kind": "done",
            "command": Value::Null,
            "reason": "goal state discarded",
            "requires_hitl": false,
            "payload_path": Value::Null,
        },
        "completion": {
            "root_complete": true,
            "workflow_complete": true,
            "requires_hitl": false,
            "criteria": ["goal state discarded"],
            "missing": Vec::<&str>::new(),
        },
    });
    println!("{}", serde_json::to_string(&env)?);
    Ok(())
}

fn emit_error(message: &str) -> Result<()> {
    let env = json!({
        "schema_version": "aw.cli.v1",
        "status": "error",
        "action": "goal_error",
        "message": message,
        "requires_hitl": false,
        "next": {
            "kind": "none",
            "command": Value::Null,
            "reason": message,
            "requires_hitl": false,
            "payload_path": Value::Null,
        },
        "completion": {
            "root_complete": false,
            "workflow_complete": false,
            "requires_hitl": false,
            "criteria": Vec::<&str>::new(),
            "missing": [message],
        },
    });
    println!("{}", serde_json::to_string(&env)?);
    anyhow::bail!("{message}")
}

fn goal_summary_json(state: &GoalState) -> Value {
    json!({
        "id": state.id,
        "intent": state.intent,
        "gates": state.gates,
        "created_at": state.created_at,
        "budget_checks": state.budget_checks,
        "budget_minutes": state.budget_minutes,
        "checks_run": state.checks_run,
    })
}

fn print_check_outcome(outcome: &CheckOutcome) -> Result<()> {
    let state = &outcome.state;
    let gate_reports: Vec<Value> = outcome
        .gates
        .iter()
        .map(|gate| {
            json!({
                "command": gate.command,
                "success": gate.success,
                "timed_out": gate.timed_out,
                "output_tail": gate.output_tail,
            })
        })
        .collect();
    let failing = outcome.gates.iter().find(|gate| !gate.success);

    let (status, action, message, workflow_complete, next_kind, next_command, next_reason) =
        match outcome.status {
            CheckStatus::Done => (
                "done",
                "goal_check_done",
                format!(
                    "goal {} complete: all {} gate(s) green",
                    state.id,
                    state.gates.len()
                ),
                true,
                "done",
                None,
                "all recorded gates passed".to_string(),
            ),
            CheckStatus::Blocked => {
                let failing = failing.expect("blocked status implies a failing gate");
                (
                    "blocked",
                    "goal_check_blocked",
                    format!("goal {} still red: `{}` failed", state.id, failing.command),
                    false,
                    "run_command",
                    Some(format!("aw goal check {}", state.id)),
                    format!("gate `{}` has not passed yet", failing.command),
                )
            }
            CheckStatus::GaveUp => (
                "gave_up",
                "goal_gave_up",
                format!(
                    "goal {} gave up: budget/expiry exhausted after {} check(s)",
                    state.id, state.checks_run
                ),
                false,
                "none",
                None,
                "budget or 24h expiry exhausted; intent preserved for manual follow-up".to_string(),
            ),
        };

    let env = json!({
        "schema_version": "aw.cli.v1",
        "status": status,
        "action": action,
        "message": message,
        "goal": goal_summary_json(state),
        "gates": gate_reports,
        "requires_hitl": false,
        "next": {
            "kind": next_kind,
            "command": next_command,
            "reason": next_reason,
            "requires_hitl": false,
            "payload_path": Value::Null,
        },
        "completion": {
            "root_complete": workflow_complete,
            "workflow_complete": workflow_complete,
            "requires_hitl": false,
            "criteria": if workflow_complete {
                vec!["all recorded gates passed"]
            } else {
                Vec::<&str>::new()
            },
            "missing": if workflow_complete {
                Vec::<&str>::new()
            } else if outcome.status == CheckStatus::GaveUp {
                vec!["budget/expiry exhausted"]
            } else {
                vec!["at least one gate still failing"]
            },
        },
    });
    println!("{}", serde_json::to_string(&env)?);
    Ok(())
}

// -- state helpers --------------------------------------------------------

fn set_goal(
    project_root: &Path,
    intent: String,
    gates: Vec<String>,
    budget_checks: Option<u32>,
    budget_minutes: Option<u32>,
) -> Result<GoalState> {
    let dir = goals_path(project_root);
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("creating goal state directory {}", dir.display()))?;
    let id = new_goal_id(&dir);
    let state = GoalState {
        id,
        intent,
        gates,
        created_at: Utc::now().to_rfc3339(),
        budget_checks,
        budget_minutes,
        checks_run: 0,
    };
    write_goal(project_root, &state)?;
    Ok(state)
}

fn new_goal_id(dir: &Path) -> String {
    loop {
        let candidate = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
        if !dir.join(format!("{candidate}.json")).exists() {
            return candidate;
        }
    }
}

fn write_goal(project_root: &Path, state: &GoalState) -> Result<()> {
    let path = goal_state_path(project_root, &state.id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating goal state directory {}", parent.display()))?;
    }
    let content = serde_json::to_string_pretty(state)?;
    std::fs::write(&path, content)
        .with_context(|| format!("writing goal state {}", path.display()))?;
    Ok(())
}

fn load_goal(project_root: &Path, id: &str) -> Result<GoalState> {
    let path = goal_state_path(project_root, id);
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("no goal `{id}` recorded (looked for {})", path.display()))?;
    let state: GoalState = serde_json::from_str(&content)
        .with_context(|| format!("goal state at {} is not valid JSON", path.display()))?;
    Ok(state)
}

fn clear_goal(project_root: &Path, id: &str) -> Result<()> {
    let path = goal_state_path(project_root, id);
    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("removing goal state {}", path.display()))?;
    }
    Ok(())
}

fn list_goal_ids(project_root: &Path) -> Result<Vec<String>> {
    let dir = goals_path(project_root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut ids: Vec<String> = std::fs::read_dir(&dir)
        .with_context(|| format!("reading goal state directory {}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(str::to_string)
            } else {
                None
            }
        })
        .collect();
    ids.sort();
    Ok(ids)
}

fn resolve_goal_id(project_root: &Path, id: Option<&str>) -> Result<String> {
    if let Some(id) = id {
        return Ok(id.to_string());
    }
    let ids = list_goal_ids(project_root)?;
    match ids.len() {
        0 => anyhow::bail!(
            "no goals recorded for this workspace; run `aw goal set --gate <command> <intent>` first"
        ),
        1 => Ok(ids.into_iter().next().expect("checked len == 1")),
        _ => anyhow::bail!(
            "{} goals recorded for this workspace ({}); pass an explicit id",
            ids.len(),
            ids.join(", ")
        ),
    }
}

fn check_goal(project_root: &Path, id: &str) -> Result<CheckOutcome> {
    let mut state = load_goal(project_root, id)?;

    if is_expired(&state) {
        clear_goal(project_root, id)?;
        return Ok(CheckOutcome {
            state,
            status: CheckStatus::GaveUp,
            gates: Vec::new(),
        });
    }

    state.checks_run += 1;
    if let Some(budget) = state.budget_checks {
        if state.checks_run > budget {
            clear_goal(project_root, id)?;
            return Ok(CheckOutcome {
                state,
                status: CheckStatus::GaveUp,
                gates: Vec::new(),
            });
        }
    }

    let gates: Vec<GateOutcome> = state
        .gates
        .iter()
        .map(|gate| {
            run_gate(
                project_root,
                gate,
                Duration::from_secs(DEFAULT_GATE_TIMEOUT_SECS),
            )
        })
        .collect();
    let all_green = gates.iter().all(|gate| gate.success);

    if all_green {
        clear_goal(project_root, id)?;
        Ok(CheckOutcome {
            state,
            status: CheckStatus::Done,
            gates,
        })
    } else {
        write_goal(project_root, &state)?;
        Ok(CheckOutcome {
            state,
            status: CheckStatus::Blocked,
            gates,
        })
    }
}

fn is_expired(state: &GoalState) -> bool {
    let Ok(created_at) = DateTime::parse_from_rfc3339(&state.created_at) else {
        // Unparseable created_at: treat as expired rather than looping forever.
        return true;
    };
    let created_at: DateTime<Utc> = created_at.with_timezone(&Utc);
    let age = Utc::now().signed_duration_since(created_at);
    if age.num_hours() >= MAX_GOAL_AGE_HOURS {
        return true;
    }
    if let Some(budget_minutes) = state.budget_minutes {
        if age.num_minutes() >= i64::from(budget_minutes) {
            return true;
        }
    }
    false
}

fn run_gate(project_root: &Path, gate: &str, timeout: Duration) -> GateOutcome {
    let mut command = protected_shell_command(project_root, gate);
    command.current_dir(project_root);
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            return GateOutcome {
                command: gate.to_string(),
                success: false,
                output_tail: format!("failed to spawn gate command: {err}"),
                timed_out: false,
            };
        }
    };

    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();
    let stdout_reader = std::thread::spawn(move || {
        let mut buf = String::new();
        if let Some(pipe) = stdout_pipe.as_mut() {
            let _ = pipe.read_to_string(&mut buf);
        }
        buf
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut buf = String::new();
        if let Some(pipe) = stderr_pipe.as_mut() {
            let _ = pipe.read_to_string(&mut buf);
        }
        buf
    });

    let start = Instant::now();
    let (status, timed_out) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (Some(status), false),
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    break (None, true);
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => break (None, false),
        }
    };

    let stdout = stdout_reader.join().unwrap_or_default();
    let stderr = stderr_reader.join().unwrap_or_default();
    let success = status.map(|status| status.success()).unwrap_or(false);
    let output_tail = if timed_out {
        format!(
            "gate command exceeded {}s timeout and was killed",
            timeout.as_secs()
        )
    } else {
        tail_output(&stdout, &stderr)
    };

    GateOutcome {
        command: gate.to_string(),
        success,
        output_tail,
        timed_out,
    }
}

fn tail_output(stdout: &str, stderr: &str) -> String {
    let mut combined = String::new();
    if !stdout.trim().is_empty() {
        combined.push_str(stdout.trim_end());
    }
    if !stderr.trim().is_empty() {
        if !combined.is_empty() {
            combined.push('\n');
        }
        combined.push_str(stderr.trim_end());
    }
    let lines: Vec<&str> = combined.lines().collect();
    if lines.len() > OUTPUT_TAIL_LINES {
        lines[lines.len() - OUTPUT_TAIL_LINES..].join("\n")
    } else {
        combined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_root() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path().to_path_buf();
        (tmp, root)
    }

    #[test]
    fn goal_state_round_trips_through_set_and_load() {
        let (_tmp, root) = project_root();
        let state = set_goal(
            &root,
            "ship the thing".to_string(),
            vec!["true".to_string()],
            None,
            None,
        )
        .unwrap();

        let loaded = load_goal(&root, &state.id).unwrap();
        assert_eq!(loaded.id, state.id);
        assert_eq!(loaded.intent, "ship the thing");
        assert_eq!(loaded.gates, vec!["true".to_string()]);
        assert_eq!(loaded.checks_run, 0);
    }

    #[test]
    fn check_all_green_clears_state_and_reports_done() {
        let (_tmp, root) = project_root();
        let state = set_goal(
            &root,
            "trivial".to_string(),
            vec!["true".to_string()],
            None,
            None,
        )
        .unwrap();

        let outcome = check_goal(&root, &state.id).unwrap();
        assert_eq!(outcome.status, CheckStatus::Done);
        assert!(outcome.gates.iter().all(|gate| gate.success));
        assert!(!goal_state_path(&root, &state.id).exists());
    }

    #[test]
    fn check_red_gate_keeps_state_and_reports_blocked() {
        let (_tmp, root) = project_root();
        let state = set_goal(
            &root,
            "trivial".to_string(),
            vec!["false".to_string()],
            None,
            None,
        )
        .unwrap();

        let outcome = check_goal(&root, &state.id).unwrap();
        assert_eq!(outcome.status, CheckStatus::Blocked);
        assert!(outcome.gates.iter().any(|gate| !gate.success));
        assert!(goal_state_path(&root, &state.id).exists());

        let reloaded = load_goal(&root, &state.id).unwrap();
        assert_eq!(reloaded.checks_run, 1);
    }

    #[test]
    fn check_exhausted_budget_gives_up_and_clears_state() {
        let (_tmp, root) = project_root();
        let mut state = set_goal(
            &root,
            "trivial".to_string(),
            vec!["false".to_string()],
            Some(1),
            None,
        )
        .unwrap();
        state.checks_run = 1;
        write_goal(&root, &state).unwrap();

        let outcome = check_goal(&root, &state.id).unwrap();
        assert_eq!(outcome.status, CheckStatus::GaveUp);
        assert_eq!(outcome.state.intent, "trivial");
        assert!(!goal_state_path(&root, &state.id).exists());
    }

    #[test]
    fn check_expired_goal_gives_up_and_clears_state() {
        let (_tmp, root) = project_root();
        let mut state = set_goal(
            &root,
            "stale".to_string(),
            vec!["true".to_string()],
            None,
            None,
        )
        .unwrap();
        state.created_at = (Utc::now() - chrono::Duration::hours(25)).to_rfc3339();
        write_goal(&root, &state).unwrap();

        let outcome = check_goal(&root, &state.id).unwrap();
        assert_eq!(outcome.status, CheckStatus::GaveUp);
        assert_eq!(outcome.state.intent, "stale");
        assert!(!goal_state_path(&root, &state.id).exists());
    }

    #[test]
    fn goal_state_is_isolated_per_workspace() {
        let (_tmp_a, root_a) = project_root();
        let (_tmp_b, root_b) = project_root();

        let state_a = set_goal(
            &root_a,
            "a".to_string(),
            vec!["true".to_string()],
            None,
            None,
        )
        .unwrap();

        assert!(load_goal(&root_b, &state_a.id).is_err());
        assert_eq!(list_goal_ids(&root_b).unwrap(), Vec::<String>::new());
        assert_eq!(list_goal_ids(&root_a).unwrap(), vec![state_a.id]);
    }

    #[test]
    fn resolve_goal_id_requires_explicit_id_when_multiple_recorded() {
        let (_tmp, root) = project_root();
        set_goal(
            &root,
            "one".to_string(),
            vec!["true".to_string()],
            None,
            None,
        )
        .unwrap();
        set_goal(
            &root,
            "two".to_string(),
            vec!["true".to_string()],
            None,
            None,
        )
        .unwrap();

        assert!(resolve_goal_id(&root, None).is_err());
    }

    #[test]
    fn resolve_goal_id_defaults_to_sole_goal() {
        let (_tmp, root) = project_root();
        let state = set_goal(
            &root,
            "one".to_string(),
            vec!["true".to_string()],
            None,
            None,
        )
        .unwrap();

        assert_eq!(resolve_goal_id(&root, None).unwrap(), state.id);
    }
}

// CODEGEN-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/cli/goal.rs
    action: add
    impl_mode: codegen
    section: source
    description: |
      Issue #1897: new `aw goal` verb family (`set`/`check`/`show`/`list`/
      `clear`) — a CLI-owned verifiable-condition loop for bounded work
      outside the WI/TD/EC tracked lifecycle. `aw goal set` records prose
      intent plus one or more required `--gate <command>` machine-runnable
      conditions (rejecting with remediation on zero gates) plus optional
      `--budget-checks`/`--budget-minutes`, as workspace-scoped ephemeral
      JSON under `/tmp/aw/workspaces/<workspace>/goals/{goal_id}.json`
      (`goals_path`/`goal_state_path`, `shared::workspace`). `aw goal check`
      runs every recorded gate with a bounded 600s per-command timeout via a
      reader-thread + `try_wait()` poll loop (no pipe-buffer deadlock) and
      reports deterministically: all green clears the state and emits a
      terminal `done` envelope; any red keeps the state, reports the
      failing gate's output tail, and emits
      `next.command = aw goal check <id>`; a `--budget-checks`/
      `--budget-minutes` exhaustion or the hard 24h expiry ceiling since
      `created_at` clears the state and emits a terminal `gave_up` status
      without dropping the recorded intent from the report. Every emitted
      envelope is an `aw.cli.v1` chain-valid JSON object. Replaces the
      retired generic `goal-loop` Stop-hook skill: gates are executable
      commands only, never LLM-judged/prose-only conditions, and this does
      not replace `aw wi run`/`aw capability run` completion semantics or
      cross session/workspace boundaries.
  - path: apps/agentic-workflow/src/cli/goal.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Issue #1899: `aw goal` becomes the single unified loop verb. Two new
      `GoalCommand` variants -- `Wi(GoalWiArgs)` and
      `Capability(GoalCapabilityArgs)` -- re-home the retired `aw wi run
      <id>` and `aw capability run [<cap-id>] --project <p>` lifecycle root
      types as `aw goal wi <id>` / `aw goal capability [<cap-id>] --project
      <p>`. `GoalWiArgs` is a standalone struct (field-shape mirror of the
      old `crate::cli::issues::WiRunArgs`, not a re-export) so the retired
      `aw wi run <id>` clap leaf can keep parsing for its
      `emit_retired_verb_redirect` envelope without depending on this
      canonical form. `GoalCapabilityArgs` mirrors the field subset of
      `capability::CapabilityRunArgs` the shared engine needs plus a
      mandatory `--project` (no parent command to inherit it from). `run`
      became `pub async fn` and dispatches the two new variants to
      `run_goal_wi`/`run_goal_capability`, which delegate straight into the
      same shared root-driven workflow runner
      (`crate::cli::run::run_wi_root`, `crate::cli::run::run_capability_root`,
      `crate::cli::capability::run_capability_tick`) the retired verbs used
      -- envelope semantics (aw.cli.v1 schema, `completion.workflow_complete`,
      `requires_hitl`, `hitl_question`, progress JSONL, re-run-same-root
      convention) are unchanged, this is a re-home not a rewrite. The
      pre-existing `set`/`check`/`show`/`list`/`clear` ad-hoc leaves and
      their engine are untouched.
  - path: apps/agentic-workflow/src/cli/goal.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Issue #1899 R3+R7 completion. `aw wi run <id>`'s clap leaf
      (`src/cli/issues.rs`) is now actually retired -- `run_wi_run` calls
      `crate::cli::run::emit_retired_verb_redirect` naming `aw goal wi <id>`
      instead of re-entering the run engine, and `wi.run` in
      `VERB_LIFECYCLE_REGISTRY` (`chain.rs`) is reclassified
      `Migration`/non-mutating with a sunset criterion; `GoalWiArgs` stays
      the canonical args struct this slice's redirect target uses. New
      `GoalCommand::Backlog(GoalBacklogArgs)` variant + `run_goal_backlog`
      re-home `aw goal backlog --project <p>` (R7): a tracker-driven drain
      of every open work item for a project, one WI per invocation, handed
      off via the same `aw goal wi <id>` command the shared engine already
      emits elsewhere. `run_goal_backlog` is a thin delegate straight into
      `crate::cli::run::run_backlog_root`; the drain/park/select logic lives
      there (see `agentic-workflow-cli.md#schema`'s `run.rs` Changes entry).
```
