// HANDWRITE-BEGIN gap="sift-event-module" tracker="1657" reason="Export the versioned event model and governance policy as one semantic event boundary."
mod governance;
mod model;

pub use governance::{GovernancePolicy, GovernancePolicySet};
pub use model::{
    decode_event_json, AttributeValue, EventEnvelopeV1, IncomingEvent, InstrumentationScope,
    MetricExemplar, MetricPoint, MetricTemporality, OperationalEventV2, SignalKind,
    EVENT_SCHEMA_URL, EVENT_SCHEMA_VERSION, EVENT_SCHEMA_VERSION_V1,
};

/// Compatibility name for callers of the bootstrap Sift API. New code should
/// prefer [`OperationalEventV2`] when the schema generation is relevant.
pub use model::OperationalEventV2 as EventEnvelope;

<!-- marker: sift-event-module path: projects/sift/src/event/mod.rs reason: Export the versioned event model and governance policy as one semantic event boundary. -->
// HANDWRITE-END
