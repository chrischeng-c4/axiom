//! Exact CPU brute-force index — THE correctness oracle.
//!
//! [`CpuFlatIndex`] borrows a [`Collection`] and, per query, scores every stored
//! row with the same per-metric convention the GPU kernel uses, then runs the
//! shared [`topk`](super::topk) selection. No graph, no approximation: it is
//! ground truth the GPU backend is checked against.

use crate::collection::{l2_normalize, Collection, Metric};
use crate::index::{topk, Neighbor, VectorIndex};
use crate::payload::{Filter, Payload};

/// Exact brute-force nearest-neighbor index over a borrowed [`Collection`].
pub struct CpuFlatIndex<'a> {
    collection: &'a Collection,
}

impl<'a> CpuFlatIndex<'a> {
    /// Borrow `collection` as the corpus to scan. No build cost.
    pub fn new(collection: &'a Collection) -> Self {
        Self { collection }
    }
}

/// Per-row score under `metric`, matching `gpu/flat.wgsl` exactly:
/// - L2 → sum of squared differences (smaller = better).
/// - Dot / Cosine → dot product (larger = better); Cosine rows/queries are
///   already unit-normalized so this is cosine similarity.
fn score(metric: Metric, query: &[f32], row: &[f32]) -> f32 {
    match metric {
        Metric::L2 => query
            .iter()
            .zip(row)
            .map(|(q, r)| {
                let d = q - r;
                d * d
            })
            .sum(),
        Metric::Dot | Metric::Cosine => query.iter().zip(row).map(|(q, r)| q * r).sum(),
    }
}

impl CpuFlatIndex<'_> {
    /// The shared exact scan: score every physical row that survives the keep-set
    /// (`live AND filter`), assigning the metric's worst-case sentinel to dropped
    /// rows, and take the top `min(k, #kept)`. `filter == None` is the unfiltered
    /// query (keep = live), so a tombstoned row is excluded through the SAME
    /// sentinel path a filtered-out row is — the collection's live bit is just one
    /// more clause AND-ed into the keep-set. Iterates all `capacity` physical rows
    /// (tombstoned rows still occupy slots) and masks, rather than iterating the
    /// live count.
    fn masked_search(&self, query: &[f32], k: usize, filter: Option<&Filter>) -> Vec<Neighbor> {
        let dim = self.collection.dim();
        let cap = self.collection.capacity();
        let metric = self.collection.metric();
        if query.len() != dim || cap == 0 || k == 0 {
            return Vec::new();
        }
        // Cosine normalizes the query so the stored (already-unit) rows give a
        // true cosine similarity via dot; L2/Dot use the query as-is.
        let q = match metric {
            Metric::Cosine => l2_normalize(query),
            _ => query.to_vec(),
        };
        let live = self.collection.live();
        let sentinel = metric.worst_score();
        let mut nkeep = 0usize;
        let scores: Vec<f32> = (0..cap)
            .map(|i| {
                let keep = live[i] && filter.is_none_or(|f| f.matches(self.collection.payload(i)));
                if keep {
                    nkeep += 1;
                    score(metric, &q, self.collection.row(i))
                } else {
                    sentinel
                }
            })
            .collect();
        // Cap the result to the number of kept rows: sentinels sort last, so the
        // top `min(k, nkeep)` are exactly the best live-and-matching rows.
        topk(
            &scores,
            metric,
            k.min(nkeep),
            self.collection.external_ids(),
        )
    }
}

impl VectorIndex for CpuFlatIndex<'_> {
    fn search_knn(&self, query: &[f32], k: usize) -> Vec<Neighbor> {
        self.masked_search(query, k, None)
    }

    fn num_vectors(&self) -> usize {
        // Physical row count: rows are addressed 0..capacity and `row_payload`
        // indexes physical rows, so this is the correct over-fetch ceiling.
        self.collection.capacity()
    }

    fn row_payload(&self, row: u32) -> &Payload {
        self.collection.payload(row as usize)
    }

    /// Exact filtered oracle: the top `min(k, #matching-and-live)` rows whose
    /// payload matches `filter` AND are live. Tombstoned rows are excluded exactly
    /// like filtered-out rows (both get the sentinel), so this stays the ground
    /// truth for the GPU filtered + deleted paths.
    fn search_knn_filtered(&self, query: &[f32], k: usize, filter: &Filter) -> Vec<Neighbor> {
        self.masked_search(query, k, Some(filter))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l2_finds_exact_nearest() {
        let mut c = Collection::new("t", 2, Metric::L2);
        c.add("a", &[0.0, 0.0]).unwrap();
        c.add("b", &[1.0, 0.0]).unwrap();
        c.add("c", &[5.0, 5.0]).unwrap();
        let idx = CpuFlatIndex::new(&c);
        let out = idx.search_knn(&[0.9, 0.0], 2);
        assert_eq!(out[0].external_id, "b");
        assert_eq!(out[1].external_id, "a");
    }

    #[test]
    fn dot_orders_by_largest() {
        let mut c = Collection::new("t", 2, Metric::Dot);
        c.add("a", &[1.0, 0.0]).unwrap();
        c.add("b", &[10.0, 0.0]).unwrap();
        let idx = CpuFlatIndex::new(&c);
        let out = idx.search_knn(&[1.0, 0.0], 2);
        assert_eq!(out[0].external_id, "b");
        assert_eq!(out[1].external_id, "a");
    }
}
