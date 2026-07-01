//! beam — GPU-native vector database (library side of the `beam` crate).
//!
//! Beam owns vector-first storage, GPU ANN index lifecycle, batch ingest,
//! compaction/rebuild, and GPU vector-query execution. It is deliberately
//! distinct from **lumen**: Lumen owns mixed search / ranking / dedup; Beam is
//! the GPU-native vector service optimized for ANN indexes and GPU memory
//! tiers (see `README.md`, epic #769).
//!
//! This first slice is the CLI shell only — no HTTP, storage, or GPU runtime —
//! so the crate stays CPU/GPU-neutral (no CUDA/Metal/wgpu/vector/ANN deps). The
//! binary (`src/main.rs`) wires the standard `llm`/`upgrade`/`issue` verbs
//! through shared `cli-std` and exposes placeholder service verbs.

/// One-line statement of the Beam/Lumen boundary, surfaced in `beam llm`.
pub const LUMEN_BOUNDARY: &str =
    "Beam owns the GPU vector DB and index lifecycle; Lumen owns mixed search, ranking, and dedup.";

/// Render the tracked diagnostic every not-yet-built service verb prints, so the
/// message shape stays consistent across `serve`/`collections`/`index`/`query`/
/// `dockerfile`/`k8s` until each feature lands.
pub fn not_implemented(feature: &str) -> String {
    format!("not implemented yet: {feature}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_implemented_has_stable_prefix() {
        assert_eq!(
            not_implemented("vector query"),
            "not implemented yet: vector query"
        );
    }

    #[test]
    fn boundary_names_beam_and_lumen() {
        assert!(LUMEN_BOUNDARY.contains("Beam"));
        assert!(LUMEN_BOUNDARY.contains("Lumen"));
        assert!(LUMEN_BOUNDARY.contains("mixed search"));
    }
}
