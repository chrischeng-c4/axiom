// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-src-lib-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
pub mod discover;
pub mod model;
pub mod render;
pub mod router;

pub use discover::{
    discover_base_with_kubectl, normalize_base_workload, BaseContainerContract, BaseContainerPort,
    BaseEnvVar, BaseServicePort, BaseWorkloadContract,
};
pub use model::{
    BaseSpec, CleanupAction, CleanupPlan, PreviewEnvironment, PreviewPhase, PreviewSpec, RouteSpec,
};
pub use render::{render_files, RenderFile, RenderInput};
pub use router::{resolve_route, ResolvedRoute, RouteBinding, RouteRequest};

// </HANDWRITE>
