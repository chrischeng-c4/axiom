// HANDWRITE-BEGIN gap="sift-event-module" tracker="1657" reason="Export the versioned event model and governance policy as one semantic event boundary."
mod governance;
mod model;

pub use governance::{GovernancePolicy, GovernancePolicySet};
pub use model::{
    decode_event_json, AttributeValue, ContentBlobRef, IncomingEvent, InstrumentationScope,
    MetricExemplar, MetricPoint, MetricTemporality, OperationalEventV2, SignalKind,
    EVENT_SCHEMA_URL, EVENT_SCHEMA_VERSION,
};

/// Short name used by the internal ingest and storage layers.
pub use model::OperationalEventV2 as EventEnvelope;

// HANDWRITE-END
