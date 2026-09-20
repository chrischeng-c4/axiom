//! Strict, dependency-free receipts for qualifying durable performance cells.
//!
//! The ignored release workload writes one receipt only after its HTTP,
//! checkpoint, merge, recovery, and ledger assertions have passed. A later
//! aggregate verifier accepts exactly the sixteen approved matrix cells. It
//! rejects a diagnostic, failed, incomplete, mismatched, or duplicate receipt.
//!
//! This module uses a small JSON reader instead of a permissive map. Receipt
//! objects have fixed keys and integer-only measurements. This lets a Python
//! release verifier consume the same bytes without trusting a prose report.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub const SCHEMA_VERSION: u64 = 1;
pub const RECEIPT_KIND: &str = "lumen.durable-perf-cell";
pub const INPUT_SECONDS: u64 = 30 * 60;
pub const DOCOPS_PER_SECOND: u64 = 100;
pub const QUERY_QPS: u64 = 10;
pub const COMPLETED_PERCENT: u64 = 95;
pub const QUERY_P99_LIMIT_MS: u64 = 1_000;
pub const QUERY_MAX_LIMIT_MS: u64 = 5_000;
pub const DRAIN_LIMIT_MS: u64 = 60_000;
pub const RESTART_LIMIT_MS: u64 = 30_000;
pub const RSS_LIMIT_BYTES: u64 = 12 * 1024 * 1024 * 1024;
pub const DOCKER_CPUS_MILLI: u64 = 2_500;
pub const DOCKER_MEMORY_BYTES: u64 = 16 * 1024 * 1024 * 1024;
pub const SNAPSHOT_SECONDS: u64 = 15;
pub const HOT_DOCUMENTS: u64 = 500_000;
pub const IDLE_COLLECTIONS: u64 = 181;
pub const IDLE_DOCUMENTS_PER_COLLECTION: u64 = 100;
pub const FIELD_COUNT: u64 = 14;
pub const WORKLOAD_SEED: u64 = 0x4246_0610_D00D_B1E5;
pub const NGRAM_TEXT_FIELDS: u64 = 3;
pub const VECTOR_DIMENSIONS: u64 = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub repository: String,
    pub run_id: String,
    pub run_attempt: String,
    pub commit: String,
    pub image_reference: String,
    pub actual_image_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell {
    pub id: String,
    pub endpoint: String,
    pub batch_size: u64,
    pub backend: String,
}

