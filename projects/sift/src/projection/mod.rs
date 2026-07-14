// HANDWRITE-BEGIN gap="sift-projection-module" tracker="1660" reason="Export projection contracts, runtime, replay state, and the embedded Lumen adapter."
mod error_report;
mod logging;
mod lumen;
mod metric;
mod model;
mod runtime;
mod trace;

pub use logging::{
    LogPage, LogQuery, LogRecordV1, LoggingProjection, DEFAULT_RETAINED_LOG_RECORDS,
    LOGGING_SCHEMA_VERSION, MAX_LOG_QUERY_LIMIT, PROJECTION_LOGGING_STORE,
};
pub use lumen::EmbeddedLumenProjection;
pub use metric::{
    HistogramKind, MetricAggregation, MetricChunkV1, MetricHistogramV1, MetricPage, MetricPointV1,
    MetricProjection, MetricQuery, MetricRollupV1, MetricSeriesResultV1,
    DEFAULT_METRIC_CARDINALITY_LIMIT, DEFAULT_RETAINED_POINTS_PER_SERIES, MAX_METRIC_QUERY_LIMIT,
    METRIC_CHUNK_POINTS, METRIC_SCHEMA_VERSION, PROJECTION_METRIC_STORE, ROLLUP_WINDOWS_SECONDS,
};
pub use model::{
    error_lifecycle_key, ErrorLifecycleState, ErrorLifecycleV1, ProjectionCheckpoint,
    ProjectionDescriptor, ProjectionLag, ProjectionStateEnvelope, RebuildComparison, ReplayJob,
    ReplayState, SiftControlState, PROJECTION_STATE_FORMAT_VERSION, SIFT_COMMAND_FORMAT_VERSION,
};
pub use runtime::{
    Projection, ProjectionRuntime, PROJECTION_BATCH_SIZE, PROJECTION_EVENT_INDEX,
    PROJECTION_RETRY_AFTER_SECONDS,
};
pub use trace::{
    SpanEventV1, SpanLinkV1, SpanRecordV1, TraceProjection, TraceResultV1, PROJECTION_TRACE_STORE,
    TRACE_SCHEMA_VERSION,
};

// HANDWRITE-END
pub use error_report::{
    fingerprint as error_fingerprint, ErrorGroupV1, ErrorOccurrenceV1, ErrorPage, ErrorQuery,
    ErrorReportProjection, ERROR_FINGERPRINT_VERSION, ERROR_REPORT_SCHEMA_VERSION,
    MAX_ERROR_QUERY_LIMIT, PROJECTION_ERROR_REPORT_STORE,
};
