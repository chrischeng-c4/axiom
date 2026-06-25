//! `h2c` — shared HTTP/2 cleartext (h2c) client helpers for the ecosystem.
//!
//! Several components (loom → keep/relay, lumen's relay WAL, relay's raft peer
//! transport) talk to each other over **h2c** (HTTP/2 over cleartext, via
//! prior-knowledge — no TLS, no ALPN). Each used to hand-roll
//! `reqwest::Client::builder().http2_prior_knowledge().build()`. This crate
//! centralizes that, plus the connection-pool sizing that actually makes h2c
//! fast.
//!
//! ## Why a pool — the connection-count heuristic
//!
//! A single h2 connection multiplexes every stream, but all of its framing /
//! HPACK work serializes through one read/write task, so throughput bottlenecks
//! on **one core**. Spreading streams over a *few* connections recovers
//! multi-core throughput while keeping the connection count far below
//! HTTP/1.1's one-per-concurrent-request. Empirically (see
//! `examples/conn_sweep.rs`) throughput saturates around `ln(concurrency)`
//! connections, after which extra connections only add sockets. So:
//!
//! ```text
//! connections = clamp(ceil(ln(concurrency)), 1, cpu_parallelism)
//! ```
//!
//! `ln` grows so slowly it self-caps below the core count for any realistic
//! concurrency (`ln(22026) ≈ 10`), which is exactly why it tracks the knee
//! without ever over-provisioning.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Recommended number of h2c connections for a target peak `concurrency`, using
/// the available CPU parallelism as the upper cap.
///
/// See the crate docs for the rationale. Equivalent to
/// [`recommended_h2c_connections_for`] with `parallelism = available cores`.
pub fn recommended_h2c_connections(concurrency: usize) -> usize {
    recommended_h2c_connections_for(concurrency, cpu_parallelism())
}

/// Like [`recommended_h2c_connections`] but with an explicit core cap, for
/// deterministic sizing and testing.
///
/// `connections = clamp(ceil(ln(concurrency)), 1, parallelism)`.
pub fn recommended_h2c_connections_for(concurrency: usize, parallelism: usize) -> usize {
    let cap = parallelism.max(1);
    if concurrency <= 2 {
        return 1;
    }
    let ln = (concurrency as f64).ln().ceil() as usize;
    ln.clamp(1, cap)
}

/// Available CPU parallelism (`std::thread::available_parallelism`), or 1.
pub fn cpu_parallelism() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

/// Build a single-connection h2c client — the drop-in replacement for
/// `reqwest::Client::builder().http2_prior_knowledge().build()`.
pub fn h2c_client() -> reqwest::Result<reqwest::Client> {
    h2c_builder(None, None).build()
}

/// Like [`h2c_client`] with an optional per-request `timeout` and `user_agent`.
pub fn h2c_client_with(
    timeout: Option<Duration>,
    user_agent: Option<&str>,
) -> reqwest::Result<reqwest::Client> {
    h2c_builder(timeout, user_agent).build()
}

fn h2c_builder(timeout: Option<Duration>, user_agent: Option<&str>) -> reqwest::ClientBuilder {
    let mut b = reqwest::Client::builder().http2_prior_knowledge();
    if let Some(t) = timeout {
        b = b.timeout(t);
    }
    if let Some(ua) = user_agent {
        b = b.user_agent(ua.to_string());
    }
    b
}

/// A round-robin pool of h2c clients. Each underlying [`reqwest::Client`] owns
/// one connection that multiplexes many streams; requests are dispatched across
/// them round-robin so framing spreads over multiple cores.
///
/// Size it from a target concurrency with [`H2cPool::for_concurrency`] (uses
/// [`recommended_h2c_connections`]) or pin the count with
/// [`H2cPool::with_connections`]. Cheap to [`Clone`] (shares the clients and the
/// cursor); clone freely across tasks.
///
/// ```no_run
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = h2c::H2cPool::for_concurrency(256)?; // ~6 connections
/// let resp = pool.get("http://keep:7117/healthz").send().await?;
/// # let _ = resp; Ok(()) }
/// ```
#[derive(Clone)]
pub struct H2cPool {
    clients: Arc<Vec<reqwest::Client>>,
    next: Arc<AtomicUsize>,
}

impl H2cPool {
    /// Build a pool sized by [`recommended_h2c_connections`] for `concurrency`.
    pub fn for_concurrency(concurrency: usize) -> reqwest::Result<Self> {
        Self::with_connections(recommended_h2c_connections(concurrency))
    }

    /// Build a pool of exactly `n` connections (clamped to at least 1).
    pub fn with_connections(n: usize) -> reqwest::Result<Self> {
        Self::with_connections_and(n, None, None)
    }

    /// Build a pool of `n` connections, each with the given `timeout`/`user_agent`.
    pub fn with_connections_and(
        n: usize,
        timeout: Option<Duration>,
        user_agent: Option<&str>,
    ) -> reqwest::Result<Self> {
        let clients = (0..n.max(1))
            .map(|_| h2c_builder(timeout, user_agent).build())
            .collect::<reqwest::Result<Vec<_>>>()?;
        Ok(Self {
            clients: Arc::new(clients),
            next: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Number of underlying connections.
    pub fn connections(&self) -> usize {
        self.clients.len()
    }

    /// The next client in round-robin order.
    pub fn client(&self) -> &reqwest::Client {
        let i = self.next.fetch_add(1, Ordering::Relaxed) % self.clients.len();
        &self.clients[i]
    }

    /// Round-robin `GET`.
    pub fn get<U: reqwest::IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        self.client().get(url)
    }

    /// Round-robin `POST`.
    pub fn post<U: reqwest::IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        self.client().post(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heuristic_is_log_shaped() {
        // ceil(ln(c)), capped well above so the log shape shows through
        assert_eq!(recommended_h2c_connections_for(16, 64), 3); // ln=2.77
        assert_eq!(recommended_h2c_connections_for(64, 64), 5); // ln=4.16
        assert_eq!(recommended_h2c_connections_for(256, 64), 6); // ln=5.55
        assert_eq!(recommended_h2c_connections_for(1024, 64), 7); // ln=6.93
        assert_eq!(recommended_h2c_connections_for(4096, 64), 9); // ln=8.32
    }

    #[test]
    fn heuristic_clamps_to_cores_and_floor() {
        // never exceeds the core cap
        assert_eq!(recommended_h2c_connections_for(1_000_000, 4), 4);
        // tiny concurrency → a single connection
        assert_eq!(recommended_h2c_connections_for(0, 8), 1);
        assert_eq!(recommended_h2c_connections_for(1, 8), 1);
        assert_eq!(recommended_h2c_connections_for(2, 8), 1);
        // core cap is at least 1 even if passed 0
        assert_eq!(recommended_h2c_connections_for(1024, 0), 1);
    }

    #[test]
    fn pool_round_robins_across_connections() {
        let pool = H2cPool::with_connections(3).unwrap();
        assert_eq!(pool.connections(), 3);
        let ptr = |c: &reqwest::Client| c as *const reqwest::Client;
        let a = ptr(pool.client());
        let b = ptr(pool.client());
        let c = ptr(pool.client());
        let d = ptr(pool.client());
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
        assert_eq!(a, d); // wraps back to the first connection
    }

    #[test]
    fn pool_floor_is_one_connection() {
        assert_eq!(H2cPool::with_connections(0).unwrap().connections(), 1);
    }
}
