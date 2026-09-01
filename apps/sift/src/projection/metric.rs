// HANDWRITE-BEGIN gap="sift-metric-projection" tracker="1667" reason="Define series identity, chunks, temporality, histograms, exemplars, overflow, rollups, typed query, snapshot, and rebuild."
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        atomic::{AtomicU64, Ordering},
        RwLock,
    },
};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;

use crate::{
    AttributeValue, MetricExemplar, MetricPoint, MetricTemporality, SignalKind, StoredEvent,
};

use super::{model::ProjectionDescriptor, runtime::Projection};

pub const PROJECTION_METRIC_STORE: &str = "metric-store";
pub const METRIC_SCHEMA_VERSION: u32 = 2;
pub const METRIC_CHUNK_POINTS: usize = 256;
pub const DEFAULT_METRIC_CARDINALITY_LIMIT: usize = 10_000;
pub const DEFAULT_RETAINED_POINTS_PER_SERIES: usize = 100_000;
pub const MAX_METRIC_QUERY_LIMIT: usize = 1_000;
pub const ROLLUP_WINDOWS_SECONDS: [u64; 2] = [60, 3_600];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum HistogramKind {
    Explicit,
    Exponential,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct MetricHistogramV1 {
    pub kind: HistogramKind,
    pub count: u64,
    pub sum: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub explicit_bounds: Vec<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bucket_counts: Vec<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<i32>,
    #[serde(default)]
    pub zero_count: u64,
    #[serde(default)]
    pub positive_offset: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub positive_bucket_counts: Vec<u64>,
    #[serde(default)]
    pub negative_offset: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub negative_bucket_counts: Vec<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

impl MetricHistogramV1 {
    fn validate(&self) -> Result<()> {
        if !self.sum.is_finite()
            || self.min.is_some_and(|value| !value.is_finite())
            || self.max.is_some_and(|value| !value.is_finite())
            || self.explicit_bounds.iter().any(|value| !value.is_finite())
        {
            bail!("histogram numbers must be finite");
        }
        if self.min.zip(self.max).is_some_and(|(min, max)| min > max) {
            bail!("histogram min must not exceed max");
        }
        match self.kind {
            HistogramKind::Explicit => {
                if self.scale.is_some()
                    || !self.positive_bucket_counts.is_empty()
                    || !self.negative_bucket_counts.is_empty()
                {
                    bail!("explicit histogram cannot contain exponential buckets");
                }
                if self.bucket_counts.len() != self.explicit_bounds.len() + 1 {
                    bail!("explicit histogram bucket_counts must have bounds length plus one");
                }
                if !self
                    .explicit_bounds
                    .windows(2)
                    .all(|window| window[0] < window[1])
                {
                    bail!("explicit histogram bounds must be strictly increasing");
                }
                if self.bucket_counts.iter().sum::<u64>() != self.count {
                    bail!("explicit histogram buckets must sum to count");
                }
            }
            HistogramKind::Exponential => {
                if self.scale.is_none()
                    || !self.explicit_bounds.is_empty()
                    || !self.bucket_counts.is_empty()
                {
                    bail!("exponential histogram requires scale and no explicit buckets");
                }
                let total = self.zero_count
                    + self.positive_bucket_counts.iter().sum::<u64>()
                    + self.negative_bucket_counts.iter().sum::<u64>();
                if total != self.count {
                    bail!("exponential histogram buckets must sum to count");
                }
            }
        }
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.validate()?;
        other.validate()?;
        if self.kind != other.kind
            || self.explicit_bounds != other.explicit_bounds
            || self.scale != other.scale
            || self.positive_offset != other.positive_offset
            || self.negative_offset != other.negative_offset
            || self.bucket_counts.len() != other.bucket_counts.len()
            || self.positive_bucket_counts.len() != other.positive_bucket_counts.len()
            || self.negative_bucket_counts.len() != other.negative_bucket_counts.len()
        {
            bail!("histogram schemas are not merge compatible");
        }
        self.count = self.count.saturating_add(other.count);
        self.sum += other.sum;
        self.zero_count = self.zero_count.saturating_add(other.zero_count);
        merge_counts(&mut self.bucket_counts, &other.bucket_counts);
        merge_counts(
            &mut self.positive_bucket_counts,
            &other.positive_bucket_counts,
        );
        merge_counts(
            &mut self.negative_bucket_counts,
            &other.negative_bucket_counts,
        );
        self.min = optional_min(self.min, other.min);
        self.max = optional_max(self.max, other.max);
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct MetricPointV1 {
    pub cursor: u64,
    pub event_id: String,
    pub occurred_at: String,
    pub time_unix_nano: i64,
    pub value: f64,
    #[serde(default, skip_serializing_if = "is_false")]
    pub stale: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub histogram: Option<MetricHistogramV1>,
    #[serde(default)]
    pub exemplars: Vec<MetricExemplar>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct MetricChunkV1 {
    pub start_time_unix_nano: i64,
    pub end_time_unix_nano: i64,
    pub points: Vec<MetricPointV1>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct MetricRollupV1 {
    pub window_seconds: u64,
    pub start_time_unix_nano: i64,
    pub end_time_unix_nano: i64,
    pub point_count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub last: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub histogram: Option<MetricHistogramV1>,
    #[serde(default)]
    pub exemplars: Vec<MetricExemplar>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MetricAggregation {
    #[default]
    Raw,
    Sum,
    Avg,
    Min,
    Max,
    Count,
    Rate,
}

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct MetricQuery {
    pub project: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub resource_equals: BTreeMap<String, String>,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub attribute_equals: BTreeMap<String, AttributeValue>,
    #[serde(default)]
    pub aggregation: MetricAggregation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_series_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_cursor: Option<u64>,
    #[serde(default = "default_query_limit")]
    pub limit: usize,
}

impl MetricQuery {
    pub fn for_project(project: impl Into<String>) -> Self {
        Self {
            project: project.into(),
            environment: None,
            name: None,
            start_time: None,
            end_time: None,
            resource_equals: BTreeMap::new(),
            attribute_equals: BTreeMap::new(),
            aggregation: MetricAggregation::Raw,
            after_series_id: None,
            min_cursor: None,
            limit: default_query_limit(),
        }
    }

    fn validate(&self) -> Result<(Option<i64>, Option<i64>)> {
        if self.project.trim().is_empty() {
            bail!("project must not be empty");
        }
        if self.limit == 0 || self.limit > MAX_METRIC_QUERY_LIMIT {
            bail!("limit must be between 1 and {MAX_METRIC_QUERY_LIMIT}");
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
pub struct MetricSeriesResultV1 {
    pub series_id: String,
    pub project: String,
    pub environment: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub temporality: MetricTemporality,
    pub resource: BTreeMap<String, String>,
    #[schema(value_type = Object)]
    pub attributes: BTreeMap<String, AttributeValue>,
    pub overflow: bool,
    pub points: Vec<MetricPointV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aggregate: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub histogram: Option<MetricHistogramV1>,
    pub reset_count: u64,
    pub rollups: Vec<MetricRollupV1>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct MetricPage {
    pub series: Vec<MetricSeriesResultV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_series_id: Option<String>,
    pub projection_cursor: u64,
    pub has_more: bool,
    pub overflowed_series: u64,
    pub overflowed_points: u64,
}

#[derive(Clone, Deserialize, Serialize)]
struct StoredMetricSeries {
    series_id: String,
    project: String,
    environment: String,
    name: String,
    unit: Option<String>,
    temporality: MetricTemporality,
    resource: BTreeMap<String, String>,
    attributes: BTreeMap<String, AttributeValue>,
    overflow: bool,
    chunks: Vec<MetricChunkV1>,
    #[serde(default)]
    point_count: usize,
    rollups: Vec<MetricRollupV1>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct MetricState {
    series: BTreeMap<String, StoredMetricSeries>,
    cursor_by_event_id: BTreeMap<String, u64>,
    exact_identities: BTreeMap<String, BTreeSet<String>>,
    overflowed_identities: BTreeSet<String>,
    overflowed_points: u64,
}

#[derive(Deserialize, Serialize)]
struct MetricSnapshot {
    state: MetricState,
    cardinality_limit: usize,
    retained_points_per_series: usize,
}

#[derive(Default, Deserialize)]
struct MetricPayload {
    #[serde(default)]
    histogram: Option<MetricHistogramV1>,
}

pub struct MetricProjection {
    state: RwLock<MetricState>,
    cardinality_limit: usize,
    retained_points_per_series: usize,
    maintenance_work_points: AtomicU64,
}

impl MetricProjection {
    pub fn new() -> Self {
        Self::with_limits(
            DEFAULT_METRIC_CARDINALITY_LIMIT,
            DEFAULT_RETAINED_POINTS_PER_SERIES,
        )
        .expect("default metric limits are valid")
    }

    pub fn with_limits(
        cardinality_limit: usize,
        retained_points_per_series: usize,
    ) -> Result<Self> {
        if cardinality_limit == 0 || retained_points_per_series == 0 {
            bail!("metric cardinality and retention limits must be non-zero");
        }
        Ok(Self {
            state: RwLock::new(MetricState::default()),
            cardinality_limit,
            retained_points_per_series,
            maintenance_work_points: AtomicU64::new(0),
        })
    }

    #[doc(hidden)]
    pub fn maintenance_work_points(&self) -> u64 {
        self.maintenance_work_points.load(Ordering::Relaxed)
    }

    pub fn query(&self, query: &MetricQuery) -> Result<MetricPage> {
        let (start, end) = query.validate()?;
        let state = self.state.read().expect("metric projection lock poisoned");
        let mut results = Vec::new();
        for series in state.series.values() {
            if series.project != query.project
                || query
                    .environment
                    .as_ref()
                    .is_some_and(|environment| &series.environment != environment)
                || query.name.as_ref().is_some_and(|name| &series.name != name)
                || query
                    .after_series_id
                    .as_ref()
                    .is_some_and(|after| &series.series_id <= after)
                || !query
                    .resource_equals
                    .iter()
                    .all(|(key, value)| series.resource.get(key) == Some(value))
                || !query
                    .attribute_equals
                    .iter()
                    .all(|(key, value)| series.attributes.get(key) == Some(value))
            {
                continue;
            }
            let all_points = flatten_points(series);
            let points = all_points
                .iter()
                .filter(|point| start.is_none_or(|start| point.time_unix_nano >= start))
                .filter(|point| end.is_none_or(|end| point.time_unix_nano < end))
                .cloned()
                .collect::<Vec<_>>();
            if points.is_empty() {
                continue;
            }
            let numeric_points = points
                .iter()
                .filter(|point| !point.stale)
                .cloned()
                .collect::<Vec<_>>();
            let (aggregate, reset_count, histogram) = if numeric_points.is_empty() {
                (None, 0, None)
            } else {
                let (semantic_total, reset_count) =
                    semantic_total(series.temporality, &numeric_points);
                (
                    aggregate(query.aggregation, semantic_total, &numeric_points),
                    reset_count,
                    merge_histograms(&numeric_points)?,
                )
            };
            let rollups = make_rollups(&all_points)?
                .into_iter()
                .filter(|rollup| start.is_none_or(|start| rollup.end_time_unix_nano > start))
                .filter(|rollup| end.is_none_or(|end| rollup.start_time_unix_nano < end))
                .collect();
            results.push(MetricSeriesResultV1 {
                series_id: series.series_id.clone(),
                project: series.project.clone(),
                environment: series.environment.clone(),
                name: series.name.clone(),
                unit: series.unit.clone(),
                temporality: series.temporality,
                resource: series.resource.clone(),
                attributes: series.attributes.clone(),
                overflow: series.overflow,
                points,
                aggregate,
                histogram,
                reset_count,
                rollups,
            });
        }
        results.sort_by(|left, right| left.series_id.cmp(&right.series_id));
        let has_more = results.len() > query.limit;
        results.truncate(query.limit);
        Ok(MetricPage {
            next_series_id: results.last().map(|series| series.series_id.clone()),
            series: results,
            projection_cursor: state
                .cursor_by_event_id
                .values()
                .copied()
                .max()
                .unwrap_or(0),
            has_more,
            overflowed_series: state.overflowed_identities.len() as u64,
            overflowed_points: state.overflowed_points,
        })
    }
}

impl Default for MetricProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl Projection for MetricProjection {
    fn descriptor(&self) -> ProjectionDescriptor {
        ProjectionDescriptor {
            name: PROJECTION_METRIC_STORE.into(),
            schema_version: METRIC_SCHEMA_VERSION,
            retention: format!(
                "{} points per series; 60s and 3600s rollups",
                self.retained_points_per_series
            ),
        }
    }

    fn apply_idempotent(&self, stored: &StoredEvent) -> Result<()> {
        if stored.event.signal != SignalKind::Metric {
            return Ok(());
        }
        let event = &stored.event;
        let metric = event
            .metric
            .as_ref()
            .context("metric signal requires a direct metric point")?;
        let payload: MetricPayload =
            serde_json::from_value(event.payload.clone()).context("decode metric payload")?;
        if let Some(histogram) = &payload.histogram {
            histogram.validate()?;
        }
        let timestamp = parse_time(&event.occurred_at)?;
        let exact_id = sha256(
            identity_material(
                &event.project,
                &event.environment,
                metric,
                &event.resource,
                &event.attributes,
                false,
            )?
            .as_bytes(),
        );
        let mut state = self.state.write().expect("metric projection lock poisoned");
        if state
            .cursor_by_event_id
            .get(&event.event_id)
            .is_some_and(|cursor| *cursor >= stored.cursor)
        {
            return Ok(());
        }
        let replaces_existing = state.cursor_by_event_id.remove(&event.event_id).is_some();
        let known = state.series.contains_key(&exact_id);
        let current_count = state
            .exact_identities
            .get(&event.project)
            .map_or(0, BTreeSet::len);
        let overflow = !known && current_count >= self.cardinality_limit;
        let series_id = if overflow {
            state.overflowed_identities.insert(exact_id.clone());
            state.overflowed_points = state.overflowed_points.saturating_add(1);
            sha256(
                identity_material(
                    &event.project,
                    &event.environment,
                    metric,
                    &BTreeMap::new(),
                    &BTreeMap::new(),
                    true,
                )?
                .as_bytes(),
            )
        } else {
            state
                .exact_identities
                .entry(event.project.clone())
                .or_default()
                .insert(exact_id.clone());
            exact_id
        };
        let point = MetricPointV1 {
            cursor: stored.cursor,
            event_id: event.event_id.clone(),
            occurred_at: event.occurred_at.clone(),
            time_unix_nano: timestamp,
            value: metric.value,
            stale: metric.stale,
            histogram: payload.histogram,
            exemplars: metric.exemplars.clone(),
        };
        let (removed_event_ids, current_retained, touched) = {
            let series =
                state
                    .series
                    .entry(series_id.clone())
                    .or_insert_with(|| StoredMetricSeries {
                        series_id,
                        project: event.project.clone(),
                        environment: event.environment.clone(),
                        name: metric.name.clone(),
                        unit: metric.unit.clone(),
                        temporality: metric.temporality,
                        resource: if overflow {
                            BTreeMap::from([("sift.metric.overflow".into(), "true".into())])
                        } else {
                            event.resource.clone()
                        },
                        attributes: if overflow {
                            BTreeMap::new()
                        } else {
                            event.attributes.clone()
                        },
                        overflow,
                        chunks: Vec::new(),
                        point_count: 0,
                        rollups: Vec::new(),
                    });
            if series.temporality != metric.temporality {
                bail!("metric series temporality changed without changing identity");
            }
            if series.point_count == 0 && !series.chunks.is_empty() {
                series.point_count = series.chunks.iter().map(|chunk| chunk.points.len()).sum();
            }
            let append_in_order = !replaces_existing
                && last_point(series).is_none_or(|last| point_order(last, &point).is_le());
            let (removed, current_retained, touched) = if append_in_order {
                push_metric_point(series, point);
                let mut removed = Vec::new();
                while series.point_count > self.retained_points_per_series {
                    if let Some(event_id) = pop_oldest_metric_point(series) {
                        removed.push(event_id);
                    } else {
                        break;
                    }
                }
                series.rollups.clear();
                let touched = 1_u64.saturating_add(removed.len() as u64);
                (removed, true, touched)
            } else {
                let mut points = flatten_points(series);
                points.retain(|existing| existing.event_id != event.event_id);
                points.push(point);
                points.sort_by(point_order);
                let removed = if points.len() > self.retained_points_per_series {
                    points
                        .drain(..points.len() - self.retained_points_per_series)
                        .map(|point| point.event_id)
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
                let current_retained = points
                    .iter()
                    .any(|retained| retained.event_id == event.event_id);
                let touched = points.len() as u64 + removed.len() as u64;
                series.chunks = make_chunks(&points);
                series.point_count = points.len();
                series.rollups.clear();
                (removed, current_retained, touched)
            };
            (removed, current_retained, touched)
        };
        self.maintenance_work_points
            .fetch_add(touched, Ordering::Relaxed);
        for event_id in removed_event_ids {
            state.cursor_by_event_id.remove(&event_id);
        }
        if current_retained {
            state
                .cursor_by_event_id
                .insert(event.event_id.clone(), stored.cursor);
        }
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&MetricSnapshot {
            state: self
                .state
                .read()
                .expect("metric projection lock poisoned")
                .clone(),
            cardinality_limit: self.cardinality_limit,
            retained_points_per_series: self.retained_points_per_series,
        })
        .map_err(Into::into)
    }

    fn restore(&self, bytes: &[u8]) -> Result<()> {
        let snapshot: MetricSnapshot =
            serde_json::from_slice(bytes).context("decode metric projection snapshot")?;
        if snapshot.cardinality_limit != self.cardinality_limit
            || snapshot.retained_points_per_series != self.retained_points_per_series
        {
            bail!("metric projection snapshot limits do not match configured limits");
        }
        let mut state = snapshot.state;
        for series in state.series.values_mut() {
            series.point_count = series.chunks.iter().map(|chunk| chunk.points.len()).sum();
            series.rollups.clear();
        }
        *self.state.write().expect("metric projection lock poisoned") = state;
        Ok(())
    }

    fn semantic_digest(&self) -> Result<String> {
        Ok(sha256(&self.snapshot()?))
    }
}

fn identity_material(
    project: &str,
    environment: &str,
    metric: &MetricPoint,
    resource: &BTreeMap<String, String>,
    attributes: &BTreeMap<String, AttributeValue>,
    overflow: bool,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "project": project,
        "environment": environment,
        "name": metric.name,
        "unit": metric.unit,
        "temporality": metric.temporality,
        "resource": resource,
        "attributes": attributes,
        "overflow": overflow,
    }))
    .map_err(Into::into)
}

fn flatten_points(series: &StoredMetricSeries) -> Vec<MetricPointV1> {
    series
        .chunks
        .iter()
        .flat_map(|chunk| chunk.points.iter().cloned())
        .collect()
}

fn last_point(series: &StoredMetricSeries) -> Option<&MetricPointV1> {
    series.chunks.last()?.points.last()
}

fn point_order(left: &MetricPointV1, right: &MetricPointV1) -> std::cmp::Ordering {
    left.time_unix_nano
        .cmp(&right.time_unix_nano)
        .then_with(|| left.cursor.cmp(&right.cursor))
        .then_with(|| left.event_id.cmp(&right.event_id))
}

fn push_metric_point(series: &mut StoredMetricSeries, point: MetricPointV1) {
    let needs_chunk = series
        .chunks
        .last()
        .is_none_or(|chunk| chunk.points.len() == METRIC_CHUNK_POINTS);
    if needs_chunk {
        series.chunks.push(MetricChunkV1 {
            start_time_unix_nano: point.time_unix_nano,
            end_time_unix_nano: point.time_unix_nano,
            points: Vec::with_capacity(METRIC_CHUNK_POINTS),
        });
    }
    let chunk = series.chunks.last_mut().expect("metric chunk was created");
    if chunk.points.is_empty() {
        chunk.start_time_unix_nano = point.time_unix_nano;
    }
    chunk.end_time_unix_nano = point.time_unix_nano;
    chunk.points.push(point);
    series.point_count = series.point_count.saturating_add(1);
}

fn pop_oldest_metric_point(series: &mut StoredMetricSeries) -> Option<String> {
    let point = series.chunks.first_mut()?.points.remove(0);
    series.point_count = series.point_count.saturating_sub(1);
    if series.chunks[0].points.is_empty() {
        series.chunks.remove(0);
    } else {
        series.chunks[0].start_time_unix_nano = series.chunks[0].points[0].time_unix_nano;
    }
    Some(point.event_id)
}

fn make_chunks(points: &[MetricPointV1]) -> Vec<MetricChunkV1> {
    points
        .chunks(METRIC_CHUNK_POINTS)
        .filter_map(|points| {
            Some(MetricChunkV1 {
                start_time_unix_nano: points.first()?.time_unix_nano,
                end_time_unix_nano: points.last()?.time_unix_nano,
                points: points.to_vec(),
            })
        })
        .collect()
}

fn make_rollups(points: &[MetricPointV1]) -> Result<Vec<MetricRollupV1>> {
    let mut rollups = Vec::new();
    for window_seconds in ROLLUP_WINDOWS_SECONDS {
        let window_nanos = (window_seconds as i64) * 1_000_000_000;
        let mut groups = BTreeMap::<i64, Vec<&MetricPointV1>>::new();
        for point in points.iter().filter(|point| !point.stale) {
            let start = point.time_unix_nano.div_euclid(window_nanos) * window_nanos;
            groups.entry(start).or_default().push(point);
        }
        for (start, points) in groups {
            let values = points.iter().map(|point| point.value).collect::<Vec<_>>();
            let histogram =
                merge_histogram_refs(points.iter().filter_map(|point| point.histogram.as_ref()))?;
            let mut exemplars = points
                .iter()
                .flat_map(|point| point.exemplars.iter().cloned())
                .collect::<Vec<_>>();
            exemplars.sort_by(|left, right| {
                left.trace_id
                    .cmp(&right.trace_id)
                    .then_with(|| left.span_id.cmp(&right.span_id))
            });
            exemplars.dedup_by(|left, right| {
                left.trace_id == right.trace_id && left.span_id == right.span_id
            });
            rollups.push(MetricRollupV1 {
                window_seconds,
                start_time_unix_nano: start,
                end_time_unix_nano: start + window_nanos,
                point_count: points.len() as u64,
                sum: values.iter().sum(),
                min: values.iter().copied().fold(f64::INFINITY, f64::min),
                max: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                last: points.last().expect("non-empty rollup").value,
                histogram,
                exemplars,
            });
        }
    }
    rollups.sort_by(|left, right| {
        left.window_seconds
            .cmp(&right.window_seconds)
            .then_with(|| left.start_time_unix_nano.cmp(&right.start_time_unix_nano))
    });
    Ok(rollups)
}

fn semantic_total(temporality: MetricTemporality, points: &[MetricPointV1]) -> (f64, u64) {
    match temporality {
        MetricTemporality::Gauge => (points.last().map_or(0.0, |point| point.value), 0),
        MetricTemporality::Delta => (points.iter().map(|point| point.value).sum(), 0),
        MetricTemporality::Cumulative => {
            let mut total = 0.0;
            let mut resets = 0;
            for window in points.windows(2) {
                if window[1].value >= window[0].value {
                    total += window[1].value - window[0].value;
                } else {
                    resets += 1;
                    total += window[1].value.max(0.0);
                }
            }
            (total, resets)
        }
    }
}

fn aggregate(
    aggregation: MetricAggregation,
    semantic_total: f64,
    points: &[MetricPointV1],
) -> Option<f64> {
    match aggregation {
        MetricAggregation::Raw => Some(semantic_total),
        MetricAggregation::Sum => Some(points.iter().map(|point| point.value).sum()),
        MetricAggregation::Avg => {
            Some(points.iter().map(|point| point.value).sum::<f64>() / points.len() as f64)
        }
        MetricAggregation::Min => points.iter().map(|point| point.value).reduce(f64::min),
        MetricAggregation::Max => points.iter().map(|point| point.value).reduce(f64::max),
        MetricAggregation::Count => Some(points.len() as f64),
        MetricAggregation::Rate => {
            let duration = points
                .last()
                .zip(points.first())
                .map(|(last, first)| last.time_unix_nano - first.time_unix_nano)
                .unwrap_or(0);
            (duration > 0).then_some(semantic_total / (duration as f64 / 1_000_000_000.0))
        }
    }
}

fn merge_histograms(points: &[MetricPointV1]) -> Result<Option<MetricHistogramV1>> {
    merge_histogram_refs(points.iter().filter_map(|point| point.histogram.as_ref()))
}

fn merge_histogram_refs<'a>(
    histograms: impl Iterator<Item = &'a MetricHistogramV1>,
) -> Result<Option<MetricHistogramV1>> {
    let mut merged: Option<MetricHistogramV1> = None;
    for histogram in histograms {
        match &mut merged {
            Some(merged) => merged.merge(histogram)?,
            None => {
                histogram.validate()?;
                merged = Some(histogram.clone());
            }
        }
    }
    Ok(merged)
}

fn merge_counts(left: &mut [u64], right: &[u64]) {
    for (left, right) in left.iter_mut().zip(right) {
        *left = left.saturating_add(*right);
    }
}

fn optional_min(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (left, right) => left.or(right),
    }
}

fn optional_max(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (left, right) => left.or(right),
    }
}

fn parse_optional_time(name: &str, value: Option<&str>) -> Result<Option<i64>> {
    value
        .map(|value| parse_time(value).with_context(|| format!("{name} must be RFC3339")))
        .transpose()
}

fn parse_time(value: &str) -> Result<i64> {
    DateTime::parse_from_rfc3339(value)
        .with_context(|| format!("invalid RFC3339 timestamp `{value}`"))?
        .with_timezone(&Utc)
        .timestamp_nanos_opt()
        .context("timestamp is outside nanosecond range")
}

fn sha256(bytes: impl AsRef<[u8]>) -> String {
    hex::encode(Sha256::digest(bytes.as_ref()))
}

const fn default_query_limit() -> usize {
    100
}

fn is_false(value: &bool) -> bool {
    !*value
}
// HANDWRITE-END
