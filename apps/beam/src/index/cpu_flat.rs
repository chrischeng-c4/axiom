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

impl VectorIndex for CpuFlatIndex<'_> {
    fn search_knn(&self, query: &[f32], k: usize) -> Vec<Neighbor> {
        let dim = self.collection.dim();
        let n = self.collection.len();
        let metric = self.collection.metric();
        if query.len() != dim || n == 0 || k == 0 {
            return Vec::new();
        }
        // Cosine normalizes the query so the stored (already-unit) rows give a
        // true cosine similarity via dot; L2/Dot use the query as-is.
        let q = match metric {
            Metric::Cosine => l2_normalize(query),
            _ => query.to_vec(),
        };
        let scores: Vec<f32> = (0..n)
            .map(|i| score(metric, &q, self.collection.row(i)))
            .collect();
        topk(&scores, metric, k, self.collection.external_ids())
    }

    fn num_vectors(&self) -> usize {
        self.collection.len()
    }

    fn row_payload(&self, row: u32) -> &Payload {
        self.collection.payload(row as usize)
    }

    /// Exact filtered oracle: score every row, then assign the metric's
    /// worst-case sentinel to rows whose payload fails `filter` and take the
    /// top `min(k, #matching)` — so only matching rows can be returned and the
    /// result length is exactly the number of matches when fewer than `k` match.
    fn search_knn_filtered(&self, query: &[f32], k: usize, filter: &Filter) -> Vec<Neighbor> {
        let dim = self.collection.dim();
        let n = self.collection.len();
        let metric = self.collection.metric();
        if query.len() != dim || n == 0 || k == 0 {
            return Vec::new();
        }
        let q = match metric {
            Metric::Cosine => l2_normalize(query),
            _ => query.to_vec(),
        };
        let sentinel = metric.worst_score();
        let mut nmatch = 0usize;
        let scores: Vec<f32> = (0..n)
            .map(|i| {
                if filter.matches(self.collection.payload(i)) {
                    nmatch += 1;
                    score(metric, &q, self.collection.row(i))
                } else {
                    sentinel
                }
            })
            .collect();
        // Cap the result to the number of matching rows: sentinels sort last, so
        // the top `min(k, nmatch)` are exactly the best matching rows.
        topk(&scores, metric, k.min(nmatch), self.collection.external_ids())
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
