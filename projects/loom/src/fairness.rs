//! Fair dispatch (#107) — weighted fair-share allocation of a bounded dispatch
//! budget across competing runs.
//!
//! When many runs have ready nodes at once, the scheduler must not let one big
//! run starve the others. `fair_allocate` splits a global per-tick budget across
//! runs by weight (weighted fair queueing), capped by what each run can actually
//! use (its ready count) and bounded by the budget (lazy bounded
//! materialization — we never dispatch more than the budget per tick). Leftover
//! from capped runs is redistributed to runs that still have demand.

use crate::model::WorkflowRunId;

/// One run's current dispatch demand.
#[derive(Debug, Clone)]
pub struct RunDemand {
    pub id: WorkflowRunId,
    /// Nodes ready to dispatch right now.
    pub ready: usize,
    /// Relative share weight (e.g. priority / quota); 0 is treated as 1.
    pub weight: u32,
}

/// Allocate `budget` dispatch slots across `runs` by weighted fair share, each
/// capped by its `ready` count. Deterministic: ties break by input order.
/// Returns `(run_id, granted)` for every run granted ≥ 1 slot.
pub fn fair_allocate(runs: &[RunDemand], budget: usize) -> Vec<(WorkflowRunId, usize)> {
    let mut granted: Vec<usize> = vec![0; runs.len()];
    let total_demand: usize = runs.iter().map(|r| r.ready).sum();
    let mut remaining = budget.min(total_demand);

    // Weighted fair fill: each slot goes to the run with the lowest
    // allocated/weight ratio that still has unmet demand. This is the discrete
    // form of weighted fair queueing and is starvation-free.
    while remaining > 0 {
        let mut best: Option<usize> = None;
        let mut best_ratio = f64::INFINITY;
        for (i, r) in runs.iter().enumerate() {
            if granted[i] >= r.ready {
                continue; // capped by ready count
            }
            let w = r.weight.max(1) as f64;
            let ratio = granted[i] as f64 / w;
            if ratio < best_ratio - f64::EPSILON {
                best_ratio = ratio;
                best = Some(i);
            }
        }
        match best {
            Some(i) => {
                granted[i] += 1;
                remaining -= 1;
            }
            None => break, // no run has unmet demand
        }
    }

    runs.iter()
        .enumerate()
        .filter(|(i, _)| granted[*i] > 0)
        .map(|(i, r)| (r.id.clone(), granted[i]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demand(id: &str, ready: usize, weight: u32) -> RunDemand {
        RunDemand { id: WorkflowRunId::new(id), ready, weight }
    }

    fn granted(v: &[(WorkflowRunId, usize)], id: &str) -> usize {
        v.iter().find(|(r, _)| r.0 == id).map(|(_, n)| *n).unwrap_or(0)
    }

    #[test]
    fn equal_weights_split_budget_evenly() {
        let runs = [demand("a", 10, 1), demand("b", 10, 1)];
        let alloc = fair_allocate(&runs, 6);
        assert_eq!(granted(&alloc, "a"), 3);
        assert_eq!(granted(&alloc, "b"), 3);
    }

    #[test]
    fn weight_biases_the_share() {
        // a has 3x the weight of b → roughly 3:1 of the budget.
        let runs = [demand("a", 10, 3), demand("b", 10, 1)];
        let alloc = fair_allocate(&runs, 8);
        assert_eq!(granted(&alloc, "a"), 6);
        assert_eq!(granted(&alloc, "b"), 2);
    }

    #[test]
    fn budget_caps_total_dispatched() {
        let runs = [demand("a", 100, 1), demand("b", 100, 1)];
        let alloc: usize = fair_allocate(&runs, 5).iter().map(|(_, n)| n).sum();
        assert_eq!(alloc, 5);
    }

    #[test]
    fn leftover_from_capped_run_redistributes() {
        // a can only use 1; the rest of the budget should flow to b.
        let runs = [demand("a", 1, 1), demand("b", 10, 1)];
        let alloc = fair_allocate(&runs, 6);
        assert_eq!(granted(&alloc, "a"), 1);
        assert_eq!(granted(&alloc, "b"), 5);
    }

    #[test]
    fn never_exceeds_demand() {
        let runs = [demand("a", 2, 1), demand("b", 3, 1)];
        let alloc = fair_allocate(&runs, 100);
        assert_eq!(granted(&alloc, "a"), 2);
        assert_eq!(granted(&alloc, "b"), 3);
    }
}
