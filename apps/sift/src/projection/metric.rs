// HANDWRITE-BEGIN gap="sift-metric-projection" tracker="1667" reason="Define series identity, chunks, temporality, histograms, exemplars, overflow, rollups, typed query, snapshot, and rebuild."
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    ops::Bound::{Excluded, Unbounded},
    path::{Component, Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex, RwLock,
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
pub const METRIC_SCHEMA_VERSION: u32 = 5;
pub const METRIC_CHUNK_POINTS: usize = 256;
pub const DEFAULT_METRIC_MEMTABLE_BYTES: usize = 256 * 1024 * 1024;
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
    #[serde(default)]
    sealed_chunks: Vec<SealedMetricChunk>,
    chunks: Vec<MetricChunkV1>,
    #[serde(default)]
    point_count: usize,
    rollups: Vec<MetricRollupV1>,
    #[serde(skip)]
    resident_bytes: usize,
}

#[derive(Clone, Deserialize, Serialize)]
struct SealedMetricChunk {
    key: String,
    start_time_unix_nano: i64,
    end_time_unix_nano: i64,
    point_count: usize,
    sha256: String,
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct MetricState {
    series: BTreeMap<String, StoredMetricSeries>,
    exact_identities: BTreeMap<String, BTreeSet<String>>,
    overflowed_identities: BTreeSet<String>,
    overflowed_points: u64,
    #[serde(default)]
    projection_cursor: u64,
    #[serde(skip)]
    memtable_bytes: usize,
}

#[derive(Deserialize)]
struct MetricSnapshot {
    state: MetricState,
    cardinality_limit: usize,
    retained_points_per_series: usize,
}

#[derive(Serialize)]
struct MetricSnapshotRef<'a> {
    state: &'a MetricState,
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
    memtable_limit_bytes: usize,
    chunk_store: Option<MetricChunkStore>,
    maintenance_work_points: AtomicU64,
}

