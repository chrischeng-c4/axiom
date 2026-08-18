// HANDWRITE-BEGIN gap="missing-generator:logic:7c31a9e4" tracker="#2620" reason="Own the controller's own metric set, its Prometheus exposition, and the scrape listener; the generator has no primitive for a self-observing async runtime surface."
//! The operator's view of itself.
//!
//! Every service in the kit runs the same controller, so until now every
//! service was blind in the same place: [`crate::controller::run`] started a
//! reconcile loop and nothing else, exposing no port, counting no reconciles,
//! and losing every error into a bare requeue. A control plane that cannot say
//! whether it is converging is indistinguishable, from outside the process,
//! from one that is idle — which is the exact failure this module exists to
//! make impossible (#2620).
//!
//! The metric set is deliberately small and derived from one question: *is this
//! operator doing its job?* That needs a rate of work (`_reconcile_total`), a
//! rate of failure (`_reconcile_errors_total`), a latency distribution
//! (`_reconcile_duration_seconds`), and which replica is actually allowed to
//! act (`_leader`). Anything beyond that is a service's own business and
//! belongs on the instance endpoint, not here.
//!
//! Names are prefixed from the service's `MANAGER` (`lumen-operator` →
//! `lumen_operator_`), so the six services sharing this controller land in one
//! Prometheus without colliding, and a query written against one reads the same
//! for all of them.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use metrics_prometheus::{Bucket, Counter, Histogram, Sample};

/// Reconcile duration buckets, in milliseconds — the unit
/// [`ControllerMetrics::observe`] records — published as seconds.
///
/// The spread is chosen for what a reconcile actually is: a handful of
/// server-side applies and GETs against the apiserver. Sub-10ms means the
/// leader gate short-circuited or nothing changed; the interesting mass sits
/// between 25ms and 1s; anything past 10s is a reconcile that is effectively
/// stuck behind a slow or throttled apiserver, and the `+Inf` row is enough to
/// alert on that without a wider tail.
const RECONCILE_BUCKETS_MS: &[Bucket<'static>] = &[
    Bucket::new("0.005", 5),
    Bucket::new("0.01", 10),
    Bucket::new("0.025", 25),
    Bucket::new("0.05", 50),
    Bucket::new("0.1", 100),
    Bucket::new("0.25", 250),
    Bucket::new("0.5", 500),
    Bucket::new("1", 1_000),
    Bucket::new("2.5", 2_500),
    Bucket::new("5", 5_000),
    Bucket::new("10", 10_000),
];

/// Environment variable naming the address the scrape listener binds.
pub const METRICS_ADDR_ENV: &str = "OPERATOR_METRICS_ADDR";

/// Default scrape address: all interfaces, port 9090.
pub const DEFAULT_METRICS_ADDR: &str = "0.0.0.0:9090";

/// The controller's own metrics, shared by every reconcile and read at scrape
/// time.
///
/// Leadership is *not* a field here. It lives in
/// [`crate::lease::Election::is_leader`] and is read when the exposition is
/// rendered, because a cached copy would be wrong in precisely the case that
/// matters: an operator with zero CRs runs no reconciles, so a gauge only
/// written from the reconcile path would report stale leadership forever, and
/// a lease handover on an idle cluster would be invisible.
#[derive(Debug)]
pub struct ControllerMetrics {
    prefix: String,
    reconcile_total: Counter,
    reconcile_errors_total: Counter,
    reconcile_duration: Histogram,
}

impl ControllerMetrics {
    /// Build the metric set for a service, deriving the Prometheus name prefix
    /// from its field-manager name (`lumen-operator` → `lumen_operator`).
    pub fn new(manager: &str) -> Self {
        Self {
            prefix: manager.replace('-', "_"),
            reconcile_total: Counter::new(),
            reconcile_errors_total: Counter::new(),
            reconcile_duration: Histogram::new(RECONCILE_BUCKETS_MS),
        }
    }

    /// Record one completed reconcile attempt and how long it took.
    ///
    /// Called for failures too: the denominator of an error *rate* has to be
    /// every attempt, or a controller that fails everything reports an error
    /// ratio of zero over zero.
    pub fn observe(&self, elapsed: Duration) {
        self.reconcile_total.incr();
        self.reconcile_duration.observe(elapsed.as_millis() as u64);
    }

    /// Record that a reconcile returned an error.
    pub fn observe_error(&self) {
        self.reconcile_errors_total.incr();
    }

    pub fn reconcile_total(&self) -> u64 {
        self.reconcile_total.get()
    }

    pub fn reconcile_errors_total(&self) -> u64 {
        self.reconcile_errors_total.get()
    }

