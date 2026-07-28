// SPEC-MANAGED: apps/agentic-workflow/tech-design/src/agentic_workflow/work_items/coordination_authority.py
//! Public AW coordination authority commands.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use serde_json::json;

use crate::coordination::authority::{
    interrupt_dispatch, load_state, open_state, record_decision, satisfy_gate, save_state,
    submit_event, CoordinationState, EventRejection, ReconciliationOutcome,
};
use crate::coordination::protocol::{
    DispatchDocument, GateDocument, MessageDocument, TaskDocument,
};

#[derive(Debug, Args)]
pub struct CoordinationArgs {
    #[command(subcommand)]
    command: CoordinationCommand,
}

#[derive(Debug, Subcommand)]
enum CoordinationCommand {
    /// Establish a new AW-owned task/dispatch/gate snapshot.
    Open {
        #[arg(long)]
        task: PathBuf,
        #[arg(long)]
        dispatch: PathBuf,
        #[arg(long)]
        gates: PathBuf,
    },
    /// Record AW-verified evidence for one required gate.
    SatisfyGate {
        task_id: String,
        #[arg(long)]
        gate: String,
        #[arg(long)]
        evidence: String,
    },
    /// Durably interrupt the active dispatch.
    Interrupt {
        task_id: String,
        #[arg(long)]
        reason: String,
    },
    /// Submit one untrusted worker event for AW reconciliation.
    Submit {
        task_id: String,
        #[arg(long)]
        event: PathBuf,
    },
    /// Record a human-authorized decision in AW-owned state.
    Decide {
        task_id: String,
        #[arg(long)]
        gate: String,
        #[arg(long)]
        choice: String,
        #[arg(long)]
        evidence: String,
    },
    /// Read one AW-owned coordination snapshot.
    Show { task_id: String },
    /// Print one canonical public coordination JSON Schema.
    Schema {
        #[arg(value_enum)]
        document_kind: CoordinationSchemaKind,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CoordinationSchemaKind {
    Message,
}

pub fn run(args: CoordinationArgs) -> Result<()> {
    let project_root = crate::cli::find_project_root()?;
    match args.command {
        CoordinationCommand::Open {
            task,
            dispatch,
            gates,
        } => run_open(&project_root, &task, &dispatch, &gates),
        CoordinationCommand::SatisfyGate {
            task_id,
            gate,
            evidence,
        } => run_satisfy_gate(&project_root, &task_id, &gate, &evidence),
        CoordinationCommand::Interrupt { task_id, reason } => {
            run_interrupt(&project_root, &task_id, &reason)
        }
        CoordinationCommand::Submit { task_id, event } => {
            run_submit(&project_root, &task_id, &event)
        }
        CoordinationCommand::Decide {
            task_id,
            gate,
            choice,
            evidence,
        } => run_decide(&project_root, &task_id, &gate, &choice, &evidence),
        CoordinationCommand::Show { task_id } => run_show(&project_root, &task_id),
        CoordinationCommand::Schema { document_kind } => run_schema(document_kind),
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path, label: &str) -> Result<T> {
    let bytes =
        fs::read(path).with_context(|| format!("failed to read {label}: {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("invalid {label} JSON: {}", path.display()))
}

fn print_rejected(reason: &str, decision_advanced: bool) {
    println!(
        "{}",
        json!({
            "schema_version": "aw.cli.v1",
            "status": "rejected",
            "authority": "aw",
            "reason": reason,
            "completion_advanced": false,
            "decision_advanced": decision_advanced,
            "terminal": true
        })
    );
}

fn reject<T>(reason: impl Into<String>) -> Result<T> {
    let reason = reason.into();
    print_rejected(&reason, false);
    anyhow::bail!(reason)
}

fn reject_event<T>(rejection: EventRejection) -> Result<T> {
    println!(
        "{}",
        json!({
            "schema_version": "aw.cli.v1",
            "status": "rejected",
            "code": rejection.code.as_str(),
            "authority": "aw",
            "reason": rejection.reason,
            "completion_advanced": false,
            "decision_advanced": false,
            "next": {
                "kind": "run_command",
                "command": rejection.remediation,
                "reason": "inspect the authoritative coordination contract before retrying"
            },
            "terminal": true
        })
    );
    anyhow::bail!("coordination event rejected")
}

fn run_open(project_root: &Path, task: &Path, dispatch: &Path, gates: &Path) -> Result<()> {
    let task: TaskDocument = match read_json(task, "task") {
        Ok(value) => value,
        Err(error) => return reject(error.to_string()),
    };
    let dispatch: DispatchDocument = match read_json(dispatch, "dispatch") {
        Ok(value) => value,
        Err(error) => return reject(error.to_string()),
    };
    let gates: Vec<GateDocument> = match read_json(gates, "gate inventory") {
        Ok(value) => value,
        Err(error) => return reject(error.to_string()),
    };
    let state = match open_state(task, dispatch, gates) {
        Ok(value) => value,
        Err(error) => return reject(error.to_string()),
    };
    let state_path = save_state(project_root, &state)?;
    println!(
        "{}",
        json!({
            "schema_version": "aw.cli.v1",
            "status": "open",
            "authority": "aw",
            "task_id": state.task.task_id,
            "state_path": state_path,
            "completion_advanced": false,
            "decision_advanced": false,
            "terminal": true
        })
    );
    Ok(())
}

fn run_satisfy_gate(
    project_root: &Path,
    task_id: &str,
    gate_id: &str,
    evidence: &str,
) -> Result<()> {
    let mut state = load_state(project_root, task_id)?;
    if let Err(error) = satisfy_gate(&mut state, gate_id, evidence) {
        return reject(error.to_string());
    }
    save_state(project_root, &state)?;
    println!(
        "{}",
        json!({
            "schema_version": "aw.cli.v1",
            "status": "satisfied",
            "authority": "aw",
            "task_id": task_id,
            "gate_id": gate_id,
            "completion_advanced": false,
            "decision_advanced": false,
            "terminal": true
        })
    );
    Ok(())
}

fn run_interrupt(project_root: &Path, task_id: &str, reason: &str) -> Result<()> {
    let mut state = load_state(project_root, task_id)?;
    if let Err(error) = interrupt_dispatch(&mut state, reason) {
        return reject(error.to_string());
    }
    save_state(project_root, &state)?;
    println!(
        "{}",
        json!({
            "schema_version": "aw.cli.v1",
            "status": "interrupted",
            "authority": "aw",
            "task_id": task_id,
            "dispatch_status": "interrupted",
            "completion_advanced": false,
            "decision_advanced": false,
            "terminal": true
        })
    );
    Ok(())
}

fn outcome_json(task_id: &str, outcome: &ReconciliationOutcome) -> serde_json::Value {
    json!({
        "schema_version": "aw.cli.v1",
        "status": outcome.status,
        "authority": "aw",
        "task_id": task_id,
        "reason": outcome.reason,
        "completion_advanced": outcome.completion_advanced,
        "decision_advanced": outcome.decision_advanced,
        "requires_hitl": outcome.requires_hitl,
        "terminal": true
    })
}

fn run_submit(project_root: &Path, task_id: &str, event: &Path) -> Result<()> {
    let mut state = load_state(project_root, task_id)?;
    let event: MessageDocument = match read_json(event, "coordination event") {
        Ok(value) => value,
        Err(error) => return reject_event(EventRejection::invalid(error.to_string())),
    };
    let outcome = match submit_event(&mut state, event) {
        Ok(value) => value,
        Err(rejection) => return reject_event(rejection),
    };
    save_state(project_root, &state)?;
    println!("{}", outcome_json(task_id, &outcome));
    Ok(())
}

fn run_decide(
    project_root: &Path,
    task_id: &str,
    gate_id: &str,
    choice: &str,
    evidence: &str,
) -> Result<()> {
    let mut state = load_state(project_root, task_id)?;
    let outcome = match record_decision(&mut state, gate_id, choice, evidence) {
        Ok(value) => value,
        Err(error) => return reject(error.to_string()),
    };
    if outcome.decision_advanced {
        save_state(project_root, &state)?;
    }
    println!("{}", outcome_json(task_id, &outcome));
    Ok(())
}

fn run_show(project_root: &Path, task_id: &str) -> Result<()> {
    let state: CoordinationState = load_state(project_root, task_id)?;
    println!(
        "{}",
        json!({
            "schema_version": "aw.cli.v1",
            "status": "ok",
            "authority": "aw",
            "task": state.task,
            "dispatch": state.dispatch,
            "gates": state.gates,
            "completion_advanced": state.completion_advanced,
            "decision": state.decision,
            "events": state.events,
            "interrupt_reason": state.interrupt_reason,
            "terminal": true
        })
    );
    Ok(())
}

/// @spec #2588
fn run_schema(document_kind: CoordinationSchemaKind) -> Result<()> {
    let (name, source) = match document_kind {
        CoordinationSchemaKind::Message => (
            "message",
            include_str!("../../schemas/coordination/message.schema.json"),
        ),
    };
    let schema: serde_json::Value = serde_json::from_str(source)?;
    println!(
        "{}",
        json!({
            "schema_version": "aw.cli.v1",
            "status": "ok",
            "document_kind": name,
            "schema": schema,
            "terminal": true
        })
    );
    Ok(())
}
