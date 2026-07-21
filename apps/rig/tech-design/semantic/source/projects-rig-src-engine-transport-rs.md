---
id: projects-rig-src-engine-transport-rs
capability_refs:
  - id: scenario-engine
    role: primary
    claim: scenario-step-dsl-execution
    coverage: partial
    rationale: "This source unit implements rig scenario discovery, execution, verdict, or report behavior used by the scenario engine."
fill_sections: [overview, source, changes]
---

# Standardized apps/rig/src/engine/transport.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/rig/src/engine/transport.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `HttpTransport` | apps/rig/src/engine/transport.rs | struct | pub | 39 |  |
| `PostgresTransport` | apps/rig/src/engine/transport.rs | struct | pub | 143 |  |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Pluggable load transports — the per-operation work the open-loop
//! [`loadgen`](super::loadgen) scheduler drives.
//!
//! The scheduler (ticks, workers, coordinated-omission, percentiles,
//! achieved_qps, honesty) is protocol-agnostic; a [`Transport`] supplies the
//! "do one operation" half. HTTP is built in; Postgres is behind the
//! `postgres` feature. Two transports measured on this ONE scheduler are
//! comparable by construction — a unified thin Rust client whose overhead is
//! a near-constant floor across targets, so the ratio reflects the backend +
//! protocol, not the client library (the whole point of comparing lumen vs pg
//! fairly).

use crate::scenario::interp::VarStore;
use crate::scenario::step::HttpRequest;

use super::http;

/// Per-worker operation handle: built once per worker thread, then driven once
/// per scheduled tick. `execute` returns `Ok` on success, `Err(reason)` on a
/// failure that counts toward `error_rate`.
pub trait OpWorker: Send {
    fn execute(&mut self) -> Result<(), String>;
}

/// A protocol the load scheduler can drive. [`connect`](Transport::connect)
/// builds one [`OpWorker`] per worker thread (a pg connection + prepared
/// statement, or a stateless HTTP sender), so per-worker state is never shared
/// across threads.
pub trait Transport: Send + Sync {
    fn connect(&self) -> Result<Box<dyn OpWorker>, String>;
}

// ---------------------------------------------------------------------------
// HTTP (built in) — thin ureq client, the existing load path.
// ---------------------------------------------------------------------------

pub struct HttpTransport {
    pub request: HttpRequest,
    pub vars: VarStore,
}

impl Transport for HttpTransport {
    fn connect(&self) -> Result<Box<dyn OpWorker>, String> {
        Ok(Box::new(HttpWorker {
            agent: http::agent_for(&self.request),
            request: self.request.clone(),
            vars: self.vars.clone(),
        }))
    }
}

struct HttpWorker {
    agent: ureq::Agent,
    request: HttpRequest,
    vars: VarStore,
}

impl OpWorker for HttpWorker {
    fn execute(&mut self) -> Result<(), String> {
        match http::execute_with_agent(&self.agent, &self.request, &self.vars) {
            Ok(o) if o.violation.is_none() => Ok(()),
            Ok(o) => Err(o
                .violation
                .unwrap_or_else(|| "expectation violated".to_string())),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::step::HttpExpect;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::time::Duration;

    fn read_request(stream: &mut TcpStream) -> std::io::Result<bool> {
        let mut request = Vec::new();
        let mut buf = [0_u8; 1024];
        loop {
            let read = stream.read(&mut buf)?;
            if read == 0 {
                return Ok(false);
            }
            request.extend_from_slice(&buf[..read]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                return Ok(true);
            }
        }
    }

    #[test]
    fn http_worker_reuses_one_keep_alive_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let mut connections = 0_u32;
            let mut requests = 0_u32;
            while requests < 2 {
                let (mut stream, _) = listener.accept().unwrap();
                connections += 1;
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                while requests < 2 && read_request(&mut stream).unwrap_or(false) {
                    requests += 1;
                    stream
                        .write_all(
                            b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\nconnection: keep-alive\r\n\r\n{}",
                        )
                        .unwrap();
                }
            }
            connections
        });

        let transport = HttpTransport {
            request: HttpRequest {
                method: "GET".into(),
                url: format!("http://{addr}/healthz"),
                body: None,
                expect: HttpExpect::default(),
            },
            vars: VarStore::new(),
        };
        let mut worker = transport.connect().unwrap();
        worker.execute().unwrap();
        worker.execute().unwrap();

        assert_eq!(server.join().unwrap(), 1);
    }
}

// ---------------------------------------------------------------------------
// Postgres (feature = "postgres") — sync `postgres` client, prepared once per
// worker. Keeps loadgen's no-async-runtime thread model.
// ---------------------------------------------------------------------------

#[cfg(feature = "postgres")]
pub struct PostgresTransport {
    /// libpq-style DSN, e.g. `postgresql://user@127.0.0.1/db`.
    pub dsn: String,
    /// The SQL prepared once per worker and executed each tick (literals inline;
    /// no bind params in v1 — the query is fixed per cell).
    pub sql: String,
}

#[cfg(feature = "postgres")]
impl Transport for PostgresTransport {
    fn connect(&self) -> Result<Box<dyn OpWorker>, String> {
        let mut client = postgres::Client::connect(&self.dsn, postgres::NoTls)
            .map_err(|e| format!("pg connect `{}` failed: {e}", self.dsn))?;
        let stmt = client
            .prepare(&self.sql)
            .map_err(|e| format!("pg prepare failed: {e}"))?;
        Ok(Box::new(PgWorker { client, stmt }))
    }
}

#[cfg(feature = "postgres")]
struct PgWorker {
    client: postgres::Client,
    stmt: postgres::Statement,
}

#[cfg(feature = "postgres")]
impl OpWorker for PgWorker {
    fn execute(&mut self) -> Result<(), String> {
        self.client
            .query(&self.stmt, &[])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/rig/src/engine/transport.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `apps/rig/src/engine/transport.rs` captured during rig
      standardization onto the codegen ladder.
```
