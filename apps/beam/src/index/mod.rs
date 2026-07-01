//! Vector-index contract + the shared top-k selection both backends use.
//!
//! An index answers a single question — the `k` nearest rows to a query — via
//! [`VectorIndex::search_knn`]. Two backends implement it:
//!
//! - [`cpu_flat::CpuFlatIndex`] — exact CPU brute force, the correctness oracle.
//! - [`crate::gpu::GpuFlatIndex`] — the same exact scan, but the per-row
//!   distances are computed by a Metal (wgpu) compute kernel.
//!
//! Both compute the SAME per-row score under a shared convention (see
//! [`crate::collection::Metric::code`]) and hand it to [`topk`], so their result
//! row-sets agree.
//!
//! ## Ordering contract
//!
//! `search_knn` returns neighbors **best-first**, length `min(k, n)`:
//!
//! - [`Metric::L2`](crate::collection::Metric::L2): `score` is squared Euclidean
//!   distance — **smaller score first**.
//! - [`Metric::Dot`](crate::collection::Metric::Dot) /
//!   [`Metric::Cosine`](crate::collection::Metric::Cosine): `score` is a dot
//!   product — **larger score first**.

use std::cmp::Ordering;

use crate::collection::Metric;

pub mod cpu_flat;
pub mod ivf_pq;

/// One search result: the stored row index, its external id, and its raw score
/// under the collection's metric (squared-L2 distance, or dot product).
#[derive(Debug, Clone, PartialEq)]
pub struct Neighbor {
    /// Row index into the collection (`0..n`).
    pub row: u32,
    /// The external id stored for that row.
    pub external_id: String,
    /// Raw metric score: squared-L2 distance (L2) or dot product (Dot/Cosine).
    pub score: f32,
}

/// The nearest-neighbor query contract shared by every backend.
pub trait VectorIndex {
    /// Return the `k` nearest neighbors to `query`, best-first (see the module
    /// ordering contract). The result length is `min(k, n)`; an empty result is
    /// returned for `k == 0`, an empty collection, or a dimension mismatch.
    fn search_knn(&self, query: &[f32], k: usize) -> Vec<Neighbor>;
}

/// Select the top-`k` rows from `scores` (one score per row) under `metric`,
/// returning them best-first as [`Neighbor`]s. Shared by the CPU oracle and the
/// GPU backend so both order identically.
///
/// For [`Metric::L2`] smaller is better (ascending); for Dot/Cosine larger is
/// better (descending). Uses a partial selection (`select_nth_unstable`) so the
/// hot path stays `O(n)` for the common `k << n`.
pub fn topk(scores: &[f32], metric: Metric, k: usize, external_ids: &[String]) -> Vec<Neighbor> {
    let n = scores.len();
    let want = k.min(n);
    if want == 0 {
        return Vec::new();
    }

    // Order two rows best-first for `metric`. NaN scores sink to the bottom.
    let better = |a: usize, b: usize| -> Ordering {
        let (sa, sb) = (scores[a], scores[b]);
        let ord = sa.partial_cmp(&sb).unwrap_or(Ordering::Equal);
        if metric.larger_is_better() {
            ord.reverse()
        } else {
            ord
        }
    };

    let mut idx: Vec<usize> = (0..n).collect();
    if want < n {
        idx.select_nth_unstable_by(want - 1, |&a, &b| better(a, b));
        idx.truncate(want);
    }
    idx.sort_by(|&a, &b| better(a, b));

    idx.into_iter()
        .map(|i| Neighbor {
            row: i as u32,
            external_id: external_ids[i].clone(),
            score: scores[i],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topk_l2_smaller_first() {
        let scores = [5.0, 1.0, 3.0, 2.0];
        let ids: Vec<String> = (0..4).map(|i| format!("id-{i}")).collect();
        let out = topk(&scores, Metric::L2, 2, &ids);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].row, 1); // score 1.0
        assert_eq!(out[1].row, 3); // score 2.0
    }

    #[test]
    fn topk_dot_larger_first() {
        let scores = [5.0, 1.0, 3.0, 2.0];
        let ids: Vec<String> = (0..4).map(|i| format!("id-{i}")).collect();
        let out = topk(&scores, Metric::Dot, 2, &ids);
        assert_eq!(out[0].row, 0); // score 5.0
        assert_eq!(out[1].row, 2); // score 3.0
    }

    #[test]
    fn topk_clamps_to_n() {
        let scores = [1.0, 2.0];
        let ids: Vec<String> = vec!["a".into(), "b".into()];
        assert_eq!(topk(&scores, Metric::L2, 10, &ids).len(), 2);
        assert!(topk(&scores, Metric::L2, 0, &ids).is_empty());
    }
}
