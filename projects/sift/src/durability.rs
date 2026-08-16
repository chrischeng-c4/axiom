// HANDWRITE-BEGIN gap="sift-framed-journal-state-machine" tracker="1605" reason="Implement CRC-framed event journal snapshot/restore and the RaftStateMachine adapter."
//! Shared durability and Raft state-machine adapter for Sift's raw journal.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use anyhow::{bail, Context, Result};
use chrono::DateTime;
use raft_host::{Index, RaftStateMachine};
use serde::{Deserialize, Serialize};

use crate::{
    projection::{
        AuditExportManifestV1, AuditLegalHoldV1, ErrorLifecycleState, ErrorLifecycleV1, ReplayJob,
        SiftControlState, SIFT_COMMAND_FORMAT_VERSION,
    },
    AppendResult, DurableJournal, EventEnvelope, IncomingEvent, SignalKind, StoredEvent,
};

const CONTROL_STATE_FILE: &str = "sift-control-state.json";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SiftCommandV1 {
    AppendEvent { event: Box<EventEnvelope> },
    UpsertReplayJob { job: Box<ReplayJob> },
    TransitionErrorGroup { lifecycle: Box<ErrorLifecycleV1> },
    UpsertAuditLegalHold { hold: Box<AuditLegalHoldV1> },
    RecordAuditExport { export: Box<AuditExportManifestV1> },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct JournalSnapshot {
    pub applied_index: u64,
    pub events: Vec<StoredEvent>,
    #[serde(default)]
    pub replay_jobs: BTreeMap<String, ReplayJob>,
    #[serde(default)]
    pub error_lifecycles: BTreeMap<String, ErrorLifecycleV1>,
    #[serde(default)]
    pub audit_legal_holds: BTreeMap<String, AuditLegalHoldV1>,
    #[serde(default)]
    pub audit_exports: BTreeMap<String, AuditExportManifestV1>,
}

impl JournalSnapshot {
    pub(crate) fn from_state(events: Vec<StoredEvent>, control: &SiftControlState) -> Self {
        Self {
            applied_index: control.applied_index,
            events,
            replay_jobs: control.replay_jobs.clone(),
            error_lifecycles: control.error_lifecycles.clone(),
            audit_legal_holds: control.audit_legal_holds.clone(),
            audit_exports: control.audit_exports.clone(),
        }
    }

    pub(crate) fn from_events(events: Vec<StoredEvent>) -> Self {
        let applied_index = events.last().map(|event| event.cursor).unwrap_or(0);
        Self {
            applied_index,
            events,
            replay_jobs: BTreeMap::new(),
            error_lifecycles: BTreeMap::new(),
            audit_legal_holds: BTreeMap::new(),
            audit_exports: BTreeMap::new(),
        }
    }
}

/// The only replicated applier for Sift events. RaftHost calls this adapter in
/// committed-index order, so an acknowledged replica write has passed through
/// the same durable journal boundary on every voter.
pub struct SiftStateMachine {
    journal: Arc<DurableJournal>,
    control_path: PathBuf,
    control: Mutex<SiftControlState>,
    append_outcomes: Mutex<BTreeMap<u64, AppendResult>>,
    applied_index: AtomicU64,
}

impl SiftStateMachine {
    /// Compatibility constructor for existing embedders. New service startup
    /// uses [`open`](Self::open) so initialization failures remain explicit.
    pub fn new(journal: Arc<DurableJournal>) -> Self {
        let data_dir = journal
            .snapshot_path
            .parent()
            .expect("Sift snapshot path must have a data directory")
            .to_path_buf();
        Self::open(data_dir, journal).expect("open Sift state machine control state")
    }

    pub fn open(data_dir: impl AsRef<Path>, journal: Arc<DurableJournal>) -> Result<Self> {
        let control_path = data_dir.as_ref().join(CONTROL_STATE_FILE);
        let mut control =
            if control_path.exists() {
                serde_json::from_slice::<SiftControlState>(&fs::read(&control_path).with_context(
                    || format!("read Sift control state {}", control_path.display()),
                )?)
                .with_context(|| format!("decode Sift control state {}", control_path.display()))?
            } else {
                SiftControlState::default()
            };
        if control.format_version != SIFT_COMMAND_FORMAT_VERSION {
            bail!(
                "unsupported Sift control state format {}",
                control.format_version
            );
        }
        // An old Sift journal had no separate control file. A raw append may
        // also have reached fsync immediately before a process crash prevented
        // the control write. In both cases the durable raw cursor is a valid
        // lower bound for the next local commit index.
        control.applied_index = control.applied_index.max(journal.last_cursor());
        persist_control(&control_path, &control)?;
        let applied_index = control.applied_index;
        Ok(Self {
            journal,
            control_path,
            control: Mutex::new(control),
            append_outcomes: Mutex::new(BTreeMap::new()),
            applied_index: AtomicU64::new(applied_index),
        })
    }

    pub fn applied_commit_index(&self) -> u64 {
        self.applied_index.load(Ordering::Acquire)
    }

    pub fn apply_local(&self, index: u64, command: &[u8]) -> Result<()> {
        <Self as RaftStateMachine>::apply(self, index, command)
    }

    pub fn replay_job(&self, id: &str) -> Option<ReplayJob> {
        self.control
            .lock()
            .expect("Sift control state lock poisoned")
            .replay_jobs
            .get(id)
            .cloned()
    }

    pub fn error_lifecycle(&self, project: &str, fingerprint: &str) -> Option<ErrorLifecycleV1> {
        self.control
            .lock()
            .expect("Sift control state lock poisoned")
            .error_lifecycles
            .get(&crate::projection::error_lifecycle_key(
                project,
                fingerprint,
            ))
            .cloned()
    }

    pub fn audit_legal_hold(&self, project: &str, id: &str) -> Option<AuditLegalHoldV1> {
        self.control
            .lock()
            .expect("Sift control state lock poisoned")
            .audit_legal_holds
            .get(&crate::projection::audit_control_key(project, id))
            .cloned()
    }

    pub fn audit_legal_holds(&self, project: &str) -> Vec<AuditLegalHoldV1> {
        self.control
            .lock()
            .expect("Sift control state lock poisoned")
            .audit_legal_holds
            .values()
            .filter(|hold| hold.project == project && hold.active)
            .cloned()
            .collect()
    }

    pub fn audit_export(&self, project: &str, id: &str) -> Option<AuditExportManifestV1> {
        self.control
            .lock()
            .expect("Sift control state lock poisoned")
            .audit_exports
            .get(&crate::projection::audit_control_key(project, id))
            .cloned()
    }

    pub fn take_append_outcome(&self, index: u64) -> Option<AppendResult> {
        self.append_outcomes
            .lock()
            .expect("Sift append outcome lock poisoned")
            .remove(&index)
    }
}

impl RaftStateMachine for SiftStateMachine {
    fn apply(&self, index: Index, command: &[u8]) -> Result<()> {
        if index <= self.applied_index.load(Ordering::Acquire) {
            return Ok(());
        }
        let command = decode_command(command)?;
        let mut control = self
            .control
            .lock()
            .expect("Sift control state lock poisoned");
        match command {
            SiftCommandV1::AppendEvent { event } => {
                // Raw cursors remain dense when non-event commands interleave.
                // A retry after raw fsync/control-write failure is event-id
                // idempotent at this boundary.
                let result = self.journal.append(*event)?.with_commit_index(index);
                let mut outcomes = self
                    .append_outcomes
                    .lock()
                    .expect("Sift append outcome lock poisoned");
                outcomes.insert(index, result);
                while outcomes.len() > 4_096 {
                    if let Some(oldest) = outcomes.keys().next().copied() {
                        outcomes.remove(&oldest);
                    }
                }
            }
            SiftCommandV1::UpsertReplayJob { mut job } => {
                job.commit_index = index;
                control.replay_jobs.insert(job.id.clone(), *job);
            }
            SiftCommandV1::TransitionErrorGroup { mut lifecycle } => {
                validate_error_lifecycle(&lifecycle)?;
                lifecycle.commit_index = index;
                let key = lifecycle.key();
                let previous = control
                    .error_lifecycles
                    .get(&key)
                    .map(|value| value.state)
                    .unwrap_or(ErrorLifecycleState::Open);
                for signal in [SignalKind::AuditEvent, SignalKind::ChangeEvent] {
                    let suffix = if signal == SignalKind::AuditEvent {
                        "audit"
                    } else {
                        "change"
                    };
                    let mut event = EventEnvelope::for_project(
                        lifecycle.project.clone(),
                        "control",
                        format!(
                            "error-lifecycle:{}:{}:{index}:{suffix}",
                            lifecycle.project, lifecycle.fingerprint
                        ),
                        signal,
                        serde_json::json!({
                            "kind": "error_group_lifecycle_transition",
                            "fingerprint": lifecycle.fingerprint,
                            "from": previous,
                            "to": lifecycle.state,
                            "actor": lifecycle.actor,
                            "reason": lifecycle.reason,
                            "muted_until": lifecycle.muted_until,
                            "occurrence_cursor": lifecycle.occurrence_cursor,
                            "commit_index": index,
                        }),
                    );
                    event.occurred_at.clone_from(&lifecycle.updated_at);
                    event.observed_at.clone_from(&lifecycle.updated_at);
                    event.resource.insert("service.name".into(), "sift".into());
                    self.journal.append(event)?;
                }
                control.error_lifecycles.insert(key, *lifecycle);
            }
            SiftCommandV1::UpsertAuditLegalHold { mut hold } => {
                validate_audit_legal_hold(&hold)?;
                hold.commit_index = index;
                let key = hold.key();
                let previous_active = control
                    .audit_legal_holds
                    .get(&key)
                    .is_some_and(|previous| previous.active);
                append_control_evidence(
                    &self.journal,
                    &hold.project,
                    &hold.updated_at,
                    &format!("audit-hold:{}:{}:{index}", hold.project, hold.id),
                    true,
                    serde_json::json!({
                        "kind": "audit_legal_hold_transition",
                        "action": if hold.active { "audit.hold.activate" } else { "audit.hold.release" },
                        "target": hold.id.clone(),
                        "actor": hold.actor.clone(),
                        "reason": hold.reason.clone(),
                        "start_time": hold.start_time.clone(),
                        "end_time": hold.end_time.clone(),
                        "from_active": previous_active,
                        "to_active": hold.active,
                        "commit_index": index,
                    }),
                )?;
                control.audit_legal_holds.insert(key, *hold);
            }
            SiftCommandV1::RecordAuditExport { mut export } => {
                validate_audit_export(&export)?;
                export.commit_index = index;
                let key = export.key();
                if control.audit_exports.contains_key(&key) {
                    bail!("audit export id `{}` already exists", export.id);
                }
                append_control_evidence(
                    &self.journal,
                    &export.project,
                    &export.exported_at,
                    &format!("audit-export:{}:{}:{index}", export.project, export.id),
                    false,
                    serde_json::json!({
                        "kind": "audit_controlled_export",
                        "action": "audit.export",
                        "target": export.id.clone(),
                        "actor": export.actor.clone(),
                        "record_count": export.record_count,
                        "content_sha256": export.content_sha256.clone(),
                        "start_time": export.start_time.clone(),
                        "end_time": export.end_time.clone(),
                        "commit_index": index,
                    }),
                )?;
                control.audit_exports.insert(key, *export);
            }
        }
        control.applied_index = index;
        persist_control(&self.control_path, &control)?;
        self.applied_index.store(index, Ordering::Release);
        Ok(())
    }

    fn snapshot(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        let control = self
            .control
            .lock()
            .expect("Sift control state lock poisoned");
        let bytes = serde_json::to_vec(&JournalSnapshot::from_state(
            self.journal.snapshot_events(),
            &control,
        ))?;
        writer.write_all(&bytes)?;
        Ok(())
    }

    fn restore(&self, reader: &mut dyn std::io::Read) -> Result<()> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        let snapshot: JournalSnapshot = serde_json::from_slice(&bytes)?;
        self.journal.restore_snapshot(snapshot.events)?;
        let restored = SiftControlState {
            format_version: SIFT_COMMAND_FORMAT_VERSION,
            applied_index: snapshot.applied_index,
            replay_jobs: snapshot.replay_jobs,
            error_lifecycles: snapshot.error_lifecycles,
            audit_legal_holds: snapshot.audit_legal_holds,
            audit_exports: snapshot.audit_exports,
        };
        persist_control(&self.control_path, &restored)?;
        *self
            .control
            .lock()
            .expect("Sift control state lock poisoned") = restored;
        self.applied_index
            .store(snapshot.applied_index, Ordering::Release);
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied_index.load(Ordering::Acquire)
    }
}