    /// Render the Prometheus text exposition. `leader` is passed in rather than
    /// stored — see the note on [`ControllerMetrics`].
    pub fn render(&self, leader: bool) -> String {
        let p = &self.prefix;
        let mut out = metrics_prometheus::render(&[
            Sample::new(
                &format!("{p}_reconcile_total"),
                "counter",
                "Reconcile attempts by this operator replica, successful or not.",
                self.reconcile_total.get(),
            ),
            Sample::new(
                &format!("{p}_reconcile_errors_total"),
                "counter",
                "Reconcile attempts that returned an error.",
                self.reconcile_errors_total.get(),
            ),
            Sample::new(
                &format!("{p}_leader"),
                "gauge",
                "1 on the replica currently holding the leader-election Lease, 0 otherwise.",
                u64::from(leader),
            ),
        ]);
        out.push_str(&self.reconcile_duration.render(
            &format!("{p}_reconcile_duration_seconds"),
            "Wall-clock duration of one reconcile.",
            1_000,
        ));
        out
    }
}

/// The address the scrape listener should bind, from [`METRICS_ADDR_ENV`] or
/// [`DEFAULT_METRICS_ADDR`].
///
/// An unparseable override falls back to the default *loudly*: silently not
/// listening because someone typo'd a port is the failure mode this whole
/// module exists to remove.
pub fn metrics_addr() -> SocketAddr {
    let raw = std::env::var(METRICS_ADDR_ENV).unwrap_or_else(|_| DEFAULT_METRICS_ADDR.to_string());
    raw.parse().unwrap_or_else(|error| {
        tracing::error!(
            %error, address = %raw, env = METRICS_ADDR_ENV,
            "unparseable metrics bind address; falling back to {DEFAULT_METRICS_ADDR}"
        );
        DEFAULT_METRICS_ADDR.parse().expect("default addr parses")
    })
}

