// SPEC-MANAGED: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-admin-plane" tracker="#1290" reason="Admin plane needs generator primitives that do not exist yet.">
//! `GET /metrics` rendering (TD Logic section `metrics_req` node, Schema
//! section `AdminMetricsLine`): folds every `AdminState.pools` entry's
//! `ConnectionBudget::active()` and `BackendPool::stats()` into Prometheus
//! text exposition format 0.0.4 gauge lines (AC4), one triple per pool:
//! `pgpool_frontend_active`, `pgpool_backend_active`, `pgpool_backend_idle`,
//! each labeled `pool="<name>"`.

use crate::admin::state::{AdminState, NamedPool};
use metrics_prometheus::{render_labeled, Label, LabeledSample, SampleGroup};

/// Content-type header value the TD's e2e test asserts verbatim.
pub const CONTENT_TYPE: &str = "text/plain;version=0.0.4";

/// Renders the full Prometheus text-format body for every pool in `state`.
pub fn render(state: &AdminState) -> String {
    let frontend_active = pool_samples(state, |pool| pool.budget.active());
    let backend_active = pool_samples(state, |pool| pool.pool.stats().backend_active);
    let backend_idle = pool_samples(state, |pool| pool.pool.stats().backend_idle);

    render_labeled(&[
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
    ])
}

fn pool_samples<'a>(
    state: &'a AdminState,
    value_of: impl Fn(&NamedPool) -> usize,
) -> Vec<LabeledSample<'a>> {
    state
        .pools
        .iter()
        .map(|pool| {
            LabeledSample::new(
                vec![Label::new("pool", pool.name.as_str())],
                value_of(pool) as u64,
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
}
// </HANDWRITE>