fn validate_error_lifecycle(lifecycle: &ErrorLifecycleV1) -> Result<()> {
    if lifecycle.project.trim().is_empty()
        || lifecycle.fingerprint.trim().is_empty()
        || lifecycle.actor.trim().is_empty()
    {
        bail!("error lifecycle project, fingerprint, and actor must not be empty");
    }
    DateTime::parse_from_rfc3339(&lifecycle.updated_at)
        .context("error lifecycle updated_at must be RFC3339")?;
    match (lifecycle.state, lifecycle.muted_until.as_deref()) {
        (ErrorLifecycleState::Muted, Some(until)) => {
            DateTime::parse_from_rfc3339(until)
                .context("muted error lifecycle requires RFC3339 muted_until")?;
        }
        (ErrorLifecycleState::Muted, None) => {
            bail!("muted error lifecycle requires muted_until")
        }
        (_, Some(_)) => bail!("only muted error lifecycle may set muted_until"),
        (_, None) => {}
    }
    Ok(())
}

fn validate_audit_legal_hold(hold: &AuditLegalHoldV1) -> Result<()> {
    if hold.id.trim().is_empty()
        || hold.project.trim().is_empty()
        || hold.reason.trim().is_empty()
        || hold.actor.trim().is_empty()
    {
        bail!("audit hold id, project, reason, and actor must not be empty");
    }
    let start = DateTime::parse_from_rfc3339(&hold.start_time)
        .context("audit hold start_time must be RFC3339")?;
    let end = DateTime::parse_from_rfc3339(&hold.end_time)
        .context("audit hold end_time must be RFC3339")?;
    DateTime::parse_from_rfc3339(&hold.updated_at)
        .context("audit hold updated_at must be RFC3339")?;
    if start >= end {
        bail!("audit hold start_time must be earlier than end_time");
    }
    Ok(())
}

