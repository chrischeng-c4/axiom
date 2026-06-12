// SPEC-MANAGED: projects/arena/tech-design/semantic/source/projects-arena-src-measure-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Per-(cell, target) measurement. Dispatches on the target's transport and
//! drives it through rig's ONE open-loop scheduler ([`run_transport`]), so
//! every target — HTTP or Postgres — is measured by the same thin Rust client
//! on the same schedule, and the resulting numbers are comparable by
//! construction. The protocol/client knowledge lives entirely in rig's
//! transports; arena only picks which one and reads the folded [`LoadStats`].

use std::sync::Arc;

use rig::engine::loadgen::{run_transport, LoadStats, Schedule};
use rig::engine::transport::{HttpTransport, PostgresTransport, Transport};
use rig::scenario::interp::VarStore;

use crate::spec::{CellTarget, LoadShape, TargetSpec};

/// Measure one (target, cell): build the target's transport from the cell's
/// opaque payload (http `request` or postgres `query`) and drive it under the
/// shared schedule. `Err` only on a connect/template abort (per-op failures
/// surface as `error_rate`/`failed` inside the returned stats).
/// @spec projects/arena/tech-design/semantic/source/projects-arena-src-measure-rs.md#source
pub fn measure(
    target: &TargetSpec,
    cell: &CellTarget,
    load: &LoadShape,
) -> Result<LoadStats, String> {
    let schedule = Schedule {
        target_qps: load.target_qps,
        workers: load.workers,
        duration_secs: load.duration_secs,
        warmup_secs: load.warmup_secs,
    };
    let transport: Arc<dyn Transport> = match target.kind.as_str() {
        "service" | "http" => {
            let request = cell
                .request
                .clone()
                .ok_or_else(|| "http target cell is missing `request`".to_string())?;
            Arc::new(HttpTransport {
                request,
                vars: VarStore::new(),
            })
        }
        "postgres" => {
            let dsn = target
                .dsn
                .clone()
                .ok_or_else(|| "postgres target is missing `dsn`".to_string())?;
            let sql = cell
                .query
                .clone()
                .ok_or_else(|| "postgres target cell is missing `query`".to_string())?;
            Arc::new(PostgresTransport { dsn, sql })
        }
        other => {
            return Err(format!(
                "target kind `{other}` is not supported (use `http` or `postgres`)"
            ))
        }
    };
    let stats = run_transport(&schedule, &transport);
    if let Some(abort) = stats.abort {
        return Err(abort);
    }
    Ok(stats)
}

/// `true` when the offered load was actually achieved (so the latency
/// percentiles are trustworthy). Below the honesty ratio the target saturated
/// and its tail is a queueing artifact, not a real measurement.
/// @spec projects/arena/tech-design/semantic/source/projects-arena-src-measure-rs.md#source
pub fn load_is_honest(load: &LoadShape, stats: &LoadStats) -> bool {
    let target = load.target_qps.max(1) as f64;
    stats.achieved_qps >= target * rig::scenario::load::ACHIEVED_QPS_HONESTY_RATIO
}

/// `true` when every request failed — the target is effectively unreachable and
/// its scalar is meaningless.
/// @spec projects/arena/tech-design/semantic/source/projects-arena-src-measure-rs.md#source
pub fn fully_failed(stats: &LoadStats) -> bool {
    stats.total > 0 && stats.failed == stats.total
}
// CODEGEN-END
