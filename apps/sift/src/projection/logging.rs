// HANDWRITE-BEGIN gap="sift-logging-projection" tracker="1664" reason="Define the log record/query/page schema, fixed-field embedded Lumen index, retention, snapshot, restore, and typed query behavior."
use std::{
    collections::{BTreeMap, HashSet},
    sync::RwLock,
};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use index_text::{
    Analyzer, FieldSpec, MatchOperator, MemoryTextIndex, TextDocument, TextIndex,
    TextIndexSnapshot, TextQuery, TextSchema,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;

use crate::{AttributeValue, SignalKind, StoredEvent};

use super::{model::ProjectionDescriptor, runtime::Projection};

pub const PROJECTION_LOGGING_STORE: &str = "logging-store";
pub const LOGGING_SCHEMA_VERSION: u32 = 3;
pub const DEFAULT_RETAINED_LOG_RECORDS: usize = 100_000;
pub const MAX_LOG_QUERY_LIMIT: usize = 1_000;

const RESOURCE_TYPE: &str = "gcp.resource.type";
const SERVICE_NAME: &str = "service.name";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct LogRecordV1 {
    pub cursor: u64,
    pub event_id: String,
    pub project: String,
    pub environment: String,
    pub occurred_at: String,
    pub observed_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    pub body_text: String,
    #[schema(value_type = Object)]
    pub json_payload: serde_json::Value,
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
    pub coexistence_key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct LogQuery {
    pub project: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub attribute_equals: BTreeMap<String, AttributeValue>,
    #[serde(default)]
    pub after_cursor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_cursor: Option<u64>,
    #[serde(default = "default_query_limit")]
    pub limit: usize,
}

impl LogQuery {
    pub fn for_project(project: impl Into<String>) -> Self {
        Self {
            project: project.into(),
            environment: None,
            start_time: None,
            end_time: None,
            severity: None,
            resource_type: None,
            service_name: None,
            trace_id: None,
            span_id: None,
            request_id: None,
            session_id: None,
            text: None,
            attribute_equals: BTreeMap::new(),
            after_cursor: 0,
            min_cursor: None,
            limit: default_query_limit(),
        }
    }

    fn validate(&self) -> Result<QueryBounds> {
        if self.project.trim().is_empty() {
            bail!("project must not be empty");
        }
        if self.limit == 0 || self.limit > MAX_LOG_QUERY_LIMIT {
            bail!("limit must be between 1 and {MAX_LOG_QUERY_LIMIT}");
        }
        let start = parse_optional_time("start_time", self.start_time.as_deref())?;
        let end = parse_optional_time("end_time", self.end_time.as_deref())?;
        if start.zip(end).is_some_and(|(start, end)| start >= end) {
            bail!("start_time must be earlier than end_time");
        }
        Ok(QueryBounds { start, end })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct LogPage {
    pub records: Vec<LogRecordV1>,
    pub next_cursor: u64,
    pub projection_cursor: u64,
    pub has_more: bool,
}

#[derive(Default)]
struct LoggingState {
    records: BTreeMap<u64, LogRecordV1>,
    projection_cursor: u64,
}

#[derive(Deserialize, Serialize)]
struct LoggingSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    text_index: Option<TextIndexSnapshot>,
    records: BTreeMap<u64, LogRecordV1>,
    projection_cursor: u64,
    max_records: usize,
}

#[derive(Serialize)]
struct SemanticState<'a> {
    records: &'a BTreeMap<u64, LogRecordV1>,
    projection_cursor: u64,
    max_records: usize,
}

struct QueryBounds {
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
}

pub struct LoggingProjection {
    text_index: MemoryTextIndex,
    state: RwLock<LoggingState>,
    max_records: usize,
}

impl LoggingProjection {
    pub fn new() -> Result<Self> {
        Self::with_max_records(DEFAULT_RETAINED_LOG_RECORDS)
    }

    pub fn with_max_records(max_records: usize) -> Result<Self> {
        if max_records == 0 {
            bail!("logging retention must keep at least one record");
        }
        let text_index =
            MemoryTextIndex::new(fixed_schema()?).context("create shared logging text index")?;
        Ok(Self {
            text_index,
            state: RwLock::new(LoggingState::default()),
            max_records,
        })
    }

    pub fn query(&self, query: &LogQuery) -> Result<LogPage> {
        let bounds = query.validate()?;
        let candidates = match query.text.as_deref().map(str::trim) {
            Some(text) if !text.is_empty() => Some(
                self.text_index
                    .search(
                        &TextQuery::match_text("body", text, MatchOperator::All),
                        self.max_records,
                    )?
                    .into_iter()
                    .map(|hit| hit.external_id)
                    .collect::<HashSet<_>>(),
            ),
            _ => None,
        };

        let state = self
            .state
            .read()
            .expect("logging projection state lock poisoned");
        let projection_cursor = state.records.keys().next_back().copied().unwrap_or(0);
        let mut matching = state
            .records
            .range((query.after_cursor.saturating_add(1))..)
            .filter(|(_, record)| record_matches(record, query, &bounds, candidates.as_ref()))
            .map(|(_, record)| record.clone())
            .take(query.limit + 1)
            .collect::<Vec<_>>();
        let has_more = matching.len() > query.limit;
        matching.truncate(query.limit);
        let next_cursor = matching
            .last()
            .map(|record| record.cursor)
            .unwrap_or(query.after_cursor);
        Ok(LogPage {
            records: matching,
            next_cursor,
            projection_cursor,
            has_more,
        })
    }

    fn index(&self, record: &LogRecordV1) -> Result<()> {
        self.text_index.upsert(index_document(record))?;
        Ok(())
    }
}

impl Projection for LoggingProjection {
    fn descriptor(&self) -> ProjectionDescriptor {
        ProjectionDescriptor {
            name: PROJECTION_LOGGING_STORE.into(),
            schema_version: LOGGING_SCHEMA_VERSION,
            retention: format!("latest-{0}-records", self.max_records),
        }
    }

    fn apply_idempotent(&self, stored: &StoredEvent) -> Result<()> {
        if stored.event.signal != SignalKind::Log {
            return Ok(());
        }
        let record = normalize(stored);
        if self
            .state
            .read()
            .expect("logging projection state lock poisoned")
            .projection_cursor
            >= record.cursor
        {
            return Ok(());
        }
        self.index(&record)?;
        let mut state = self
            .state
            .write()
            .expect("logging projection state lock poisoned");
        state.projection_cursor = state.projection_cursor.max(record.cursor);
        state.records.insert(record.cursor, record);
        while state.records.len() > self.max_records {
            let Some(oldest) = state.records.keys().next().copied() else {
                break;
            };
            self.text_index.delete(&projection_row_key(oldest), None)?;
            state.records.remove(&oldest);
        }
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        let state = self
            .state
            .read()
            .expect("logging projection state lock poisoned");
        canonical_json(&LoggingSnapshot {
            text_index: Some(self.text_index.snapshot()?),
            records: state.records.clone(),
            projection_cursor: state.projection_cursor,
            max_records: self.max_records,
        })
    }

    fn restore(&self, bytes: &[u8]) -> Result<()> {
        let snapshot: LoggingSnapshot =
            serde_json::from_slice(bytes).context("decode logging projection snapshot")?;
        if snapshot.max_records != self.max_records {
            bail!(
                "logging snapshot retention {} does not match configured {}",
                snapshot.max_records,
                self.max_records
            );
        }
        match snapshot.text_index {
            Some(text_index) if self.text_index.restore(&text_index).is_ok() => {}
            _ => self
                .text_index
                .rebuild(snapshot.records.values().map(index_document).collect())
                .context("rebuild logging index from retained projection records")?,
        }
        *self
            .state
            .write()
            .expect("logging projection state lock poisoned") = LoggingState {
            records: snapshot.records,
            projection_cursor: snapshot.projection_cursor,
        };
        Ok(())
    }

    fn semantic_digest(&self) -> Result<String> {
        let state = self
            .state
            .read()
            .expect("logging projection state lock poisoned");
        Ok(hex::encode(Sha256::digest(serde_json::to_vec(
            &SemanticState {
                records: &state.records,
                projection_cursor: state.projection_cursor,
                max_records: self.max_records,
            },
        )?)))
    }
}

fn normalize(stored: &StoredEvent) -> LogRecordV1 {
    let event = &stored.event;
    let json_payload = event
        .payload
        .get("jsonPayload")
        .cloned()
        .unwrap_or_else(|| event.payload.clone());
    let body_text = event
        .attributes
        .get("otel.log.body")
        .and_then(AttributeValue::as_str)
        .map(str::to_owned)
        .or_else(|| {
            json_payload
                .get("message")
                .or_else(|| json_payload.get("body"))
                .or_else(|| event.payload.get("body"))
                .or_else(|| event.payload.get("message"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| json_payload.to_string());
    LogRecordV1 {
        cursor: stored.cursor,
        event_id: event.event_id.clone(),
        project: event.project.clone(),
        environment: event.environment.clone(),
        occurred_at: event.occurred_at.clone(),
        observed_at: event.observed_at.clone(),
        severity: event.severity.clone(),
        body_text,
        json_payload,
        resource: event.resource.clone(),
        attributes: event.attributes.clone(),
        trace_id: event.trace_id.clone(),
        span_id: event.span_id.clone(),
        request_id: event.request_id.clone(),
        session_id: event.session_id.clone(),
        coexistence_key: format!("{}:{}", event.project, event.event_id),
    }
}

fn record_matches(
    record: &LogRecordV1,
    query: &LogQuery,
    bounds: &QueryBounds,
    candidates: Option<&HashSet<String>>,
) -> bool {
    if record.project != query.project
        || query
            .environment
            .as_ref()
            .is_some_and(|value| record.environment != *value)
        || query.severity.as_ref().is_some_and(|value| {
            !record
                .severity
                .as_deref()
                .is_some_and(|severity| severity.eq_ignore_ascii_case(value))
        })
        || query
            .resource_type
            .as_ref()
            .is_some_and(|value| record.resource.get(RESOURCE_TYPE) != Some(value))
        || query
            .service_name
            .as_ref()
            .is_some_and(|value| record.resource.get(SERVICE_NAME) != Some(value))
        || !matches_optional(&record.trace_id, &query.trace_id)
        || !matches_optional(&record.span_id, &query.span_id)
        || !matches_optional(&record.request_id, &query.request_id)
        || !matches_optional(&record.session_id, &query.session_id)
        || !query
            .attribute_equals
            .iter()
            .all(|(key, value)| record.attributes.get(key) == Some(value))
        || candidates.is_some_and(|ids| !ids.contains(&projection_row_key(record.cursor)))
    {
        return false;
    }
    let Ok(occurred_at) = DateTime::parse_from_rfc3339(&record.occurred_at) else {
        return false;
    };
    let occurred_at = occurred_at.with_timezone(&Utc);
    bounds.start.is_none_or(|start| occurred_at >= start)
        && bounds.end.is_none_or(|end| occurred_at < end)
}

fn matches_optional(actual: &Option<String>, expected: &Option<String>) -> bool {
    expected
        .as_ref()
        .is_none_or(|value| actual.as_ref() == Some(value))
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

fn default_query_limit() -> usize {
    100
}

fn fixed_schema() -> Result<TextSchema> {
    let mut fields = BTreeMap::new();
    fields.insert("body".into(), FieldSpec::text(Analyzer::WhitespaceLower));
    for name in [
        "project",
        "environment",
        "severity",
        "resource_type",
        "service_name",
        "trace_id",
        "span_id",
        "request_id",
        "session_id",
        "occurred_at",
        "coexistence_key",
    ] {
        fields.insert(name.into(), FieldSpec::keyword());
    }
    TextSchema::new(fields).map_err(Into::into)
}

fn index_document(record: &LogRecordV1) -> TextDocument {
    let mut document = TextDocument::new(projection_row_key(record.cursor), record.cursor)
        .with_field("body", &record.body_text)
        .with_field("project", &record.project)
        .with_field("environment", &record.environment)
        .with_field("occurred_at", &record.occurred_at)
        .with_field("coexistence_key", &record.coexistence_key);
    for (field, value) in [
        ("severity", record.severity.as_deref()),
        (
            "resource_type",
            record.resource.get(RESOURCE_TYPE).map(String::as_str),
        ),
        (
            "service_name",
            record.resource.get(SERVICE_NAME).map(String::as_str),
        ),
        ("trace_id", record.trace_id.as_deref()),
        ("span_id", record.span_id.as_deref()),
        ("request_id", record.request_id.as_deref()),
        ("session_id", record.session_id.as_deref()),
    ] {
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            document = document.with_field(field, value);
        }
    }
    document
}

fn projection_row_key(cursor: u64) -> String {
    format!("cursor-{cursor:020}")
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let value: serde_json::Value = serde_json::from_slice(&serde_json::to_vec(value)?)?;
    serde_json::to_vec(&value).map_err(Into::into)
}

// HANDWRITE-END