fn validate_audit_export(export: &AuditExportManifestV1) -> Result<()> {
    if export.id.trim().is_empty()
        || export.project.trim().is_empty()
        || export.actor.trim().is_empty()
    {
        bail!("audit export id, project, and actor must not be empty");
    }
    if export.content_sha256.len() != 64
        || !export
            .content_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("audit export content_sha256 must be a 64-character hex digest");
    }
    DateTime::parse_from_rfc3339(&export.exported_at)
        .context("audit export exported_at must be RFC3339")?;
    Ok(())
}

fn append_control_evidence(
    journal: &DurableJournal,
    project: &str,
    occurred_at: &str,
    event_id_prefix: &str,
    include_change: bool,
    payload: serde_json::Value,
) -> Result<()> {
    let signals: &[SignalKind] = if include_change {
        &[SignalKind::AuditEvent, SignalKind::ChangeEvent]
    } else {
        &[SignalKind::AuditEvent]
    };
    for signal in signals {
        let suffix = if *signal == SignalKind::AuditEvent {
            "audit"
        } else {
            "change"
        };
        let mut event = EventEnvelope::for_project(
            project,
            "control",
            format!("{event_id_prefix}:{suffix}"),
            *signal,
            payload.clone(),
        );
        event.occurred_at = occurred_at.to_string();
        event.observed_at = occurred_at.to_string();
        event.resource.insert("service.name".into(), "sift".into());
        journal.append(event)?;
    }
    Ok(())
}

fn decode_command(bytes: &[u8]) -> Result<SiftCommandV1> {
    match serde_json::from_slice::<SiftCommandV1>(bytes) {
        Ok(command) => Ok(command),
        Err(command_error) => serde_json::from_slice::<IncomingEvent>(bytes)
            .map(|event| SiftCommandV1::AppendEvent {
                event: Box::new(event.into_inner()),
            })
            .with_context(|| {
                format!("decode Sift command v1 or legacy bare event: {command_error}")
            }),
    }
}

fn persist_control(path: &Path, control: &SiftControlState) -> Result<()> {
    service_durability::atomic_write(
        path,
        &serde_json::to_vec_pretty(control)?,
        service_durability::FsyncPolicy::Always,
    )
    .with_context(|| format!("atomically persist Sift control state {}", path.display()))
}

// HANDWRITE-END
