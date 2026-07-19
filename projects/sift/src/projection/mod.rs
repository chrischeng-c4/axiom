// HANDWRITE-BEGIN gap="sift-projection-module" tracker="1660" reason="Export projection contracts, runtime, replay state, and the embedded Lumen adapter."
mod audit_change;
mod error_report;
mod logging;
mod lumen;
mod metric;
mod model;
mod profile;
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
    audit_control_key, error_lifecycle_key, AuditExportManifestV1, AuditLegalHoldV1,
    ErrorLifecycleState, ErrorLifecycleV1, ProjectionCheckpoint, ProjectionDescriptor,
    ProjectionLag, ProjectionStateEnvelope, RebuildComparison, ReplayJob, ReplayState,
    SiftControlState, PROJECTION_STATE_FORMAT_VERSION, SIFT_COMMAND_FORMAT_VERSION,
};
pub use profile::{
    ProfileFlamegraphEntryV1, ProfileFunctionV1, ProfileFunctionValueV1, ProfileLineV1,
    ProfileLocationV1, ProfileMappingV1, ProfilePage, ProfileProjection, ProfileQuery,
    ProfileRecordV1, ProfileStackSampleV1, ProfileView, DEFAULT_PROFILE_RETENTION_DAYS,
    MAX_PROFILE_QUERY_LIMIT, MAX_PROFILE_TOP_LIMIT, PROFILE_RECORD_SCHEMA, PROFILE_SCHEMA_VERSION,
    PROJECTION_PROFILE_STORE,
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
pub use audit_change::{
    export_content_sha256, AuditChangeProjection, AuditChangeRecordV1, AuditExportResponseV1,
    AuditPage, AuditQuery, AUDIT_CHANGE_SCHEMA_VERSION, AUDIT_RECORD_SCHEMA,
    DEFAULT_AUDIT_RETENTION_DAYS, MAX_AUDIT_QUERY_LIMIT, PROJECTION_AUDIT_CHANGE_STORE,
};
pub use error_report::{
    fingerprint as error_fingerprint, ErrorGroupV1, ErrorOccurrenceV1, ErrorPage, ErrorQuery,
    ErrorReportProjection, ERROR_FINGERPRINT_VERSION, ERROR_REPORT_SCHEMA_VERSION,
    MAX_ERROR_QUERY_LIMIT, PROJECTION_ERROR_REPORT_STORE,
};
