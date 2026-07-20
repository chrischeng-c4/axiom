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
//! - [`collection`] — the in-memory row-major vector store + [`collection::Metric`],
//!   with external-id addressing and delete / update / upsert (tombstones + an
//!   append-only vector store; the live-mask folds into every index's keep-bitmask).
//! - [`payload`] — row attribute payloads ([`payload::Payload`]) and the
//!   composable [`payload::Filter`] for filtered k-NN (metadata + filtered
//!   search, the vector-DB table stakes).
//! - [`index`] — the [`index::VectorIndex`] contract + shared top-k, the
//!   [`index::cpu_flat::CpuFlatIndex`] exact CPU oracle, and the
//!   [`index::ivf_pq::IvfPqIndex`] IVF-PQ (IVFADC) approximate index.
//! - [`gpu`] — [`gpu::GpuContext`] (wgpu / Metal), the
//!   [`gpu::GpuFlatIndex`] GPU brute-force index, and the
//!   [`gpu::ivfpq::GpuIvfScanner`] GPU IVF-PQ candidate scan — all matched to the
//!   oracle.
//! - [`dataset`] — deterministic (fixed-seed LCG) synthetic corpora + queries.
//! - [`persist`] — durable save/load: the collection segment + trained IVF-PQ
//!   model persist to disk (GPU buffers are rebuilt on load, never persisted), so
//!   a cold start reproduces identical search results without retraining.
//! - [`bench`] — the `beam bench` GPU-vs-CPU parity + timing demo.
//! - [`service`] — the `beam serve` HTTP/2 (h2c) service layer: the in-process
//!   collection registry + REST routes that turn the engine into a callable
//!   vector database.

pub mod bench;
pub mod collection;
pub mod dataset;
pub mod gpu;
pub mod index;
pub mod payload;
pub mod persist;
pub mod service;
pub mod spec;
pub mod dx;
pub mod backup;
pub mod domain;
pub mod infrastructure;
pub mod application;
pub mod operator;

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
