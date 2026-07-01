//! beam — GPU-native vector database (library side of the `beam` crate).
//!
//! Beam owns vector-first storage, GPU ANN index lifecycle, batch ingest,
//! compaction/rebuild, and GPU vector-query execution. It is deliberately
//! distinct from **lumen**: Lumen owns mixed search / ranking / dedup; Beam is
//! the GPU-native vector service optimized for ANN indexes and GPU memory
//! tiers (see `README.md`, epic #769).
//!
//! The binary (`src/main.rs`) wires the standard `llm`/`upgrade`/`issue` verbs
//! through shared `cli-std`, the placeholder service verbs, and the real
//! `beam bench` verb.
//!
//! ## Vector-search engine
//!
//! The first real vector-search engine lives here:
//!
//! - [`collection`] — the in-memory row-major vector store + [`collection::Metric`].
//! - [`index`] — the [`index::VectorIndex`] contract + shared top-k, and the
//!   [`index::cpu_flat::CpuFlatIndex`] exact CPU oracle.
//! - [`gpu`] — [`gpu::GpuContext`] (wgpu / Metal) and the
//!   [`gpu::GpuFlatIndex`] GPU brute-force index whose results match the oracle.
//! - [`dataset`] — deterministic (fixed-seed LCG) synthetic corpora + queries.
//! - [`bench`] — the `beam bench` GPU-vs-CPU parity + timing demo.

pub mod bench;
pub mod collection;
pub mod dataset;
pub mod gpu;
pub mod index;

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