struct MetricChunkStore {
    root: PathBuf,
    obsolete: Mutex<storage_durable::FramedLogWriter>,
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
        Self::build(
            cardinality_limit,
            retained_points_per_series,
            usize::MAX,
            None,
        )
    }

    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_limits(
            data_dir,
            DEFAULT_METRIC_CARDINALITY_LIMIT,
            DEFAULT_RETAINED_POINTS_PER_SERIES,
            DEFAULT_METRIC_MEMTABLE_BYTES,
        )
    }

    #[doc(hidden)]
    pub fn open_with_limits(
        data_dir: impl AsRef<Path>,
        cardinality_limit: usize,
        retained_points_per_series: usize,
        memtable_limit_bytes: usize,
    ) -> Result<Self> {
        let chunk_store = MetricChunkStore::open(
            data_dir
                .as_ref()
                .join("indexes")
                .join(PROJECTION_METRIC_STORE)
                .join("chunks"),
        )?;
        Self::build(
            cardinality_limit,
            retained_points_per_series,
            memtable_limit_bytes,
            Some(chunk_store),
        )
    }

    fn build(
        cardinality_limit: usize,
        retained_points_per_series: usize,
        memtable_limit_bytes: usize,
        chunk_store: Option<MetricChunkStore>,
    ) -> Result<Self> {
        if cardinality_limit == 0 || retained_points_per_series == 0 {
            bail!("metric cardinality and retention limits must be non-zero");
        }
        if memtable_limit_bytes == 0 {
            bail!("metric memtable limit must be non-zero");
        }
        Ok(Self {
            state: RwLock::new(MetricState::default()),
            cardinality_limit,
            retained_points_per_series,
            memtable_limit_bytes,
            chunk_store,
            maintenance_work_points: AtomicU64::new(0),
        })
    }

    #[doc(hidden)]
    pub fn maintenance_work_points(&self) -> u64 {
        self.maintenance_work_points.load(Ordering::Relaxed)
    }

    #[doc(hidden)]
    pub fn memtable_bytes(&self) -> usize {
        self.state
            .read()
            .expect("metric projection lock poisoned")
            .memtable_bytes
    }

    #[doc(hidden)]
    pub fn sealed_chunk_count(&self) -> usize {
        self.state
            .read()
            .expect("metric projection lock poisoned")
            .series
            .values()
            .map(|series| series.sealed_chunks.len())
            .sum()
    }

    fn all_points(&self, series: &StoredMetricSeries) -> Result<Vec<MetricPointV1>> {
        let mut points = Vec::with_capacity(series.point_count);
        for sealed in &series.sealed_chunks {
            points.extend(self.read_sealed_chunk(sealed)?.points);
        }
        points.extend(
            series
                .chunks
                .iter()
                .flat_map(|chunk| chunk.points.iter().cloned()),
        );
        Ok(points)
    }

    fn last_point(&self, series: &StoredMetricSeries) -> Result<Option<MetricPointV1>> {
        if let Some(point) = series.chunks.last().and_then(|chunk| chunk.points.last()) {
            return Ok(Some(point.clone()));
        }
        let Some(sealed) = series.sealed_chunks.last() else {
            return Ok(None);
        };
        Ok(self.read_sealed_chunk(sealed)?.points.last().cloned())
    }

    fn read_sealed_chunk(&self, sealed: &SealedMetricChunk) -> Result<MetricChunkV1> {
        self.chunk_store
            .as_ref()
            .context("metric projection has no disk chunk store")?
            .read(sealed)
    }

    fn seal_full_chunks(&self, series: &mut StoredMetricSeries) -> Result<()> {
        let Some(store) = &self.chunk_store else {
            return Ok(());
        };
        while series
            .chunks
            .first()
            .is_some_and(|chunk| chunk.points.len() >= METRIC_CHUNK_POINTS)
        {
            let resident_bytes = metric_chunk_memory_bytes(&series.chunks[0])?;
            let sealed = store.write(&series.series_id, &series.chunks[0])?;
            series.chunks.remove(0);
            series.resident_bytes = series.resident_bytes.saturating_sub(resident_bytes);
            series.sealed_chunks.push(sealed);
        }
        Ok(())
    }

    fn evict_series_to_limit(
        &self,
        series: &mut StoredMetricSeries,
        mutation_cursor: u64,
    ) -> Result<u64> {
        let mut work = 0_u64;
        while series.point_count > self.retained_points_per_series {
            if let Some(sealed) = series.sealed_chunks.first().cloned() {
                let chunk = self.read_sealed_chunk(&sealed)?;
                if let Some(store) = &self.chunk_store {
                    store.mark_obsolete(mutation_cursor, std::slice::from_ref(&sealed.key))?;
                }
                series.sealed_chunks.remove(0);
                series.point_count = series.point_count.saturating_sub(chunk.points.len());
                work = work.saturating_add(chunk.points.len() as u64);
                continue;
            }
            let Some(_) = pop_oldest_metric_point(series)? else {
                break;
            };
            work = work.saturating_add(1);
        }
        Ok(work)
    }

    fn enforce_memtable_limit(&self, state: &mut MetricState) -> Result<()> {
        let Some(store) = &self.chunk_store else {
            if state.memtable_bytes > self.memtable_limit_bytes {
                bail!("metric memtable exceeded its limit without a disk chunk store");
            }
            return Ok(());
        };
        while state.memtable_bytes > self.memtable_limit_bytes {
            let series_id = state
                .series
                .iter()
                .find(|(_, series)| !series.chunks.is_empty())
                .map(|(series_id, _)| series_id.clone())
                .context("metric memtable accounting exceeds limit without resident chunks")?;
            let series = state
                .series
                .get_mut(&series_id)
                .context("selected metric series disappeared")?;
            let resident_bytes = metric_chunk_memory_bytes(&series.chunks[0])?;
            let sealed = store.write(&series.series_id, &series.chunks[0])?;
            series.chunks.remove(0);
            series.resident_bytes = series.resident_bytes.saturating_sub(resident_bytes);
            series.sealed_chunks.push(sealed);
            state.memtable_bytes = state.memtable_bytes.saturating_sub(resident_bytes);
        }
        Ok(())
    }

    pub fn query(&self, query: &MetricQuery) -> Result<MetricPage> {
        let (start, end) = query.validate()?;
        let state = self.state.read().expect("metric projection lock poisoned");
        let start_series = query
            .after_series_id
            .as_ref()
            .map_or(Unbounded, |series_id| Excluded(series_id.clone()));
        let mut results = Vec::with_capacity(query.limit.saturating_add(1));
        for series in state
            .series
            .range((start_series, Unbounded))
            .map(|(_, series)| series)
        {
            if series.project != query.project
                || query
                    .environment
                    .as_ref()
                    .is_some_and(|environment| &series.environment != environment)
                || query.name.as_ref().is_some_and(|name| &series.name != name)
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
            let all_points = self.all_points(series)?;
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
            if results.len() > query.limit {
                break;
            }
        }
        let has_more = results.len() > query.limit;
        results.truncate(query.limit);
        Ok(MetricPage {
            next_series_id: results.last().map(|series| series.series_id.clone()),
            series: results,
            projection_cursor: state.projection_cursor,
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
        if state.projection_cursor >= stored.cursor {
            return Ok(());
        }
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
        let (touched, before, after) = {
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
                        sealed_chunks: Vec::new(),
                        chunks: Vec::new(),
                        point_count: 0,
                        rollups: Vec::new(),
                        resident_bytes: 0,
                    });
            if series.temporality != metric.temporality {
                bail!("metric series temporality changed without changing identity");
            }
            if series.point_count == 0
                && (!series.sealed_chunks.is_empty() || !series.chunks.is_empty())
            {
                series.point_count = series
                    .sealed_chunks
                    .iter()
                    .map(|chunk| chunk.point_count)
                    .chain(series.chunks.iter().map(|chunk| chunk.points.len()))
                    .sum();
            }
            let before = series.resident_bytes;
            let last_point = self.last_point(series)?;
            let append_in_order = last_point
                .as_ref()
                .is_none_or(|last| point_order(last, &point).is_le());
            let touched = if append_in_order {
                push_metric_point(series, point)?;
                self.seal_full_chunks(series)?;
                let eviction_work = self.evict_series_to_limit(series, stored.cursor)?;
                series.rollups.clear();
                1_u64.saturating_add(eviction_work)
            } else {
                let mut points = self.all_points(series)?;
                points.push(point);
                points.sort_by(point_order);
                let removed = points.len().saturating_sub(self.retained_points_per_series);
                if removed > 0 {
                    points.drain(..removed);
                }
                let touched = points.len() as u64 + removed as u64;
                if let Some(store) = &self.chunk_store {
                    let old_keys = series
                        .sealed_chunks
                        .iter()
                        .map(|chunk| chunk.key.clone())
                        .collect::<Vec<_>>();
                    store.mark_obsolete(stored.cursor, &old_keys)?;
                }
                series.sealed_chunks.clear();
                series.chunks = make_chunks(&points);
                series.point_count = points.len();
                series.rollups.clear();
                series.resident_bytes = metric_series_memory_bytes(series)?;
                self.seal_full_chunks(series)?;
                touched
            };
            let after = series.resident_bytes;
            (touched, before, after)
        };
        state.memtable_bytes = state
            .memtable_bytes
            .saturating_sub(before)
            .saturating_add(after);
        self.maintenance_work_points
            .fetch_add(touched, Ordering::Relaxed);
        state.projection_cursor = state.projection_cursor.max(stored.cursor);
        self.enforce_memtable_limit(&mut state)?;
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        let state = self.state.read().expect("metric projection lock poisoned");
        serde_json::to_vec(&MetricSnapshotRef {
            state: &state,
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
        let mut memtable_bytes = 0_usize;
        let mut projection_cursor = state.projection_cursor;
        for series in state.series.values_mut() {
            if !series.sealed_chunks.is_empty() && self.chunk_store.is_none() {
                bail!("metric snapshot contains disk chunks but projection has no chunk store");
            }
            for sealed in &series.sealed_chunks {
                self.read_sealed_chunk(sealed)?;
            }
            series.point_count = series
                .sealed_chunks
                .iter()
                .map(|chunk| chunk.point_count)
                .chain(series.chunks.iter().map(|chunk| chunk.points.len()))
                .sum();
            series.rollups.clear();
            series.resident_bytes = metric_series_memory_bytes(series)?;
            memtable_bytes = memtable_bytes.saturating_add(series.resident_bytes);
            for point in series.chunks.iter().flat_map(|chunk| &chunk.points) {
                projection_cursor = projection_cursor.max(point.cursor);
            }
        }
        state.memtable_bytes = memtable_bytes;
        state.projection_cursor = projection_cursor;
        self.enforce_memtable_limit(&mut state)?;
        *self.state.write().expect("metric projection lock poisoned") = state;
        Ok(())
    }

    fn checkpoint_committed(&self) -> Result<()> {
        let Some(store) = &self.chunk_store else {
            return Ok(());
        };
        let state = self.state.read().expect("metric projection lock poisoned");
        let committed_cursor = state.projection_cursor;
        let referenced = state
            .series
            .values()
            .flat_map(|series| series.sealed_chunks.iter().map(|chunk| chunk.key.clone()))
            .collect::<BTreeSet<_>>();
        drop(state);
        store.cleanup_obsolete(committed_cursor, &referenced)
    }

    fn semantic_digest(&self) -> Result<String> {
        Ok(sha256(&self.snapshot()?))
    }
}

impl MetricChunkStore {
    fn open(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root)
            .with_context(|| format!("create metric chunk root {}", root.display()))?;
        storage_durable::set_private_directory_mode(&root)?;
        let obsolete = storage_durable::FramedLogWriter::open(
            root.join("obsolete.framed"),
            storage_durable::FsyncPolicy::Always,
        )?;
        Ok(Self {
            root,
            obsolete: Mutex::new(obsolete),
        })
    }

    fn write(&self, series_id: &str, chunk: &MetricChunkV1) -> Result<SealedMetricChunk> {
        let first = chunk
            .points
            .first()
            .context("cannot seal an empty metric chunk")?;
        let last = chunk.points.last().context("metric chunk lost its tail")?;
        validate_series_id(series_id)?;
        let bytes = serde_json::to_vec(chunk).context("encode sealed metric chunk")?;
        let digest = sha256(&bytes);
        let key = format!(
            "{series_id}/{:020}-{:020}-{digest}.json",
            first.cursor, last.cursor
        );
        let path = self.path_for(&key)?;
        let parent = path
            .parent()
            .context("metric chunk path has no parent directory")?;
        fs::create_dir_all(parent)
            .with_context(|| format!("create metric series directory {}", parent.display()))?;
        storage_durable::set_private_directory_mode(parent)?;
        storage_durable::atomic_write(&path, &bytes, storage_durable::FsyncPolicy::Always)
            .with_context(|| format!("persist sealed metric chunk {}", path.display()))?;
        storage_durable::set_private_file_mode(&path)?;
        Ok(SealedMetricChunk {
            key,
            start_time_unix_nano: chunk.start_time_unix_nano,
            end_time_unix_nano: chunk.end_time_unix_nano,
            point_count: chunk.points.len(),
            sha256: digest,
        })
    }

    fn read(&self, sealed: &SealedMetricChunk) -> Result<MetricChunkV1> {
        let path = self.path_for(&sealed.key)?;
        let bytes = fs::read(&path)
            .with_context(|| format!("read sealed metric chunk {}", path.display()))?;
        if sha256(&bytes) != sealed.sha256 {
            bail!("sealed metric chunk {} checksum mismatch", sealed.key);
        }
        let chunk: MetricChunkV1 =
            serde_json::from_slice(&bytes).context("decode sealed metric chunk")?;
        if chunk.points.len() != sealed.point_count
            || chunk.start_time_unix_nano != sealed.start_time_unix_nano
            || chunk.end_time_unix_nano != sealed.end_time_unix_nano
            || chunk.points.is_empty()
        {
            bail!("sealed metric chunk {} metadata mismatch", sealed.key);
        }
        Ok(chunk)
    }

    fn mark_obsolete(&self, cursor: u64, keys: &[String]) -> Result<()> {
        if keys.is_empty() {
            return Ok(());
        }
        let mut ledger = self
            .obsolete
            .lock()
            .expect("metric obsolete ledger lock poisoned");
        for key in keys {
            self.path_for(key)?;
            ledger.append(cursor, key.as_bytes())?;
        }
        ledger.sync_strict()
    }

    fn cleanup_obsolete(&self, committed_cursor: u64, referenced: &BTreeSet<String>) -> Result<()> {
        let mut ledger = self
            .obsolete
            .lock()
            .expect("metric obsolete ledger lock poisoned");
        ledger.flush()?;
        let ledger_path = self.root.join("obsolete.framed");
        let mut cursor = storage_durable::FramedLogCursor::open(&ledger_path)?;
        let mut safe_through = None;
        while let Some(frame) = cursor.next_frame()? {
            if frame.seq > committed_cursor {
                break;
            }
            let key =
                String::from_utf8(frame.payload).context("decode obsolete metric chunk key")?;
            if !referenced.contains(&key) {
                self.remove_key(&key)?;
            }
            safe_through = Some(frame.seq);
        }
        drop(cursor);
        if let Some(through) = safe_through {
            ledger.truncate_through(through)?;
        }
        Ok(())
    }

    fn remove_key(&self, key: &str) -> Result<()> {
        let path = self.path_for(key)?;
        match fs::remove_file(&path) {
            Ok(()) => storage_durable::sync_parent_dir(&path),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => {
                Err(error).with_context(|| format!("remove metric chunk {}", path.display()))
            }
        }
    }

    fn path_for(&self, key: &str) -> Result<PathBuf> {
        let relative = Path::new(key);
        let components = relative.components().collect::<Vec<_>>();
        if components.len() != 2
            || components
                .iter()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            bail!("metric chunk key is not a safe relative path");
        }
        let series_id = components[0].as_os_str().to_string_lossy();
        validate_series_id(&series_id)?;
        Ok(self.root.join(relative))
    }
}

