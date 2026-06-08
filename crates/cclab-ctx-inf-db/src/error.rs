use crate::types::{EntityId, RelationId};

#[derive(Debug, thiserror::Error)]
pub enum CtxInfError {
    #[error("entity not found: {0}")]
    EntityNotFound(EntityId),

    #[error("relation not found: {0}")]
    RelationNotFound(RelationId),

    #[error("version conflict: expected {expected}, actual {actual}")]
    VersionConflict { expected: u64, actual: u64 },

    #[error("invalid entity type: {0}")]
    InvalidEntityType(String),

    #[error("invalid relation type: {0}")]
    InvalidRelationType(String),

    #[error("dangling reference: entity {0} does not exist")]
    DanglingReference(EntityId),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("WAL error: {0}")]
    Wal(String),

    #[error("snapshot error: {0}")]
    Snapshot(String),

    #[error("recovery error: {0}")]
    Recovery(String),

    #[error("GPU unavailable: {0}")]
    GpuUnavailable(String),

    #[error(
        "out of memory: {tier} tier, requested {requested} bytes, available {available} bytes"
    )]
    OutOfMemory {
        tier: String,
        requested: usize,
        available: usize,
    },
}

pub type Result<T> = std::result::Result<T, CtxInfError>;
