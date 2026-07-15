// SPEC-MANAGED: apps/pgpool/tech-design/semantic/pgpool-stateless-deployment-instance.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-k8s-instance" tracker="#1561" reason="Pgpool Kubernetes instance composition is currently hand-authored.">
//! Kubernetes control-plane models and deterministic manifest rendering.

mod budget;
mod control;
mod instance;
mod reserve;

pub use budget::{
    AllocationError, AllocationState, EndpointAllocator, EndpointCapacity, GlobalConnectionBudget,
    PodAllocation,
};
pub use control::{
    BackendPoolObservation, ControlPlaneError, ControlPlaneStatus, DrainProgress,
    EndpointControlStatus, PgpoolControlPlane, PodControlPhase, PodControlStatus,
};
pub use instance::{
    render_instance_yaml, render_manifests, spec_for_profile, InstanceProfile, PgpoolInstanceSpec,
};
pub use reserve::{
    ReserveLeaseError, ReserveLeaseGrant, ReserveLeaseKey, ReserveLeaseLedger, ReserveLeaseRequest,
    ReserveLeaseState,
};
// </HANDWRITE>
