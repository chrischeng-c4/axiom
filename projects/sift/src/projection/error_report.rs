// HANDWRITE-BEGIN gap="sift-error-report-projection" tracker="1666" reason="Define exception normalization, fingerprints, groups, occurrences, query, snapshot, and rebuild semantics."
use std::{
    any::Any,
    collections::{BTreeMap, BTreeSet},
    sync::RwLock,
};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;

use crate::{AttributeValue, SignalKind, StoredEvent};

use super::{
    model::{ErrorLifecycleState, ErrorLifecycleV1, ProjectionDescriptor},
    runtime::Projection,
};

pub const PROJECTION_ERROR_REPORT_STORE: &str = "error-report-store";
pub const ERROR_REPORT_SCHEMA_VERSION: u32 = 1;
pub const ERROR_FINGERPRINT_VERSION: &str = "sift.error.fingerprint.v1";
pub const MAX_ERROR_QUERY_LIMIT: usize = 1_000;
const DEFAULT_ERROR_QUERY_LIMIT: usize = 100;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct ErrorOccurrenceV1 {
    pub cursor: u64,
    pub event_id: String,
    pub occurred_at: String,
    pub environment: String,
    pub exception_type: String,
    pub message: String,
    pub stacktrace: String,
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
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct ErrorGroupV1 {
    pub project: String,
    pub fingerprint: String,
    pub fingerprint_version: String,
    pub state: ErrorLifecycleState,
    pub reopened: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub muted_until: Option<String>,
    pub first_seen: String,
    pub last_seen: String,
    pub first_cursor: u64,
    pub last_cursor: u64,
    pub occurrence_count: u64,
    pub exception_types: Vec<String>,
    pub sample_message: String,
    pub occurrences: Vec<ErrorOccurrenceV1>,
    pub correlations: BTreeMap<String, Vec<String>>,
    pub projection_cursor: u64,
}

impl ErrorGroupV1 {
    pub fn apply_lifecycle(
        mut self,
        lifecycle: Option<&ErrorLifecycleV1>,
        now: DateTime<Utc>,
    ) -> Self {
        let Some(lifecycle) = lifecycle else {
            return self;
        };
        self.state = lifecycle.state;
        self.muted_until = lifecycle.muted_until.clone();
        if lifecycle.state == ErrorLifecycleState::Resolved
            && self.last_cursor > lifecycle.occurrence_cursor
        {
            self.state = ErrorLifecycleState::Open;
            self.reopened = true;
            self.muted_until = None;
        } else if lifecycle.state == ErrorLifecycleState::Muted
            && lifecycle
                .muted_until
                .as_deref()
                .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
                .is_some_and(|until| until.with_timezone(&Utc) <= now)
        {
            self.state = ErrorLifecycleState::Open;
            self.muted_until = None;
        }
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct ErrorQuery {
    pub project: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<ErrorLifecycleState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default)]
    pub after_cursor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_cursor: Option<u64>,
    #[serde(default = "default_query_limit")]
    pub limit: usize,
}

impl ErrorQuery {
    pub fn for_project(project: impl Into<String>) -> Self {
        Self {
            project: project.into(),
            state: None,
            trace_id: None,
            session_id: None,
            text: None,
            after_cursor: 0,
            min_cursor: None,
            limit: default_query_limit(),
        }
    }

    fn validate(&self) -> Result<()> {
        if self.project.trim().is_empty() {
            bail!("project must not be empty");
        }
        if self.limit == 0 || self.limit > MAX_ERROR_QUERY_LIMIT {
            bail!("limit must be between 1 and {MAX_ERROR_QUERY_LIMIT}");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct ErrorPage {
    pub groups: Vec<ErrorGroupV1>,
    pub next_cursor: u64,
    pub projection_cursor: u64,
    pub has_more: bool,
}

#[derive(Default, Deserialize, Serialize)]
struct ErrorReportState {
    groups: BTreeMap<String, ErrorGroupV1>,
    cursor_by_event_id: BTreeMap<String, u64>,
}

#[derive(Default, Deserialize)]
struct ExceptionPayload {
    #[serde(default, alias = "exception.type", alias = "type")]
    exception_type: String,
    #[serde(default, alias = "exception.message", alias = "message")]
    message: String,
    #[serde(
        default,
        alias = "exception.stacktrace",
        alias = "stacktrace",
        alias = "stack"
    )]
    stacktrace: String,
}

pub struct ErrorReportProjection {
    state: RwLock<ErrorReportState>,
}

impl ErrorReportProjection {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(ErrorReportState::default()),
        }
    }

    pub fn query(&self, query: &ErrorQuery) -> Result<ErrorPage> {
        query.validate()?;
        let state = self.state.read().expect("error projection lock poisoned");
        let projection_cursor = state
            .cursor_by_event_id
            .values()
            .copied()
            .max()
            .unwrap_or(0);
        let text = query.text.as_deref().map(|value| value.to_lowercase());
        let mut groups = state
            .groups
            .values()
            .filter(|group| {
                group.project == query.project && group.last_cursor > query.after_cursor
            })
            .filter(|group| {
                query.trace_id.as_ref().is_none_or(|trace_id| {
                    group
                        .correlations
                        .get("trace_ids")
                        .is_some_and(|values| values.contains(trace_id))
                })
            })
            .filter(|group| {
                query.session_id.as_ref().is_none_or(|session_id| {
                    group
                        .correlations
                        .get("session_ids")
                        .is_some_and(|values| values.contains(session_id))
                })
            })
            .filter(|group| {
                text.as_ref().is_none_or(|text| {
                    group.sample_message.to_lowercase().contains(text)
                        || group
                            .exception_types
                            .iter()
                            .any(|value| value.to_lowercase().contains(text))
                })
            })
            .cloned()
            .collect::<Vec<_>>();
        groups.sort_by(|left, right| {
            left.last_cursor
                .cmp(&right.last_cursor)
                .then_with(|| left.fingerprint.cmp(&right.fingerprint))
        });
        let has_more = groups.len() > query.limit;
        groups.truncate(query.limit);
        let next_cursor = groups
            .last()
            .map(|group| group.last_cursor)
            .unwrap_or(query.after_cursor);
        Ok(ErrorPage {
            groups,
            next_cursor,
            projection_cursor,
            has_more,
        })
    }

    pub fn get_group(&self, project: &str, fingerprint: &str) -> Result<Option<ErrorGroupV1>> {
        if project.trim().is_empty() || fingerprint.trim().is_empty() {
            bail!("project and fingerprint must not be empty");
        }
        Ok(self
            .state
            .read()
            .expect("error projection lock poisoned")
            .groups
            .get(&group_key(project, fingerprint))
            .cloned())
    }
}

