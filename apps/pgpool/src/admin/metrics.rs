// SPEC-MANAGED: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-admin-plane" tracker="#1290" reason="Admin plane needs generator primitives that do not exist yet.">
//! `GET /metrics` rendering (TD Logic section `metrics_req` node, Schema
//! section `AdminMetricsLine`): folds every `AdminState.pools` entry's
//! `ConnectionBudget::active()` and `BackendPool::stats()` into Prometheus
//! text exposition format 0.0.4 gauge lines (AC4), one triple per pool:
//! `pgpool_frontend_active`, `pgpool_backend_active`, `pgpool_backend_idle`,
//! each labeled `pool="<name>"`.

use std::fmt::Write;

use crate::admin::state::{AdminState, NamedPool};
use crate::pool::{RESERVE_GRANTED_METRIC, RESERVE_QUEUED_METRIC, RESERVE_SPENT_METRIC};
use metrics_prometheus::{render_labeled, Label, LabeledSample, SampleGroup};

/// Content-type header value the TD's e2e test asserts verbatim.
pub const CONTENT_TYPE: &str = "text/plain;version=0.0.4";

// <HANDWRITE gap="missing-generator:logic" tracker="#1892" reason="Capture one BackendPool stats snapshot per pool per Prometheus render.">
/// Renders the full Prometheus text-format body for every pool in `state`.
pub fn render(state: &AdminState) -> String {
    let snapshots = pool_metric_snapshots(state);
    let frontend_active = pool_samples(&snapshots, |snapshot| snapshot.frontend_active);
    let backend_active = pool_samples(&snapshots, |snapshot| snapshot.stats.backend_active);
    let backend_idle = pool_samples(&snapshots, |snapshot| snapshot.stats.backend_idle);
    let reserve_queued = pool_samples(&snapshots, |snapshot| snapshot.stats.reserve_queued);
    let reserve_granted = pool_samples(&snapshots, |snapshot| snapshot.stats.reserve_granted);
    let reserve_spent = pool_samples(&snapshots, |snapshot| snapshot.stats.reserve_spent);

    let mut out = render_labeled(&[
        SampleGroup::new(
            "pgpool_frontend_active",
            "gauge",
            "Frontend connections currently admitted by the connection budget.",
            &frontend_active,
        ),
        SampleGroup::new(
            "pgpool_backend_active",
            "gauge",
            "Backend connections currently leased out.",
            &backend_active,
        ),
        SampleGroup::new(
            "pgpool_backend_idle",
            "gauge",
            "Backend connections currently sitting idle in the pool.",
            &backend_idle,
        ),
        SampleGroup::new(
            RESERVE_QUEUED_METRIC,
            "gauge",
            "Reserve backend units waiting for asynchronous allocator admission.",
            &reserve_queued,
        ),
        SampleGroup::new(
            RESERVE_GRANTED_METRIC,
            "gauge",
            "Reserve backend units granted in the local lease cache.",
            &reserve_granted,
        ),
        SampleGroup::new(
            RESERVE_SPENT_METRIC,
            "gauge",
            "Reserve backend units currently spent by a physical backend lifecycle.",
            &reserve_spent,
        ),
    ]);
    out.push_str(&render_transaction_phase_metrics(state));
    out
}
// </HANDWRITE>

fn render_transaction_phase_metrics(state: &AdminState) -> String {
    let mut out = String::new();
    let mut wrote_help = false;
    for pool in state.pools.iter() {
        let Some(snapshot) = pool.pool.transaction_phase_telemetry() else {
            continue;
        };
        if !wrote_help {
            let _ = writeln!(
                out,
                "# HELP pgpool_transaction_phase_seconds Aggregate duration of explicitly enabled transaction-pool phases."
            );
            let _ = writeln!(out, "# TYPE pgpool_transaction_phase_seconds summary");
            wrote_help = true;
        }
        for metric in snapshot.metrics {
            let labels = format!(
                "pool=\"{}\",phase=\"{}\",outcome=\"{}\"",
                pool.name, metric.phase, metric.outcome
            );
            let _ = writeln!(
                out,
                "pgpool_transaction_phase_seconds_count{{{labels}}} {}",
                metric.count
            );
            let _ = writeln!(
                out,
                "pgpool_transaction_phase_seconds_sum{{{labels}}} {:.9}",
                metric.total_seconds
            );
        }
    }
    out
}

struct PoolMetricSnapshot<'a> {
    pool: &'a NamedPool,
    frontend_active: usize,
    stats: crate::pool::BackendPoolStats,
}