impl Cell {
    pub fn new(endpoint: impl Into<String>, batch_size: u64, backend: impl Into<String>) -> Self {
        let endpoint = endpoint.into();
        let backend = backend.into();
        Self {
            id: format!("{endpoint}-{batch_size}-{backend}"),
            endpoint,
            batch_size,
            backend,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limits {
    pub input_seconds: u64,
    pub docops_per_second: u64,
    pub query_qps: u64,
    pub completed_percent: u64,
    pub query_p99_limit_ms: u64,
    pub query_max_limit_ms: u64,
    pub drain_limit_ms: u64,
    pub rss_limit_bytes: u64,
    pub docker_cpus_milli: u64,
    pub docker_memory_bytes: u64,
    pub snapshot_seconds: u64,
    pub hot_documents: u64,
    pub idle_collections: u64,
    pub idle_documents_per_collection: u64,
    pub field_count: u64,
    pub seed: u64,
    pub ngram_text_fields: u64,
    pub vector_dimensions: u64,
}

impl Limits {
    pub fn approved() -> Self {
        Self {
            input_seconds: INPUT_SECONDS,
            docops_per_second: DOCOPS_PER_SECOND,
            query_qps: QUERY_QPS,
            completed_percent: COMPLETED_PERCENT,
            query_p99_limit_ms: QUERY_P99_LIMIT_MS,
            query_max_limit_ms: QUERY_MAX_LIMIT_MS,
            drain_limit_ms: DRAIN_LIMIT_MS,
            rss_limit_bytes: RSS_LIMIT_BYTES,
            docker_cpus_milli: DOCKER_CPUS_MILLI,
            docker_memory_bytes: DOCKER_MEMORY_BYTES,
            snapshot_seconds: SNAPSHOT_SECONDS,
            hot_documents: HOT_DOCUMENTS,
            idle_collections: IDLE_COLLECTIONS,
            idle_documents_per_collection: IDLE_DOCUMENTS_PER_COLLECTION,
            field_count: FIELD_COUNT,
            seed: WORKLOAD_SEED,
            ngram_text_fields: NGRAM_TEXT_FIELDS,
            vector_dimensions: VECTOR_DIMENSIONS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measurement {
    pub input_duration_ms: u64,
    pub observed_input_elapsed_ms: u64,
    pub requests_offered: u64,
    pub requests_finished: u64,
    pub requests_completed: u64,
    pub requests_started_in_input: u64,
    pub requests_finished_in_input: u64,
    pub requests_completed_in_input: u64,
    pub requests_started_per_second_milli: u64,
    pub requests_completed_per_second_milli: u64,
    pub request_errors: u64,
    pub client_cancellations: u64,
    pub items_offered: u64,
    pub items_completed: u64,
    pub items_failed: u64,
    pub items_started_in_input: u64,
    pub items_completed_in_input: u64,
    pub items_started_per_second_milli: u64,
    pub items_completed_per_second_milli: u64,
    pub docops_offered: u64,
    pub docops_completed: u64,
    pub docops_offered_in_input: u64,
    pub docops_completed_in_input: u64,
    pub index_requests_started_in_input: u64,
    pub replace_requests_started_in_input: u64,
    pub unindex_requests_started_in_input: u64,
    pub docops_offered_per_second_milli: u64,
    pub docops_completed_per_second_milli: u64,
    pub docops_completion_percent_milli: u64,
    pub request_latency_p99_ms: u64,
    pub request_latency_max_ms: u64,
    pub queries_offered: u64,
    pub queries_completed: u64,
    pub queries_started_in_input: u64,
    pub queries_completed_in_input: u64,
    pub hot_queries_started_in_input: u64,
    pub idle_queries_started_in_input: u64,
    pub queries_started_per_second_milli: u64,
    pub queries_completed_per_second_milli: u64,
    pub query_errors_or_timeouts: u64,
    pub query_latency_p99_ms: u64,
    pub query_latency_max_ms: u64,
    pub request_drain_ms: u64,
    pub query_drain_ms: u64,
    pub checkpoint_delta: u64,
    pub merge_delta: u64,
    pub checkpoint_bytes: u64,
    pub merge_read_bytes: u64,
    pub merge_write_bytes: u64,
    pub capture_hold_ns_total: u64,
    pub pending_delta_bytes: u64,
    pub pending_delta_layers: u64,
    pub backpressure_events: u64,
    pub segment_disk_bytes: u64,
    pub peak_rss_bytes: u64,
    pub restart_duration_ms: u64,
    pub restart_recovered: bool,
    pub live_mutation_readback: bool,
    pub cold_mutation_readback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub qualifying: bool,
    pub diagnostic: bool,
    pub succeeded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    pub schema_version: u64,
    pub kind: String,
    pub binding: Binding,
    pub cell: Cell,
    pub limits: Limits,
    pub measurement: Measurement,
    pub outcome: Outcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregateExpectation {
    pub repository: String,
    pub run_id: String,
    pub run_attempt: String,
    pub commit: String,
    pub image_reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aggregate {
    pub actual_image_id: String,
    pub cells: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiptError {
    Json(String),
    MissingKey {
        context: &'static str,
        key: &'static str,
    },
    UnknownKey {
        context: &'static str,
        key: String,
    },
    Type {
        context: &'static str,
        key: &'static str,
    },
    Invalid(String),
    BindingMismatch(String),
    DuplicateCell(String),
    UnexpectedCell(String),
    MissingCell(String),
    CellCount {
        actual: usize,
        expected: usize,
    },
    Io(String),
}

impl Display for ReceiptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(detail) => write!(formatter, "invalid receipt JSON: {detail}"),
            Self::MissingKey { context, key } => write!(formatter, "{context} is missing {key}"),
            Self::UnknownKey { context, key } => {
                write!(formatter, "{context} has unknown key {key}")
            }
            Self::Type { context, key } => {
                write!(formatter, "{context}.{key} has the wrong JSON type")
            }
            Self::Invalid(detail) => {
                write!(formatter, "invalid durable performance receipt: {detail}")
            }
            Self::BindingMismatch(detail) => write!(
                formatter,
                "receipt binding differs from the qualifying run: {detail}"
            ),
            Self::DuplicateCell(cell) => {
                write!(formatter, "duplicate qualifying receipt for cell {cell}")
            }
            Self::UnexpectedCell(cell) => {
                write!(formatter, "receipt has an unapproved matrix cell {cell}")
            }
            Self::MissingCell(cell) => {
                write!(formatter, "missing qualifying receipt for cell {cell}")
            }
            Self::CellCount { actual, expected } => write!(
                formatter,
                "receipt aggregate has {actual} cells, expected {expected}"
            ),
            Self::Io(detail) => write!(formatter, "cannot write performance receipt: {detail}"),
        }
    }
}

impl std::error::Error for ReceiptError {}

type Result<T> = std::result::Result<T, ReceiptError>;

impl Receipt {
    pub fn to_json(&self) -> String {
        object(&[
            ("schema_version", number(self.schema_version)),
            ("kind", string(&self.kind)),
            ("binding", self.binding.to_json()),
            ("cell", self.cell.to_json()),
            ("limits", self.limits.to_json()),
            ("measurement", self.measurement.to_json()),
            ("outcome", self.outcome.to_json()),
        ])
    }

    pub fn from_json(bytes: &str) -> Result<Self> {
        let root = parse_json(bytes)?;
        let mut root = object_value(root, "receipt")?;
        require_exact_keys(
            &root,
            "receipt",
            &[
                "schema_version",
                "kind",
                "binding",
                "cell",
                "limits",
                "measurement",
                "outcome",
            ],
        )?;
        Ok(Self {
            schema_version: take_number(&mut root, "receipt", "schema_version")?,
            kind: take_string(&mut root, "receipt", "kind")?,
            binding: Binding::from_json(take_value(&mut root, "receipt", "binding")?)?,
            cell: Cell::from_json(take_value(&mut root, "receipt", "cell")?)?,
            limits: Limits::from_json(take_value(&mut root, "receipt", "limits")?)?,
            measurement: Measurement::from_json(take_value(&mut root, "receipt", "measurement")?)?,
            outcome: Outcome::from_json(take_value(&mut root, "receipt", "outcome")?)?,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(ReceiptError::Invalid(format!(
                "schema_version must be {SCHEMA_VERSION}, got {}",
                self.schema_version
            )));
        }
        if self.kind != RECEIPT_KIND {
            return Err(ReceiptError::Invalid(format!(
                "kind must be {RECEIPT_KIND}, got {}",
                self.kind
            )));
        }
        validate_binding(&self.binding)?;
        validate_cell(&self.cell)?;
        if self.limits != Limits::approved() {
            return Err(ReceiptError::Invalid(
                "limits do not equal the approved durable workload configuration".to_owned(),
            ));
        }
        if !self.outcome.qualifying || self.outcome.diagnostic || !self.outcome.succeeded {
            return Err(ReceiptError::Invalid(
                "a qualifying receipt must be qualifying=true, diagnostic=false, succeeded=true"
                    .to_owned(),
            ));
        }
        validate_measurement(&self.measurement, &self.limits)
    }
}

impl Binding {
    fn to_json(&self) -> String {
        object(&[
            ("repository", string(&self.repository)),
            ("run_id", string(&self.run_id)),
            ("run_attempt", string(&self.run_attempt)),
            ("commit", string(&self.commit)),
            ("image_reference", string(&self.image_reference)),
            ("actual_image_id", string(&self.actual_image_id)),
        ])
    }

    fn from_json(value: Json) -> Result<Self> {
        let mut object = object_value(value, "binding")?;
        require_exact_keys(
            &object,
            "binding",
            &[
                "repository",
                "run_id",
                "run_attempt",
                "commit",
                "image_reference",
                "actual_image_id",
            ],
        )?;
        Ok(Self {
            repository: take_string(&mut object, "binding", "repository")?,
            run_id: take_string(&mut object, "binding", "run_id")?,
            run_attempt: take_string(&mut object, "binding", "run_attempt")?,
            commit: take_string(&mut object, "binding", "commit")?,
            image_reference: take_string(&mut object, "binding", "image_reference")?,
            actual_image_id: take_string(&mut object, "binding", "actual_image_id")?,
        })
    }
}

impl Cell {
    fn to_json(&self) -> String {
        object(&[
            ("id", string(&self.id)),
            ("endpoint", string(&self.endpoint)),
            ("batch_size", number(self.batch_size)),
            ("backend", string(&self.backend)),
        ])
    }

    fn from_json(value: Json) -> Result<Self> {
        let mut object = object_value(value, "cell")?;
        require_exact_keys(
            &object,
            "cell",
            &["id", "endpoint", "batch_size", "backend"],
        )?;
        Ok(Self {
            id: take_string(&mut object, "cell", "id")?,
            endpoint: take_string(&mut object, "cell", "endpoint")?,
            batch_size: take_number(&mut object, "cell", "batch_size")?,
            backend: take_string(&mut object, "cell", "backend")?,
        })
    }
}

impl Limits {
    fn to_json(&self) -> String {
        object(&[
            ("input_seconds", number(self.input_seconds)),
            ("docops_per_second", number(self.docops_per_second)),
            ("query_qps", number(self.query_qps)),
            ("completed_percent", number(self.completed_percent)),
            ("query_p99_limit_ms", number(self.query_p99_limit_ms)),
            ("query_max_limit_ms", number(self.query_max_limit_ms)),
            ("drain_limit_ms", number(self.drain_limit_ms)),
            ("rss_limit_bytes", number(self.rss_limit_bytes)),
            ("docker_cpus_milli", number(self.docker_cpus_milli)),
            ("docker_memory_bytes", number(self.docker_memory_bytes)),
            ("snapshot_seconds", number(self.snapshot_seconds)),
            ("hot_documents", number(self.hot_documents)),
            ("idle_collections", number(self.idle_collections)),
            (
                "idle_documents_per_collection",
                number(self.idle_documents_per_collection),
            ),
            ("field_count", number(self.field_count)),
            ("seed", number(self.seed)),
            ("ngram_text_fields", number(self.ngram_text_fields)),
            ("vector_dimensions", number(self.vector_dimensions)),
        ])
    }

    fn from_json(value: Json) -> Result<Self> {
        let mut object = object_value(value, "limits")?;
        require_exact_keys(
            &object,
            "limits",
            &[
                "input_seconds",
                "docops_per_second",
                "query_qps",
                "completed_percent",
                "query_p99_limit_ms",
                "query_max_limit_ms",
                "drain_limit_ms",
                "rss_limit_bytes",
                "docker_cpus_milli",
                "docker_memory_bytes",
                "snapshot_seconds",
                "hot_documents",
                "idle_collections",
                "idle_documents_per_collection",
                "field_count",
                "seed",
                "ngram_text_fields",
                "vector_dimensions",
            ],
        )?;
        Ok(Self {
            input_seconds: take_number(&mut object, "limits", "input_seconds")?,
            docops_per_second: take_number(&mut object, "limits", "docops_per_second")?,
            query_qps: take_number(&mut object, "limits", "query_qps")?,
            completed_percent: take_number(&mut object, "limits", "completed_percent")?,
            query_p99_limit_ms: take_number(&mut object, "limits", "query_p99_limit_ms")?,
            query_max_limit_ms: take_number(&mut object, "limits", "query_max_limit_ms")?,
            drain_limit_ms: take_number(&mut object, "limits", "drain_limit_ms")?,
            rss_limit_bytes: take_number(&mut object, "limits", "rss_limit_bytes")?,
            docker_cpus_milli: take_number(&mut object, "limits", "docker_cpus_milli")?,
            docker_memory_bytes: take_number(&mut object, "limits", "docker_memory_bytes")?,
            snapshot_seconds: take_number(&mut object, "limits", "snapshot_seconds")?,
            hot_documents: take_number(&mut object, "limits", "hot_documents")?,
            idle_collections: take_number(&mut object, "limits", "idle_collections")?,
            idle_documents_per_collection: take_number(
                &mut object,
                "limits",
                "idle_documents_per_collection",
            )?,
            field_count: take_number(&mut object, "limits", "field_count")?,
            seed: take_number(&mut object, "limits", "seed")?,
            ngram_text_fields: take_number(&mut object, "limits", "ngram_text_fields")?,
            vector_dimensions: take_number(&mut object, "limits", "vector_dimensions")?,
        })
    }
}

impl Measurement {
    fn to_json(&self) -> String {
        object(&[
            ("input_duration_ms", number(self.input_duration_ms)),
            (
                "observed_input_elapsed_ms",
                number(self.observed_input_elapsed_ms),
            ),
            ("requests_offered", number(self.requests_offered)),
            ("requests_finished", number(self.requests_finished)),
            ("requests_completed", number(self.requests_completed)),
            (
                "requests_started_in_input",
                number(self.requests_started_in_input),
            ),
            (
                "requests_finished_in_input",
                number(self.requests_finished_in_input),
            ),
            (
                "requests_completed_in_input",
                number(self.requests_completed_in_input),
            ),
            (
                "requests_started_per_second_milli",
                number(self.requests_started_per_second_milli),
            ),
            (
                "requests_completed_per_second_milli",
                number(self.requests_completed_per_second_milli),
            ),
            ("request_errors", number(self.request_errors)),
            ("client_cancellations", number(self.client_cancellations)),
            ("items_offered", number(self.items_offered)),
            ("items_completed", number(self.items_completed)),
            ("items_failed", number(self.items_failed)),
            (
                "items_started_in_input",
                number(self.items_started_in_input),
            ),
            (
                "items_completed_in_input",
                number(self.items_completed_in_input),
            ),
            (
                "items_started_per_second_milli",
                number(self.items_started_per_second_milli),
            ),
            (
                "items_completed_per_second_milli",
                number(self.items_completed_per_second_milli),
            ),
            ("docops_offered", number(self.docops_offered)),
            ("docops_completed", number(self.docops_completed)),
            (
                "docops_offered_in_input",
                number(self.docops_offered_in_input),
            ),
            (
                "docops_completed_in_input",
                number(self.docops_completed_in_input),
            ),
            (
                "index_requests_started_in_input",
                number(self.index_requests_started_in_input),
            ),
            (
                "replace_requests_started_in_input",
                number(self.replace_requests_started_in_input),
            ),
            (
                "unindex_requests_started_in_input",
                number(self.unindex_requests_started_in_input),
            ),
            (
                "docops_offered_per_second_milli",
                number(self.docops_offered_per_second_milli),
            ),
            (
                "docops_completed_per_second_milli",
                number(self.docops_completed_per_second_milli),
            ),
            (
                "docops_completion_percent_milli",
                number(self.docops_completion_percent_milli),
            ),
            (
                "request_latency_p99_ms",
                number(self.request_latency_p99_ms),
            ),
            (
                "request_latency_max_ms",
                number(self.request_latency_max_ms),
            ),
            ("queries_offered", number(self.queries_offered)),
            ("queries_completed", number(self.queries_completed)),
            (
                "queries_started_in_input",
                number(self.queries_started_in_input),
            ),
            (
                "queries_completed_in_input",
                number(self.queries_completed_in_input),
            ),
            (
                "hot_queries_started_in_input",
                number(self.hot_queries_started_in_input),
            ),
            (
                "idle_queries_started_in_input",
                number(self.idle_queries_started_in_input),
            ),
            (
                "queries_started_per_second_milli",
                number(self.queries_started_per_second_milli),
            ),
            (
                "queries_completed_per_second_milli",
                number(self.queries_completed_per_second_milli),
            ),
            (
                "query_errors_or_timeouts",
                number(self.query_errors_or_timeouts),
            ),
            ("query_latency_p99_ms", number(self.query_latency_p99_ms)),
            ("query_latency_max_ms", number(self.query_latency_max_ms)),
            ("request_drain_ms", number(self.request_drain_ms)),
            ("query_drain_ms", number(self.query_drain_ms)),
            ("checkpoint_delta", number(self.checkpoint_delta)),
            ("merge_delta", number(self.merge_delta)),
            ("checkpoint_bytes", number(self.checkpoint_bytes)),
            ("merge_read_bytes", number(self.merge_read_bytes)),
            ("merge_write_bytes", number(self.merge_write_bytes)),
            ("capture_hold_ns_total", number(self.capture_hold_ns_total)),
            ("pending_delta_bytes", number(self.pending_delta_bytes)),
            ("pending_delta_layers", number(self.pending_delta_layers)),
            ("backpressure_events", number(self.backpressure_events)),
            ("segment_disk_bytes", number(self.segment_disk_bytes)),
            ("peak_rss_bytes", number(self.peak_rss_bytes)),
            ("restart_duration_ms", number(self.restart_duration_ms)),
            ("restart_recovered", boolean(self.restart_recovered)),
            (
                "live_mutation_readback",
                boolean(self.live_mutation_readback),
            ),
            (
                "cold_mutation_readback",
                boolean(self.cold_mutation_readback),
            ),
        ])
    }

    fn from_json(value: Json) -> Result<Self> {
        let mut object = object_value(value, "measurement")?;
        require_exact_keys(
            &object,
            "measurement",
            &[
                "input_duration_ms",
                "observed_input_elapsed_ms",
                "requests_offered",
                "requests_finished",
                "requests_completed",
                "requests_started_in_input",
                "requests_finished_in_input",
                "requests_completed_in_input",
                "requests_started_per_second_milli",
                "requests_completed_per_second_milli",
                "request_errors",
                "client_cancellations",
                "items_offered",
                "items_completed",
                "items_failed",
                "items_started_in_input",
                "items_completed_in_input",
                "items_started_per_second_milli",
                "items_completed_per_second_milli",
                "docops_offered",
                "docops_completed",
                "docops_offered_in_input",
                "docops_completed_in_input",
                "index_requests_started_in_input",
                "replace_requests_started_in_input",
                "unindex_requests_started_in_input",
                "docops_offered_per_second_milli",
                "docops_completed_per_second_milli",
                "docops_completion_percent_milli",
                "request_latency_p99_ms",
                "request_latency_max_ms",
                "queries_offered",
                "queries_completed",
                "queries_started_in_input",
                "queries_completed_in_input",
                "hot_queries_started_in_input",
                "idle_queries_started_in_input",
                "queries_started_per_second_milli",
                "queries_completed_per_second_milli",
                "query_errors_or_timeouts",
                "query_latency_p99_ms",
                "query_latency_max_ms",
                "request_drain_ms",
                "query_drain_ms",
                "checkpoint_delta",
                "merge_delta",
                "checkpoint_bytes",
                "merge_read_bytes",
                "merge_write_bytes",
                "capture_hold_ns_total",
                "pending_delta_bytes",
                "pending_delta_layers",
                "backpressure_events",
                "segment_disk_bytes",
                "peak_rss_bytes",
                "restart_duration_ms",
                "restart_recovered",
                "live_mutation_readback",
                "cold_mutation_readback",
            ],
        )?;
        Ok(Self {
            input_duration_ms: take_number(&mut object, "measurement", "input_duration_ms")?,
            observed_input_elapsed_ms: take_number(
                &mut object,
                "measurement",
                "observed_input_elapsed_ms",
            )?,
            requests_offered: take_number(&mut object, "measurement", "requests_offered")?,
            requests_finished: take_number(&mut object, "measurement", "requests_finished")?,
            requests_completed: take_number(&mut object, "measurement", "requests_completed")?,
            requests_started_in_input: take_number(
                &mut object,
                "measurement",
                "requests_started_in_input",
            )?,
            requests_finished_in_input: take_number(
                &mut object,
                "measurement",
                "requests_finished_in_input",
            )?,
            requests_completed_in_input: take_number(
                &mut object,
                "measurement",
                "requests_completed_in_input",
            )?,
            requests_started_per_second_milli: take_number(
                &mut object,
                "measurement",
                "requests_started_per_second_milli",
            )?,
            requests_completed_per_second_milli: take_number(
                &mut object,
                "measurement",
                "requests_completed_per_second_milli",
            )?,
            request_errors: take_number(&mut object, "measurement", "request_errors")?,
            client_cancellations: take_number(&mut object, "measurement", "client_cancellations")?,
            items_offered: take_number(&mut object, "measurement", "items_offered")?,
            items_completed: take_number(&mut object, "measurement", "items_completed")?,
            items_failed: take_number(&mut object, "measurement", "items_failed")?,
            items_started_in_input: take_number(
                &mut object,
                "measurement",
                "items_started_in_input",
            )?,
            items_completed_in_input: take_number(
                &mut object,
                "measurement",
                "items_completed_in_input",
            )?,
            items_started_per_second_milli: take_number(
                &mut object,
                "measurement",
                "items_started_per_second_milli",
            )?,
            items_completed_per_second_milli: take_number(
                &mut object,
                "measurement",
                "items_completed_per_second_milli",
            )?,
            docops_offered: take_number(&mut object, "measurement", "docops_offered")?,
            docops_completed: take_number(&mut object, "measurement", "docops_completed")?,
            docops_offered_in_input: take_number(
                &mut object,
                "measurement",
                "docops_offered_in_input",
            )?,
            docops_completed_in_input: take_number(
                &mut object,
                "measurement",
                "docops_completed_in_input",
            )?,
            index_requests_started_in_input: take_number(
                &mut object,
                "measurement",
                "index_requests_started_in_input",
            )?,
            replace_requests_started_in_input: take_number(
                &mut object,
                "measurement",
                "replace_requests_started_in_input",
            )?,
            unindex_requests_started_in_input: take_number(
                &mut object,
                "measurement",
                "unindex_requests_started_in_input",
            )?,
            docops_offered_per_second_milli: take_number(
                &mut object,
                "measurement",
                "docops_offered_per_second_milli",
            )?,
            docops_completed_per_second_milli: take_number(
                &mut object,
                "measurement",
                "docops_completed_per_second_milli",
            )?,
            docops_completion_percent_milli: take_number(
                &mut object,
                "measurement",
                "docops_completion_percent_milli",
            )?,
            request_latency_p99_ms: take_number(
                &mut object,
                "measurement",
                "request_latency_p99_ms",
            )?,
            request_latency_max_ms: take_number(
                &mut object,
                "measurement",
                "request_latency_max_ms",
            )?,
            queries_offered: take_number(&mut object, "measurement", "queries_offered")?,
            queries_completed: take_number(&mut object, "measurement", "queries_completed")?,
            queries_started_in_input: take_number(
                &mut object,
                "measurement",
                "queries_started_in_input",
            )?,
            queries_completed_in_input: take_number(
                &mut object,
                "measurement",
                "queries_completed_in_input",
            )?,
            hot_queries_started_in_input: take_number(
                &mut object,
                "measurement",
                "hot_queries_started_in_input",
            )?,
            idle_queries_started_in_input: take_number(
                &mut object,
                "measurement",
                "idle_queries_started_in_input",
            )?,
            queries_started_per_second_milli: take_number(
                &mut object,
                "measurement",
                "queries_started_per_second_milli",
            )?,
            queries_completed_per_second_milli: take_number(
                &mut object,
                "measurement",
                "queries_completed_per_second_milli",
            )?,
            query_errors_or_timeouts: take_number(
                &mut object,
                "measurement",
                "query_errors_or_timeouts",
            )?,
            query_latency_p99_ms: take_number(&mut object, "measurement", "query_latency_p99_ms")?,
            query_latency_max_ms: take_number(&mut object, "measurement", "query_latency_max_ms")?,
            request_drain_ms: take_number(&mut object, "measurement", "request_drain_ms")?,
            query_drain_ms: take_number(&mut object, "measurement", "query_drain_ms")?,
            checkpoint_delta: take_number(&mut object, "measurement", "checkpoint_delta")?,
            merge_delta: take_number(&mut object, "measurement", "merge_delta")?,
            checkpoint_bytes: take_number(&mut object, "measurement", "checkpoint_bytes")?,
            merge_read_bytes: take_number(&mut object, "measurement", "merge_read_bytes")?,
            merge_write_bytes: take_number(&mut object, "measurement", "merge_write_bytes")?,
            capture_hold_ns_total: take_number(
                &mut object,
                "measurement",
                "capture_hold_ns_total",
            )?,
            pending_delta_bytes: take_number(&mut object, "measurement", "pending_delta_bytes")?,
            pending_delta_layers: take_number(&mut object, "measurement", "pending_delta_layers")?,
            backpressure_events: take_number(&mut object, "measurement", "backpressure_events")?,
            segment_disk_bytes: take_number(&mut object, "measurement", "segment_disk_bytes")?,
            peak_rss_bytes: take_number(&mut object, "measurement", "peak_rss_bytes")?,
            restart_duration_ms: take_number(&mut object, "measurement", "restart_duration_ms")?,
            restart_recovered: take_boolean(&mut object, "measurement", "restart_recovered")?,
            live_mutation_readback: take_boolean(
                &mut object,
                "measurement",
                "live_mutation_readback",
            )?,
            cold_mutation_readback: take_boolean(
                &mut object,
                "measurement",
                "cold_mutation_readback",
            )?,
        })
    }
}

impl Outcome {
    fn to_json(&self) -> String {
        object(&[
            ("qualifying", boolean(self.qualifying)),
            ("diagnostic", boolean(self.diagnostic)),
            ("succeeded", boolean(self.succeeded)),
        ])
    }

    fn from_json(value: Json) -> Result<Self> {
        let mut object = object_value(value, "outcome")?;
        require_exact_keys(
            &object,
            "outcome",
            &["qualifying", "diagnostic", "succeeded"],
        )?;
        Ok(Self {
            qualifying: take_boolean(&mut object, "outcome", "qualifying")?,
            diagnostic: take_boolean(&mut object, "outcome", "diagnostic")?,
            succeeded: take_boolean(&mut object, "outcome", "succeeded")?,
        })
    }
}

pub fn expected_cells() -> Vec<Cell> {
    let mut cells = Vec::with_capacity(16);
    for backend in ["flat-cpu", "hnsw-cpu"] {
        for (endpoint, batches) in [
            ("index", &[1_u64, 100, 1_000][..]),
            ("replace", &[1_u64, 32][..]),
            ("unindex", &[1_u64, 100, 1_000][..]),
        ] {
            for batch_size in batches {
                cells.push(Cell::new(endpoint, *batch_size, backend));
            }
        }
    }
    cells
}

/// One explicit qualifying matrix row for the release workflow.
///
/// The receipt schema owns this enumeration so the workflow oracle and the
/// aggregate verifier cannot silently drift to different cells. Exactly one
/// row also runs the existing product and supply-chain gates; the other rows
/// run the same qualifying performance command only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualifyingWorkflowCell {
    pub cell: Cell,
    pub product_gates: bool,
}

pub fn qualifying_workflow_cells() -> Vec<QualifyingWorkflowCell> {
    expected_cells()
        .into_iter()
        .map(|cell| QualifyingWorkflowCell {
            product_gates: cell.id == "index-1-flat-cpu",
            cell,
        })
        .collect()
}

pub fn validate_aggregate(
    expected: &AggregateExpectation,
    receipts: &[Receipt],
) -> Result<Aggregate> {
    let expected_cells = expected_cells();
    let expected_ids = expected_cells
        .iter()
        .map(|cell| cell.id.clone())
        .collect::<BTreeSet<_>>();
    if receipts.len() != expected_ids.len() {
        return Err(ReceiptError::CellCount {
            actual: receipts.len(),
            expected: expected_ids.len(),
        });
    }
    let mut actual_image_id = None;
    let mut seen = BTreeSet::new();
    for receipt in receipts {
        receipt.validate()?;
        validate_binding_against(expected, &receipt.binding)?;
        if !expected_ids.contains(&receipt.cell.id) {
            return Err(ReceiptError::UnexpectedCell(receipt.cell.id.clone()));
        }
        if !seen.insert(receipt.cell.id.clone()) {
            return Err(ReceiptError::DuplicateCell(receipt.cell.id.clone()));
        }
        match &actual_image_id {
            None => actual_image_id = Some(receipt.binding.actual_image_id.clone()),
            Some(prior) if prior != &receipt.binding.actual_image_id => {
                return Err(ReceiptError::BindingMismatch(format!(
                    "actual_image_id {} differs from {}",
                    receipt.binding.actual_image_id, prior
                )))
            }
            Some(_) => {}
        }
    }
    for cell in expected_ids.difference(&seen) {
        return Err(ReceiptError::MissingCell(cell.clone()));
    }
    Ok(Aggregate {
        actual_image_id: actual_image_id.expect("exact nonempty receipt set"),
        cells: seen,
    })
}

pub fn write_new(path: &Path, receipt: &Receipt) -> Result<()> {
    receipt.validate()?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| ReceiptError::Io(format!("{}: {error}", path.display())))?;
    output
        .write_all(receipt.to_json().as_bytes())
        .and_then(|_| output.write_all(b"\n"))
        .and_then(|_| output.sync_all())
        .map_err(|error| ReceiptError::Io(format!("{}: {error}", path.display())))
}

fn validate_binding(binding: &Binding) -> Result<()> {
    for (name, value) in [
        ("repository", &binding.repository),
        ("run_id", &binding.run_id),
        ("run_attempt", &binding.run_attempt),
    ] {
        if value.is_empty() || value.chars().any(char::is_control) {
            return Err(ReceiptError::Invalid(format!("{name} is empty or unsafe")));
        }
    }
    if binding.commit.len() < 7
        || binding.commit.len() > 64
        || !binding.commit.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(ReceiptError::Invalid(
            "commit must be a 7..64 hexadecimal Git object name".to_owned(),
        ));
    }
    if !immutable_repository_digest(&binding.image_reference) {
        return Err(ReceiptError::Invalid(
            "image_reference must be an immutable repository@sha256:<64hex> digest".to_owned(),
        ));
    }
    if !image_id(&binding.actual_image_id) {
        return Err(ReceiptError::Invalid(
            "actual_image_id must be sha256:<64hex>".to_owned(),
        ));
    }
    Ok(())
}

fn validate_binding_against(expected: &AggregateExpectation, binding: &Binding) -> Result<()> {
    for (name, actual, wanted) in [
        ("repository", &binding.repository, &expected.repository),
        ("run_id", &binding.run_id, &expected.run_id),
        ("run_attempt", &binding.run_attempt, &expected.run_attempt),
        ("commit", &binding.commit, &expected.commit),
        (
            "image_reference",
            &binding.image_reference,
            &expected.image_reference,
        ),
    ] {
        if actual != wanted {
            return Err(ReceiptError::BindingMismatch(format!(
                "{name} is {actual}, expected {wanted}"
            )));
        }
    }
    Ok(())
}

fn validate_cell(cell: &Cell) -> Result<()> {
    let allowed_batch = match cell.endpoint.as_str() {
        "index" | "unindex" => matches!(cell.batch_size, 1 | 100 | 1_000),
        "replace" => matches!(cell.batch_size, 1 | 32),
        _ => false,
    };
    if !allowed_batch {
        return Err(ReceiptError::Invalid(format!(
            "cell has unapproved endpoint/batch: {}/{}",
            cell.endpoint, cell.batch_size
        )));
    }
    if !matches!(cell.backend.as_str(), "flat-cpu" | "hnsw-cpu") {
        return Err(ReceiptError::Invalid(format!(
            "cell has unapproved backend {}",
            cell.backend
        )));
    }
    let expected_id = format!("{}-{}-{}", cell.endpoint, cell.batch_size, cell.backend);
    if cell.id != expected_id {
        return Err(ReceiptError::Invalid(format!(
            "cell id {} must equal {expected_id}",
            cell.id
        )));
    }
    Ok(())
}

fn validate_measurement(measurement: &Measurement, limits: &Limits) -> Result<()> {
    let minimum_input_ms = limits.input_seconds.saturating_mul(1_000);
    if measurement.input_duration_ms != minimum_input_ms {
        return Err(ReceiptError::Invalid(format!(
            "input membership duration {}ms must equal {}ms",
            measurement.input_duration_ms, minimum_input_ms
        )));
    }
    if measurement.observed_input_elapsed_ms < measurement.input_duration_ms {
        return Err(ReceiptError::Invalid(
            "observed input clock did not cross the end of the membership window".to_owned(),
        ));
    }
    let required_docops = limits
        .docops_per_second
        .saturating_mul(limits.input_seconds);
    let required_queries = limits.query_qps.saturating_mul(limits.input_seconds);
    if measurement.docops_offered_in_input < required_docops {
        return Err(ReceiptError::Invalid(format!(
            "input offered {} document operations, expected at least {required_docops}",
            measurement.docops_offered_in_input
        )));
    }
    if measurement.docops_completed_in_input.saturating_mul(100)
        < measurement
            .docops_offered_in_input
            .saturating_mul(limits.completed_percent)
    {
        return Err(ReceiptError::Invalid(
            "completed document operations are below the approved percentage".to_owned(),
        ));
    }
    if measurement.queries_started_in_input < required_queries
        || measurement.queries_completed_in_input < required_queries
    {
        return Err(ReceiptError::Invalid(format!(
            "input queries started={} completed={}, expected at least {required_queries}",
            measurement.queries_started_in_input, measurement.queries_completed_in_input
        )));
    }
    let expected_docops_offered_rate = rate_milli(
        measurement.docops_offered_in_input,
        measurement.input_duration_ms,
    )?;
    let expected_docops_completed_rate = rate_milli(
        measurement.docops_completed_in_input,
        measurement.input_duration_ms,
    )?;
    let expected_query_started_rate = rate_milli(
        measurement.queries_started_in_input,
        measurement.input_duration_ms,
    )?;
    let expected_query_completed_rate = rate_milli(
        measurement.queries_completed_in_input,
        measurement.input_duration_ms,
    )?;
    let expected_request_started_rate = rate_milli(
        measurement.requests_started_in_input,
        measurement.input_duration_ms,
    )?;
    let expected_request_completed_rate = rate_milli(
        measurement.requests_completed_in_input,
        measurement.input_duration_ms,
    )?;
    let expected_item_started_rate = rate_milli(
        measurement.items_started_in_input,
        measurement.input_duration_ms,
    )?;
    let expected_item_completed_rate = rate_milli(
        measurement.items_completed_in_input,
        measurement.input_duration_ms,
    )?;
    let expected_completion = percent_milli(
        measurement.docops_completed_in_input,
        measurement.docops_offered_in_input,
    )?;
    if measurement.docops_offered_per_second_milli != expected_docops_offered_rate
        || measurement.docops_completed_per_second_milli != expected_docops_completed_rate
        || measurement.queries_started_per_second_milli != expected_query_started_rate
        || measurement.queries_completed_per_second_milli != expected_query_completed_rate
        || measurement.requests_started_per_second_milli != expected_request_started_rate
        || measurement.requests_completed_per_second_milli != expected_request_completed_rate
        || measurement.items_started_per_second_milli != expected_item_started_rate
        || measurement.items_completed_per_second_milli != expected_item_completed_rate
        || measurement.docops_completion_percent_milli != expected_completion
    {
        return Err(ReceiptError::Invalid(
            "integer rate or completion fields do not equal their measured totals".to_owned(),
        ));
    }
    if measurement.requests_offered == 0
        || measurement.requests_finished != measurement.requests_offered
        || measurement.requests_completed != measurement.requests_offered
        || measurement.requests_started_in_input == 0
        || measurement.requests_started_in_input > measurement.requests_offered
        || measurement.requests_finished_in_input > measurement.requests_finished
        || measurement.requests_completed_in_input > measurement.requests_completed
        || measurement.requests_finished_in_input > measurement.requests_started_in_input
        || measurement.requests_completed_in_input > measurement.requests_finished_in_input
        || measurement.items_offered == 0
        || measurement.items_completed != measurement.items_offered
        || measurement.items_started_in_input == 0
        || measurement.items_started_in_input > measurement.items_offered
        || measurement.items_completed_in_input > measurement.items_completed
        || measurement.items_completed_in_input > measurement.items_started_in_input
        || measurement.docops_offered < measurement.docops_offered_in_input
        || measurement.docops_completed < measurement.docops_completed_in_input
        || measurement.docops_completed_in_input > measurement.docops_offered_in_input
        || measurement.docops_completed != measurement.docops_offered
        || measurement.docops_offered > measurement.items_offered
        || measurement.docops_completed > measurement.items_completed
        || measurement.queries_offered < measurement.queries_started_in_input
        || measurement.queries_completed < measurement.queries_completed_in_input
        || measurement.queries_completed != measurement.queries_offered
    {
        return Err(ReceiptError::Invalid(
            "request, item, document, or query totals are incomplete".to_owned(),
        ));
    }
    let mutation_request_starts = measurement
        .index_requests_started_in_input
        .checked_add(measurement.replace_requests_started_in_input)
        .and_then(|sum| sum.checked_add(measurement.unindex_requests_started_in_input))
        .ok_or_else(|| {
            ReceiptError::Invalid("mutation request input counts overflow".to_owned())
        })?;
    if measurement.index_requests_started_in_input == 0
        || measurement.replace_requests_started_in_input == 0
        || measurement.unindex_requests_started_in_input == 0
        || mutation_request_starts != measurement.requests_started_in_input
    {
        return Err(ReceiptError::Invalid(
            "each mutation endpoint must contribute actual in-window request bodies".to_owned(),
        ));
    }
    if measurement.hot_queries_started_in_input == 0
        || measurement.idle_queries_started_in_input == 0
        || measurement
            .hot_queries_started_in_input
            .checked_add(measurement.idle_queries_started_in_input)
            .ok_or_else(|| ReceiptError::Invalid("query-class input counts overflow".to_owned()))?
            != measurement.queries_started_in_input
    {
        return Err(ReceiptError::Invalid(
            "hot and idle queries must both contribute to the combined in-window QPS".to_owned(),
        ));
    }
    if measurement.request_errors != 0
        || measurement.client_cancellations != 0
        || measurement.items_failed != 0
        || measurement.query_errors_or_timeouts != 0
    {
        return Err(ReceiptError::Invalid(
            "qualifying receipt reports request, item, cancellation, or query failure".to_owned(),
        ));
    }
    if measurement.request_latency_p99_ms > measurement.request_latency_max_ms
        || measurement.request_latency_p99_ms > limits.query_p99_limit_ms
        || measurement.request_latency_max_ms > limits.query_max_limit_ms
        || measurement.query_latency_p99_ms > limits.query_p99_limit_ms
        || measurement.query_latency_p99_ms > measurement.query_latency_max_ms
        || measurement.query_latency_max_ms > limits.query_max_limit_ms
    {
        return Err(ReceiptError::Invalid(
            "latency observations exceed their approved upper bound".to_owned(),
        ));
    }
    if measurement.request_drain_ms > limits.drain_limit_ms
        || measurement.query_drain_ms > limits.drain_limit_ms
    {
        return Err(ReceiptError::Invalid(
            "drain observations exceed the approved limit".to_owned(),
        ));
    }
    if measurement.checkpoint_delta == 0 || measurement.merge_delta == 0 {
        return Err(ReceiptError::Invalid(
            "qualifying receipt requires completed checkpoint and merge deltas".to_owned(),
        ));
    }
    if measurement.checkpoint_bytes == 0
        || measurement.merge_read_bytes == 0
        || measurement.merge_write_bytes == 0
        || measurement.segment_disk_bytes == 0
    {
        return Err(ReceiptError::Invalid(
            "qualifying receipt lacks completed checkpoint/merge I/O or disk evidence".to_owned(),
        ));
    }
    if measurement.peak_rss_bytes == 0 || measurement.peak_rss_bytes > limits.rss_limit_bytes {
        return Err(ReceiptError::Invalid(
            "peak RSS is missing or exceeds the approved limit".to_owned(),
        ));
    }
    if !measurement.restart_recovered {
        return Err(ReceiptError::Invalid(
            "restart did not recover the durable workload".to_owned(),
        ));
    }
    if measurement.restart_duration_ms > RESTART_LIMIT_MS {
        return Err(ReceiptError::Invalid(format!(
            "restart duration {}ms exceeds the approved {}ms limit",
            measurement.restart_duration_ms, RESTART_LIMIT_MS
        )));
    }
    if !measurement.live_mutation_readback || !measurement.cold_mutation_readback {
        return Err(ReceiptError::Invalid(
            "qualifying receipt lacks live or cold mutation target/content readback".to_owned(),
        ));
    }
    Ok(())
}

pub fn rate_milli(count: u64, duration_ms: u64) -> Result<u64> {
    count
        .checked_mul(1_000_000)
        .and_then(|scaled| scaled.checked_div(duration_ms))
        .ok_or_else(|| ReceiptError::Invalid("rate cannot divide by zero or overflow".to_owned()))
}

pub fn percent_milli(completed: u64, offered: u64) -> Result<u64> {
    completed
        .checked_mul(100_000)
        .and_then(|scaled| scaled.checked_div(offered))
        .ok_or_else(|| {
            ReceiptError::Invalid(
                "completion percentage needs offered document operations".to_owned(),
            )
        })
}

fn immutable_repository_digest(value: &str) -> bool {
    let Some((repository, digest)) = value.rsplit_once("@sha256:") else {
        return false;
    };
    !repository.is_empty()
        && digest.len() == 64
        && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn image_id(value: &str) -> bool {
    value.len() == "sha256:".len() + 64
        && value.starts_with("sha256:")
        && value["sha256:".len()..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
}

fn object(fields: &[(&str, String)]) -> String {
    let mut output = String::from("{");
    for (index, (key, value)) in fields.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&string(key));
        output.push(':');
        output.push_str(value);
    }
    output.push('}');
    output
}

fn string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                use std::fmt::Write as _;
                write!(&mut output, "\\u{:04x}", character as u32).expect("write control escape");
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

fn number(value: u64) -> String {
    value.to_string()
}

fn boolean(value: bool) -> String {
    value.to_string()
}

#[derive(Debug)]
enum Json {
    Object(BTreeMap<String, Json>),
    String(String),
    Number(u64),
    Boolean(bool),
}

fn parse_json(input: &str) -> Result<Json> {
    let mut parser = Parser {
        input: input.as_bytes(),
        position: 0,
    };
    let value = parser.value()?;
    parser.whitespace();
    if parser.position != parser.input.len() {
        return Err(ReceiptError::Json(format!(
            "trailing byte at {}",
            parser.position
        )));
    }
    Ok(value)
}

struct Parser<'a> {
    input: &'a [u8],
    position: usize,
}

impl Parser<'_> {
    fn whitespace(&mut self) {
        while self
            .input
            .get(self.position)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            self.position += 1;
        }
    }

    fn value(&mut self) -> Result<Json> {
        self.whitespace();
        match self.input.get(self.position).copied() {
            Some(b'{') => self.object(),
            Some(b'"') => self.string().map(Json::String),
            Some(b't') => self.literal(b"true", Json::Boolean(true)),
            Some(b'f') => self.literal(b"false", Json::Boolean(false)),
            Some(byte) if byte.is_ascii_digit() => self.number().map(Json::Number),
            Some(byte) => Err(ReceiptError::Json(format!(
                "unsupported value byte {:?} at {}",
                byte as char, self.position
            ))),
            None => Err(ReceiptError::Json("unexpected end of input".to_owned())),
        }
    }

    fn literal(&mut self, literal: &[u8], value: Json) -> Result<Json> {
        if self.input[self.position..].starts_with(literal) {
            self.position += literal.len();
            Ok(value)
        } else {
            Err(ReceiptError::Json(format!(
                "expected literal at {}",
                self.position
            )))
        }
    }

    fn object(&mut self) -> Result<Json> {
        self.expect(b'{')?;
        self.whitespace();
        let mut fields = BTreeMap::new();
        if self.peek() == Some(b'}') {
            self.position += 1;
            return Ok(Json::Object(fields));
        }
        loop {
            self.whitespace();
            let key = self.string()?;
            self.whitespace();
            self.expect(b':')?;
            let value = self.value()?;
            if fields.insert(key.clone(), value).is_some() {
                return Err(ReceiptError::Json(format!(
                    "duplicate key {key} at {}",
                    self.position
                )));
            }
            self.whitespace();
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b'}') => {
                    self.position += 1;
                    return Ok(Json::Object(fields));
                }
                _ => {
                    return Err(ReceiptError::Json(format!(
                        "object separator expected at {}",
                        self.position
                    )))
                }
            }
        }
    }

    fn string(&mut self) -> Result<String> {
        self.expect(b'"')?;
        let mut output = String::new();
        loop {
            let byte = *self
                .input
                .get(self.position)
                .ok_or_else(|| ReceiptError::Json("unterminated string".to_owned()))?;
            self.position += 1;
            match byte {
                b'"' => return Ok(output),
                b'\\' => {
                    let escape = *self.input.get(self.position).ok_or_else(|| {
                        ReceiptError::Json("unterminated string escape".to_owned())
                    })?;
                    self.position += 1;
                    match escape {
                        b'"' => output.push('"'),
                        b'\\' => output.push('\\'),
                        b'/' => output.push('/'),
                        b'b' => output.push('\u{0008}'),
                        b'f' => output.push('\u{000c}'),
                        b'n' => output.push('\n'),
                        b'r' => output.push('\r'),
                        b't' => output.push('\t'),
                        b'u' => {
                            let scalar = self.hex4()?;
                            let character = char::from_u32(scalar).ok_or_else(|| {
                                ReceiptError::Json("invalid Unicode scalar".to_owned())
                            })?;
                            output.push(character);
                        }
                        _ => {
                            return Err(ReceiptError::Json(format!(
                                "unsupported string escape at {}",
                                self.position
                            )))
                        }
                    }
                }
                0..=0x1f => {
                    return Err(ReceiptError::Json(
                        "unescaped control byte in string".to_owned(),
                    ))
                }
                byte if byte.is_ascii() => output.push(byte as char),
                byte => {
                    let start = self.position - 1;
                    let width = utf8_width(byte).ok_or_else(|| {
                        ReceiptError::Json(format!("invalid UTF-8 start at {start}"))
                    })?;
                    let end = start + width;
                    let slice = self
                        .input
                        .get(start..end)
                        .ok_or_else(|| ReceiptError::Json("truncated UTF-8 string".to_owned()))?;
                    let decoded = std::str::from_utf8(slice).map_err(|error| {
                        ReceiptError::Json(format!("invalid UTF-8 string: {error}"))
                    })?;
                    output.push_str(decoded);
                    self.position = end;
                }
            }
        }
    }

    fn hex4(&mut self) -> Result<u32> {
        let digits = self
            .input
            .get(self.position..self.position + 4)
            .ok_or_else(|| ReceiptError::Json("truncated Unicode escape".to_owned()))?;
        self.position += 4;
        let mut output = 0_u32;
        for digit in digits {
            output = output
                .checked_mul(16)
                .and_then(|value| match digit {
                    b'0'..=b'9' => Some(value + u32::from(digit - b'0')),
                    b'a'..=b'f' => Some(value + u32::from(digit - b'a' + 10)),
                    b'A'..=b'F' => Some(value + u32::from(digit - b'A' + 10)),
                    _ => None,
                })
                .ok_or_else(|| ReceiptError::Json("invalid Unicode escape".to_owned()))?;
        }
        Ok(output)
    }

    fn number(&mut self) -> Result<u64> {
        let start = self.position;
        if self.peek() == Some(b'0') {
            self.position += 1;
            if self
                .input
                .get(self.position)
                .is_some_and(|byte| byte.is_ascii_digit())
            {
                return Err(ReceiptError::Json("leading zero in integer".to_owned()));
            }
        } else {
            while self
                .input
                .get(self.position)
                .is_some_and(|byte| byte.is_ascii_digit())
            {
                self.position += 1;
            }
        }
        std::str::from_utf8(&self.input[start..self.position])
            .expect("digits are UTF-8")
            .parse::<u64>()
            .map_err(|error| ReceiptError::Json(format!("invalid integer: {error}")))
    }

    fn expect(&mut self, byte: u8) -> Result<()> {
        self.whitespace();
        if self.peek() == Some(byte) {
            self.position += 1;
            Ok(())
        } else {
            Err(ReceiptError::Json(format!(
                "expected {:?} at {}",
                byte as char, self.position
            )))
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.position).copied()
    }
}

