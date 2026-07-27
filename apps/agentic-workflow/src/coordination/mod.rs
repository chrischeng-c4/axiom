//! Client-independent coordination protocol shared by AW and native clients.

pub mod authority;
pub mod protocol;

pub use authority::{CoordinationDecision, CoordinationState, ReconciliationOutcome};
pub use protocol::{
    CoordinationAuthority, CoordinationVersion, DispatchDocument, DispatchStatus, GateDocument,
    GateStatus, GateType, MessageDocument, MessageType, TaskDocument,
    AW_COORDINATION_SCHEMA_VERSION,
};