fn pool_metric_snapshots(state: &AdminState) -> Vec<PoolMetricSnapshot<'_>> {
    state
        .pools
        .iter()
        .map(|pool| PoolMetricSnapshot {
            pool,
            frontend_active: pool.budget.active(),
            stats: pool.pool.stats(),
        })
        .collect()
}

fn pool_samples<'a>(
    snapshots: &'a [PoolMetricSnapshot<'a>],
    value_of: impl Fn(&PoolMetricSnapshot<'_>) -> usize,
) -> Vec<LabeledSample<'a>> {
    snapshots
        .iter()
        .map(|snapshot| {
            LabeledSample::new(
                vec![Label::new("pool", snapshot.pool.name.as_str())],
                value_of(snapshot) as u64,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::{BackendPool, PoolConfig};
    use crate::proxy::BackendEndpointConfig;
    use crate::wire::WireCodecConfig;
    use crate::PoolMode;
    use server_lifecycle::ConnectionBudget;
    use std::time::Duration;

    fn one_pool_state(name: &str) -> AdminState {
        let pool = BackendPool::new(PoolConfig {
            endpoint: BackendEndpointConfig {
                host: "127.0.0.1".to_string(),
                port: 5432,
            },
            max_backend_connections: 4,
            acquire_timeout: Duration::from_millis(50),
            backend_connect_timeout: Duration::from_millis(50),
            wire: WireCodecConfig::default(),
        });
        AdminState::new(
            server_lifecycle::DrainController::new(),
            vec![NamedPool {
                name: name.to_string(),
                mode: PoolMode::Transaction,
                budget: ConnectionBudget::new(10),
                pool,
            }],
        )
    }

    /// verify: admin::metrics_renders_prometheus_text_format_gauges_per_pool (R4)
    #[test]
    fn renders_all_three_gauges_labeled_per_pool() {
        let state = one_pool_state("default");
        let body = render(&state);
        assert!(body.contains("pgpool_frontend_active{pool=\"default\"} 0"));
        assert!(body.contains("pgpool_backend_active{pool=\"default\"} 0"));
        assert!(body.contains("pgpool_backend_idle{pool=\"default\"} 0"));
        assert!(body.contains("# TYPE pgpool_frontend_active gauge"));
    }

    /// verify: admin::metrics_gauge_values_match_pool_stats_at_request_time (R4)
    #[test]
    fn gauge_values_track_live_budget_state() {
        let state = one_pool_state("default");
        let permit = state.pools[0]
            .budget
            .try_acquire()
            .expect("budget has room");
        let body = render(&state);
        assert!(body.contains("pgpool_frontend_active{pool=\"default\"} 1"));
        drop(permit);
        let body = render(&state);
        assert!(body.contains("pgpool_frontend_active{pool=\"default\"} 0"));
    }

    #[test]
    fn pool_label_uses_shared_prometheus_escaping() {
        let state = one_pool_state("west\"\\edge\nblue");
        let body = render(&state);
        assert!(body.contains("pgpool_frontend_active{pool=\"west\\\"\\\\edge\\nblue\"} 0"));
    }

    #[test]
    fn metrics_render_uses_one_snapshot_per_pool() {
        let state = one_pool_state("default");
        let snapshots = pool_metric_snapshots(&state);
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].stats, state.pools[0].pool.stats());
        assert!(render(&state).contains("pgpool_reserve_spent{pool=\"default\"} 0"));
    }

    #[test]
    fn phase_metrics_are_absent_until_explicitly_enabled_and_stay_aggregate() {
        let disabled = one_pool_state("default");
        assert!(!render(&disabled).contains("pgpool_transaction_phase_seconds"));

        let pool = BackendPool::new_with_transaction_phase_telemetry(PoolConfig {
            endpoint: BackendEndpointConfig {
                host: "127.0.0.1".to_string(),
                port: 5432,
            },
            max_backend_connections: 4,
            acquire_timeout: Duration::from_millis(50),
            backend_connect_timeout: Duration::from_millis(50),
            wire: WireCodecConfig::default(),
        });
        pool.record_transaction_phase(
            crate::pool::telemetry::TransactionPhase::Acquire,
            crate::pool::telemetry::TransactionPhaseOutcome::Success,
            Duration::from_nanos(7),
        );
        let enabled = AdminState::new(
            server_lifecycle::DrainController::new(),
            vec![NamedPool {
                name: "default".to_string(),
                mode: PoolMode::Transaction,
                budget: ConnectionBudget::new(10),
                pool,
            }],
        );
        let body = render(&enabled);
        assert!(body.contains("phase=\"acquire\",outcome=\"success\"} 1"));
        assert!(body.contains("phase=\"relay\",outcome=\"failure\"} 0"));
        assert!(!body.contains("client="));
        assert!(!body.contains("query="));
    }
}
// </HANDWRITE>
