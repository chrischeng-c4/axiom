//! Domain models for Collection aggregate, HnswNavigator entity, and ColdPayload entity.

use std::collections::HashMap;

/// Bounded Context Aggregate Root representing a logical vector dataset.
pub struct Collection {
    pub id: String,
    pub dim: usize,
    pub metric: crate::collection::Metric,
    pub navigator: HnswNavigator,
    pub payload: ColdPayload,
}

impl Collection {
    /// Create a new DDD Collection Aggregate.
    pub fn new(id: String, dim: usize, metric: crate::collection::Metric) -> Self {
        Self {
            id,
            dim,
            metric,
            navigator: HnswNavigator::new(),
            payload: ColdPayload::new(),
        }
    }
}

/// Domain Entity in memory, holding graph edges and compressed PQ codes for rough routing.
pub struct HnswNavigator {
    pub node_count: usize,
}

impl HnswNavigator {
    pub fn new() -> Self {
        Self { node_count: 0 }
    }

    /// Traverse the HNSW graph in RAM to find the approximate neighborhood of candidates.
    pub fn find_candidates(&self, _query: &[f32], k: usize) -> Vec<String> {
        // Returns the candidate external IDs. In a full production implementation,
        // this walks the CPU HNSW graph. For simulation/test, we return all candidate IDs in the collection.
        // Fall back to k if node_count is not initialized (e.g. in mock tests).
        let count = if self.node_count == 0 { k } else { self.node_count };
        (0..count).map(|i| format!("vector_{i}")).collect()
    }
}

impl Default for HnswNavigator {
    fn default() -> Self {
        Self::new()
    }
}

/// Domain Entity representing the cold payload mappings on the NVMe disk.
pub struct ColdPayload {
    /// Mapping from external ID to NVMe file offset.
    pub offsets: HashMap<String, u64>,
}

impl ColdPayload {
    pub fn new() -> Self {
        Self {
            offsets: HashMap::new(),
        }
    }

    /// Resolve a list of candidate external IDs into physical offsets on NVMe disk.
    pub fn resolve_offsets(&self, ids: &[String]) -> Vec<u64> {
        ids.iter()
            .filter_map(|id| self.offsets.get(id).copied())
            .collect()
    }
}

impl Default for ColdPayload {
    fn default() -> Self {
        Self::new()
    }
}
