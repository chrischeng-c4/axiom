//! Per-(cell, target) measurement. v1 = the service flavor only: assemble a
//! rig [`LoadProfile`] from the target's load shape + the cell's request, drive
//! it through [`rig::engine::loadgen`], and hand back the full [`LoadStats`] so
//! the engine can reduce to the comparable scalar AND judge load-honesty.

use rig::engine::loadgen::{self, LoadStats};
use rig::scenario::interp::VarStore;
use rig::scenario::load::LoadProfile;
use rig::scenario::step::HttpRequest;

use crate::spec::LoadShape;

/// Drive one service target with the cell's request and return the folded
/// load stats. `Err` only on a template/transport abort (per-request failures
/// surface as `error_rate`/`failed` inside the returned stats).
pub fn measure_service(load: &LoadShape, request: &HttpRequest) -> Result<LoadStats, String> {
    let profile = LoadProfile {
        target_qps: load.target_qps,
        workers: load.workers,
        duration_secs: load.duration_secs,
        warmup_secs: load.warmup_secs,
        request: request.clone(),
    };
    let stats = loadgen::run(&profile, &VarStore::new());
    if let Some(abort) = stats.abort {
        return Err(abort);
    }
    Ok(stats)
}

/// `true` when the offered load was actually achieved (so the latency
/// percentiles are trustworthy). Below the honesty ratio the target saturated
/// and its tail is a queueing artifact, not a real measurement.
pub fn load_is_honest(load: &LoadShape, stats: &LoadStats) -> bool {
    let target = load.target_qps.max(1) as f64;
    stats.achieved_qps >= target * rig::scenario::load::ACHIEVED_QPS_HONESTY_RATIO
}

/// `true` when every request failed — the target is effectively unreachable and
/// its scalar is meaningless.
pub fn fully_failed(stats: &LoadStats) -> bool {
    stats.total > 0 && stats.failed == stats.total
}