fn validate_series_id(series_id: &str) -> Result<()> {
    if series_id.len() != 64 || !series_id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("metric series id is not a SHA-256 hex digest");
    }
    Ok(())
}

fn metric_series_memory_bytes(series: &StoredMetricSeries) -> Result<usize> {
    series.chunks.iter().try_fold(0_usize, |total, chunk| {
        Ok(total.saturating_add(metric_chunk_memory_bytes(chunk)?))
    })
}

fn metric_chunk_memory_bytes(chunk: &MetricChunkV1) -> Result<usize> {
    chunk.points.iter().try_fold(0_usize, |total, point| {
        Ok(total.saturating_add(metric_point_memory_bytes(point)?))
    })
}

fn metric_point_memory_bytes(point: &MetricPointV1) -> Result<usize> {
    Ok(serde_json::to_vec(point)
        .context("measure metric memtable point")?
        .len()
        .saturating_add(std::mem::size_of::<MetricPointV1>()))
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

fn point_order(left: &MetricPointV1, right: &MetricPointV1) -> std::cmp::Ordering {
    left.time_unix_nano
        .cmp(&right.time_unix_nano)
        .then_with(|| left.cursor.cmp(&right.cursor))
        .then_with(|| left.event_id.cmp(&right.event_id))
}

fn push_metric_point(series: &mut StoredMetricSeries, point: MetricPointV1) -> Result<()> {
    let resident_bytes = metric_point_memory_bytes(&point)?;
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
    series.resident_bytes = series.resident_bytes.saturating_add(resident_bytes);
    Ok(())
}

fn pop_oldest_metric_point(series: &mut StoredMetricSeries) -> Result<Option<String>> {
    let Some(chunk) = series.chunks.first_mut() else {
        return Ok(None);
    };
    let point = chunk.points.remove(0);
    let resident_bytes = metric_point_memory_bytes(&point)?;
    series.point_count = series.point_count.saturating_sub(1);
    if series.chunks[0].points.is_empty() {
        series.chunks.remove(0);
    } else {
        series.chunks[0].start_time_unix_nano = series.chunks[0].points[0].time_unix_nano;
    }
    series.resident_bytes = series.resident_bytes.saturating_sub(resident_bytes);
    Ok(Some(point.event_id))
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
