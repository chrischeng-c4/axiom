// SPEC-MANAGED: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-session-proxy" tracker="#1288" reason="Session-mode proxy needs generator primitives that do not exist yet.">
//! End-to-end coverage against a real local Postgres (AC1-AC4). Every test
//! here graceful-skips (prints why, then returns) when the environment's
//! Postgres isn't reachable, per the repo's "real services over mocks, skip
//! gracefully" testing convention -- see `apps/pgpool/CLAUDE.md`/root
//! `CLAUDE.md` Testing section.

use std::net::SocketAddr;
use std::time::Duration;

use server_core::{BindConfig, ConnectionBudget};
use tokio::io::AsyncBufReadExt;

use pgpool::pool::{BackendPool, PoolConfig};
use pgpool::proxy::{BackendEndpointConfig, SessionHandler, SessionProxyConfig};
use pgpool::wire::WireCodecConfig;

/// Confirms the local Postgres is reachable, the current OS user can log in
/// via `trust` auth against the `postgres` database, and a trivial query
/// round-trips -- all in one probe, so every AC test below can graceful-skip
/// with a single check instead of failing when this exact local setup isn't
/// present (e.g. a future CI image without Homebrew Postgres running).
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
/// first row's `column`, parsed as `i32`.
///
/// Deliberately avoids `Client::query`/`query_one` here: those drive
/// tokio-postgres's default extended query protocol
/// (Parse/Bind/Describe/Execute/Sync), whose backend replies
/// (ParseComplete/BindComplete/ParameterDescription/NoData) are outside the
/// currently-implemented wire codec's message set -- a pre-existing gap in
/// the separate wire-codec TD (tracker #1287,
/// `apps/pgpool/src/wire/backend.rs`), which only decodes
/// Authentication*/ParameterStatus/BackendKeyData/ReadyForQuery/
/// RowDescription/DataRow/CommandComplete/ErrorResponse/NoticeResponse.
/// Since this session-proxy work item (#1288) treats any undecodable frame
/// as a fatal `FrameError` on the affected leg (by design -- see
/// `proxy::frame_error_on_either_leg_ends_session_without_forwarding_bad_bytes`
/// in `tests/proxy.rs`), a real extended-protocol query trips that same
/// path and the session closes before the row reaches the client. The
/// simple query protocol's responses (RowDescription/DataRow/
/// CommandComplete/ReadyForQuery) are all in the implemented set, so it
/// exercises a genuine round trip through the real proxy without depending
/// on a different module's unrelated gap -- confirmed manually too: `psql`
/// (which also speaks the simple query protocol) round-trips cleanly
/// through a manually-started `pgpool serve` against real Postgres.
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

fn session_proxy_config(backend_addr: SocketAddr, max_frontend: usize) -> SessionProxyConfig {
    let backend = BackendEndpointConfig {
        host: backend_addr.ip().to_string(),
        port: backend_addr.port(),
    };
    let backend_pool = BackendPool::new(PoolConfig {
        endpoint: backend.clone(),
        max_backend_connections: 64,
        acquire_timeout: Duration::from_secs(5),
        backend_connect_timeout: Duration::from_secs(5),
        wire: WireCodecConfig::default(),
    });
    SessionProxyConfig {
        backend,
        frontend_budget: ConnectionBudget::new(max_frontend),
        backend_connect_timeout: Duration::from_secs(5),
        drain_timeout: Duration::from_secs(5),
        wire: WireCodecConfig::default(),
        backend_pool,
    }
}

