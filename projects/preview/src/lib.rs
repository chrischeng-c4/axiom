// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-src-lib-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
pub mod apply;
pub mod discover;
pub mod model;
pub mod render;
pub mod router;

pub use apply::{
    apply_manifest_paths, apply_rendered_manifests, apply_summary_markdown,
    manifest_inventory_for_env, manifest_inventory_from_dir, render_gitops_bundle, ApplyOptions,
    ApplySummary, GitopsBundleFile, ManifestInventory, ManifestInventoryEntry,
};
pub use discover::{
    discover_base_with_kubectl, normalize_base_workload, BaseContainerContract, BaseContainerPort,
    BaseEnvVar, BaseServicePort, BaseWorkloadContract,
};
pub use model::{
    BaseSpec, CleanupAction, CleanupPlan, PreviewEnvironment, PreviewPhase, PreviewSpec, RouteSpec,
};
pub use render::{render_files, RenderFile, RenderInput};
pub use router::{
    load_route_table_from_kubectl, load_route_table_from_rendered_dir, resolve_route,
    resolve_route_with_base, BaseRoute, ResolvedRoute, RouteBinding, RouteDecision, RouteOutcome,
    RouteRequest,
};

// </HANDWRITE>
