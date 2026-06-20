//! HTTP request observability: per-route request counts + latency histograms,
//! rendered into the Prometheus exposition served at `/metrics`.
//!
//! This lives entirely at the HTTP layer (a `route_layer` middleware reads the
//! matched route pattern). It never touches the engine write hot path, so it
//! can't regress the engine perf-gate. The shared mutex is hit once per HTTP
//! request — negligible against the per-request HTTP/2 + JSON cost.

use std::collections::HashMap;
use std::time::Instant;

use axum::extract::{MatchedPath, State};
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use parking_lot::Mutex;

/// Latency histogram bucket upper bounds, in seconds (Prometheus `le`).
const BUCKETS: [f64; 14] = [
    0.0005, 0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

#[derive(Default)]
struct RouteStat {
    /// Count per HTTP status code.
    status: HashMap<u16, u64>,
    /// Non-cumulative per-bucket counts; index BUCKETS.len() is the +Inf overflow.
    bucket: [u64; BUCKETS.len() + 1],
    sum_secs: f64,
    count: u64,
}

/// Per-(method, route-pattern) request metrics.
#[derive(Default)]
pub struct HttpMetrics {
    inner: Mutex<HashMap<(String, String), RouteStat>>,
}

impl HttpMetrics {
    fn record(&self, method: &str, route: &str, status: u16, secs: f64) {
        let mut g = self.inner.lock();
        let stat = g.entry((method.to_string(), route.to_string())).or_default();
        *stat.status.entry(status).or_insert(0) += 1;
        let idx = BUCKETS
            .iter()
            .position(|&b| secs <= b)
            .unwrap_or(BUCKETS.len());
        stat.bucket[idx] += 1;
        stat.sum_secs += secs;
        stat.count += 1;
    }

    /// Render the Prometheus exposition for the recorded request metrics.
    pub fn render(&self) -> String {
        let g = self.inner.lock();
        let mut out = String::new();
        out.push_str("# HELP keep_http_requests_total HTTP requests by route and status.\n");
        out.push_str("# TYPE keep_http_requests_total counter\n");
        for ((method, route), stat) in g.iter() {
            for (status, n) in &stat.status {
                out.push_str(&format!(
                    "keep_http_requests_total{{method=\"{}\",route=\"{}\",status=\"{}\"}} {}\n",
                    method,
                    esc(route),
                    status,
                    n
                ));
            }
        }
        out.push_str(
            "# HELP keep_http_request_duration_seconds HTTP request latency by route.\n",
        );
        out.push_str("# TYPE keep_http_request_duration_seconds histogram\n");
        for ((method, route), stat) in g.iter() {
            let r = esc(route);
            let mut cum = 0u64;
            for (i, b) in BUCKETS.iter().enumerate() {
                cum += stat.bucket[i];
                out.push_str(&format!(
                    "keep_http_request_duration_seconds_bucket{{method=\"{}\",route=\"{}\",le=\"{}\"}} {}\n",
                    method, r, b, cum
                ));
            }
            cum += stat.bucket[BUCKETS.len()];
            out.push_str(&format!(
                "keep_http_request_duration_seconds_bucket{{method=\"{}\",route=\"{}\",le=\"+Inf\"}} {}\n",
                method, r, cum
            ));
            out.push_str(&format!(
                "keep_http_request_duration_seconds_sum{{method=\"{}\",route=\"{}\"}} {}\n",
                method, r, stat.sum_secs
            ));
            out.push_str(&format!(
                "keep_http_request_duration_seconds_count{{method=\"{}\",route=\"{}\"}} {}\n",
                method, r, stat.count
            ));
        }
        out
    }
}

/// Escape a Prometheus label value (backslash, quote, newline).
fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

/// `route_layer` middleware: time the request and record it against its matched
/// route pattern (so high-cardinality keys collapse to `/v1/kv/{key}` etc.).
pub async fn track(
    State(metrics): State<std::sync::Arc<HttpMetrics>>,
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().as_str().to_string();
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());
    let start = Instant::now();
    let resp = next.run(req).await;
    metrics.record(&method, &route, resp.status().as_u16(), start.elapsed().as_secs_f64());
    resp
}