/// Serve `GET /metrics` on `addr` until the process ends.
///
/// HTTP/1.1 on purpose: this is a Prometheus scrape target, and Prometheus
/// scrapes over HTTP/1.1. The kit's `service-http::serve_h2c` is the wrong
/// transport here even though it is the house default elsewhere.
pub async fn serve<F>(addr: SocketAddr, metrics: Arc<ControllerMetrics>, leader: F)
where
    F: Fn() -> bool + Clone + Send + Sync + 'static,
{
    let app = axum::Router::new().route(
        "/metrics",
        axum::routing::get(move || {
            let metrics = metrics.clone();
            let leader = leader.clone();
            async move {
                (
                    [("content-type", "text/plain; version=0.0.4")],
                    metrics.render(leader()),
                )
            }
        }),
    );

    // A failed bind must not kill the operator. Losing metrics is bad; losing
    // reconciliation because port 9090 was taken is worse, and the loss is not
    // silent either way — the absence alert is written against the scrape
    // target's `up` series, so a listener that never came up fires the same
    // page as a process that died.
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(%error, %addr, "metrics listener failed to bind; operator continues without a scrape endpoint");
            return;
        }
    };
    tracing::info!(%addr, "serving controller metrics on /metrics");
    if let Err(error) = axum::serve(listener, app).await {
        tracing::error!(%error, "metrics listener stopped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(rendered: &str, line_prefix: &str) -> Option<String> {
        rendered
            .lines()
            .find(|l| l.starts_with(line_prefix))
            .map(str::to_string)
    }

    /// The prefix is what keeps six operators' series apart in one Prometheus,
    /// and `-` is not a legal character in a metric name — a naive
    /// `format!("{manager}_...")` would emit `lumen-operator_reconcile_total`,
    /// which Prometheus rejects at scrape time rather than at build time.
    #[test]
    fn the_manager_name_becomes_a_legal_metric_prefix() {
        let rendered = ControllerMetrics::new("lumen-operator").render(false);
        assert!(
            rendered.contains("lumen_operator_reconcile_total"),
            "{rendered}"
        );
        // Only the metric names are checked — HELP text is free prose and may
        // legitimately contain hyphens ("leader-election").
        for line in rendered.lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
            let name = line.split(['{', ' ']).next().unwrap_or_default();
            assert!(
                !name.contains('-'),
                "`{name}` is not a legal Prometheus metric name"
            );
        }
    }

    /// Pins the full family set and each `# TYPE`. An alert is written against
    /// an exact series name, so renaming one here silently breaks a rule that
    /// lives in a YAML file no compiler reads.
    #[test]
    fn every_declared_family_is_present_with_its_type() {
        let rendered = ControllerMetrics::new("tape-operator").render(true);
        for (name, kind) in [
            ("tape_operator_reconcile_total", "counter"),
            ("tape_operator_reconcile_errors_total", "counter"),
            ("tape_operator_leader", "gauge"),
            ("tape_operator_reconcile_duration_seconds", "histogram"),
        ] {
            assert!(
                rendered.contains(&format!("# TYPE {name} {kind}\n")),
                "missing `# TYPE {name} {kind}` in:\n{rendered}"
            );
        }
    }

    /// The leader gauge is the one series a human reads to answer "which
    /// replica is actually in charge", so both values have to be exact — a
    /// gauge that is only ever 1 is worse than no gauge.
    #[test]
    fn the_leader_gauge_reports_both_states() {
        let metrics = ControllerMetrics::new("lumen-operator");
        assert_eq!(
            body(&metrics.render(true), "lumen_operator_leader "),
            Some("lumen_operator_leader 1".to_string())
        );
        assert_eq!(
            body(&metrics.render(false), "lumen_operator_leader "),
            Some("lumen_operator_leader 0".to_string())
        );
    }

    /// An error rate is `errors / total`. If a failing reconcile were counted
    /// only as an error, the denominator would exclude exactly the events the
    /// numerator counts, and an operator failing 100% of its reconciles would
    /// report a ratio of `n/0`.
    #[test]
    fn a_failed_reconcile_counts_in_both_the_numerator_and_the_denominator() {
        let metrics = ControllerMetrics::new("lumen-operator");
        metrics.observe(Duration::from_millis(30));
        metrics.observe(Duration::from_millis(40));
        metrics.observe_error();

        assert_eq!(metrics.reconcile_total(), 2);
        assert_eq!(metrics.reconcile_errors_total(), 1);
        let rendered = metrics.render(true);
        assert!(rendered.contains("lumen_operator_reconcile_total 2"), "{rendered}");
        assert!(
            rendered.contains("lumen_operator_reconcile_errors_total 1"),
            "{rendered}"
        );
    }

    /// Durations are observed in milliseconds and published as seconds. Getting
    /// the divisor wrong is invisible in a unit test that only asserts the
    /// series exists, and shows up in production as a latency SLO off by 1000x.
    #[test]
    fn durations_are_recorded_in_milliseconds_and_published_in_seconds() {
        let metrics = ControllerMetrics::new("lumen-operator");
        metrics.observe(Duration::from_millis(30));
        metrics.observe(Duration::from_millis(1_500));
        let rendered = metrics.render(true);

        assert_eq!(
            body(&rendered, "lumen_operator_reconcile_duration_seconds_sum"),
            Some("lumen_operator_reconcile_duration_seconds_sum 1.530".to_string()),
            "{rendered}"
        );
        // 30ms lands in the 0.05 bucket, 1.5s in the 2.5 bucket — cumulative,
        // so the 1s row still shows only the first observation.
        assert!(
            rendered.contains("lumen_operator_reconcile_duration_seconds_bucket{le=\"1\"} 1"),
            "{rendered}"
        );
        assert!(
            rendered.contains("lumen_operator_reconcile_duration_seconds_bucket{le=\"2.5\"} 2"),
            "{rendered}"
        );
        assert!(
            rendered.contains("lumen_operator_reconcile_duration_seconds_count 2"),
            "{rendered}"
        );
    }

    /// A fresh operator with no CRs must still publish every series at zero.
    /// Absent series and zero-valued series are not the same thing to
    /// Prometheus: `rate()` over a series that has never appeared returns
    /// nothing, so an alert on it can never fire.
    #[test]
    fn an_idle_operator_still_publishes_every_series() {
        let rendered = ControllerMetrics::new("defer-operator").render(false);
        for line in [
            "defer_operator_reconcile_total 0",
            "defer_operator_reconcile_errors_total 0",
            "defer_operator_leader 0",
            "defer_operator_reconcile_duration_seconds_count 0",
            "defer_operator_reconcile_duration_seconds_sum 0.000",
        ] {
            assert!(rendered.contains(line), "missing `{line}` in:\n{rendered}");
        }
    }

    #[test]
    fn an_unparseable_address_override_falls_back_to_the_default() {
        // Serial by construction: the env var is process-global, so this test
        // sets and clears it around a single assertion rather than relying on
        // test ordering.
        std::env::set_var(METRICS_ADDR_ENV, "not-an-address");
        let addr = metrics_addr();
        std::env::remove_var(METRICS_ADDR_ENV);
        assert_eq!(addr.to_string(), DEFAULT_METRICS_ADDR);
    }
}
// HANDWRITE-END
