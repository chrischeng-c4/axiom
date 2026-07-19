// HANDWRITE-BEGIN gap="sift-profile-projection" tracker="1669" reason="Define blob-backed profile records, OTel normalization, retention, typed analysis, snapshot, and rebuild semantics."
use std::{
    any::Any,
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, RwLock},
};

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::{
    AttributeValue, ContentBlobRef, DurableJournal, InstrumentationScope, SignalKind, StoredEvent,
};

use super::{model::ProjectionDescriptor, runtime::Projection};

pub const PROJECTION_PROFILE_STORE: &str = "profile-store";
pub const PROFILE_SCHEMA_VERSION: u32 = 1;
pub const PROFILE_RECORD_SCHEMA: &str = "sift.profile.v1";
pub const DEFAULT_PROFILE_RETENTION_DAYS: i64 = 30;
pub const MAX_PROFILE_QUERY_LIMIT: usize = 1_000;
pub const MAX_PROFILE_TOP_LIMIT: usize = 1_000;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct ProfileMappingV1 {
    pub id: usize,
    pub memory_start: u64,
    pub memory_limit: u64,
    pub file_offset: u64,
    pub filename: String,
    #[schema(value_type = Object)]
    pub attributes: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct ProfileFunctionV1 {
    pub id: usize,
    pub name: String,
    pub system_name: String,
    pub filename: String,
    pub start_line: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct ProfileLineV1 {
    pub function_id: usize,
    pub line: i64,
    pub column: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct ProfileLocationV1 {
    pub id: usize,
    pub mapping_id: usize,
    pub address: u64,
    pub lines: Vec<ProfileLineV1>,
    #[schema(value_type = Object)]
    pub attributes: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct ProfileStackSampleV1 {
    pub frames: Vec<String>,
    pub value: f64,
    #[schema(value_type = Object)]
    pub labels: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct ProfileRecordV1 {
    pub schema: String,
    pub cursor: u64,
    pub event_id: String,
    pub profile_id: String,
    pub project: String,
    pub environment: String,
    pub occurred_at: String,
    pub start_time: String,
    pub end_time: String,
    pub sample_type: String,
    pub unit: String,
    pub period: i64,
    pub resource: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instrumentation_scope: Option<InstrumentationScope>,
    #[schema(value_type = Object)]
    pub attributes: BTreeMap<String, AttributeValue>,
    #[schema(value_type = Object)]
    pub profile_labels: BTreeMap<String, String>,
    pub mappings: Vec<ProfileMappingV1>,
    pub functions: Vec<ProfileFunctionV1>,
    pub locations: Vec<ProfileLocationV1>,
    pub samples: Vec<ProfileStackSampleV1>,
    pub blob_refs: Vec<ContentBlobRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_payload_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    pub retention_expires_at: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProfileView {
    #[default]
    List,
    Flamegraph,
    TopFunctions,
    Diff,
}

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct ProfileQuery {
    pub project: String,
    #[serde(default)]
    pub view: ProfileView,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison_profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
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
    #[serde(default = "default_top_limit")]
    pub top_limit: usize,
}

impl ProfileQuery {
    pub fn for_project(project: impl Into<String>) -> Self {
        Self {
            project: project.into(),
            view: ProfileView::List,
            profile_id: None,
            baseline_profile_id: None,
            comparison_profile_id: None,
            trace_id: None,
            span_id: None,
            start_time: None,
            end_time: None,
            after_cursor: 0,
            min_cursor: None,
            limit: default_query_limit(),
            top_limit: default_top_limit(),
        }
    }

    fn validate(&self) -> Result<ProfileQueryBounds> {
        if self.project.trim().is_empty() {
            bail!("project must not be empty");
        }
        if self.limit == 0 || self.limit > MAX_PROFILE_QUERY_LIMIT {
            bail!("limit must be between 1 and {MAX_PROFILE_QUERY_LIMIT}");
        }
        if self.top_limit == 0 || self.top_limit > MAX_PROFILE_TOP_LIMIT {
            bail!("top_limit must be between 1 and {MAX_PROFILE_TOP_LIMIT}");
        }
        match self.view {
            ProfileView::Flamegraph | ProfileView::TopFunctions => {
                if self.profile_id.as_deref().is_none_or(str::is_empty) {
                    bail!("profile_id is required for the selected profile view");
                }
            }
            ProfileView::Diff => {
                if self
                    .baseline_profile_id
                    .as_deref()
                    .is_none_or(str::is_empty)
                    || self
                        .comparison_profile_id
                        .as_deref()
                        .is_none_or(str::is_empty)
                {
                    bail!("baseline_profile_id and comparison_profile_id are required for diff");
                }
            }
            ProfileView::List => {}
        }
        let start = parse_optional_time("start_time", self.start_time.as_deref())?;
        let end = parse_optional_time("end_time", self.end_time.as_deref())?;
        if start.zip(end).is_some_and(|(start, end)| start >= end) {
            bail!("start_time must be earlier than end_time");
        }
        Ok(ProfileQueryBounds { start, end })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct ProfileFlamegraphEntryV1 {
    pub frames: Vec<String>,
    pub value: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct ProfileFunctionValueV1 {
    pub function: String,
    pub inclusive: f64,
    pub self_value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct ProfilePage {
    pub records: Vec<ProfileRecordV1>,
    pub flamegraph: Vec<ProfileFlamegraphEntryV1>,
    pub functions: Vec<ProfileFunctionValueV1>,
    pub next_cursor: u64,
    pub projection_cursor: u64,
    pub has_more: bool,
}

#[derive(Default, Deserialize, Serialize)]
struct ProfileState {
    records: BTreeMap<u64, ProfileRecordV1>,
    cursor_by_event_id: BTreeMap<String, u64>,
    cursor_by_profile_id: BTreeMap<String, u64>,
}

struct ProfileQueryBounds {
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
}

pub struct ProfileProjection {
    journal: Arc<DurableJournal>,
    state: RwLock<ProfileState>,
    retention_days: i64,
}

impl ProfileProjection {
    pub fn new(journal: Arc<DurableJournal>) -> Self {
        Self::with_retention_days(journal, DEFAULT_PROFILE_RETENTION_DAYS)
            .expect("default profile retention is valid")
    }

    pub fn with_retention_days(journal: Arc<DurableJournal>, retention_days: i64) -> Result<Self> {
        if retention_days <= 0 {
            bail!("profile retention days must be positive");
        }
        Ok(Self {
            journal,
            state: RwLock::new(ProfileState::default()),
            retention_days,
        })
    }

    pub fn query(&self, query: &ProfileQuery, now: DateTime<Utc>) -> Result<ProfilePage> {
        let bounds = query.validate()?;
        let state = self.state.read().expect("profile projection lock poisoned");
        let mut records = state
            .records
            .range(query.after_cursor.saturating_add(1)..)
            .map(|(_, record)| record)
            .filter(|record| record.project == query.project)
            .filter(|record| {
                query
                    .profile_id
                    .as_ref()
                    .is_none_or(|id| &record.profile_id == id)
            })
            .filter(|record| {
                query
                    .trace_id
                    .as_ref()
                    .is_none_or(|id| profile_has_trace(record, id))
            })
            .filter(|record| {
                query
                    .span_id
                    .as_ref()
                    .is_none_or(|id| profile_has_span(record, id))
            })
            .filter(|record| retained(record, now))
            .filter(|record| in_time_bounds(record, &bounds))
            .take(query.limit.saturating_add(1))
            .cloned()
            .collect::<Vec<_>>();
        let has_more = records.len() > query.limit;
        records.truncate(query.limit);
        let next_cursor = records
            .last()
            .map(|record| record.cursor)
            .unwrap_or(query.after_cursor);

        let (flamegraph, functions) = match query.view {
            ProfileView::List => (Vec::new(), Vec::new()),
            ProfileView::Flamegraph => {
                let record = required_profile(
                    &state,
                    &query.project,
                    query.profile_id.as_deref().unwrap_or_default(),
                    now,
                )?;
                (flamegraph(record), Vec::new())
            }
            ProfileView::TopFunctions => {
                let record = required_profile(
                    &state,
                    &query.project,
                    query.profile_id.as_deref().unwrap_or_default(),
                    now,
                )?;
                (Vec::new(), top_functions(record, query.top_limit))
            }
            ProfileView::Diff => {
                let baseline = required_profile(
                    &state,
                    &query.project,
                    query.baseline_profile_id.as_deref().unwrap_or_default(),
                    now,
                )?;
                let comparison = required_profile(
                    &state,
                    &query.project,
                    query.comparison_profile_id.as_deref().unwrap_or_default(),
                    now,
                )?;
                (
                    Vec::new(),
                    diff_functions(baseline, comparison, query.top_limit)?,
                )
            }
        };
        Ok(ProfilePage {
            records,
            flamegraph,
            functions,
            next_cursor,
            projection_cursor: 0,
            has_more,
        })
    }

    fn materialize(&self, event: &StoredEvent) -> Result<ProfileRecordV1> {
        self.journal
            .storage()
            .validate_blob_refs(&event.event.blob_refs)
            .context("profile event references unavailable durable content")?;
        let payload = load_profile_payload(&self.journal, event)?;
        let profile = payload.get("profile").unwrap_or(&payload);
        let dictionary = payload.get("dictionary").unwrap_or(&Value::Null);
        let strings = value_array(dictionary, "stringTable", "string_table")
            .iter()
            .map(|value| value.as_str().unwrap_or_default().to_string())
            .collect::<Vec<_>>();
        let attribute_table = profile_attributes(dictionary, &strings);
        let mappings = profile_mappings(dictionary, &strings, &attribute_table);
        let functions = profile_functions(dictionary, &strings);
        let locations = profile_locations(dictionary, &attribute_table);
        let samples = profile_samples(profile, dictionary, &strings, &functions, &locations)?;
        let profile_id = value_string(profile, "profileId", "profile_id")
            .filter(|value| !value.is_empty())
            .map(normalize_id)
            .unwrap_or_else(|| event.event.event_id.clone());
        let start_nanos = value_u64(profile, "timeUnixNano", "time_unix_nano").max(value_u64(
            profile,
            "startTimeUnixNano",
            "start_time_unix_nano",
        ));
        let duration_nanos = value_u64(profile, "durationNano", "duration_nano");
        let start_time = nanos_time(start_nanos).unwrap_or_else(|| event.event.occurred_at.clone());
        let end_time = nanos_time(start_nanos.saturating_add(duration_nanos))
            .unwrap_or_else(|| start_time.clone());
        let sample_type = profile
            .get("sampleType")
            .or_else(|| profile.get("sample_type"));
        let type_index = sample_type
            .map(|value| value_usize(value, "typeStrindex", "type_strindex"))
            .unwrap_or_default();
        let unit_index = sample_type
            .map(|value| value_usize(value, "unitStrindex", "unit_strindex"))
            .unwrap_or_default();
        let occurred = DateTime::parse_from_rfc3339(&event.event.occurred_at)
            .context("profile occurred_at must be RFC3339")?
            .with_timezone(&Utc);
        Ok(ProfileRecordV1 {
            schema: PROFILE_RECORD_SCHEMA.into(),
            cursor: event.cursor,
            event_id: event.event.event_id.clone(),
            profile_id,
            project: event.event.project.clone(),
            environment: event.event.environment.clone(),
            occurred_at: event.event.occurred_at.clone(),
            start_time,
            end_time,
            sample_type: strings
                .get(type_index)
                .filter(|value| !value.is_empty())
                .cloned()
                .unwrap_or_else(|| "samples".into()),
            unit: strings
                .get(unit_index)
                .filter(|value| !value.is_empty())
                .cloned()
                .unwrap_or_else(|| "count".into()),
            period: value_i64(profile, "period", "period"),
            resource: event.event.resource.clone(),
            instrumentation_scope: event.event.instrumentation_scope.clone(),
            attributes: event.event.attributes.clone(),
            profile_labels: referenced_attributes(profile, &attribute_table),
            mappings,
            functions,
            locations,
            samples,
            blob_refs: event.event.blob_refs.clone(),
            original_payload_format: value_string(
                profile,
                "originalPayloadFormat",
                "original_payload_format",
            )
            .filter(|value| !value.is_empty())
            .map(str::to_string),
            trace_id: event.event.trace_id.clone(),
            span_id: event.event.span_id.clone(),
            retention_expires_at: (occurred + Duration::days(self.retention_days)).to_rfc3339(),
        })
    }
}

impl Projection for ProfileProjection {
    fn descriptor(&self) -> ProjectionDescriptor {
        ProjectionDescriptor {
            name: PROJECTION_PROFILE_STORE.into(),
            schema_version: PROFILE_SCHEMA_VERSION,
            retention: format!("{} days", self.retention_days),
        }
    }

    fn apply_idempotent(&self, event: &StoredEvent) -> Result<()> {
        if event.event.signal != SignalKind::Profile {
            return Ok(());
        }
        if self
            .state
            .read()
            .expect("profile projection lock poisoned")
            .cursor_by_event_id
            .contains_key(&event.event.event_id)
        {
            return Ok(());
        }
        let record = self.materialize(event)?;
        let mut state = self
            .state
            .write()
            .expect("profile projection lock poisoned");
        if state.cursor_by_event_id.contains_key(&event.event.event_id) {
            return Ok(());
        }
        let key = profile_key(&record.project, &record.profile_id);
        if let Some(existing) = state.cursor_by_profile_id.get(&key) {
            bail!(
                "profile id {} in project {} already belongs to cursor {existing}",
                record.profile_id,
                record.project
            );
        }
        state
            .cursor_by_event_id
            .insert(record.event_id.clone(), record.cursor);
        state.cursor_by_profile_id.insert(key, record.cursor);
        state.records.insert(record.cursor, record);
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        let state = self.state.read().expect("profile projection lock poisoned");
        Ok(serde_json::to_vec(&*state)?)
    }

    fn restore(&self, state: &[u8]) -> Result<()> {
        let restored: ProfileState = serde_json::from_slice(state)?;
        validate_restored_state(&restored)?;
        for record in restored.records.values() {
            self.journal
                .storage()
                .validate_blob_refs(&record.blob_refs)
                .with_context(|| {
                    format!(
                        "restore profile projection record {} durable blobs",
                        record.profile_id
                    )
                })?;
        }
        *self
            .state
            .write()
            .expect("profile projection lock poisoned") = restored;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn load_profile_payload(journal: &DurableJournal, event: &StoredEvent) -> Result<Value> {
    let Some(marker) = event.event.payload.get("profileBlob") else {
        return Ok(event.event.payload.clone());
    };
    let reference: ContentBlobRef =
        serde_json::from_value(marker.clone()).context("decode profileBlob content reference")?;
    if !event
        .event
        .blob_refs
        .iter()
        .any(|candidate| candidate == &reference)
    {
        bail!("profileBlob reference is absent from event blob_refs");
    }
    let bytes = journal.storage().read_blob(&reference.hash)?;
    if bytes.len() as u64 != reference.size {
        bail!("profileBlob durable size does not match reference");
    }
    serde_json::from_slice(&bytes).context("decode JSON profile blob")
}

fn profile_mappings(
    dictionary: &Value,
    strings: &[String],
    attributes: &[BTreeMap<String, String>],
) -> Vec<ProfileMappingV1> {
    value_array(dictionary, "mappingTable", "mapping_table")
        .iter()
        .enumerate()
        .map(|(id, mapping)| ProfileMappingV1 {
            id,
            memory_start: value_u64(mapping, "memoryStart", "memory_start"),
            memory_limit: value_u64(mapping, "memoryLimit", "memory_limit"),
            file_offset: value_u64(mapping, "fileOffset", "file_offset"),
            filename: strings
                .get(value_usize(
                    mapping,
                    "filenameStrindex",
                    "filename_strindex",
                ))
                .cloned()
                .unwrap_or_default(),
            attributes: referenced_attributes(mapping, attributes),
        })
        .collect()
}

fn profile_functions(dictionary: &Value, strings: &[String]) -> Vec<ProfileFunctionV1> {
    value_array(dictionary, "functionTable", "function_table")
        .iter()
        .enumerate()
        .map(|(id, function)| ProfileFunctionV1 {
            id,
            name: strings
                .get(value_usize(function, "nameStrindex", "name_strindex"))
                .cloned()
                .unwrap_or_default(),
            system_name: strings
                .get(value_usize(
                    function,
                    "systemNameStrindex",
                    "system_name_strindex",
                ))
                .cloned()
                .unwrap_or_default(),
            filename: strings
                .get(value_usize(
                    function,
                    "filenameStrindex",
                    "filename_strindex",
                ))
                .cloned()
                .unwrap_or_default(),
            start_line: value_i64(function, "startLine", "start_line"),
        })
        .collect()
}

fn profile_locations(
    dictionary: &Value,
    attributes: &[BTreeMap<String, String>],
) -> Vec<ProfileLocationV1> {
    value_array(dictionary, "locationTable", "location_table")
        .iter()
        .enumerate()
        .map(|(id, location)| ProfileLocationV1 {
            id,
            mapping_id: value_usize(location, "mappingIndex", "mapping_index"),
            address: value_u64(location, "address", "address"),
            lines: value_array(location, "lines", "lines")
                .iter()
                .map(|line| ProfileLineV1 {
                    function_id: value_usize(line, "functionIndex", "function_index"),
                    line: value_i64(line, "line", "line"),
                    column: value_i64(line, "column", "column"),
                })
                .collect(),
            attributes: referenced_attributes(location, attributes),
        })
        .collect()
}

fn profile_samples(
    profile: &Value,
    dictionary: &Value,
    strings: &[String],
    functions: &[ProfileFunctionV1],
    locations: &[ProfileLocationV1],
) -> Result<Vec<ProfileStackSampleV1>> {
    let attributes = profile_attributes(dictionary, strings);
    let stacks = value_array(dictionary, "stackTable", "stack_table");
    let links = value_array(dictionary, "linkTable", "link_table");
    value_array(profile, "samples", "samples")
        .iter()
        .map(|sample| {
            let frames = if let Some(direct) = sample.get("frames").and_then(Value::as_array) {
                direct
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            } else {
                let stack_index = value_usize(sample, "stackIndex", "stack_index");
                let stack = stacks.get(stack_index).with_context(|| {
                    format!("profile sample references missing stack {stack_index}")
                })?;
                let mut frames = Vec::new();
                for location_index in index_array(stack, "locationIndices", "location_indices")
                    .into_iter()
                    .rev()
                {
                    let location = locations.get(location_index).with_context(|| {
                        format!("profile stack references missing location {location_index}")
                    })?;
                    for line in &location.lines {
                        let function = functions.get(line.function_id).with_context(|| {
                            format!(
                                "profile location references missing function {}",
                                line.function_id
                            )
                        })?;
                        let name = if function.name.is_empty() {
                            function.system_name.clone()
                        } else {
                            function.name.clone()
                        };
                        if !name.is_empty() {
                            frames.push(name);
                        }
                    }
                }
                frames
            };
            let values = value_array(sample, "values", "values")
                .iter()
                .filter_map(number_f64)
                .collect::<Vec<_>>();
            let timestamps = value_array(sample, "timestampsUnixNano", "timestamps_unix_nano");
            let value = if values.is_empty() {
                timestamps.len() as f64
            } else {
                values.iter().sum()
            };
            if !value.is_finite() || value < 0.0 {
                bail!("profile sample value must be finite and non-negative");
            }
            let link = links.get(value_usize(sample, "linkIndex", "link_index"));
            Ok(ProfileStackSampleV1 {
                frames,
                value,
                labels: referenced_attributes(sample, &attributes),
                trace_id: link.and_then(|value| profile_id(value, "traceId", "trace_id")),
                span_id: link.and_then(|value| profile_id(value, "spanId", "span_id")),
            })
        })
        .collect()
}

fn profile_attributes(dictionary: &Value, strings: &[String]) -> Vec<BTreeMap<String, String>> {
    value_array(dictionary, "attributeTable", "attribute_table")
        .iter()
        .map(|attribute| {
            let key = strings
                .get(value_usize(attribute, "keyStrindex", "key_strindex"))
                .cloned()
                .unwrap_or_default();
            let value = attribute
                .get("value")
                .and_then(any_value_string)
                .unwrap_or_default();
            let unit = strings
                .get(value_usize(attribute, "unitStrindex", "unit_strindex"))
                .filter(|unit| !unit.is_empty());
            BTreeMap::from([(
                key,
                unit.map_or(value.clone(), |unit| format!("{value} {unit}")),
            )])
        })
        .collect()
}

fn referenced_attributes(
    source: &Value,
    table: &[BTreeMap<String, String>],
) -> BTreeMap<String, String> {
    index_array(source, "attributeIndices", "attribute_indices")
        .into_iter()
        .filter_map(|index| table.get(index))
        .flat_map(|attributes| attributes.clone())
        .filter(|(key, _)| !key.is_empty())
        .collect()
}

fn flamegraph(record: &ProfileRecordV1) -> Vec<ProfileFlamegraphEntryV1> {
    let mut totals = BTreeMap::<Vec<String>, f64>::new();
    for sample in &record.samples {
        *totals.entry(sample.frames.clone()).or_default() += sample.value;
    }
    totals
        .into_iter()
        .map(|(frames, value)| ProfileFlamegraphEntryV1 { frames, value })
        .collect()
}

fn function_totals(record: &ProfileRecordV1) -> BTreeMap<String, (f64, f64)> {
    let mut totals = BTreeMap::<String, (f64, f64)>::new();
    for sample in &record.samples {
        for function in &sample.frames {
            totals.entry(function.clone()).or_default().0 += sample.value;
        }
        if let Some(function) = sample.frames.last() {
            totals.entry(function.clone()).or_default().1 += sample.value;
        }
    }
    totals
}

fn top_functions(record: &ProfileRecordV1, limit: usize) -> Vec<ProfileFunctionValueV1> {
    let mut values = function_totals(record)
        .into_iter()
        .map(
            |(function, (inclusive, self_value))| ProfileFunctionValueV1 {
                function,
                inclusive,
                self_value,
                baseline: None,
                comparison: None,
                delta: None,
            },
        )
        .collect::<Vec<_>>();
    values.sort_by(|left, right| {
        right
            .inclusive
            .total_cmp(&left.inclusive)
            .then_with(|| left.function.cmp(&right.function))
    });
    values.truncate(limit);
    values
}

fn diff_functions(
    baseline: &ProfileRecordV1,
    comparison: &ProfileRecordV1,
    limit: usize,
) -> Result<Vec<ProfileFunctionValueV1>> {
    if baseline.sample_type != comparison.sample_type || baseline.unit != comparison.unit {
        bail!(
            "profile diff requires matching sample type and unit; baseline={}/{} comparison={}/{}",
            baseline.sample_type,
            baseline.unit,
            comparison.sample_type,
            comparison.unit
        );
    }
    let baseline_totals = function_totals(baseline);
    let comparison_totals = function_totals(comparison);
    let names = baseline_totals
        .keys()
        .chain(comparison_totals.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut values = names
        .into_iter()
        .map(|function| {
            let baseline_value = baseline_totals.get(&function).copied().unwrap_or_default();
            let comparison_value = comparison_totals
                .get(&function)
                .copied()
                .unwrap_or_default();
            ProfileFunctionValueV1 {
                function,
                inclusive: comparison_value.0,
                self_value: comparison_value.1,
                baseline: Some(baseline_value.0),
                comparison: Some(comparison_value.0),
                delta: Some(comparison_value.0 - baseline_value.0),
            }
        })
        .collect::<Vec<_>>();
    values.sort_by(|left, right| {
        right
            .delta
            .unwrap_or_default()
            .abs()
            .total_cmp(&left.delta.unwrap_or_default().abs())
            .then_with(|| left.function.cmp(&right.function))
    });
    values.truncate(limit);
    Ok(values)
}

fn required_profile<'a>(
    state: &'a ProfileState,
    project: &str,
    profile_id: &str,
    now: DateTime<Utc>,
) -> Result<&'a ProfileRecordV1> {
    let cursor = state
        .cursor_by_profile_id
        .get(&profile_key(project, profile_id))
        .with_context(|| format!("profile {profile_id} was not found in project {project}"))?;
    let record = state
        .records
        .get(cursor)
        .context("profile id index references a missing record")?;
    if !retained(record, now) {
        bail!("profile {profile_id} is outside hot retention");
    }
    Ok(record)
}

fn retained(record: &ProfileRecordV1, now: DateTime<Utc>) -> bool {
    DateTime::parse_from_rfc3339(&record.retention_expires_at)
        .map(|expires| expires.with_timezone(&Utc) > now)
        .unwrap_or(false)
}

fn in_time_bounds(record: &ProfileRecordV1, bounds: &ProfileQueryBounds) -> bool {
    DateTime::parse_from_rfc3339(&record.start_time)
        .map(|time| {
            let time = time.with_timezone(&Utc);
            bounds.start.is_none_or(|start| time >= start)
                && bounds.end.is_none_or(|end| time < end)
        })
        .unwrap_or(false)
}

fn profile_has_trace(record: &ProfileRecordV1, trace_id: &str) -> bool {
    record.trace_id.as_deref() == Some(trace_id)
        || record
            .samples
            .iter()
            .any(|sample| sample.trace_id.as_deref() == Some(trace_id))
}

fn profile_has_span(record: &ProfileRecordV1, span_id: &str) -> bool {
    record.span_id.as_deref() == Some(span_id)
        || record
            .samples
            .iter()
            .any(|sample| sample.span_id.as_deref() == Some(span_id))
}

fn validate_restored_state(state: &ProfileState) -> Result<()> {
    if state.records.len() != state.cursor_by_event_id.len()
        || state.records.len() != state.cursor_by_profile_id.len()
    {
        bail!("profile snapshot indexes do not match record count");
    }
    for (cursor, record) in &state.records {
        if cursor != &record.cursor
            || state.cursor_by_event_id.get(&record.event_id) != Some(cursor)
            || state
                .cursor_by_profile_id
                .get(&profile_key(&record.project, &record.profile_id))
                != Some(cursor)
        {
            bail!("profile snapshot contains an inconsistent record index");
        }
    }
    Ok(())
}

fn value_array<'a>(value: &'a Value, camel: &str, snake: &str) -> &'a [Value] {
    value
        .get(camel)
        .or_else(|| value.get(snake))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn index_array(value: &Value, camel: &str, snake: &str) -> Vec<usize> {
    value_array(value, camel, snake)
        .iter()
        .filter_map(number_usize)
        .collect()
}

fn value_string<'a>(value: &'a Value, camel: &str, snake: &str) -> Option<&'a str> {
    value
        .get(camel)
        .or_else(|| value.get(snake))
        .and_then(Value::as_str)
}

fn value_u64(value: &Value, camel: &str, snake: &str) -> u64 {
    value
        .get(camel)
        .or_else(|| value.get(snake))
        .and_then(|value| value.as_u64().or_else(|| value.as_str()?.parse().ok()))
        .unwrap_or_default()
}

fn value_i64(value: &Value, camel: &str, snake: &str) -> i64 {
    value
        .get(camel)
        .or_else(|| value.get(snake))
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
        .unwrap_or_default()
}

fn value_usize(value: &Value, camel: &str, snake: &str) -> usize {
    value
        .get(camel)
        .or_else(|| value.get(snake))
        .and_then(number_usize)
        .unwrap_or_default()
}

fn number_usize(value: &Value) -> Option<usize> {
    value
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .or_else(|| value.as_i64().and_then(|value| usize::try_from(value).ok()))
        .or_else(|| value.as_str()?.parse().ok())
}

fn number_f64(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_i64().map(|value| value as f64))
        .or_else(|| value.as_u64().map(|value| value as f64))
        .or_else(|| value.as_str()?.parse().ok())
}

fn any_value_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::Object(object) => object.values().find_map(any_value_string),
        Value::Array(values) => Some(
            values
                .iter()
                .filter_map(any_value_string)
                .collect::<Vec<_>>()
                .join(","),
        ),
        Value::Null => None,
    }
}

fn profile_id(value: &Value, camel: &str, snake: &str) -> Option<String> {
    value_string(value, camel, snake).map(normalize_id)
}

fn normalize_id(value: &str) -> String {
    if value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return value.to_ascii_lowercase();
    }
    BASE64
        .decode(value)
        .ok()
        .map(hex::encode)
        .unwrap_or_else(|| value.to_string())
}

fn nanos_time(value: u64) -> Option<String> {
    if value == 0 {
        return None;
    }
    DateTime::<Utc>::from_timestamp(
        i64::try_from(value / 1_000_000_000).ok()?,
        (value % 1_000_000_000) as u32,
    )
    .map(|time| time.to_rfc3339())
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

fn default_top_limit() -> usize {
    50
}

fn profile_key(project: &str, profile_id: &str) -> String {
    format!("{project}\u{0}{profile_id}")
}
// HANDWRITE-END