impl Default for ErrorReportProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl Projection for ErrorReportProjection {
    fn descriptor(&self) -> ProjectionDescriptor {
        ProjectionDescriptor {
            name: PROJECTION_ERROR_REPORT_STORE.into(),
            schema_version: ERROR_REPORT_SCHEMA_VERSION,
            retention: "raw-journal-retention; occurrence-detail project-policy".into(),
        }
    }

    fn apply_idempotent(&self, stored: &StoredEvent) -> Result<()> {
        if stored.event.signal != SignalKind::Exception {
            return Ok(());
        }
        let event = &stored.event;
        let payload = decode_payload(&event.payload)?;
        let fingerprint = fingerprint(
            &payload.exception_type,
            &payload.message,
            &payload.stacktrace,
        );
        let occurrence = ErrorOccurrenceV1 {
            cursor: stored.cursor,
            event_id: event.event_id.clone(),
            occurred_at: event.occurred_at.clone(),
            environment: event.environment.clone(),
            exception_type: payload.exception_type.clone(),
            message: payload.message.clone(),
            stacktrace: payload.stacktrace,
            resource: event.resource.clone(),
            attributes: event.attributes.clone(),
            trace_id: event.trace_id.clone(),
            span_id: event.span_id.clone(),
            request_id: event.request_id.clone(),
            session_id: event.session_id.clone(),
        };
        let mut state = self.state.write().expect("error projection lock poisoned");
        if state
            .cursor_by_event_id
            .get(&event.event_id)
            .is_some_and(|cursor| *cursor >= stored.cursor)
        {
            return Ok(());
        }
        let key = group_key(&event.project, &fingerprint);
        let group = state.groups.entry(key).or_insert_with(|| ErrorGroupV1 {
            project: event.project.clone(),
            fingerprint: fingerprint.clone(),
            fingerprint_version: ERROR_FINGERPRINT_VERSION.into(),
            state: ErrorLifecycleState::Open,
            reopened: false,
            muted_until: None,
            first_seen: event.occurred_at.clone(),
            last_seen: event.occurred_at.clone(),
            first_cursor: stored.cursor,
            last_cursor: stored.cursor,
            occurrence_count: 0,
            exception_types: Vec::new(),
            sample_message: payload.message.clone(),
            occurrences: Vec::new(),
            correlations: BTreeMap::new(),
            projection_cursor: stored.cursor,
        });
        group.first_cursor = group.first_cursor.min(stored.cursor);
        group.last_cursor = group.last_cursor.max(stored.cursor);
        if event.occurred_at < group.first_seen {
            group.first_seen.clone_from(&event.occurred_at);
        }
        if event.occurred_at > group.last_seen {
            group.last_seen.clone_from(&event.occurred_at);
            group.sample_message.clone_from(&payload.message);
        }
        group.occurrence_count += 1;
        if !group.exception_types.contains(&payload.exception_type) {
            group.exception_types.push(payload.exception_type);
            group.exception_types.sort();
        }
        group.occurrences.push(occurrence);
        group.occurrences.sort_by(|left, right| {
            left.cursor
                .cmp(&right.cursor)
                .then_with(|| left.event_id.cmp(&right.event_id))
        });
        group.correlations = correlations(&group.occurrences);
        group.projection_cursor = stored.cursor;
        state
            .cursor_by_event_id
            .insert(event.event_id.clone(), stored.cursor);
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&*self.state.read().expect("error projection lock poisoned"))
            .map_err(Into::into)
    }

    fn restore(&self, bytes: &[u8]) -> Result<()> {
        *self.state.write().expect("error projection lock poisoned") =
            serde_json::from_slice(bytes).context("decode error-report projection snapshot")?;
        Ok(())
    }

    fn semantic_digest(&self) -> Result<String> {
        Ok(hex::encode(Sha256::digest(self.snapshot()?)))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn decode_payload(value: &serde_json::Value) -> Result<ExceptionPayload> {
    let payload: ExceptionPayload =
        serde_json::from_value(value.clone()).context("decode canonical exception payload")?;
    if payload.exception_type.trim().is_empty() {
        bail!("exception payload requires exception.type or type");
    }
    Ok(payload)
}

pub fn fingerprint(exception_type: &str, message: &str, stacktrace: &str) -> String {
    let material = format!(
        "{ERROR_FINGERPRINT_VERSION}\n{}\n{}\n{}",
        normalize(exception_type),
        normalize(message),
        application_frames(stacktrace).join("\n")
    );
    hex::encode(Sha256::digest(material.as_bytes()))
}

fn normalize(value: &str) -> String {
    let uuid = Regex::new(
        r"(?i)\b[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}\b",
    )
    .expect("valid UUID regex");
    let hex = Regex::new(r"(?i)\b(?:0x)?[0-9a-f]{12,}\b").expect("valid hex regex");
    let number = Regex::new(r"\b\d+\b").expect("valid number regex");
    let whitespace = Regex::new(r"\s+").expect("valid whitespace regex");
    let value = uuid.replace_all(value, "<id>");
    let value = hex.replace_all(&value, "<id>");
    let value = number.replace_all(&value, "<n>");
    whitespace.replace_all(value.trim(), " ").to_lowercase()
}

fn application_frames(stacktrace: &str) -> Vec<String> {
    stacktrace
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| {
            let lower = line.to_lowercase();
            !["std::", "core::", "tokio::", "runtime.goexit", "java.base/"]
                .iter()
                .any(|marker| lower.contains(marker))
        })
        .take(8)
        .map(normalize)
        .collect()
}

fn correlations(occurrences: &[ErrorOccurrenceV1]) -> BTreeMap<String, Vec<String>> {
    let mut values = BTreeMap::<String, BTreeSet<String>>::new();
    for occurrence in occurrences {
        for (name, value) in [
            ("trace_ids", occurrence.trace_id.as_ref()),
            ("span_ids", occurrence.span_id.as_ref()),
            ("request_ids", occurrence.request_id.as_ref()),
            ("session_ids", occurrence.session_id.as_ref()),
        ] {
            if let Some(value) = value {
                values.entry(name.into()).or_default().insert(value.clone());
            }
        }
    }
    values
        .into_iter()
        .map(|(name, values)| (name, values.into_iter().collect()))
        .collect()
}

fn group_key(project: &str, fingerprint: &str) -> String {
    format!("{project}\u{1f}{fingerprint}")
}

const fn default_query_limit() -> usize {
    DEFAULT_ERROR_QUERY_LIMIT
}
// HANDWRITE-END
