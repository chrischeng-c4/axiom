// HANDWRITE-BEGIN gap="sift-ingest-module" tracker="1658" reason="Export the bounded ingest admission, batch, GCP, and OTLP semantic modules."
//! Bounded transport adapters for canonical operational events.
//!
//! This module owns transport decoding and admission only. Accepted items are
//! returned as `OperationalEventV2` and must be appended through Sift's shared
//! state machine by the HTTP boundary.

pub mod batch;
pub mod gcp;
pub mod limits;
pub mod otlp;

pub use batch::{
    BatchItemResult, BatchOutcome, EventWriteRequest, EventWriteResponse, IngestErrorDetail,
};
pub use limits::{AdmissionController, AdmissionError, AdmissionPermit, IngestLimits};
// HANDWRITE-END
