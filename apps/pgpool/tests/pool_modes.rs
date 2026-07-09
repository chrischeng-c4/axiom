// SPEC-MANAGED: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#unit-test
// <HANDWRITE gap="missing-generator:logic:pgpool-backend-pool" tracker="#1289" reason="Backend pool needs generator primitives that do not exist yet.">
//! End-to-end coverage of transaction-mode backend pooling against a real
//! local Postgres (AC1-AC5). Every test here graceful-skips (prints why,
//! then returns) when the environment's Postgres isn't reachable, per the
//! repo's "real services over mocks, skip gracefully" testing convention --
//! see `apps/pgpool/CLAUDE.md`/root `CLAUDE.md` Testing section. Mirrors
//! `tests/session_proxy.rs`'s real-Postgres discovery/helper pattern
//! (`real_backend_ready`, `backend_user`, `proxy_dsn`, `simple_query_i32`).
//!
//! A recurring semantic these tests lean on, confirmed by reading
//! `src/pool/backend_pool.rs`: `BackendPool::acquire_fresh()` (used only
//! for the one-time admission handshake) never idle-reuses -- it only ever
//! tries a brand-new semaphore permit or waits. A connection parked in the
//! idle set via `LeaseDisposition::ReturnToIdle` keeps its permit "spent"
//! rather than returning it to the semaphore, so it still fully occupies
//! its capacity slot even though nothing is actively using it. That means
//! a *second* client's admission cannot be unblocked merely by some other
//! lease's `ReturnToIdle` -- only a genuine `LeaseDisposition::Close` (or a
//! dead-idle-connection drop) truly frees a slot for a waiting
//! `acquire_fresh()` caller. `BackendPool::acquire()` (per-transaction) is
//! different: it rechecks the idle set on every wake, so a waiting
//! `acquire()` caller *is* correctly unblocked by another lease's
//! `ReturnToIdle`. AC3a/AC3b below are shaped around this distinction.

use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::Duration;

use server_core::{BindConfig, ConnectionBudget};

use pgpool::pool::{
    BackendPool, PoolConfig, PoolStats, TransactionHandler, TransactionProxyConfig,
};
use pgpool::proxy::{BackendEndpointConfig, SessionHandler, SessionProxyConfig};
use pgpool::wire::WireCodecConfig;

