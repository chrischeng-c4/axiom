// HANDWRITE-BEGIN gap="sift-trace-projection" tracker="1665" reason="Define span/link/event schemas, trace topology, partial diagnostics, critical path, correlations, snapshot, and rebuild semantics."
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    sync::RwLock,
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;

use crate::{AttributeValue, InstrumentationScope, SignalKind, StoredEvent};

use super::{model::ProjectionDescriptor, runtime::Projection};

pub const PROJECTION_TRACE_STORE: &str = "trace-store";
pub const TRACE_SCHEMA_VERSION: u32 = 3;
pub const MAX_TRACE_QUERY_LIMIT: usize = 1_000;
pub const DEFAULT_RETAINED_TRACE_SPANS: usize = 100_000;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct SpanLinkV1 {
    pub trace_id: String,
    pub span_id: String,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub attributes: BTreeMap<String, AttributeValue>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct SpanEventV1 {
    pub name: String,
    pub time_unix_nano: u64,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub attributes: BTreeMap<String, AttributeValue>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct SpanRecordV1 {
    pub cursor: u64,
    pub event_id: String,
    pub project: String,
    pub environment: String,
    pub trace_id: String,
    pub span_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub start_time_unix_nano: u64,
    pub end_time_unix_nano: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    #[serde(default)]
    pub links: Vec<SpanLinkV1>,
    #[serde(default)]
    pub events: Vec<SpanEventV1>,
    pub resource: BTreeMap<String, String>,
    #[schema(value_type = Object)]
    pub attributes: BTreeMap<String, AttributeValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instrumentation_scope: Option<InstrumentationScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl SpanRecordV1 {
    fn duration(&self) -> u64 {
        self.end_time_unix_nano
            .saturating_sub(self.start_time_unix_nano)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct TraceResultV1 {
    pub project: String,
    pub trace_id: String,
    pub spans: Vec<SpanRecordV1>,
    pub root_span_ids: Vec<String>,
    pub partial: bool,
    pub gaps: Vec<String>,
    pub cycles: Vec<Vec<String>>,
    pub critical_path_span_ids: Vec<String>,
    pub duration_unix_nano: u64,
    pub correlation_ids: BTreeMap<String, Vec<String>>,
    pub projection_cursor: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct TraceQuery {
    pub project: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time_unix_nano: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time_unix_nano: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_duration_unix_nano: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_duration_unix_nano: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub attributes: BTreeMap<String, AttributeValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_trace_id: Option<String>,
    #[serde(default = "default_trace_query_limit")]
    pub limit: usize,
}

impl TraceQuery {
    pub fn for_project(project: impl Into<String>) -> Self {
        Self {
            project: project.into(),
            environment: None,
            start_time_unix_nano: None,
            end_time_unix_nano: None,
            service: None,
            operation: None,
            min_duration_unix_nano: None,
            max_duration_unix_nano: None,
            status: None,
            attributes: BTreeMap::new(),
            after_trace_id: None,
            limit: default_trace_query_limit(),
        }
    }

    fn validate(&self) -> Result<()> {
        if self.project.trim().is_empty() {
            bail!("project must not be empty");
        }
        if self.limit == 0 || self.limit > MAX_TRACE_QUERY_LIMIT {
            bail!("limit must be between 1 and {MAX_TRACE_QUERY_LIMIT}");
        }
        if self
            .start_time_unix_nano
            .zip(self.end_time_unix_nano)
            .is_some_and(|(start, end)| start >= end)
        {
            bail!("start_time_unix_nano must be earlier than end_time_unix_nano");
        }
        if self
            .min_duration_unix_nano
            .zip(self.max_duration_unix_nano)
            .is_some_and(|(min, max)| min > max)
        {
            bail!("minimum trace duration must not exceed maximum trace duration");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct TracePage {
    pub traces: Vec<TraceResultV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_trace_id: Option<String>,
    pub projection_cursor: u64,
    pub has_more: bool,
}

#[derive(Default, Deserialize, Serialize)]
struct TraceState {
    traces: BTreeMap<String, BTreeMap<String, SpanRecordV1>>,
    #[serde(default)]
    location_by_cursor: BTreeMap<u64, TraceLocation>,
    conflicts: BTreeMap<String, BTreeSet<String>>,
    #[serde(default)]
    projection_cursor: u64,
}

#[derive(Clone, Deserialize, Serialize)]
struct TraceLocation {
    trace_key: String,
    span_id: String,
}

#[derive(Default, Deserialize)]
struct SpanPayload {
    #[serde(default)]
    name: String,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default, alias = "parentSpanId")]
    parent_span_id: Option<String>,
    #[serde(default, alias = "startTimeUnixNano")]
    start_time_unix_nano: u64,
    #[serde(default, alias = "endTimeUnixNano")]
    end_time_unix_nano: u64,
    #[serde(default)]
    status: Option<SpanStatusPayload>,
    #[serde(default)]
    links: Vec<SpanLinkV1>,
    #[serde(default)]
    events: Vec<SpanEventV1>,
}

#[derive(Default, Deserialize)]
struct SpanStatusPayload {
    #[serde(default)]
    code: Option<serde_json::Value>,
    #[serde(default)]
    message: Option<String>,
}

pub struct TraceProjection {
    state: RwLock<TraceState>,
    max_spans: usize,
}

impl TraceProjection {
    pub fn new() -> Self {
        Self::with_max_spans(DEFAULT_RETAINED_TRACE_SPANS)
            .expect("default trace span retention is valid")
    }

    pub fn with_max_spans(max_spans: usize) -> Result<Self> {
        if max_spans == 0 {
            bail!("trace retention must keep at least one span");
        }
        Ok(Self {
            state: RwLock::new(TraceState::default()),
            max_spans,
        })
    }

    pub fn get_trace(&self, project: &str, trace_id: &str) -> Result<Option<TraceResultV1>> {
        if project.trim().is_empty() || trace_id.trim().is_empty() {
            bail!("project and trace_id must not be empty");
        }
        let state = self.state.read().expect("trace projection lock poisoned");
        let key = trace_key(project, trace_id);
        let Some(records) = state.traces.get(&key) else {
            return Ok(None);
        };
        let mut spans = records.values().cloned().collect::<Vec<_>>();
        spans.sort_by(|left, right| {
            left.start_time_unix_nano
                .cmp(&right.start_time_unix_nano)
                .then_with(|| left.span_id.cmp(&right.span_id))
                .then_with(|| left.cursor.cmp(&right.cursor))
        });
        let by_id = spans
            .iter()
            .map(|span| (span.span_id.clone(), span))
            .collect::<BTreeMap<_, _>>();
        let mut gaps = Vec::new();
        let mut roots = Vec::new();
        for span in &spans {
            match span.parent_span_id.as_deref() {
                None | Some("") => roots.push(span.span_id.clone()),
                Some(parent) if !by_id.contains_key(parent) => {
                    roots.push(span.span_id.clone());
                    gaps.push(format!("missing_parent:{}:{parent}", span.span_id));
                }
                Some(_) => {}
            }
        }
        if let Some(conflicts) = state.conflicts.get(&key) {
            gaps.extend(
                conflicts
                    .iter()
                    .map(|span| format!("conflicting_span:{span}")),
            );
        }
        roots.sort();
        gaps.sort();
        let cycles = detect_cycles(&by_id);
        let mut children = BTreeMap::<String, Vec<String>>::new();
        for span in &spans {
            if let Some(parent) = span
                .parent_span_id
                .as_ref()
                .filter(|id| by_id.contains_key(*id))
            {
                children
                    .entry(parent.clone())
                    .or_default()
                    .push(span.span_id.clone());
            }
        }
        for values in children.values_mut() {
            values.sort();
        }
        let candidates = if roots.is_empty() {
            by_id.keys().cloned().collect::<Vec<_>>()
        } else {
            roots.clone()
        };
        let critical_path_span_ids = candidates
            .iter()
            .map(|root| best_path(root, &by_id, &children, &mut HashSet::new()))
            .max_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)))
            .map(|(_, path)| path)
            .unwrap_or_default();
        let duration_unix_nano = spans
            .iter()
            .map(|span| span.end_time_unix_nano)
            .max()
            .zip(spans.iter().map(|span| span.start_time_unix_nano).min())
            .map(|(end, start)| end.saturating_sub(start))
            .unwrap_or(0);
        Ok(Some(TraceResultV1 {
            project: project.into(),
            trace_id: trace_id.into(),
            correlation_ids: correlations(&spans),
            spans,
            root_span_ids: roots.clone(),
            partial: !gaps.is_empty() || !cycles.is_empty() || roots.is_empty(),
            gaps,
            cycles,
            critical_path_span_ids,
            duration_unix_nano,
            projection_cursor: state.projection_cursor,
        }))
    }

    pub fn query(&self, query: &TraceQuery) -> Result<TracePage> {
        query.validate()?;
        let prefix = format!("{}\u{1f}", query.project);
        let trace_ids = {
            let state = self.state.read().expect("trace projection lock poisoned");
            state
                .traces
                .keys()
                .filter_map(|key| key.strip_prefix(&prefix))
                .filter(|trace_id| {
                    query
                        .after_trace_id
                        .as_deref()
                        .is_none_or(|after| *trace_id > after)
                })
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };
        let mut traces = Vec::new();
        for trace_id in trace_ids {
            let Some(trace) = self.get_trace(&query.project, &trace_id)? else {
                continue;
            };
            if trace_matches(&trace, query) {
                traces.push(trace);
                if traces.len() > query.limit {
                    break;
                }
            }
        }
        let has_more = traces.len() > query.limit;
        traces.truncate(query.limit);
        let projection_cursor = traces
            .iter()
            .map(|trace| trace.projection_cursor)
            .max()
            .unwrap_or_else(|| {
                self.state
                    .read()
                    .expect("trace projection lock poisoned")
                    .projection_cursor
            });
        Ok(TracePage {
            next_trace_id: traces.last().map(|trace| trace.trace_id.clone()),
            traces,
            projection_cursor,
            has_more,
        })
    }
}

impl Default for TraceProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl Projection for TraceProjection {
    fn descriptor(&self) -> ProjectionDescriptor {
        ProjectionDescriptor {
            name: PROJECTION_TRACE_STORE.into(),
            schema_version: TRACE_SCHEMA_VERSION,
            retention: "raw-journal-retention".into(),
        }
    }

    fn apply_idempotent(&self, stored: &StoredEvent) -> Result<()> {
        if stored.event.signal != SignalKind::Span {
            return Ok(());
        }
        let event = &stored.event;
        let trace_id = event
            .trace_id
            .as_deref()
            .context("span event requires trace_id")?;
        let span_id = event
            .span_id
            .as_deref()
            .context("span event requires span_id")?;
        let payload: SpanPayload = serde_json::from_value(event.payload.clone())
            .context("decode canonical span payload")?;
        if payload.end_time_unix_nano < payload.start_time_unix_nano {
            bail!("span end_time_unix_nano must not precede start_time_unix_nano");
        }
        let status_code = payload
            .status
            .as_ref()
            .and_then(|status| status.code.as_ref())
            .map(value_as_string);
        let status_message = payload.status.and_then(|status| status.message);
        let record = SpanRecordV1 {
            cursor: stored.cursor,
            event_id: event.event_id.clone(),
            project: event.project.clone(),
            environment: event.environment.clone(),
            trace_id: trace_id.into(),
            span_id: span_id.into(),
            parent_span_id: payload.parent_span_id.filter(|parent| !parent.is_empty()),
            name: if payload.name.is_empty() {
                span_id.into()
            } else {
                payload.name
            },
            kind: payload.kind,
            start_time_unix_nano: payload.start_time_unix_nano,
            end_time_unix_nano: payload.end_time_unix_nano,
            status_code,
            status_message,
            links: payload.links,
            events: payload.events,
            resource: event.resource.clone(),
            attributes: event.attributes.clone(),
            instrumentation_scope: event.instrumentation_scope.clone(),
            request_id: event.request_id.clone(),
            session_id: event.session_id.clone(),
        };
        let key = trace_key(&event.project, trace_id);
        let mut state = self.state.write().expect("trace projection lock poisoned");
        if state.projection_cursor >= stored.cursor {
            return Ok(());
        }
        state.projection_cursor = state.projection_cursor.max(stored.cursor);
        let conflicting = state
            .traces
            .get(&key)
            .and_then(|trace| trace.get(span_id))
            .is_some_and(|existing| existing != &record);
        if let Some(previous_cursor) = state
            .traces
            .get(&key)
            .and_then(|trace| trace.get(span_id))
            .map(|previous| previous.cursor)
        {
            remove_tracked_cursor(&mut state, previous_cursor);
        }
        if conflicting {
            state
                .conflicts
                .entry(key.clone())
                .or_default()
                .insert(span_id.into());
        }
        state
            .traces
            .entry(key.clone())
            .or_default()
            .insert(span_id.into(), record);
        state.location_by_cursor.insert(
            stored.cursor,
            TraceLocation {
                trace_key: key,
                span_id: span_id.into(),
            },
        );
        while state.location_by_cursor.len() > self.max_spans {
            let Some(oldest_cursor) = state.location_by_cursor.keys().next().copied() else {
                break;
            };
            remove_tracked_cursor(&mut state, oldest_cursor);
        }
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        let state = self.state.read().expect("trace projection lock poisoned");
        serde_json::to_vec(&*state).map_err(Into::into)
    }

    fn restore(&self, bytes: &[u8]) -> Result<()> {
        let mut state: TraceState =
            serde_json::from_slice(bytes).context("decode trace projection snapshot")?;
        let mut rows = Vec::new();
        for (trace_key, trace) in &state.traces {
            for (span_id, record) in trace {
                rows.push((
                    record.cursor,
                    TraceLocation {
                        trace_key: trace_key.clone(),
                        span_id: span_id.clone(),
                    },
                ));
            }
        }
        state.location_by_cursor.clear();
        for (cursor, location) in rows {
            state.projection_cursor = state.projection_cursor.max(cursor);
            state.location_by_cursor.insert(cursor, location);
        }
        while state.location_by_cursor.len() > self.max_spans {
            let Some(oldest_cursor) = state.location_by_cursor.keys().next().copied() else {
                break;
            };
            remove_tracked_cursor(&mut state, oldest_cursor);
        }
        *self.state.write().expect("trace projection lock poisoned") = state;
        Ok(())
    }

    fn semantic_digest(&self) -> Result<String> {
        Ok(hex::encode(Sha256::digest(self.snapshot()?)))
    }
}

fn trace_key(project: &str, trace_id: &str) -> String {
    format!("{project}\u{1f}{trace_id}")
}

fn remove_tracked_cursor(state: &mut TraceState, cursor: u64) {
    let Some(location) = state.location_by_cursor.remove(&cursor) else {
        return;
    };
    let mut remove_trace = false;
    if let Some(trace) = state.traces.get_mut(&location.trace_key) {
        if trace
            .get(&location.span_id)
            .is_some_and(|record| record.cursor == cursor)
        {
            trace.remove(&location.span_id);
        }
        remove_trace = trace.is_empty();
    }
    if remove_trace {
        state.traces.remove(&location.trace_key);
        state.conflicts.remove(&location.trace_key);
        return;
    }
    let remove_conflicts = if let Some(conflicts) = state.conflicts.get_mut(&location.trace_key) {
        conflicts.remove(&location.span_id);
        conflicts.is_empty()
    } else {
        false
    };
    if remove_conflicts {
        state.conflicts.remove(&location.trace_key);
    }
}

fn default_trace_query_limit() -> usize {
    100
}

fn trace_matches(trace: &TraceResultV1, query: &TraceQuery) -> bool {
    if query
        .min_duration_unix_nano
        .is_some_and(|minimum| trace.duration_unix_nano < minimum)
        || query
            .max_duration_unix_nano
            .is_some_and(|maximum| trace.duration_unix_nano > maximum)
    {
        return false;
    }
    let spans = trace.spans.iter().filter(|span| {
        query
            .environment
            .as_deref()
            .is_none_or(|environment| span.environment == environment)
            && query
                .start_time_unix_nano
                .is_none_or(|start| span.end_time_unix_nano >= start)
            && query
                .end_time_unix_nano
                .is_none_or(|end| span.start_time_unix_nano < end)
    });
    let spans = spans.collect::<Vec<_>>();
    if spans.is_empty() {
        return false;
    }
    query.service.as_deref().is_none_or(|service| {
        spans
            .iter()
            .any(|span| span.resource.get("service.name").map(String::as_str) == Some(service))
    }) && query
        .operation
        .as_deref()
        .is_none_or(|operation| spans.iter().any(|span| span.name == operation))
        && query.status.as_deref().is_none_or(|status| {
            spans.iter().any(|span| {
                span.status_code
                    .as_deref()
                    .is_some_and(|actual| actual.eq_ignore_ascii_case(status))
            })
        })
        && query.attributes.iter().all(|(key, value)| {
            spans
                .iter()
                .any(|span| span.attributes.get(key) == Some(value))
        })
}

fn detect_cycles(by_id: &BTreeMap<String, &SpanRecordV1>) -> Vec<Vec<String>> {
    let mut unique = BTreeMap::<String, Vec<String>>::new();
    for start in by_id.keys() {
        let mut positions = HashMap::<String, usize>::new();
        let mut path = Vec::<String>::new();
        let mut current = start.clone();
        loop {
            if let Some(position) = positions.get(&current).copied() {
                let mut cycle = path[position..].to_vec();
                cycle.sort();
                cycle.dedup();
                unique.entry(cycle.join("\u{1f}")).or_insert(cycle);
                break;
            }
            positions.insert(current.clone(), path.len());
            path.push(current.clone());
            let Some(parent) = by_id
                .get(&current)
                .and_then(|span| span.parent_span_id.as_ref())
                .filter(|parent| by_id.contains_key(*parent))
            else {
                break;
            };
            current = parent.clone();
        }
    }
    unique.into_values().collect()
}

fn best_path(
    span_id: &str,
    by_id: &BTreeMap<String, &SpanRecordV1>,
    children: &BTreeMap<String, Vec<String>>,
    visiting: &mut HashSet<String>,
) -> (u128, Vec<String>) {
    let Some(span) = by_id.get(span_id) else {
        return (0, Vec::new());
    };
    if !visiting.insert(span_id.into()) {
        return (0, Vec::new());
    }
    let child = children
        .get(span_id)
        .into_iter()
        .flatten()
        .map(|child| best_path(child, by_id, children, visiting))
        .max_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)))
        .unwrap_or_default();
    visiting.remove(span_id);
    let mut path = Vec::with_capacity(child.1.len() + 1);
    path.push(span_id.into());
    path.extend(child.1);
    (u128::from(span.duration()) + child.0, path)
}

fn correlations(spans: &[SpanRecordV1]) -> BTreeMap<String, Vec<String>> {
    let mut values = BTreeMap::<String, BTreeSet<String>>::new();
    for span in spans {
        values
            .entry("span_ids".into())
            .or_default()
            .insert(span.span_id.clone());
        if let Some(request_id) = &span.request_id {
            values
                .entry("request_ids".into())
                .or_default()
                .insert(request_id.clone());
        }
        if let Some(session_id) = &span.session_id {
            values
                .entry("session_ids".into())
                .or_default()
                .insert(session_id.clone());
        }
        for link in &span.links {
            values
                .entry("linked_trace_ids".into())
                .or_default()
                .insert(link.trace_id.clone());
            values
                .entry("linked_span_ids".into())
                .or_default()
                .insert(link.span_id.clone());
        }
    }
    values
        .into_iter()
        .map(|(key, values)| (key, values.into_iter().collect()))
        .collect()
}

fn value_as_string(value: &serde_json::Value) -> String {
    value
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| value.to_string())
}

// HANDWRITE-END
