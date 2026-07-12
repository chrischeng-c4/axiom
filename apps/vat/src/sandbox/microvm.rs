// SPEC-MANAGED: apps/vat/tech-design/logic/vat-microvm-phase-1-isolation-microvm-sandbox-backend-for-vat-ru.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// @spec apps/vat/tech-design/logic/vat-microvm-phase-1-isolation-microvm-sandbox-backend-for-vat-ru.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MicroVmSandboxBackendPhase1DataModelAdditions {
    /// New unit variant on the existing Isolation enum in spec.rs (currently None | Seatbelt, derives clap::ValueEnum). Additive only — no existing variant renamed or removed.
    #[serde(default)]
    pub isolation_variant: Option<serde_json::Value>,
    /// New optional field on the existing EnvSpec struct in spec.rs, alongside base/workdir/env/setup/isolation/egress/gpu/limits.
    #[serde(default)]
    pub env_spec_field: Option<serde_json::Value>,
    /// New apps/vat/src/sandbox/microvm.rs struct implementing the existing Sandbox trait (same trait seatbelt::SeatbeltBackend and process::ProcessBackend already implement).
    #[serde(default)]
    pub microvm_backend_struct: Option<serde_json::Value>,
}
// CODEGEN-END