/// Confirms the local Postgres is reachable, the current OS user can log in
/// via `trust` auth against the `postgres` database, and a trivial query
/// round-trips -- all in one probe, so every AC test below can graceful-skip
/// with a single check instead of failing when this exact local setup isn't
/// present. Identical to `tests/session_proxy.rs`'s helper of the same
/// name (kept duplicated rather than shared across integration-test
/// binaries, which each compile as independent crates).
async fn real_backend_ready() -> Option<(SocketAddr, String)> {
    let addr: SocketAddr = "127.0.0.1:5432".parse().ok()?;
    let user = backend_user();
    let dsn = format!(
        "host={} port={} user={} dbname=postgres connect_timeout=2",
        addr.ip(),
        addr.port(),
        user
    );
    let (client, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .ok()?;
    tokio::spawn(connection);
    client.simple_query("SELECT 1").await.ok()?;
    Some((addr, user))
}

fn backend_user() -> String {
    std::env::var("USER").unwrap_or_else(|_| "postgres".to_string())
}

fn proxy_dsn(proxy_addr: SocketAddr, user: &str) -> String {
    format!(
        "host={} port={} user={} dbname=postgres connect_timeout=5",
        proxy_addr.ip(),
        proxy_addr.port(),
        user
    )
}

/// Runs `sql` via the simple query protocol (tag `'Q'`) and returns the
/// first row's `column`, parsed as `i32`. See `tests/session_proxy.rs`'s
/// helper of the same name for why the simple query protocol is used
/// instead of tokio-postgres's default extended query protocol (a
/// pre-existing wire-codec gap tracked under #1287, unrelated to this WI).
async fn simple_query_i32(client: &tokio_postgres::Client, sql: &str, column: &str) -> i32 {
    let messages = client
        .simple_query(sql)
        .await
        .expect("simple-query round-trip through pgpool");
    for message in messages {
        if let tokio_postgres::SimpleQueryMessage::Row(row) = message {
            if let Some(value) = row.get(column) {
                return value.parse().expect("column value parses as i32");
            }
        }
    }
    panic!("simple query {sql:?} returned no row for column {column:?}");
}

fn pool_config(
    backend_addr: SocketAddr,
    max_backend_connections: usize,
    acquire_timeout: Duration,
) -> PoolConfig {
    PoolConfig {
        endpoint: BackendEndpointConfig {
            host: backend_addr.ip().to_string(),
            port: backend_addr.port(),
        },
        max_backend_connections,
        acquire_timeout,
        backend_connect_timeout: Duration::from_secs(5),
        wire: WireCodecConfig::default(),
    }
}

/// Starts an in-process transaction-mode proxy sharing the given
/// (already-constructed) `BackendPool` and `ConnectionBudget`, so callers
/// can keep a handle to both for `stats()`/`active()` assertions. Mirrors
/// `tests/session_proxy.rs::spawn_proxy`.
async fn spawn_transaction_proxy(
    backend_pool: BackendPool,
    frontend_budget: ConnectionBudget,
) -> (
    SocketAddr,
    tokio::task::JoinHandle<()>,
    tokio::sync::oneshot::Sender<()>,
) {
    let handler = TransactionHandler::new(TransactionProxyConfig {
        frontend_budget,
        backend_pool,
        wire: WireCodecConfig::default(),
        drain_timeout: Duration::from_secs(5),
    });
    let server_config = tcp_server::TcpServerConfig::new(BindConfig::localhost(0));
    let listener = tcp_server::bind(&server_config)
        .await
        .expect("bind transaction proxy listener");
    let proxy_addr = listener.local_addr().expect("proxy listener addr");
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(tcp_server::serve(
        listener,
        server_config,
        handler,
        async move {
            let _ = shutdown_rx.await;
        },
    ));
    (proxy_addr, server, shutdown_tx)
}

/// Starts an in-process session-mode proxy sharing the given (already
/// constructed) `BackendPool` -- used only by the churn test (AC4) to mix
/// genuine session-mode activity into the same pool alongside
/// transaction-mode churn.
async fn spawn_session_proxy(
    backend_addr: SocketAddr,
    backend_pool: BackendPool,
    max_frontend: usize,
) -> (
    SocketAddr,
    tokio::task::JoinHandle<()>,
    tokio::sync::oneshot::Sender<()>,
) {
    let config = SessionProxyConfig {
        backend: BackendEndpointConfig {
            host: backend_addr.ip().to_string(),
            port: backend_addr.port(),
        },
        frontend_budget: ConnectionBudget::new(max_frontend),
        backend_connect_timeout: Duration::from_secs(5),
        drain_timeout: Duration::from_secs(5),
        wire: WireCodecConfig::default(),
        backend_pool,
    };
    let handler = SessionHandler::new(config);
    let server_config = tcp_server::TcpServerConfig::new(BindConfig::localhost(0));
    let listener = tcp_server::bind(&server_config)
        .await
        .expect("bind session proxy listener");
    let proxy_addr = listener.local_addr().expect("proxy listener addr");
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(tcp_server::serve(
        listener,
        server_config,
        handler,
        async move {
            let _ = shutdown_rx.await;
        },
    ));
    (proxy_addr, server, shutdown_tx)
}

async fn stop_proxy(
    server: tokio::task::JoinHandle<()>,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
) {
    let _ = shutdown_tx.send(());
    server.await.expect("proxy server task joins");
}

/// verify: pool_modes::transaction_mode_reuses_backend_connections_across_sequential_transactions (AC1a)
#[tokio::test]
async fn transaction_mode_reuses_backend_connections_across_sequential_transactions() {
    let Some((backend_addr, user)) = real_backend_ready().await else {
        eprintln!(
            "skipping transaction_mode_reuses_backend_connections_across_sequential_transactions: \
             no reachable local Postgres at 127.0.0.1:5432 for user {:?}",
            backend_user()
        );
        return;
    };

    let backend_pool = BackendPool::new(pool_config(backend_addr, 4, Duration::from_secs(5)));
    let (proxy_addr, server, shutdown_tx) =
        spawn_transaction_proxy(backend_pool, ConnectionBudget::new(10)).await;
    let dsn = proxy_dsn(proxy_addr, &user);

    let (client, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .expect("client admits");
    let connection_task = tokio::spawn(connection);

    // One persistent client issuing 20 sequential transactions: since each
    // transaction's lease is released (ReturnToIdle) before the next
    // frontend frame is even read, and this is the ONLY client, every
    // subsequent transaction's acquire() must reuse the very same idle
    // connection -- all 20 real Postgres backend pids must be identical.
    let mut pids = HashSet::new();
    for i in 0..20 {
        let pid = simple_query_i32(&client, "SELECT pg_backend_pid() AS pid", "pid").await;
        pids.insert(pid);
        assert!(
            pids.len() == 1,
            "transaction {i} opened a distinct physical backend connection instead of reusing the idle one; pids so far: {pids:?}"
        );
    }
    assert_eq!(
        pids.len(),
        1,
        "20 sequential transactions from one client must all reuse the single idle backend connection"
    );

    drop(client);
    connection_task
        .await
        .expect("connection task joined")
        .expect("connection driver ended cleanly");

    stop_proxy(server, shutdown_tx).await;
}

/// verify: pool_modes::concurrent_transactions_isolated_on_distinct_backends (AC1b)
#[tokio::test]
async fn concurrent_transactions_isolated_on_distinct_backends() {
    let Some((backend_addr, user)) = real_backend_ready().await else {
        eprintln!(
            "skipping concurrent_transactions_isolated_on_distinct_backends: \
             no reachable local Postgres at 127.0.0.1:5432 for user {:?}",
            backend_user()
        );
        return;
    };

    let backend_pool = BackendPool::new(pool_config(backend_addr, 4, Duration::from_secs(5)));
    let (proxy_addr, server, shutdown_tx) =
        spawn_transaction_proxy(backend_pool.clone(), ConnectionBudget::new(10)).await;
    let dsn = proxy_dsn(proxy_addr, &user);

    let (client_a, connection_a) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .expect("client A admits");
    let task_a = tokio::spawn(connection_a);
    let (client_b, connection_b) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .expect("client B admits");
    let task_b = tokio::spawn(connection_b);

    // Run both clients' slow transactions concurrently, and race a stats
    // probe partway through so we observe both leases held ACTIVE at the
    // same instant (not just sequentially, which would trivially never
    // collide).
    let (pid_a, pid_b, mid_flight_stats) = tokio::join!(
        simple_query_i32(
            &client_a,
            "SELECT pg_backend_pid() AS pid, pg_sleep(0.3) AS delay",
            "pid"
        ),
        simple_query_i32(
            &client_b,
            "SELECT pg_backend_pid() AS pid, pg_sleep(0.3) AS delay",
            "pid"
        ),
        async {
            tokio::time::sleep(Duration::from_millis(120)).await;
            backend_pool.stats()
        }
    );

    assert_ne!(
        pid_a, pid_b,
        "concurrent transactions from different clients must not share a backend connection"
    );
    assert_eq!(
        mid_flight_stats.backend_active, 2,
        "both concurrent transactions must hold distinct active backend leases at the same time"
    );

    drop(client_a);
    drop(client_b);
    task_a
        .await
        .expect("connection task A joined")
        .expect("connection A ended cleanly");
    task_b
        .await
        .expect("connection task B joined")
        .expect("connection B ended cleanly");

    stop_proxy(server, shutdown_tx).await;
}

/// verify: pool_modes::reset_between_owners_prevents_session_state_leak_across_transaction_leases (AC2)
#[tokio::test]
async fn reset_between_owners_prevents_session_state_leak_across_transaction_leases() {
    let Some((backend_addr, user)) = real_backend_ready().await else {
        eprintln!(
            "skipping reset_between_owners_prevents_session_state_leak_across_transaction_leases: \
             no reachable local Postgres at 127.0.0.1:5432 for user {:?}",
            backend_user()
        );
        return;
    };

    let backend_pool = BackendPool::new(pool_config(backend_addr, 2, Duration::from_secs(5)));
    let (proxy_addr, server, shutdown_tx) =
        spawn_transaction_proxy(backend_pool, ConnectionBudget::new(10)).await;
    let dsn = proxy_dsn(proxy_addr, &user);

    let (client, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .expect("client admits");
    let connection_task = tokio::spawn(connection);

    // Transaction 1: create session-local state (a temp table) that only
    // survives on the underlying backend connection unless DISCARD ALL runs
    // before the next owner reuses it.
    client
        .simple_query("CREATE TEMP TABLE pool_modes_leak_check (n int)")
        .await
        .expect("create temp table in transaction 1");

    // Transaction 2: same client (same wire session), but a fresh
    // per-transaction lease (BackendPool::acquire() reuse). The temp table
    // must be gone, proving release()'s DISCARD ALL ran between the two
    // leases even though both very likely reused the same physical
    // connection (this client is the only one contending for the pool).
    let result = client
        .simple_query("SELECT n FROM pool_modes_leak_check")
        .await;
    assert!(
        result.is_err(),
        "temp table created by a prior transaction lease must not be visible after the DISCARD ALL reset run between leases"
    );

    // The client must remain fully usable afterward -- the reset didn't
    // corrupt the connection, it just erased session-local state.
    let value = simple_query_i32(&client, "SELECT 1 AS one", "one").await;
    assert_eq!(value, 1);

    drop(client);
    connection_task
        .await
        .expect("connection task joined")
        .expect("connection driver ended cleanly");

    stop_proxy(server, shutdown_tx).await;
}

/// verify: pool_modes::saturation_wait_then_acquire_succeeds_when_lease_frees (AC3a)
#[tokio::test]
async fn saturation_wait_then_acquire_succeeds_when_lease_frees() {
    let Some((backend_addr, user)) = real_backend_ready().await else {
        eprintln!(
            "skipping saturation_wait_then_acquire_succeeds_when_lease_frees: \
             no reachable local Postgres at 127.0.0.1:5432 for user {:?}",
            backend_user()
        );
        return;
    };

    // max=1: client B's own admission (acquire_fresh(), never idle-reuse)
    // can only succeed once client A's slot is genuinely freed via
    // LeaseDisposition::Close -- see this file's top-level doc comment.
    let backend_pool = BackendPool::new(pool_config(backend_addr, 1, Duration::from_secs(5)));
    let (proxy_addr, server, shutdown_tx) =
        spawn_transaction_proxy(backend_pool, ConnectionBudget::new(10)).await;
    let dsn = proxy_dsn(proxy_addr, &user);

    let (client_a, connection_a) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .expect("client A admits while the pool has capacity");
    let task_a = tokio::spawn(connection_a);

    // Put client A mid-transaction (an ACTIVE lease, not idle-parked) so
    // that abruptly disconnecting it below ends the leg via EOF while a
    // lease is held -- which releases via Close (freeing the permit),
    // unlike disconnecting while idle-with-no-lease (which touches nothing;
    // the backend just stays idle-parked, as pool.rs's dropped-lease and
    // saturation offline tests already establish for the fake-backend
    // case).
    let slow_query = tokio::spawn(async move {
        let _ = client_a.simple_query("SELECT pg_sleep(2)").await;
    });
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Client B's admission must block: the pool's sole permit is spent
    // (held by A's active lease), and acquire_fresh() never idle-reuses.
    let dsn_for_b = dsn.clone();
    let connect_b =
        tokio::spawn(
            async move { tokio_postgres::connect(&dsn_for_b, tokio_postgres::NoTls).await },
        );
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(
        !connect_b.is_finished(),
        "client B's admission must still be waiting while the pool's sole slot is held ACTIVE by client A"
    );

    // Force client A's leg to end abruptly while its transaction lease is
    // still held -- a real abrupt client disconnect, releasing via Close.
    slow_query.abort();
    task_a.abort();
    let _ = slow_query.await;
    let _ = task_a.await;

    // Client B's waiting admission must now succeed well within
    // acquire_timeout (5s) because A's disconnect freed the pool's sole
    // capacity slot -- not silently dropped, per the AC3a text.
    let (client_b, connection_b) = tokio::time::timeout(Duration::from_secs(3), connect_b)
        .await
        .expect("client B's admission must unblock once A's slot frees, not hang until acquire_timeout elapses")
        .expect("connect task joined")
        .expect("client B eventually admits once A's slot is freed");
    let task_b = tokio::spawn(connection_b);

    let value = simple_query_i32(&client_b, "SELECT 1 AS one", "one").await;
    assert_eq!(
        value, 1,
        "client B must be fully usable after waiting for and acquiring the freed slot"
    );

    drop(client_b);
    task_b
        .await
        .expect("connection task B joined")
        .expect("connection B ended cleanly");

    stop_proxy(server, shutdown_tx).await;
}

/// verify: pool_modes::saturation_timeout_produces_typed_error_response (AC3b)
#[tokio::test]
async fn saturation_timeout_produces_typed_error_response() {
    let Some((backend_addr, user)) = real_backend_ready().await else {
        eprintln!(
            "skipping saturation_timeout_produces_typed_error_response: \
             no reachable local Postgres at 127.0.0.1:5432 for user {:?}",
            backend_user()
        );
        return;
    };

    // max=1 with a short acquire_timeout: client A's admission alone is
    // enough to keep the pool saturated from any other client's admission
    // perspective for the rest of the test -- acquire_fresh() never
    // idle-reuses, so A's connection sitting idle-parked after its own
    // handshake still fully occupies the sole capacity slot (see this
    // file's top-level doc comment).
    let backend_pool = BackendPool::new(pool_config(backend_addr, 1, Duration::from_millis(300)));
    let (proxy_addr, server, shutdown_tx) =
        spawn_transaction_proxy(backend_pool, ConnectionBudget::new(10)).await;
    let dsn = proxy_dsn(proxy_addr, &user);

    let (client_a, connection_a) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .expect("client A admits while the pool has capacity");
    let task_a = tokio::spawn(connection_a);

    let started = std::time::Instant::now();
    let rejected = tokio_postgres::connect(&dsn, tokio_postgres::NoTls).await;
    let elapsed = started.elapsed();

    let err = rejected
        .err()
        .expect("client B's admission must be rejected once acquire_timeout elapses with the pool still saturated");
    let code = err.code().expect("rejection must carry a SQLSTATE");
    assert_eq!(
        code,
        &tokio_postgres::error::SqlState::TOO_MANY_CONNECTIONS,
        "must be the pool's synthesized 53300 BackendPoolSaturated rejection, not some other connection error"
    );
    assert!(
        elapsed >= Duration::from_millis(300),
        "must actually wait out acquire_timeout rather than rejecting immediately, got {elapsed:?}"
    );
    assert!(
        elapsed < Duration::from_secs(3),
        "must not hang well past acquire_timeout, got {elapsed:?}"
    );

    // Client A must be entirely unaffected by client B's rejection.
    let value = simple_query_i32(&client_a, "SELECT 1 AS one", "one").await;
    assert_eq!(value, 1);

    drop(client_a);
    task_a
        .await
        .expect("connection task A joined")
        .expect("connection A ended cleanly");

    stop_proxy(server, shutdown_tx).await;
}

/// verify: pool_modes::churn_100_cycles_holds_backend_count_stable_no_leak (AC4)
#[tokio::test]
async fn churn_100_cycles_holds_backend_count_stable_no_leak() {
    let Some((backend_addr, user)) = real_backend_ready().await else {
        eprintln!(
            "skipping churn_100_cycles_holds_backend_count_stable_no_leak: \
             no reachable local Postgres at 127.0.0.1:5432 for user {:?}",
            backend_user()
        );
        return;
    };

    let backend_pool = BackendPool::new(pool_config(backend_addr, 4, Duration::from_secs(5)));
    let (txn_proxy_addr, txn_server, txn_shutdown_tx) =
        spawn_transaction_proxy(backend_pool.clone(), ConnectionBudget::new(20)).await;
    let (session_proxy_addr, session_server, session_shutdown_tx) =
        spawn_session_proxy(backend_addr, backend_pool.clone(), 4).await;
    let txn_dsn = proxy_dsn(txn_proxy_addr, &user);

    // A persistent SESSION-mode connection sharing the same backend pool,
    // alive for the whole churn loop, so the churn genuinely mixes
    // session-mode and transaction-mode activity against one pool (per the
    // AC4 text), not transaction-mode alone.
    let (session_client, session_connection) =
        tokio_postgres::connect(&proxy_dsn(session_proxy_addr, &user), tokio_postgres::NoTls)
            .await
            .expect("session-mode connection admits");
    let session_task = tokio::spawn(session_connection);
    let value = simple_query_i32(&session_client, "SELECT 1 AS one", "one").await;
    assert_eq!(value, 1);

    // One persistent transaction-mode client drives most of the churn via
    // ordinary ReturnToIdle reuse cycles.
    let (txn_client, txn_connection) = tokio_postgres::connect(&txn_dsn, tokio_postgres::NoTls)
        .await
        .expect("transaction-mode client admits");
    let txn_task = tokio::spawn(txn_connection);

    let mut peak_active = 0usize;
    let mut peak_idle = 0usize;
    for cycle in 0..100 {
        if cycle % 10 == 9 {
            // Mixed disposition: an independent short-lived connection
            // that's abruptly disconnected mid-query, exercising
            // LeaseDisposition::Close. Its admission's permit and its
            // Close-driven teardown's freed permit net to zero extra idle
            // growth (unlike a clean disconnect while idle-with-no-lease,
            // which would leave a permanent idle leftover -- this file's
            // top-level doc comment explains why idle-parked permits never
            // free on their own).
            let dsn = txn_dsn.clone();
            let (transient_client, transient_connection) =
                tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
                    .await
                    .expect("transient client admits for a Close-disposition churn cycle");
            let transient_task = tokio::spawn(transient_connection);
            let query = tokio::spawn(async move {
                let _ = transient_client.simple_query("SELECT pg_sleep(1)").await;
            });
            tokio::time::sleep(Duration::from_millis(50)).await;
            query.abort();
            transient_task.abort();
            let _ = query.await;
            let _ = transient_task.await;
            // Give the server side a moment to observe the EOF and release
            // via Close before the next cycle's stats snapshot.
            tokio::time::sleep(Duration::from_millis(50)).await;
        } else {
            let value = simple_query_i32(&txn_client, "SELECT 1 AS one", "one").await;
            assert_eq!(
                value, 1,
                "churn cycle {cycle} must still round-trip correctly"
            );
        }

        let stats = backend_pool.stats();
        peak_active = peak_active.max(stats.backend_active);
        peak_idle = peak_idle.max(stats.backend_idle);
    }

    // Bounded: baseline (session's 1 + persistent txn client's 1) plus at
    // most one transient connection momentarily in flight -- nowhere near
    // max_backend_connections (4), proving no unbounded growth/leak across
    // 100+ acquire/release cycles.
    assert!(
        peak_active + peak_idle <= 3,
        "backend connection count must stay bounded across the churn loop, got peak_active={peak_active} peak_idle={peak_idle}"
    );

    drop(txn_client);
    txn_task
        .await
        .expect("txn connection task joined")
        .expect("txn connection ended cleanly");
    drop(session_client);
    session_task
        .await
        .expect("session connection task joined")
        .expect("session connection ended cleanly");

    // Give both teardowns a moment to be observed server-side.
    tokio::time::sleep(Duration::from_millis(200)).await;

    let final_stats = backend_pool.stats();
    assert_eq!(
        final_stats.backend_active, 0,
        "no leaked active lease after every client cleanly disconnected"
    );
    // Session mode releases its whole-session lease via Close at teardown
    // (R2b), fully freeing its connection. The persistent transaction-mode
    // client disconnects cleanly while holding no lease (idle_no_lease), so
    // its last successfully-reset connection stays parked idle -- expected
    // behavior given this slice has no idle reaper (see the TD Config
    // section), not a leak.
    assert_eq!(
        final_stats.backend_idle, 1,
        "exactly the transaction-mode client's last idle-parked connection should remain, got backend_idle={}",
        final_stats.backend_idle
    );

    stop_proxy(session_server, session_shutdown_tx).await;
    stop_proxy(txn_server, txn_shutdown_tx).await;
}

/// verify: pool_modes::stats_api_reports_expected_counts_at_each_phase (AC5)
#[tokio::test]
async fn stats_api_reports_expected_counts_at_each_phase() {
    let Some((backend_addr, user)) = real_backend_ready().await else {
        eprintln!(
            "skipping stats_api_reports_expected_counts_at_each_phase: \
             no reachable local Postgres at 127.0.0.1:5432 for user {:?}",
            backend_user()
        );
        return;
    };

    let backend_pool = BackendPool::new(pool_config(backend_addr, 4, Duration::from_secs(5)));
    let frontend_budget = ConnectionBudget::new(4);
    let (proxy_addr, server, shutdown_tx) =
        spawn_transaction_proxy(backend_pool.clone(), frontend_budget.clone()).await;
    let dsn = proxy_dsn(proxy_addr, &user);

    let snapshot = |pool: &BackendPool, budget: &ConnectionBudget| {
        let backend = pool.stats();
        PoolStats {
            frontend_active: budget.active(),
            backend_active: backend.backend_active,
            backend_idle: backend.backend_idle,
        }
    };

    // The proxy's post-relay reset-to-idle (or reset-and-release after a
    // transaction) runs asynchronously right after the client-visible
    // ReadyForQuery is forwarded, so there is a short window where the
    // client already sees success but the pool snapshot hasn't caught up
    // yet. Poll with a bounded retry instead of asserting immediately.
    async fn wait_for_stats(
        pool: &BackendPool,
        budget: &ConnectionBudget,
        snapshot: impl Fn(&BackendPool, &ConnectionBudget) -> PoolStats,
        expected: PoolStats,
        message: &str,
    ) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        loop {
            let observed = snapshot(pool, budget);
            if observed == expected {
                return;
            }
            if tokio::time::Instant::now() >= deadline {
                assert_eq!(observed, expected, "{message}");
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    // Phase 0: before any client connects.
    assert_eq!(
        snapshot(&backend_pool, &frontend_budget),
        PoolStats {
            frontend_active: 0,
            backend_active: 0,
            backend_idle: 0,
        },
        "phase 0: no client connected yet"
    );

    // Phase 1: after admission -- the handshake completed, the backend was
    // reset and returned to idle, and the client holds no transaction
    // lease yet (idle_no_lease).
    let (client, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .expect("client admits");
    let connection_task = tokio::spawn(connection);
    wait_for_stats(
        &backend_pool,
        &frontend_budget,
        snapshot,
        PoolStats {
            frontend_active: 1,
            backend_active: 0,
            backend_idle: 1,
        },
        "phase 1: admitted, backend idle-parked, no transaction lease held",
    )
    .await;

    // Phase 2: after a transaction lease is acquired -- a slow query is
    // in flight. Scoped so the pinned future (and its borrow of `client`)
    // is dropped before `drop(client)` below.
    {
        let query_fut = client.simple_query("SELECT pg_sleep(0.3)");
        tokio::pin!(query_fut);
        tokio::select! {
            _ = &mut query_fut => panic!("slow query resolved before phase 2 stats could be observed"),
            _ = tokio::time::sleep(Duration::from_millis(100)) => {}
        }
        assert_eq!(
            snapshot(&backend_pool, &frontend_budget),
            PoolStats {
                frontend_active: 1,
                backend_active: 1,
                backend_idle: 0,
            },
            "phase 2: transaction lease acquired for the in-flight query"
        );

        // Phase 3: after the transaction is released back to idle.
        query_fut.await.expect("slow query completes");
    }
    wait_for_stats(
        &backend_pool,
        &frontend_budget,
        snapshot,
        PoolStats {
            frontend_active: 1,
            backend_active: 0,
            backend_idle: 1,
        },
        "phase 3: transaction lease released back to idle",
    )
    .await;

    drop(client);
    connection_task
        .await
        .expect("connection task joined")
        .expect("connection driver ended cleanly");

    stop_proxy(server, shutdown_tx).await;
}
// </HANDWRITE>
// SPEC-MANAGED: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#unit-test
// CODEGEN-BEGIN

// CODEGEN-END
