// SPEC-MANAGED: projects/rig/tech-design/semantic/source/projects-rig-src-engine-transport-rs.md#rust-source-unit
// CODEGEN-BEGIN
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
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-engine-transport-rs.md#source
pub trait OpWorker: Send {
    fn execute(&mut self) -> Result<(), String>;
}

/// A protocol the load scheduler can drive. [`connect`](Transport::connect)
/// builds one [`OpWorker`] per worker thread (a pg connection + prepared
/// statement, or a stateless HTTP sender), so per-worker state is never shared
/// across threads.
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-engine-transport-rs.md#source
pub trait Transport: Send + Sync {
    fn connect(&self) -> Result<Box<dyn OpWorker>, String>;
}

// ---------------------------------------------------------------------------
// HTTP (built in) — thin ureq client, the existing load path.
// ---------------------------------------------------------------------------

/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-engine-transport-rs.md#source
pub struct HttpTransport {
    pub request: HttpRequest,
    pub vars: VarStore,
}

/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-engine-transport-rs.md#source
impl Transport for HttpTransport {
    fn connect(&self) -> Result<Box<dyn OpWorker>, String> {
        Ok(Box::new(HttpWorker {
            request: self.request.clone(),
            vars: self.vars.clone(),
        }))
    }
}

struct HttpWorker {
    request: HttpRequest,
    vars: VarStore,
}

/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-engine-transport-rs.md#source
impl OpWorker for HttpWorker {
    fn execute(&mut self) -> Result<(), String> {
        match http::execute(&self.request, &self.vars) {
            Ok(o) if o.violation.is_none() => Ok(()),
            Ok(o) => Err(o
                .violation
                .unwrap_or_else(|| "expectation violated".to_string())),
            Err(e) => Err(e),
        }
    }
}

// ---------------------------------------------------------------------------
// Postgres (feature = "postgres") — sync `postgres` client, prepared once per
// worker. Keeps loadgen's no-async-runtime thread model.
// ---------------------------------------------------------------------------

#[cfg(feature = "postgres")]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-engine-transport-rs.md#source
pub struct PostgresTransport {
    /// libpq-style DSN, e.g. `postgresql://user@127.0.0.1/db`.
    pub dsn: String,
    /// The SQL prepared once per worker and executed each tick (literals inline;
    /// no bind params in v1 — the query is fixed per cell).
    pub sql: String,
}

#[cfg(feature = "postgres")]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-engine-transport-rs.md#source
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
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-engine-transport-rs.md#source
impl OpWorker for PgWorker {
    fn execute(&mut self) -> Result<(), String> {
        self.client
            .query(&self.stmt, &[])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}
// CODEGEN-END
