---
id: libs-h2c-src-lib-rs
summary: Lossless rust-source-unit coverage for `libs/h2c/src/lib.rs`.
capability_refs:
  - id: http2-cleartext-client-helpers
    role: primary
    claim: http2-cleartext-client-helpers-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the H2c library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/h2c/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/h2c/src/lib.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `llm` | libs/h2c/src/lib.rs | module | pub | 37 | pub mod llm; |
| `H2cError` | libs/h2c/src/lib.rs | re-export | pub | 40 | pub use error::{H2cError, Result}; |
| `Result` | libs/h2c/src/lib.rs | re-export | pub | 40 | pub use error::{H2cError, Result}; |
| `H2cManager` | libs/h2c/src/lib.rs | re-export | pub | 41 | pub use manager::{H2cManager, ManagerConfig, ManagerStats}; |
| `ManagerConfig` | libs/h2c/src/lib.rs | re-export | pub | 41 | pub use manager::{H2cManager, ManagerConfig, ManagerStats}; |
| `ManagerStats` | libs/h2c/src/lib.rs | re-export | pub | 41 | pub use manager::{H2cManager, ManagerConfig, ManagerStats}; |
| `server` | libs/h2c/src/lib.rs | module | pub | 46 | pub mod server; |
| `serve` | libs/h2c/src/lib.rs | re-export | pub | 48 | pub use server::serve; |
| `recommended_h2c_connections` | libs/h2c/src/lib.rs | function | pub | 55 | pub fn recommended_h2c_connections(concurrency: usize) -> usize { |
| `recommended_h2c_connections_for` | libs/h2c/src/lib.rs | function | pub | 63 | pub fn recommended_h2c_connections_for(concurrency: usize, parallelism: usize) -> usize { |
| `cpu_parallelism` | libs/h2c/src/lib.rs | function | pub | 73 | pub fn cpu_parallelism() -> usize { |
| `h2c_client` | libs/h2c/src/lib.rs | function | pub | 81 | pub fn h2c_client() -> reqwest::Result<reqwest::Client> { |
| `h2c_client_with` | libs/h2c/src/lib.rs | function | pub | 86 | pub fn h2c_client_with( |
| `H2cPool` | libs/h2c/src/lib.rs | struct | pub | 120 | pub struct H2cPool { |
| `for_concurrency` | libs/h2c/src/lib.rs | function | pub | 127 | pub fn for_concurrency(concurrency: usize) -> reqwest::Result<Self> { |
| `with_connections` | libs/h2c/src/lib.rs | function | pub | 132 | pub fn with_connections(n: usize) -> reqwest::Result<Self> { |
| `with_connections_and` | libs/h2c/src/lib.rs | function | pub | 137 | pub fn with_connections_and( |
| `connections` | libs/h2c/src/lib.rs | function | pub | 152 | pub fn connections(&self) -> usize { |
| `client` | libs/h2c/src/lib.rs | function | pub | 157 | pub fn client(&self) -> &reqwest::Client { |
| `get` | libs/h2c/src/lib.rs | function | pub | 163 | pub fn get<U: reqwest::IntoUrl>(&self, url: U) -> reqwest::RequestBuilder { |
| `post` | libs/h2c/src/lib.rs | function | pub | 168 | pub fn post<U: reqwest::IntoUrl>(&self, url: U) -> reqwest::RequestBuilder { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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

// The frame-level connection manager — built on the low-level `h2` crate so it
// can see GOAWAY / ping / flow-control and actively manage connections, where
// `H2cPool` (below) is the simpler reqwest-level round-robin option.
mod conn;
mod error;
pub mod llm;
mod manager;

pub use error::{H2cError, Result};
pub use manager::{H2cManager, ManagerConfig, ManagerStats};

// Server transport (`h2c::serve`) — the other half of the h2c stack. Behind the
// `server` feature so client-only consumers don't link the hyper server stack.
#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "server")]
pub use server::serve;

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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/h2c/src/lib.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/h2c/src/lib.rs` captured during libs codegen standardization.
```
