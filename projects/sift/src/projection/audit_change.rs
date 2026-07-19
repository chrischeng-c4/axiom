// HANDWRITE-BEGIN gap="sift-audit-change-projection" tracker="1668" reason="Define normalized hash-chained records, retention/hold query, export records, snapshot, integrity verification, and rebuild."
use std::{any::Any, collections::BTreeMap, sync::RwLock};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;

use crate::{AttributeValue, SignalKind, StoredEvent};

use super::{
    model::{AuditExportManifestV1, AuditLegalHoldV1, ProjectionDescriptor},
    runtime::Projection,
};

pub const PROJECTION_AUDIT_CHANGE_STORE: &str = "audit-change-store";
pub const AUDIT_CHANGE_SCHEMA_VERSION: u32 = 1;
pub const AUDIT_RECORD_SCHEMA: &str = "sift.audit.v1";
pub const DEFAULT_AUDIT_RETENTION_DAYS: i64 = 365;
pub const MAX_AUDIT_QUERY_LIMIT: usize = 1_000;
const GENESIS_HASH: &str = "GENESIS";
type AuditTimeRange = (Option<DateTime<Utc>>, Option<DateTime<Utc>>);

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct AuditChangeRecordV1 {
    pub schema: String,
    pub cursor: u64,
    pub event_id: String,
    pub project: String,
    pub environment: String,
    pub signal: SignalKind,
    pub occurred_at: String,
    pub actor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    pub action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    pub actor_missing: bool,
    pub resource: BTreeMap<String, String>,
    #[schema(value_type = Object)]
    pub attributes: BTreeMap<String, AttributeValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[schema(value_type = Object)]
    pub payload: serde_json::Value,
    pub previous_hash: String,
    pub record_hash: String,
    pub retention_expires_at: String,
    pub retained_by_hold: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct AuditQuery {
    pub project: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signal: Option<SignalKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    #[serde(default)]
    pub after_cursor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_cursor: Option<u64>,
    #[serde(default = "default_query_limit")]
    pub limit: usize,
}

impl AuditQuery {
    pub fn for_project(project: impl Into<String>) -> Self {
        Self {
            project: project.into(),
            signal: None,
            actor: None,
            action: None,
            target: None,
            trace_id: None,
            request_id: None,
            session_id: None,
            start_time: None,
            end_time: None,
            after_cursor: 0,
            min_cursor: None,
            limit: default_query_limit(),
        }
    }

    fn validate(&self) -> Result<AuditTimeRange> {
        if self.project.trim().is_empty() {
            bail!("project must not be empty");
        }
        if self.limit == 0 || self.limit > MAX_AUDIT_QUERY_LIMIT {
            bail!("limit must be between 1 and {MAX_AUDIT_QUERY_LIMIT}");
        }
        if self.signal.is_some_and(|signal| {
            !matches!(signal, SignalKind::AuditEvent | SignalKind::ChangeEvent)
        }) {
            bail!("audit query signal must be audit_event or change_event");
        }
        let start = parse_optional_time("start_time", self.start_time.as_deref())?;
        let end = parse_optional_time("end_time", self.end_time.as_deref())?;
        if start.zip(end).is_some_and(|(start, end)| start >= end) {
            bail!("start_time must be earlier than end_time");
        }
        Ok((start, end))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct AuditPage {
    pub records: Vec<AuditChangeRecordV1>,
    pub next_cursor: u64,
    pub projection_cursor: u64,
    pub has_more: bool,
    pub chain_valid: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct AuditExportResponseV1 {
    pub manifest: AuditExportManifestV1,
    pub records: Vec<AuditChangeRecordV1>,
}

#[derive(Default, Deserialize, Serialize)]
struct AuditChangeState {
    records: BTreeMap<u64, AuditChangeRecordV1>,
    cursor_by_event_id: BTreeMap<String, u64>,
    last_hash_by_project: BTreeMap<String, String>,
    last_cursor_by_project: BTreeMap<String, u64>,
}

#[derive(Serialize)]
struct HashMaterial<'a> {
    schema: &'a str,
    cursor: u64,
    event_id: &'a str,
    project: &'a str,
    environment: &'a str,
    signal: SignalKind,
    occurred_at: &'a str,
    actor: &'a str,
    subject: &'a Option<String>,
    action: &'a str,
    target: &'a Option<String>,
    actor_missing: bool,
    resource: &'a BTreeMap<String, String>,
    attributes: &'a BTreeMap<String, AttributeValue>,
    trace_id: &'a Option<String>,
    span_id: &'a Option<String>,
    request_id: &'a Option<String>,
    session_id: &'a Option<String>,
    payload: &'a serde_json::Value,
    previous_hash: &'a str,
    retention_expires_at: &'a str,
}

pub struct AuditChangeProjection {
    state: RwLock<AuditChangeState>,
    retention_days: i64,
}

impl AuditChangeProjection {
    pub fn new() -> Self {
        Self::with_retention_days(DEFAULT_AUDIT_RETENTION_DAYS)
            .expect("default audit retention is valid")
    }

    pub fn with_retention_days(retention_days: i64) -> Result<Self> {
        if retention_days <= 0 {
            bail!("audit retention days must be positive");
        }
        Ok(Self {
            state: RwLock::new(AuditChangeState::default()),
            retention_days,
        })
    }

    pub fn query(
        &self,
        query: &AuditQuery,
        holds: &[AuditLegalHoldV1],
        now: DateTime<Utc>,
    ) -> Result<AuditPage> {
        let (start, end) = query.validate()?;
        let state = self.state.read().expect("audit projection lock poisoned");
        let mut records = state
            .records
            .range((query.after_cursor.saturating_add(1))..)
            .filter(|(_, record)| record.project == query.project)
            .filter(|(_, record)| query.signal.is_none_or(|signal| record.signal == signal))
            .filter(|(_, record)| {
                query
                    .actor
                    .as_ref()
                    .is_none_or(|actor| &record.actor == actor)
            })
            .filter(|(_, record)| {
                query
                    .action
                    .as_ref()
                    .is_none_or(|action| &record.action == action)
            })
            .filter(|(_, record)| {
                query
                    .target
                    .as_ref()
                    .is_none_or(|target| record.target.as_ref() == Some(target))
            })
            .filter(|(_, record)| {
                query
                    .trace_id
                    .as_ref()
                    .is_none_or(|id| record.trace_id.as_ref() == Some(id))
            })
            .filter(|(_, record)| {
                query
                    .request_id
                    .as_ref()
                    .is_none_or(|id| record.request_id.as_ref() == Some(id))
            })
            .filter(|(_, record)| {
                query
                    .session_id
                    .as_ref()
                    .is_none_or(|id| record.session_id.as_ref() == Some(id))
            })
            .filter_map(|(_, record)| {
                let occurred = DateTime::parse_from_rfc3339(&record.occurred_at)
                    .ok()?
                    .with_timezone(&Utc);
                if start.is_some_and(|start| occurred < start)
                    || end.is_some_and(|end| occurred >= end)
                {
                    return None;
                }
                let held = holds.iter().any(|hold| hold_covers(hold, occurred));
                let expires = DateTime::parse_from_rfc3339(&record.retention_expires_at)
                    .ok()?
                    .with_timezone(&Utc);
                if now > expires && !held {
                    return None;
                }
                let mut record = record.clone();
                record.retained_by_hold = held;
                Some(record)
            })
            .take(query.limit + 1)
            .collect::<Vec<_>>();
        let has_more = records.len() > query.limit;
        records.truncate(query.limit);
        Ok(AuditPage {
            next_cursor: records
                .last()
                .map(|record| record.cursor)
                .unwrap_or(query.after_cursor),
            records,
            projection_cursor: state.records.keys().next_back().copied().unwrap_or(0),
            has_more,
            chain_valid: verify_state(&state).is_ok(),
        })
    }

    pub fn verify_integrity(&self) -> Result<()> {
        verify_state(&self.state.read().expect("audit projection lock poisoned"))
    }
}

impl Default for AuditChangeProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl Projection for AuditChangeProjection {
    fn descriptor(&self) -> ProjectionDescriptor {
        ProjectionDescriptor {
            name: PROJECTION_AUDIT_CHANGE_STORE.into(),
            schema_version: AUDIT_CHANGE_SCHEMA_VERSION,
            retention: format!(
                "{} days unless covered by active legal hold",
                self.retention_days
            ),
        }
    }

    fn apply_idempotent(&self, stored: &StoredEvent) -> Result<()> {
        if !matches!(
            stored.event.signal,
            SignalKind::AuditEvent | SignalKind::ChangeEvent
        ) {
            return Ok(());
        }
        let event = &stored.event;
        let mut state = self.state.write().expect("audit projection lock poisoned");
        if state.cursor_by_event_id.contains_key(&event.event_id) {
            return Ok(());
        }
        if state
            .last_cursor_by_project
            .get(&event.project)
            .is_some_and(|cursor| *cursor >= stored.cursor)
        {
            bail!("audit/change events must be applied in increasing project cursor order");
        }
        let actor = payload_string(&event.payload, &["actor", "principal", "user"]);
        let actor_missing = actor.is_none();
        let actor = actor.unwrap_or_else(|| "<unknown>".into());
        let subject = payload_string(&event.payload, &["subject", "subject_id"]);
        let action = payload_string(&event.payload, &["action", "kind"])
            .unwrap_or_else(|| event.signal.to_string());
        let target = payload_string(
            &event.payload,
            &["target", "target_id", "fingerprint", "version"],
        );
        let occurred = DateTime::parse_from_rfc3339(&event.occurred_at)
            .context("audit/change occurred_at must be RFC3339")?
            .with_timezone(&Utc);
        let retention_expires_at = (occurred + Duration::days(self.retention_days)).to_rfc3339();
        let previous_hash = state
            .last_hash_by_project
            .get(&event.project)
            .cloned()
            .unwrap_or_else(|| GENESIS_HASH.into());
        let mut record = AuditChangeRecordV1 {
            schema: AUDIT_RECORD_SCHEMA.into(),
            cursor: stored.cursor,
            event_id: event.event_id.clone(),
            project: event.project.clone(),
            environment: event.environment.clone(),
            signal: event.signal,
            occurred_at: event.occurred_at.clone(),
            actor,
            subject,
            action,
            target,
            actor_missing,
            resource: event.resource.clone(),
            attributes: event.attributes.clone(),
            trace_id: event.trace_id.clone(),
            span_id: event.span_id.clone(),
            request_id: event.request_id.clone(),
            session_id: event.session_id.clone(),
            payload: event.payload.clone(),
            previous_hash,
            record_hash: String::new(),
            retention_expires_at,
            retained_by_hold: false,
        };
        record.record_hash = record_hash(&record)?;
        state
            .cursor_by_event_id
            .insert(event.event_id.clone(), stored.cursor);
        state
            .last_hash_by_project
            .insert(event.project.clone(), record.record_hash.clone());
        state
            .last_cursor_by_project
            .insert(event.project.clone(), stored.cursor);
        state.records.insert(stored.cursor, record);
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&*self.state.read().expect("audit projection lock poisoned"))
            .map_err(Into::into)
    }

    fn restore(&self, bytes: &[u8]) -> Result<()> {
        let restored: AuditChangeState =
            serde_json::from_slice(bytes).context("decode audit/change projection snapshot")?;
        verify_state(&restored)?;
        *self.state.write().expect("audit projection lock poisoned") = restored;
        Ok(())
    }

    fn semantic_digest(&self) -> Result<String> {
        Ok(sha256(&self.snapshot()?))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn export_content_sha256(records: &[AuditChangeRecordV1]) -> Result<String> {
    Ok(sha256(&serde_json::to_vec(records)?))
}

fn verify_state(state: &AuditChangeState) -> Result<()> {
    let mut previous = BTreeMap::<String, String>::new();
    let mut cursors = BTreeMap::<String, u64>::new();
    for record in state.records.values() {
        if cursors
            .get(&record.project)
            .is_some_and(|cursor| *cursor >= record.cursor)
        {
            bail!(
                "audit chain cursor order is invalid for project {}",
                record.project
            );
        }
        let expected_previous = previous
            .get(&record.project)
            .map(String::as_str)
            .unwrap_or(GENESIS_HASH);
        if record.previous_hash != expected_previous {
            bail!(
                "audit chain previous hash mismatch at cursor {}",
                record.cursor
            );
        }
        if record_hash(record)? != record.record_hash {
            bail!("audit record hash mismatch at cursor {}", record.cursor);
        }
        cursors.insert(record.project.clone(), record.cursor);
        previous.insert(record.project.clone(), record.record_hash.clone());
    }
    if previous != state.last_hash_by_project || cursors != state.last_cursor_by_project {
        bail!("audit chain terminal indexes do not match records");
    }
    Ok(())
}

fn record_hash(record: &AuditChangeRecordV1) -> Result<String> {
    Ok(sha256(&serde_json::to_vec(&HashMaterial {
        schema: &record.schema,
        cursor: record.cursor,
        event_id: &record.event_id,
        project: &record.project,
        environment: &record.environment,
        signal: record.signal,
        occurred_at: &record.occurred_at,
        actor: &record.actor,
        subject: &record.subject,
        action: &record.action,
        target: &record.target,
        actor_missing: record.actor_missing,
        resource: &record.resource,
        attributes: &record.attributes,
        trace_id: &record.trace_id,
        span_id: &record.span_id,
        request_id: &record.request_id,
        session_id: &record.session_id,
        payload: &record.payload,
        previous_hash: &record.previous_hash,
        retention_expires_at: &record.retention_expires_at,
    })?))
}

fn hold_covers(hold: &AuditLegalHoldV1, occurred: DateTime<Utc>) -> bool {
    if !hold.active {
        return false;
    }
    DateTime::parse_from_rfc3339(&hold.start_time)
        .ok()
        .zip(DateTime::parse_from_rfc3339(&hold.end_time).ok())
        .is_some_and(|(start, end)| {
            let occurred = occurred.fixed_offset();
            occurred >= start && occurred < end
        })
}

fn payload_string(payload: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        payload.get(key).and_then(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .or_else(|| (!value.is_null()).then(|| value.to_string()))
        })
    })
}

fn parse_optional_time(name: &str, value: Option<&str>) -> Result<Option<DateTime<Utc>>> {
    value
        .map(|value| {
            DateTime::parse_from_rfc3339(value)
                .with_context(|| format!("{name} must be RFC3339"))
                .map(|value| value.with_timezone(&Utc))
        })
        .transpose()
}

fn sha256(bytes: impl AsRef<[u8]>) -> String {
    hex::encode(Sha256::digest(bytes.as_ref()))
}

const fn default_query_limit() -> usize {
    100
}
// HANDWRITE-END