/// Starts an in-process session-mode proxy in front of `backend_addr`.
/// Returns the proxy's bound address, its server task, and a shutdown
/// trigger the caller must fire (and then await the task) to stop it.
async fn spawn_proxy(
    backend_addr: SocketAddr,
    max_frontend: usize,
) -> (
    SocketAddr,
    tokio::task::JoinHandle<()>,
    tokio::sync::oneshot::Sender<()>,
) {
    let handler = SessionHandler::new(session_proxy_config(backend_addr, max_frontend));
    let server_config = tcp_server::TcpServerConfig::new(BindConfig::localhost(0));
    let listener = tcp_server::bind(&server_config)
        .await
        .expect("bind proxy listener");
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

/// verify: session_proxy::real_postgres_session_connects_queries_and_disconnects_cleanly (AC1)
#[tokio::test]
async fn real_postgres_session_connects_queries_and_disconnects_cleanly() {
    let Some((backend_addr, user)) = real_backend_ready().await else {
        eprintln!(
            "skipping real_postgres_session_connects_queries_and_disconnects_cleanly: \
             no reachable local Postgres at 127.0.0.1:5432 for user {:?}",
            backend_user()
        );
        return;
    };

    let (proxy_addr, server, shutdown_tx) = spawn_proxy(backend_addr, 10).await;

    let dsn = proxy_dsn(proxy_addr, &user);
    let (client, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .expect("connect through pgpool to real Postgres");
    let connection_task = tokio::spawn(connection);

    let value = simple_query_i32(&client, "SELECT 1 AS one", "one").await;
    assert_eq!(value, 1);

    // Clean disconnect: dropping the client makes tokio-postgres send
    // Terminate and close, which the proxy relays to the real backend
    // (EstablishedClosedClean).
    drop(client);
    connection_task
        .await
        .expect("connection task joined")
        .expect("connection driver ended cleanly");

    let _ = shutdown_tx.send(());
    server.await.expect("proxy server task");
}

/// verify: session_proxy::real_postgres_scram_auth_succeeds_without_credential_persistence (AC2)
///
/// Genuinely impractical to prove against a *live* SCRAM handshake in this
/// sandbox: the local Homebrew Postgres instance backing AC1/AC3/AC4 is
/// configured for `trust` auth (no password), and standing up a second
/// SCRAM-SHA-256-authenticated role/database -- or rewriting the shared
/// local instance's `pg_hba.conf` -- is out of scope for this crate's test
/// suite and would have side effects beyond it. Per the task's explicit
/// instruction, this is documented as a skip rather than faked as a pass.
///
/// What *is* covered, offline, against a fake in-memory backend (no
/// fabrication -- these assert real relayed bytes):
/// - `proxy::auth_frames_relayed_verbatim_for_cleartext_md5_and_scram`
///   (`apps/pgpool/tests/proxy.rs`) drives a full
///   AuthenticationSasl -> SaslInitialResponse -> AuthenticationSaslContinue
///   -> SaslResponse -> AuthenticationSaslFinal -> AuthenticationOk exchange
///   and asserts every relayed frame's payload is byte-identical to what was
///   sent, in both directions.
/// - `proxy::password_and_sasl_payload_bytes_are_never_retained` proves two
///   independent password-auth sessions sharing one proxy never observe or
///   leak each other's credential bytes (nothing is cached across
///   sessions).
///
/// This test becomes a real end-to-end SCRAM check the moment a
/// SCRAM-SHA-256-authenticated Postgres connection string is available via
/// `PGPOOL_TEST_SCRAM_DSN` (any libpq-style DSN or URI `tokio_postgres`
/// accepts); until then it graceful-skips.
#[tokio::test]
async fn real_postgres_scram_auth_succeeds_without_credential_persistence() {
    let Some(dsn) = std::env::var("PGPOOL_TEST_SCRAM_DSN")
        .ok()
        .filter(|v| !v.is_empty())
    else {
        eprintln!(
            "skipping real_postgres_scram_auth_succeeds_without_credential_persistence: \
             no PGPOOL_TEST_SCRAM_DSN configured -- see this test's doc comment for why a \
             real SCRAM-authenticated Postgres isn't stood up automatically here, and for the \
             offline wire-level SCRAM coverage in tests/proxy.rs that this test complements."
        );
        return;
    };

    let mut backend_config: tokio_postgres::Config = dsn
        .parse()
        .expect("PGPOOL_TEST_SCRAM_DSN parses as a tokio-postgres config/URI");
    let host = match backend_config.get_hosts().first() {
        Some(tokio_postgres::config::Host::Tcp(host)) => host.clone(),
        _ => panic!("PGPOOL_TEST_SCRAM_DSN must name a TCP host"),
    };
    let port = *backend_config.get_ports().first().unwrap_or(&5432);
    let backend_addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("PGPOOL_TEST_SCRAM_DSN host:port parses as a socket address");

    let (proxy_addr, server, shutdown_tx) = spawn_proxy(backend_addr, 10).await;

    backend_config.host(proxy_addr.ip().to_string());
    backend_config.port(proxy_addr.port());

    let (client, connection) = backend_config
        .connect(tokio_postgres::NoTls)
        .await
        .expect("SCRAM handshake succeeds through pgpool");
    let connection_task = tokio::spawn(connection);

    let value = simple_query_i32(&client, "SELECT 1 AS one", "one").await;
    assert_eq!(value, 1);

    drop(client);
    connection_task
        .await
        .expect("connection task joined")
        .expect("connection driver ended cleanly");

    let _ = shutdown_tx.send(());
    server.await.expect("proxy server task");
}

/// verify: session_proxy::budget_rejection_does_not_disrupt_existing_sessions (AC3)
#[tokio::test]
async fn budget_rejection_does_not_disrupt_existing_sessions() {
    let Some((backend_addr, user)) = real_backend_ready().await else {
        eprintln!(
            "skipping budget_rejection_does_not_disrupt_existing_sessions: \
             no reachable local Postgres at 127.0.0.1:5432 for user {:?}",
            backend_user()
        );
        return;
    };

    // Budget of 1: the first session holds the only permit.
    let (proxy_addr, server, shutdown_tx) = spawn_proxy(backend_addr, 1).await;
    let dsn = proxy_dsn(proxy_addr, &user);

    let (client, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .expect("first session connects");
    let connection_task = tokio::spawn(connection);

    // A second session attempted while the budget is exhausted must be
    // rejected (wire-level ErrorResponse, SQLSTATE 53300) without touching
    // the first session at all.
    let rejected = tokio_postgres::connect(&dsn, tokio_postgres::NoTls).await;
    assert!(
        rejected.is_err(),
        "second session must be rejected while the frontend budget is exhausted"
    );

    // The first session must still be fully usable after the rejection.
    let value = simple_query_i32(&client, "SELECT 1 AS one", "one").await;
    assert_eq!(value, 1);

    drop(client);
    connection_task
        .await
        .expect("connection task joined")
        .expect("connection driver ended cleanly");

    let _ = shutdown_tx.send(());
    server.await.expect("proxy server task");
}

/// verify: session_proxy::sigterm_drains_in_flight_session_before_exit (AC4)
#[tokio::test]
async fn sigterm_drains_in_flight_session_before_exit() {
    let Some((backend_addr, user)) = real_backend_ready().await else {
        eprintln!(
            "skipping sigterm_drains_in_flight_session_before_exit: \
             no reachable local Postgres at 127.0.0.1:5432 for user {:?}",
            backend_user()
        );
        return;
    };

    let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_pgpool"))
        .arg("serve")
        .args(["--backend-host", &backend_addr.ip().to_string()])
        .args(["--backend-port", &backend_addr.port().to_string()])
        .args(["--bind", "127.0.0.1:0"])
        .args(["--drain-timeout-ms", "3000"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("spawn `pgpool serve` subprocess");

    let stdout = child.stdout.take().expect("child stdout piped");
    let mut lines = tokio::io::BufReader::new(stdout).lines();
    let listening_line = tokio::time::timeout(Duration::from_secs(5), lines.next_line())
        .await
        .expect("pgpool serve prints its listening line before timeout")
        .expect("read listening line")
        .expect("listening line present");
    let addr_str = listening_line
        .trim()
        .rsplit(' ')
        .next()
        .expect("listening line ends with an address");
    let proxy_addr: SocketAddr = addr_str
        .parse()
        .expect("parse pgpool serve's bound address");

    let dsn = proxy_dsn(proxy_addr, &user);
    let (client, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .expect("connect to the spawned pgpool serve process");
    let connection_task = tokio::spawn(connection);

    // Confirm the session is genuinely established before signaling.
    let value = simple_query_i32(&client, "SELECT 1 AS one", "one").await;
    assert_eq!(value, 1);

    let pid = child.id().expect("child process has a pid");
    let kill_status = tokio::process::Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()
        .await
        .expect("run `kill -TERM` on the pgpool serve process");
    assert!(
        kill_status.success(),
        "kill -TERM must be delivered successfully"
    );

    // The in-flight session must still be usable during the drain grace
    // window that follows SIGTERM (the accept loop stops, but this session
    // is left alone until it finishes or the drain timeout elapses).
    let value = simple_query_i32(&client, "SELECT 2 AS two", "two").await;
    assert_eq!(value, 2);

    drop(client);
    connection_task
        .await
        .expect("connection task joined")
        .expect("connection driver ended cleanly");

    let status = tokio::time::timeout(Duration::from_secs(5), child.wait())
        .await
        .expect("pgpool serve exits within the drain timeout after the session finishes")
        .expect("wait on pgpool serve process");
    assert!(
        status.success(),
        "pgpool serve must exit cleanly after draining, got {status:?}"
    );
}
// </HANDWRITE>
