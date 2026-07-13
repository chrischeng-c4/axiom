// SPEC-MANAGED: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
// <HANDWRITE gap="missing-generator:logic:pg-wire-codec" tracker="#1287" reason="Wire protocol codec needs generator primitives that do not exist yet.">
//! `WireCodecConfig` — FrameReader bounds and codec limits, matching the TD
//! Config section byte-for-byte (values sourced from the pgcat/PgBouncer
//! defaults documented there).

use crate::wire::backend::TransactionStatus;

/// Bounds and limits consumed by the incremental [`crate::wire::FrameReader`]
/// and by field-count validation inside frontend/backend message decode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireCodecConfig {
    /// 10 MiB hard cap on a tagged frame's declared length; oversized ->
    /// `FrameError::Oversized`, connection closed by caller, never a panic.
    pub max_frame_bytes: usize,
    /// Cap on the untagged StartupMessage packet length (PgBouncer/pgcat-class
    /// default); oversized -> `FrameError::Oversized`.
    pub max_startup_bytes: usize,
    /// `BytesMut` initial reserve per connection; grows as needed up to
    /// `max_frame_bytes` while a frame is split across reads.
    pub read_buffer_initial_capacity: usize,
    /// Matches the protocol's i16 parameter-count field width; guards
    /// against a corrupt/hostile param count before allocating.
    pub max_bind_params: usize,
    /// Matches Postgres's own column-count ceiling; guards
    /// RowDescription/DataRow parsing against a corrupt field count.
    pub max_row_columns: usize,
    /// `TransactionStatus` before the first ReadyForQuery is observed on a
    /// fresh connection.
    pub initial_transaction_status: TransactionStatus,
}

impl Default for WireCodecConfig {
    fn default() -> Self {
        Self {
            max_frame_bytes: 10_485_760,
            max_startup_bytes: 10_000,
            read_buffer_initial_capacity: 8192,
            max_bind_params: 65_535,
            max_row_columns: 1600,
            initial_transaction_status: TransactionStatus::Idle,
        }
    }
}
// </HANDWRITE>
