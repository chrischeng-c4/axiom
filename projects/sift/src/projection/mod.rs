// HANDWRITE-BEGIN gap="sift-projection-module" tracker="1660" reason="Export projection contracts, runtime, replay state, and the embedded Lumen adapter."
mod lumen;
mod model;
mod runtime;

pub use lumen::EmbeddedLumenProjection;
pub use model::{
    ProjectionCheckpoint, ProjectionDescriptor, ProjectionLag, ProjectionStateEnvelope,
    RebuildComparison, ReplayJob, ReplayState, SiftControlState, PROJECTION_STATE_FORMAT_VERSION,
    SIFT_COMMAND_FORMAT_VERSION,
};
pub use runtime::{
    Projection, ProjectionRuntime, PROJECTION_BATCH_SIZE, PROJECTION_EVENT_INDEX,
    PROJECTION_RETRY_AFTER_SECONDS,
};

// HANDWRITE-END