fn utf8_width(byte: u8) -> Option<usize> {
    match byte {
        0xc2..=0xdf => Some(2),
        0xe0..=0xef => Some(3),
        0xf0..=0xf4 => Some(4),
        _ => None,
    }
}

fn object_value(value: Json, context: &'static str) -> Result<BTreeMap<String, Json>> {
    match value {
        Json::Object(object) => Ok(object),
        _ => Err(ReceiptError::Invalid(format!(
            "{context} must be an object"
        ))),
    }
}

fn require_exact_keys(
    object: &BTreeMap<String, Json>,
    context: &'static str,
    expected: &[&'static str],
) -> Result<()> {
    for key in expected {
        if !object.contains_key(*key) {
            return Err(ReceiptError::MissingKey { context, key });
        }
    }
    for key in object.keys() {
        if !expected.contains(&key.as_str()) {
            return Err(ReceiptError::UnknownKey {
                context,
                key: key.clone(),
            });
        }
    }
    Ok(())
}

fn take_value(
    object: &mut BTreeMap<String, Json>,
    context: &'static str,
    key: &'static str,
) -> Result<Json> {
    object
        .remove(key)
        .ok_or(ReceiptError::MissingKey { context, key })
}

fn take_string(
    object: &mut BTreeMap<String, Json>,
    context: &'static str,
    key: &'static str,
) -> Result<String> {
    match take_value(object, context, key)? {
        Json::String(value) => Ok(value),
        _ => Err(ReceiptError::Type { context, key }),
    }
}

fn take_number(
    object: &mut BTreeMap<String, Json>,
    context: &'static str,
    key: &'static str,
) -> Result<u64> {
    match take_value(object, context, key)? {
        Json::Number(value) => Ok(value),
        _ => Err(ReceiptError::Type { context, key }),
    }
}

fn take_boolean(
    object: &mut BTreeMap<String, Json>,
    context: &'static str,
    key: &'static str,
) -> Result<bool> {
    match take_value(object, context, key)? {
        Json::Boolean(value) => Ok(value),
        _ => Err(ReceiptError::Type { context, key }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding() -> Binding {
        Binding {
            repository: "chrischeng-c4/axiom".to_owned(),
            run_id: "1234".to_owned(),
            run_attempt: "2".to_owned(),
            commit: "0123456789abcdef0123456789abcdef01234567".to_owned(),
            image_reference: "ghcr.io/chrischeng-c4/lumen@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            actual_image_id: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
        }
    }

    fn expectation() -> AggregateExpectation {
        let binding = binding();
        AggregateExpectation {
            repository: binding.repository,
            run_id: binding.run_id,
            run_attempt: binding.run_attempt,
            commit: binding.commit,
            image_reference: binding.image_reference,
        }
    }

    fn measurement() -> Measurement {
        Measurement {
            input_duration_ms: INPUT_SECONDS * 1_000,
            observed_input_elapsed_ms: INPUT_SECONDS * 1_000,
            requests_offered: 18_000,
            requests_finished: 18_000,
            requests_completed: 18_000,
            requests_started_in_input: 18_000,
            requests_finished_in_input: 18_000,
            requests_completed_in_input: 18_000,
            requests_started_per_second_milli: 10_000,
            requests_completed_per_second_milli: 10_000,
            request_errors: 0,
            client_cancellations: 0,
            items_offered: 2_520_000,
            items_completed: 2_520_000,
            items_failed: 0,
            items_started_in_input: 2_520_000,
            items_completed_in_input: 2_520_000,
            items_started_per_second_milli: 1_400_000,
            items_completed_per_second_milli: 1_400_000,
            docops_offered: 180_000,
            docops_completed: 180_000,
            docops_offered_in_input: 180_000,
            docops_completed_in_input: 180_000,
            index_requests_started_in_input: 6_000,
            replace_requests_started_in_input: 6_000,
            unindex_requests_started_in_input: 6_000,
            docops_offered_per_second_milli: 100_000,
            docops_completed_per_second_milli: 100_000,
            docops_completion_percent_milli: 100_000,
            request_latency_p99_ms: 800,
            request_latency_max_ms: 1_200,
            queries_offered: 18_000,
            queries_completed: 18_000,
            queries_started_in_input: 18_000,
            queries_completed_in_input: 18_000,
            hot_queries_started_in_input: 14_400,
            idle_queries_started_in_input: 3_600,
            queries_started_per_second_milli: 10_000,
            queries_completed_per_second_milli: 10_000,
            query_errors_or_timeouts: 0,
            query_latency_p99_ms: 900,
            query_latency_max_ms: 1_100,
            request_drain_ms: 45,
            query_drain_ms: 50,
            checkpoint_delta: 2,
            merge_delta: 1,
            checkpoint_bytes: 9,
            merge_read_bytes: 10,
            merge_write_bytes: 11,
            capture_hold_ns_total: 12,
            pending_delta_bytes: 13,
            pending_delta_layers: 14,
            backpressure_events: 0,
            segment_disk_bytes: 15,
            peak_rss_bytes: RSS_LIMIT_BYTES - 1,
            restart_duration_ms: 16,
            restart_recovered: true,
            live_mutation_readback: true,
            cold_mutation_readback: true,
        }
    }

    fn receipt(cell: Cell) -> Receipt {
        Receipt {
            schema_version: SCHEMA_VERSION,
            kind: RECEIPT_KIND.to_owned(),
            binding: binding(),
            cell,
            limits: Limits::approved(),
            measurement: measurement(),
            outcome: Outcome {
                qualifying: true,
                diagnostic: false,
                succeeded: true,
            },
        }
    }

    fn aggregate_receipts() -> Vec<Receipt> {
        expected_cells().into_iter().map(receipt).collect()
    }

    #[test]
    fn strict_receipt_round_trip_and_exact_sixteen_cell_aggregate() {
        let receipts = aggregate_receipts();
        for receipt in &receipts {
            let encoded = receipt.to_json();
            assert_eq!(
                Receipt::from_json(&encoded).expect("parse own JSON"),
                *receipt
            );
        }
        let aggregate = validate_aggregate(&expectation(), &receipts)
            .expect("all sixteen approved receipts should validate");
        assert_eq!(aggregate.cells.len(), 16);
        assert_eq!(aggregate.actual_image_id, binding().actual_image_id);
    }

    #[test]
    fn workflow_rows_cover_the_exact_receipt_matrix_once() {
        let rows = qualifying_workflow_cells();
        assert_eq!(rows.len(), 16);
        assert_eq!(
            rows.iter().filter(|row| row.product_gates).count(),
            1,
            "exactly one matrix child may pay the full product-gate cost"
        );
        assert_eq!(
            rows.iter()
                .filter(|row| row.product_gates)
                .map(|row| row.cell.id.as_str())
                .collect::<Vec<_>>(),
            vec!["index-1-flat-cpu"]
        );
        assert_eq!(
            rows.iter().map(|row| row.cell.clone()).collect::<Vec<_>>(),
            expected_cells(),
            "workflow rows must use every receipt cell exactly once"
        );
    }

    #[test]
    fn rejects_missing_duplicate_and_extra_json_keys() {
        let encoded = receipt(expected_cells().remove(0)).to_json();
        let missing = encoded.replacen("\"kind\":\"lumen.durable-perf-cell\",", "", 1);
        assert!(matches!(
            Receipt::from_json(&missing),
            Err(ReceiptError::MissingKey { .. })
        ));
        let duplicate = encoded.replacen(
            "\"schema_version\":1,",
            "\"schema_version\":1,\"schema_version\":1,",
            1,
        );
        assert!(matches!(
            Receipt::from_json(&duplicate),
            Err(ReceiptError::Json(_))
        ));
        let extra = encoded.replacen(
            "\"kind\":\"lumen.durable-perf-cell\",",
            "\"kind\":\"lumen.durable-perf-cell\",\"unexpected\":1,",
            1,
        );
        assert!(matches!(
            Receipt::from_json(&extra),
            Err(ReceiptError::UnknownKey { .. })
        ));
    }

    #[test]
    fn aggregate_rejects_missing_duplicate_and_extra_cells() {
        let expected = expectation();
        let receipts = aggregate_receipts();
        assert!(matches!(
            validate_aggregate(&expected, &receipts[..15]),
            Err(ReceiptError::CellCount { .. })
        ));
        let mut duplicate = receipts.clone();
        duplicate[15].cell = duplicate[0].cell.clone();
        assert!(matches!(
            validate_aggregate(&expected, &duplicate),
            Err(ReceiptError::DuplicateCell(_))
        ));
        let mut extra = receipts.clone();
        extra[15].cell = Cell::new("index", 2, "flat-cpu");
        assert!(matches!(
            validate_aggregate(&expected, &extra),
            Err(ReceiptError::Invalid(_))
        ));
    }

    #[test]
    fn aggregate_rejects_mismatched_binding_and_diagnostic_or_failed_outcome() {
        let expected = expectation();
        let mut mismatched = aggregate_receipts();
        mismatched[0].binding.commit = "abcdef0123456789abcdef0123456789abcdef01".to_owned();
        assert!(matches!(
            validate_aggregate(&expected, &mismatched),
            Err(ReceiptError::BindingMismatch(_))
        ));

        let mut diagnostic = aggregate_receipts();
        diagnostic[0].outcome.diagnostic = true;
        assert!(matches!(
            validate_aggregate(&expected, &diagnostic),
            Err(ReceiptError::Invalid(_))
        ));

        let mut failed = aggregate_receipts();
        failed[0].outcome.succeeded = false;
        assert!(matches!(
            validate_aggregate(&expected, &failed),
            Err(ReceiptError::Invalid(_))
        ));
    }

    #[test]
    fn aggregate_rejects_missing_checkpoint_merge_and_underload() {
        let expected = expectation();

        let mut altered_limits = aggregate_receipts();
        altered_limits[0].limits.docker_memory_bytes -= 1;
        assert!(matches!(
            validate_aggregate(&expected, &altered_limits),
            Err(ReceiptError::Invalid(_))
        ));

        let mut no_checkpoint = aggregate_receipts();
        no_checkpoint[0].measurement.checkpoint_delta = 0;
        assert!(matches!(
            validate_aggregate(&expected, &no_checkpoint),
            Err(ReceiptError::Invalid(_))
        ));

        let mut no_merge = aggregate_receipts();
        no_merge[0].measurement.merge_delta = 0;
        assert!(matches!(
            validate_aggregate(&expected, &no_merge),
            Err(ReceiptError::Invalid(_))
        ));

        let mut underload = aggregate_receipts();
        underload[0].measurement.docops_offered_in_input = 179_999;
        underload[0].measurement.docops_completed_in_input = 179_999;
        underload[0].measurement.docops_offered = 179_999;
        underload[0].measurement.docops_completed = 179_999;
        underload[0].measurement.docops_offered_per_second_milli = 99_999;
        underload[0].measurement.docops_completed_per_second_milli = 99_999;
        assert!(matches!(
            validate_aggregate(&expected, &underload),
            Err(ReceiptError::Invalid(_))
        ));
    }

    #[test]
    fn aggregate_rejects_wrong_actual_rates_missing_mix_and_unproved_readback() {
        let expected = expectation();

        let mut wrong_request_rate = aggregate_receipts();
        wrong_request_rate[0]
            .measurement
            .requests_started_per_second_milli += 1;
        assert!(matches!(
            validate_aggregate(&expected, &wrong_request_rate),
            Err(ReceiptError::Invalid(_))
        ));

        let mut slow_request_p99 = aggregate_receipts();
        slow_request_p99[0].measurement.request_latency_p99_ms = 1_001;
        slow_request_p99[0].measurement.request_latency_max_ms = 1_001;
        assert!(matches!(
            validate_aggregate(&expected, &slow_request_p99),
            Err(ReceiptError::Invalid(_))
        ));

        let mut impossible_in_window_completion = aggregate_receipts();
        impossible_in_window_completion[0]
            .measurement
            .docops_offered = 180_001;
        impossible_in_window_completion[0]
            .measurement
            .docops_completed = 180_001;
        impossible_in_window_completion[0]
            .measurement
            .docops_completed_in_input = 180_001;
        assert!(matches!(
            validate_aggregate(&expected, &impossible_in_window_completion),
            Err(ReceiptError::Invalid(_))
        ));

        let mut impossible_query_percentile = aggregate_receipts();
        impossible_query_percentile[0]
            .measurement
            .query_latency_p99_ms = 900;
        impossible_query_percentile[0]
            .measurement
            .query_latency_max_ms = 800;
        assert!(matches!(
            validate_aggregate(&expected, &impossible_query_percentile),
            Err(ReceiptError::Invalid(_))
        ));

        let mut missing_endpoint = aggregate_receipts();
        missing_endpoint[0]
            .measurement
            .index_requests_started_in_input = 0;
        assert!(matches!(
            validate_aggregate(&expected, &missing_endpoint),
            Err(ReceiptError::Invalid(_))
        ));

        let mut missing_idle = aggregate_receipts();
        missing_idle[0].measurement.idle_queries_started_in_input = 0;
        assert!(matches!(
            validate_aggregate(&expected, &missing_idle),
            Err(ReceiptError::Invalid(_))
        ));

        let mut no_live_readback = aggregate_receipts();
        no_live_readback[0].measurement.live_mutation_readback = false;
        assert!(matches!(
            validate_aggregate(&expected, &no_live_readback),
            Err(ReceiptError::Invalid(_))
        ));

        let mut no_cold_readback = aggregate_receipts();
        no_cold_readback[0].measurement.cold_mutation_readback = false;
        assert!(matches!(
            validate_aggregate(&expected, &no_cold_readback),
            Err(ReceiptError::Invalid(_))
        ));
    }

    #[test]
    fn receipt_accepts_ninety_five_percent_inside_input_then_requires_full_drain() {
        let expected = expectation();
        let mut receipts = aggregate_receipts();
        let measurement = &mut receipts[0].measurement;
        measurement.docops_completed_in_input = 171_000;
        measurement.docops_completed_per_second_milli = 95_000;
        measurement.docops_completion_percent_milli = 95_000;
        validate_aggregate(&expected, &receipts)
            .expect("the allowed five percent may complete during drain");

        let mut incomplete_drain = receipts.clone();
        incomplete_drain[0].measurement.docops_completed = 179_999;
        assert!(matches!(
            validate_aggregate(&expected, &incomplete_drain),
            Err(ReceiptError::Invalid(_))
        ));
    }

    #[test]
    fn receipt_requires_fixed_membership_window_and_observed_clock_crossing_it() {
        let mut wrong_window = receipt(expected_cells().remove(0));
        wrong_window.measurement.input_duration_ms -= 1;
        assert!(matches!(
            wrong_window.validate(),
            Err(ReceiptError::Invalid(_))
        ));

        let mut early_clock = receipt(expected_cells().remove(0));
        early_clock.measurement.observed_input_elapsed_ms -= 1;
        assert!(matches!(
            early_clock.validate(),
            Err(ReceiptError::Invalid(_))
        ));
    }

    #[test]
    fn receipt_rejects_restart_duration_above_thirty_seconds() {
        let mut receipt = receipt(expected_cells().remove(0));
        receipt.measurement.restart_duration_ms = 30_000;
        receipt
            .validate()
            .expect("a restart that takes exactly thirty seconds remains valid");

        receipt.measurement.restart_duration_ms = 30_001;
        assert!(matches!(receipt.validate(), Err(ReceiptError::Invalid(_))));
    }

    #[test]
    fn write_new_refuses_to_overwrite_a_receipt() {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "lumen-perf-receipt-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("current time")
                .as_nanos()
        ));
        let receipt = receipt(expected_cells().remove(0));
        write_new(&path, &receipt).expect("write new receipt");
        assert!(write_new(&path, &receipt).is_err());
        std::fs::remove_file(path).expect("remove test receipt");
    }
}
