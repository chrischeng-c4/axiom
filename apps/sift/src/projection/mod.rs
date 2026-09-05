mod logging;
mod metric;
mod model;
mod runtime;
mod trace;

pub use logging::{
    LogPage, LogQuery, LogRecordV1, LoggingProjection, DEFAULT_RETAINED_LOG_RECORDS,
    LOGGING_SCHEMA_VERSION, MAX_LOG_QUERY_LIMIT, PROJECTION_LOGGING_STORE,
};
pub use metric::{
    HistogramKind, MetricAggregation, MetricChunkV1, MetricHistogramV1, MetricPage, MetricPointV1,
    MetricProjection, MetricQuery, MetricRollupV1, MetricSeriesResultV1,
    DEFAULT_METRIC_CARDINALITY_LIMIT, DEFAULT_RETAINED_POINTS_PER_SERIES, MAX_METRIC_QUERY_LIMIT,
    METRIC_CHUNK_POINTS, METRIC_SCHEMA_VERSION, PROJECTION_METRIC_STORE, ROLLUP_WINDOWS_SECONDS,
};
pub use model::{
    ProjectionCheckpoint, ProjectionDescriptor, ProjectionLag, ProjectionStateEnvelope,
    RebuildComparison, PROJECTION_STATE_FORMAT_VERSION,
};
pub use runtime::{
    Projection, ProjectionRuntime, PROJECTION_BATCH_SIZE, PROJECTION_RETRY_AFTER_SECONDS,
    PROJECTION_SNAPSHOT_INTERVAL_EVENTS,
};
pub use trace::{
    SpanEventV1, SpanLinkV1, SpanRecordV1, TracePage, TraceProjection, TraceQuery, TraceResultV1,
    DEFAULT_RETAINED_TRACE_SPANS, MAX_TRACE_QUERY_LIMIT, PROJECTION_TRACE_STORE,
    TRACE_SCHEMA_VERSION,
};
