// SPEC-MANAGED: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#unit-test
// <HANDWRITE gap="missing-generator:logic:pgpool-backend-pool" tracker="#1289" reason="Backend pool needs generator primitives that do not exist yet.">
//! Offline (no live Postgres) coverage for `BackendPool`/`BackendLease` and
//! transaction-mode's lease boundaries, one test function per TD Unit Test
//! requirement (R1-R5), each driven against a fake in-memory TCP backend
//! rather than a real Postgres instance.

use std::future::Future;
use std::time::Duration;

use bytes::BytesMut;
use server_lifecycle::ConnectionBudget;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

use pgpool::pool::{
    BackendPool, LeaseDisposition, PoolConfig, PoolError, PoolRejectionReason, PoolStats,
    ReserveLeasePolicy, ReserveLeaseRuntimeConfig,
};
use pgpool::proxy::BackendEndpointConfig;
use pgpool::wire::{
    AuthenticationOk, BackendMessage, CommandComplete, ErrorResponse, Frame, FrameReader,
    FrontendMessage, NoticeResponse, Query, ReadyForQuery, Role, StartupMessage, TransactionStatus,
    WireCodecConfig, WireMessage,
};

fn test_wire_config() -> WireCodecConfig {
    WireCodecConfig::default()
}

fn pool_config(backend_port: u16, max_backend_connections: usize) -> PoolConfig {
    PoolConfig {
        endpoint: BackendEndpointConfig {
            host: "127.0.0.1".to_string(),
            port: backend_port,
        },
        max_backend_connections,
        acquire_timeout: Duration::from_millis(200),
        backend_connect_timeout: Duration::from_millis(500),
        wire: test_wire_config(),
    }
}

/// Spawns a one-shot fake Postgres backend on an ephemeral loopback port,
/// running `script` against the single connection it accepts. Mirrors
/// `tests/proxy.rs`'s helper of the same name/shape.
async fn spawn_fake_backend<F, Fut>(script: F) -> (u16, tokio::task::JoinHandle<()>)
where
    F: FnOnce(TcpStream) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fake backend");
    let port = listener.local_addr().expect("fake backend addr").port();
    let handle = tokio::spawn(async move {
        let (stream, _) = listener
            .accept()
            .await
            .expect("accept fake backend connection");
        script(stream).await;
    });
    (port, handle)
}

async fn read_backend_frame(
    stream: &mut TcpStream,
    reader: &mut FrameReader,
) -> Option<BackendMessage> {
    loop {
        match reader.next_frame().expect("backend frame decodes") {
            Some(WireMessage::Backend(msg)) => return Some(msg),
            Some(WireMessage::Frontend(_)) => {
                unreachable!("backend-role reader only emits Backend frames")
            }
            None => {
                let mut buf = [0_u8; 4096];
                let n = stream.read(&mut buf).await.expect("read backend bytes");
                if n == 0 {
                    return None;
                }
                reader.feed(&buf[..n]);
            }
        }
    }
}

async fn write_backend(stream: &mut TcpStream, msg: &BackendMessage) {
    let mut buf = BytesMut::new();
    msg.encode(&mut buf);
    stream.write_all(&buf).await.expect("write backend frame");
}

/// Reads exactly one tagged `Query` frame directly off the raw stream,
/// bypassing `FrameReader`'s untagged-first-frame special case.
///
/// `FrameReader` with `Role::Frontend` assumes the very first frame it ever
/// decodes on a connection is an untagged StartupMessage (real Postgres wire
/// semantics: a fresh client connection always starts that way). These fake
/// backends skip the startup handshake entirely and go straight to answering
/// a `BackendPool::release()`-issued `DISCARD ALL` reset, so the incoming
/// `Query` frame genuinely is the first frame on the wire -- decoding it
/// through a fresh `Role::Frontend` `FrameReader` would misparse the tag byte
/// as part of the untagged length prefix. Manually reading the tag + declared
/// length + payload and going straight through `FrontendMessage::decode`
/// sidesteps that first-frame assumption, which in production never applies
/// here (a real startup always precedes any reset on the same connection).
async fn read_frontend_query(stream: &mut TcpStream) -> Query {
    let mut buf = BytesMut::new();
    let total = loop {
        if buf.len() >= 5 {
            let declared = i32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
            let total = 1 + declared;
            if buf.len() >= total {
                break total;
            }
        }
        let mut chunk = [0_u8; 4096];
        let n = stream.read(&mut chunk).await.expect("read query bytes");
        assert!(n > 0, "backend expected the DISCARD ALL query bytes");
        buf.extend_from_slice(&chunk[..n]);
    };
    let tag = buf[0];
    let frame_bytes = buf.split_to(total).freeze();
    assert_eq!(
        frame_bytes.as_ref(),
        b"Q\0\0\0\x10DISCARD ALL\0",
        "reset must emit the exact fixed PostgreSQL Query frame"
    );
    let payload = frame_bytes.slice(5..);
    let frame = Frame {
        tag: Some(tag),
        payload,
    };
    match FrontendMessage::decode(&frame, &test_wire_config()).expect("query frame decodes") {
        FrontendMessage::Query(query) => query,
        other => unreachable!("expected a Query frame, got {other:?}"),
    }
}

/// Runs a fake backend that accepts every connection in a loop and simply
/// holds each one open, reading and discarding any bytes until the peer
/// closes. Used by tests that release leases via `LeaseDisposition::Close`
/// (or drop a lease without releasing it at all) -- no `DISCARD ALL` reset is
/// ever sent on that path, so there's nothing to answer -- and by tests that
/// drive multiple `acquire_fresh()` cycles, each of which needs its own
/// freshly accepted connection.
async fn spawn_fake_backend_accept_and_hold() -> (u16, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fake backend");
    let port = listener.local_addr().expect("fake backend addr").port();
    let handle = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    tokio::spawn(async move {
                        let mut stream = stream;
                        let mut buf = [0_u8; 4096];
                        loop {
                            match stream.read(&mut buf).await {
                                Ok(0) | Err(_) => return,
                                Ok(_) => continue,
                            }
                        }
                    });
                }
                Err(_) => return,
            }
        }
    });
    (port, handle)
}

/// Runs a fake backend that answers exactly one `DISCARD ALL` reset with
/// `CommandComplete` + `ReadyForQuery(Idle)`, then holds the connection open
/// (so it can be reused as an idle connection by a later `acquire()`).
async fn spawn_reusable_fake_backend() -> (u16, tokio::task::JoinHandle<()>) {
    spawn_fake_backend(|stream| async move {
        let mut stream = stream;
        // Consume the DISCARD ALL Query frame.
        let _ = read_frontend_query(&mut stream).await;
        write_backend(
            &mut stream,
            &BackendMessage::CommandComplete(CommandComplete {
                tag: "DISCARD ALL".to_string(),
            }),
        )
        .await;
        write_backend(
            &mut stream,
            &BackendMessage::ReadyForQuery(ReadyForQuery {
                status: TransactionStatus::Idle,
            }),
        )
        .await;
        // Hold the socket open, idle, for reuse; drop only when the test
        // process tears everything down.
        tokio::time::sleep(Duration::from_secs(10)).await;
    })
    .await
}

// ---------------------------------------------------------------------
// R1: idle reuse, liveness check, reset-before-return-to-idle.
// ---------------------------------------------------------------------

/// Every physical backend leg uses the same low-latency TCP policy as the
/// accepted frontend server socket before it is exposed to a lease holder.
#[tokio::test]
async fn fresh_backend_lease_enables_tcp_nodelay() {
    let (port, _backend) = spawn_fake_backend_accept_and_hold().await;
    let pool = BackendPool::new(pool_config(port, 1));

    let lease = pool.acquire_fresh().await.expect("fresh acquire succeeds");
    assert!(
        lease.stream.nodelay().expect("read TCP_NODELAY option"),
        "new backend streams must disable Nagle coalescing before relay use"
    );
}

/// This fixture intentionally leaves no backend bytes readable after reset, so
/// it exercises the normal non-ready liveness path before the idle stream is
/// returned to the next lease holder.
///
/// verify: pool::acquire_reuses_idle_connection_after_liveness_check_passes (R1)
#[tokio::test]
async fn acquire_reuses_idle_connection_after_liveness_check_passes() {
    let (port, _backend) = spawn_reusable_fake_backend().await;
    let pool = BackendPool::new(pool_config(port, 4));

    let first = pool.acquire_fresh().await.expect("fresh acquire succeeds");
    assert!(first.fresh, "acquire_fresh() always yields a fresh lease");
    let first_id = first.id;
    pool.release(first_id, first.stream, LeaseDisposition::ReturnToIdle)
        .await;

    let stats = pool.stats();
    assert_eq!(
        stats.backend_idle, 1,
        "released connection parked in idle set"
    );

    let second = pool
        .acquire()
        .await
        .expect("acquire reuses idle connection");
    assert_eq!(
        second.id, first_id,
        "the same physical connection was reused"
    );
    assert!(
        !second.fresh,
        "a reused idle connection is not a fresh connect"
    );
    let stats = pool.stats();
    assert_eq!(stats.backend_active, 1);
    assert_eq!(stats.backend_idle, 0);
}

/// `MSG_PEEK` must leave readable backend bytes in the socket. The backend
/// sends one sentinel byte only after the reset has completed and the pool has
/// parked its stream idle; the next acquire liveness probe observes it without
/// consuming it, so the lease holder reads that exact byte afterwards.
///
/// verify: pool::acquire_liveness_peek_preserves_queued_backend_bytes (R3)
#[tokio::test]
async fn acquire_liveness_peek_preserves_queued_backend_bytes() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fake backend");
    let port = listener.local_addr().expect("fake backend addr").port();
    let (reset_complete_tx, reset_complete_rx) = oneshot::channel();
    let (send_sentinel_tx, send_sentinel_rx) = oneshot::channel();
    let (sentinel_written_tx, sentinel_written_rx) = oneshot::channel();
    let _backend = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept backend");
        let _ = read_frontend_query(&mut stream).await;
        write_backend(
            &mut stream,
            &BackendMessage::CommandComplete(CommandComplete {
                tag: "DISCARD ALL".to_string(),
            }),
        )
        .await;
        write_backend(
            &mut stream,
            &BackendMessage::ReadyForQuery(ReadyForQuery {
                status: TransactionStatus::Idle,
            }),
        )
        .await;
        let _ = reset_complete_tx.send(());
        let _ = send_sentinel_rx.await;
        stream
            .write_all(&[0xA5])
            .await
            .expect("write queued sentinel");
        let _ = sentinel_written_tx.send(());
        tokio::time::sleep(Duration::from_secs(10)).await;
    });

    let pool = BackendPool::new(pool_config(port, 1));
    let first = pool.acquire_fresh().await.expect("fresh acquire succeeds");
    pool.release(first.id, first.stream, LeaseDisposition::ReturnToIdle)
        .await;
    reset_complete_rx.await.expect("reset response completed");
    send_sentinel_tx
        .send(())
        .expect("backend awaits sentinel trigger");
    sentinel_written_rx.await.expect("sentinel was queued");

    let mut reused = pool.acquire().await.expect("idle backend is reused");
    assert!(!reused.fresh, "the queued-byte backend remains reusable");
    let mut sentinel = [0_u8; 1];
    reused
        .stream
        .read_exact(&mut sentinel)
        .await
        .expect("relay-side read sees byte preserved by liveness peek");
    assert_eq!(sentinel, [0xA5], "MSG_PEEK must not consume queued bytes");
}

/// A fake backend that accepts connections in a loop; the first connection
/// answers exactly one `DISCARD ALL` reset and then closes (so the pool
/// parks it as idle, but it goes dead shortly after — proving `acquire()`
/// must drop it rather than hand it back); every subsequent connection is
/// simply accepted and held open (so a fresh-connect retry has somewhere to
/// land).
async fn spawn_fake_backend_first_idle_conn_goes_dead() -> (u16, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fake backend");
    let port = listener.local_addr().expect("fake backend addr").port();
    let handle = tokio::spawn(async move {
        // First connection: answer the DISCARD ALL reset, then close.
        let (mut stream, _) = listener
            .accept()
            .await
            .expect("accept first fake backend connection");
        let _ = read_frontend_query(&mut stream).await;
        write_backend(
            &mut stream,
            &BackendMessage::CommandComplete(CommandComplete {
                tag: "DISCARD ALL".to_string(),
            }),
        )
        .await;
        write_backend(
            &mut stream,
            &BackendMessage::ReadyForQuery(ReadyForQuery {
                status: TransactionStatus::Idle,
            }),
        )
        .await;
        drop(stream); // Now dead: the peer observes EOF on its liveness peek.

        // Second (and any further) connection: accept and hold open so a
        // fresh-connect retry succeeds.
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    tokio::spawn(async move {
                        let mut stream = stream;
                        let mut buf = [0_u8; 4096];
                        loop {
                            match stream.read(&mut buf).await {
                                Ok(0) | Err(_) => return,
                                Ok(_) => continue,
                            }
                        }
                    });
                }
                Err(_) => return,
            }
        }
    });
    (port, handle)
}

/// verify: pool::acquire_drops_dead_idle_connection_and_retries (R1)
#[tokio::test]
async fn acquire_drops_dead_idle_connection_and_retries() {
    let (port, _backend) = spawn_fake_backend_first_idle_conn_goes_dead().await;
    let pool = BackendPool::new(pool_config(port, 1));

    let lease = pool.acquire_fresh().await.expect("fresh acquire succeeds");
    let dead_id = lease.id;
    pool.release(dead_id, lease.stream, LeaseDisposition::ReturnToIdle)
        .await;
    assert_eq!(pool.stats().backend_idle, 1, "reset succeeded, parked idle");

    // Give the backend a moment to actually close its end before the
    // liveness peek runs.
    tokio::time::sleep(Duration::from_millis(50)).await;

    let reused = pool
        .acquire()
        .await
        .expect("acquire() drops the dead idle entry and falls through to a fresh connect");
    assert_ne!(
        reused.id, dead_id,
        "the dead idle connection must never be handed back"
    );
    assert!(
        reused.fresh,
        "with the idle set now empty, acquire() must fresh-connect rather than block"
    );
    assert_eq!(
        pool.stats().backend_idle,
        0,
        "the dead idle entry was dropped, not left behind"
    );
}

/// verify: pool::release_return_to_idle_sends_discard_all_before_reuse (R1)
#[tokio::test]
async fn release_return_to_idle_sends_discard_all_before_reuse() {
    let (port, backend) = spawn_fake_backend(|stream| async move {
        let mut stream = stream;
        let query = read_frontend_query(&mut stream).await;
        assert_eq!(
            query.sql, "DISCARD ALL",
            "release() must reset via DISCARD ALL"
        );
        write_backend(
            &mut stream,
            &BackendMessage::CommandComplete(CommandComplete {
                tag: "DISCARD ALL".to_string(),
            }),
        )
        .await;
        write_backend(
            &mut stream,
            &BackendMessage::ReadyForQuery(ReadyForQuery {
                status: TransactionStatus::Idle,
            }),
        )
        .await;
    })
    .await;

    let pool = BackendPool::new(pool_config(port, 2));
    let lease = pool.acquire_fresh().await.expect("fresh acquire succeeds");
    pool.release(lease.id, lease.stream, LeaseDisposition::ReturnToIdle)
        .await;

    assert_eq!(
        pool.stats().backend_idle,
        1,
        "reset succeeded, connection parked idle"
    );
    backend.await.expect("fake backend task joins");
}

/// PostgreSQL may emit a notice before reset's expected completion and
/// readiness frames; that must not make an otherwise clean backend unusable.
///
/// verify: pool::release_return_to_idle_accepts_notice_before_ready (P0 #1716)
#[tokio::test]
async fn release_return_to_idle_accepts_notice_before_ready() {
    let (port, backend) = spawn_fake_backend(|stream| async move {
        let mut stream = stream;
        let query = read_frontend_query(&mut stream).await;
        assert_eq!(query.sql, "DISCARD ALL", "release must issue reset query");
        write_backend(
            &mut stream,
            &BackendMessage::NoticeResponse(NoticeResponse {
                fields: vec![
                    (b'S', "NOTICE".to_string()),
                    (b'M', "reset notice".to_string()),
                ],
            }),
        )
        .await;
        write_backend(
            &mut stream,
            &BackendMessage::CommandComplete(CommandComplete {
                tag: "DISCARD ALL".to_string(),
            }),
        )
        .await;
        write_backend(
            &mut stream,
            &BackendMessage::ReadyForQuery(ReadyForQuery {
                status: TransactionStatus::Idle,
            }),
        )
        .await;
    })
    .await;

    let pool = BackendPool::new(pool_config(port, 2));
    let lease = pool.acquire_fresh().await.expect("fresh acquire succeeds");
    pool.release(lease.id, lease.stream, LeaseDisposition::ReturnToIdle)
        .await;

    assert_eq!(
        pool.stats().backend_idle,
        1,
        "a validated reset notice must not prevent idle reuse"
    );
    backend.await.expect("fake backend task joins");
}

/// verify: pool::release_return_to_idle_closes_connection_when_reset_fails (R1)
#[tokio::test]
async fn release_return_to_idle_closes_connection_when_reset_fails() {
    let (port, backend) = spawn_fake_backend(|stream| async move {
        let mut stream = stream;
        let _ = read_frontend_query(&mut stream).await;
        // Respond with an ErrorResponse instead of CommandComplete +
        // ReadyForQuery: the reset itself fails.
        write_backend(
            &mut stream,
            &BackendMessage::ErrorResponse(ErrorResponse {
                fields: vec![
                    (b'S', "ERROR".to_string()),
                    (b'C', "58030".to_string()),
                    (b'M', "reset failed".to_string()),
                ],
            }),
        )
        .await;
    })
    .await;

    let pool = BackendPool::new(pool_config(port, 2));
    let lease = pool.acquire_fresh().await.expect("fresh acquire succeeds");
    pool.release(lease.id, lease.stream, LeaseDisposition::ReturnToIdle)
        .await;

    let stats = pool.stats();
    assert_eq!(
        stats.backend_idle, 0,
        "a failed reset must not be parked in idle"
    );
    assert_eq!(
        stats.backend_active, 0,
        "the capacity slot is freed instead"
    );
    backend.await.expect("fake backend task joins");
}

/// Cancelling one saturated reusable acquire must not lose capacity. The next
/// live waiter receives the reset-clean socket, and the sole permit remains
/// accounted for exactly once.
///
/// verify: pool::cancelled_waiter_allows_next_acquire_without_permit_leak (P0 #1691 no-go guard)
#[tokio::test]
async fn cancelled_waiter_allows_next_acquire_without_permit_leak() {
    let (port, _backend) = spawn_reusable_fake_backend().await;
    let pool = BackendPool::new(pool_config(port, 1));

    let first = pool.acquire_fresh().await.expect("fresh acquire succeeds");
    let first_id = first.id;

    let cancelled_pool = pool.clone();
    let cancelled = tokio::spawn(async move { cancelled_pool.acquire().await });
    tokio::task::yield_now().await;
    cancelled.abort();
    assert!(cancelled
        .await
        .expect_err("aborted waiter task")
        .is_cancelled());

    let next_pool = pool.clone();
    let next = tokio::spawn(async move { next_pool.acquire().await });
    tokio::task::yield_now().await;

    pool.release(first.id, first.stream, LeaseDisposition::ReturnToIdle)
        .await;

    let handed = next
        .await
        .expect("next waiter task joins")
        .expect("next reusable acquire succeeds");
    assert_eq!(
        handed.id, first_id,
        "cancelled waiter cannot lose the socket"
    );
    assert_eq!(pool.stats().backend_active, 1, "permit remains outstanding");
    assert_eq!(
        pool.stats().backend_idle,
        0,
        "no duplicate idle tuple is created"
    );
}

// ---------------------------------------------------------------------
// R2: transaction-mode lease boundaries vs. session-mode's whole-session lease.
// ---------------------------------------------------------------------

/// verify: pool::transaction_lease_acquired_on_first_frame_and_released_on_ready_for_query_idle (R2)
#[tokio::test]
async fn transaction_lease_acquired_on_first_frame_and_released_on_ready_for_query_idle() {
    // Directly exercises the pool primitives a `TransactionHandler` drives:
    // idle-no-lease (nothing acquired yet) -> acquire() on "first frame" ->
    // release(ReturnToIdle) once the backend reports ReadyForQuery(Idle).
    let (port, _backend) = spawn_reusable_fake_backend().await;
    let pool = BackendPool::new(pool_config(port, 2));

    // idle_no_lease: no lease held, nothing accounted for yet.
    assert_eq!(pool.stats().backend_active, 0);

    // "First frontend frame arrives" -> acquire_txn_backend.
    let lease = pool.acquire_fresh().await.expect("acquire for first frame");
    assert_eq!(
        pool.stats().backend_active,
        1,
        "lease now held (transaction_active)"
    );

    // Backend's ReadyForQuery reports Idle -> release(ReturnToIdle) -> back
    // to idle_no_lease.
    pool.release(lease.id, lease.stream, LeaseDisposition::ReturnToIdle)
        .await;
    let stats = pool.stats();
    assert_eq!(
        stats.backend_active, 0,
        "no lease held again (idle_no_lease)"
    );
    assert_eq!(
        stats.backend_idle, 1,
        "connection parked for the next transaction"
    );
}

/// verify: pool::session_mode_lease_held_for_whole_session_unchanged_from_1288 (R2)
#[tokio::test]
async fn session_mode_lease_held_for_whole_session_unchanged_from_1288() {
    use pgpool::proxy::{
        run_session, BackendEndpointConfig as SessionBackendEndpointConfig, SessionOutcome,
        SessionProxyConfig,
    };

    let (port, _backend) = spawn_fake_backend(|stream| async move {
        // Minimal backend: forward AuthenticationOk + ReadyForQuery, then
        // hold the connection so the session stays established until the
        // client terminates.
        let mut stream = stream;
        let mut reader = FrameReader::new(Role::Frontend, &test_wire_config());
        loop {
            match reader.next_frame().expect("frame decodes") {
                Some(WireMessage::Frontend(FrontendMessage::Startup(_))) => break,
                Some(_) => unreachable!("only Startup expected first"),
                None => {
                    let mut buf = [0_u8; 4096];
                    let n = stream.read(&mut buf).await.expect("read startup");
                    reader.feed(&buf[..n]);
                }
            }
        }
        write_backend(
            &mut stream,
            &BackendMessage::AuthenticationOk(pgpool::wire::AuthenticationOk),
        )
        .await;
        write_backend(
            &mut stream,
            &BackendMessage::ReadyForQuery(ReadyForQuery {
                status: TransactionStatus::Idle,
            }),
        )
        .await;
        // Hold until the client (session) terminates/EOFs.
        let mut buf = [0_u8; 4096];
        loop {
            match stream.read(&mut buf).await {
                Ok(0) | Err(_) => return,
                Ok(_) => continue,
            }
        }
    })
    .await;

    let backend_pool = BackendPool::new(pool_config(port, 2));
    let config = SessionProxyConfig {
        backend: SessionBackendEndpointConfig {
            host: "127.0.0.1".to_string(),
            port,
        },
        frontend_budget: ConnectionBudget::new(4),
        backend_connect_timeout: Duration::from_millis(500),
        drain_timeout: Duration::from_millis(500),
        wire: test_wire_config(),
        backend_pool: backend_pool.clone(),
    };

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind pair listener");
    let addr = listener.local_addr().expect("pair addr");
    let connect = tokio::spawn(TcpStream::connect(addr));
    let (proxy_side, _) = listener.accept().await.expect("accept pair connection");
    let mut client_side = connect.await.expect("connect joined").expect("connect");

    let session = tokio::spawn(async move { run_session(proxy_side, &config).await });

    // While the session is established, exactly one backend lease is held
    // (session mode's whole-session lease, R2b) -- give the handshake a
    // moment to complete.
    write_frontend_startup(&mut client_side).await;
    let mut probe_reader = FrameReader::new(Role::Backend, &test_wire_config());
    let mut probe_stream = client_side;
    // Read AuthenticationOk + ReadyForQuery off the client side to confirm
    // the handshake completed before asserting stats.
    let _ = read_backend_frame(&mut probe_stream, &mut probe_reader).await; // AuthenticationOk
    let _ = read_backend_frame(&mut probe_stream, &mut probe_reader).await; // ReadyForQuery

    assert_eq!(
        backend_pool.stats().backend_active,
        1,
        "exactly one lease held for the whole session"
    );

    drop(probe_stream);
    let outcome = session.await.expect("session task joins");
    assert!(
        matches!(
            outcome,
            SessionOutcome::EstablishedClosedClean | SessionOutcome::EstablishedClosedError
        ),
        "session established then closed (client dropped), got {outcome:?}"
    );
    assert_eq!(
        backend_pool.stats().backend_active,
        0,
        "lease released (Close) at session teardown"
    );
}

async fn write_frontend_startup(stream: &mut TcpStream) {
    let msg = FrontendMessage::Startup(pgpool::wire::StartupMessage {
        protocol_major: 3,
        protocol_minor: 0,
        parameters: vec![("user".to_string(), "postgres".to_string())],
    });
    let mut buf = BytesMut::new();
    msg.encode(&mut buf);
    stream.write_all(&buf).await.expect("write startup frame");
}

// ---------------------------------------------------------------------
// R3: bounded wait, saturation timeout, typed error mapping.
// ---------------------------------------------------------------------

/// verify: pool::acquire_waits_for_release_when_saturated_then_succeeds (R3)
#[tokio::test]
async fn acquire_waits_for_release_when_saturated_then_succeeds() {
    let (port, _backend) = spawn_fake_backend_accept_and_hold().await;
    let mut config = pool_config(port, 1);
    config.acquire_timeout = Duration::from_secs(5);
    let pool = BackendPool::new(config);

    let held = pool.acquire_fresh().await.expect("first acquire succeeds");

    let waiter_pool = pool.clone();
    let waiter = tokio::spawn(async move { waiter_pool.acquire().await });

    // Give the waiter a moment to actually start waiting, then free the slot.
    tokio::time::sleep(Duration::from_millis(100)).await;
    pool.release(held.id, held.stream, LeaseDisposition::Close)
        .await;

    let result = tokio::time::timeout(Duration::from_secs(2), waiter)
        .await
        .expect("waiter task completes")
        .expect("waiter task joins");
    assert!(
        result.is_ok(),
        "waiter acquires once the slot is freed: {result:?}"
    );
}

/// verify: pool::acquire_times_out_with_saturated_error_after_acquire_timeout (R3)
#[tokio::test]
async fn acquire_times_out_with_saturated_error_after_acquire_timeout() {
    let (port, _backend) = spawn_reusable_fake_backend().await;
    let mut config = pool_config(port, 1);
    config.acquire_timeout = Duration::from_millis(150);
    let pool = BackendPool::new(config);

    let held = pool.acquire_fresh().await.expect("first acquire succeeds");

    let start = std::time::Instant::now();
    let result = pool.acquire().await;
    let elapsed = start.elapsed();

    match result {
        Err(PoolError::Saturated { max, .. }) => assert_eq!(max, 1),
        other => panic!("expected PoolError::Saturated, got {other:?}"),
    }
    assert!(
        elapsed >= Duration::from_millis(140),
        "acquire() must wait roughly acquire_timeout before giving up, waited {elapsed:?}"
    );

    // Cleanup: release the held lease so the fake backend task can be
    // dropped without leaking a warning.
    pool.release(held.id, held.stream, LeaseDisposition::Close)
        .await;
}

/// Repeated non-matching startup-replay publications wake the existing
/// acquisition loop, but must not extend its original acquire deadline.
///
/// verify: pool::saturated_waiter_keeps_deadline_across_spurious_wakeups (P0 #1698)
#[tokio::test]
async fn saturated_waiter_keeps_deadline_across_spurious_wakeups() {
    let (port, _backend) = spawn_fake_backend_accept_and_hold().await;
    let mut config = pool_config(port, 1);
    config.acquire_timeout = Duration::from_millis(180);
    let pool = BackendPool::new(config);
    let held = pool.acquire_fresh().await.expect("first acquire succeeds");

    let waiting_startup = StartupMessage {
        protocol_major: 3,
        protocol_minor: 0,
        parameters: vec![("user".to_string(), "waiting".to_string())],
    };
    let waiter_pool = pool.clone();
    let started = std::time::Instant::now();
    let waiter =
        tokio::spawn(async move { waiter_pool.acquire_for_startup(&waiting_startup).await });

    // These entries cannot satisfy the waiting startup identity, but each
    // publishes the same Notify path that a backend return uses.
    for i in 0..3 {
        tokio::time::sleep(Duration::from_millis(25)).await;
        pool.publish_startup_replay(
            StartupMessage {
                protocol_major: 3,
                protocol_minor: 0,
                parameters: vec![("user".to_string(), format!("other-{i}"))],
            },
            vec![BackendMessage::AuthenticationOk(AuthenticationOk)],
        );
    }

    let result = waiter.await.expect("waiter task joins");
    let elapsed = started.elapsed();
    assert!(matches!(result, Err(PoolError::Saturated { max: 1, .. })));
    assert!(
        elapsed >= Duration::from_millis(165),
        "wakeups must not make the acquire deadline fire early: waited {elapsed:?}"
    );
    assert!(
        elapsed < Duration::from_millis(350),
        "wakeups must not extend the fixed acquire deadline: waited {elapsed:?}"
    );

    pool.release(held.id, held.stream, LeaseDisposition::Close)
        .await;
}

/// verify: pool::saturated_pool_error_maps_to_synthesized_error_response_53300 (R3)
#[tokio::test]
async fn saturated_pool_error_maps_to_synthesized_error_response_53300() {
    let reason = PoolRejectionReason::BackendPoolSaturated;
    let message = reason.synthesized_error_response();
    match message {
        BackendMessage::ErrorResponse(ErrorResponse { fields }) => {
            let sqlstate = fields
                .iter()
                .find(|(tag, _)| *tag == b'C')
                .map(|(_, value)| value.as_str());
            assert_eq!(sqlstate, Some("53300"));
        }
        other => panic!("expected ErrorResponse, got {other:?}"),
    }
}

// ---------------------------------------------------------------------
// R4: PoolStats composition.
// ---------------------------------------------------------------------

/// verify: pool::stats_snapshot_reports_frontend_backend_active_and_idle_counts (R4)
#[tokio::test]
async fn stats_snapshot_reports_frontend_backend_active_and_idle_counts() {
    let (port, _backend) = spawn_reusable_fake_backend().await;
    let pool = BackendPool::new(pool_config(port, 4));
    let frontend_budget = ConnectionBudget::new(8);

    let _permit = frontend_budget.try_acquire().expect("frontend permit");
    let lease = pool.acquire_fresh().await.expect("acquire succeeds");

    let backend_stats = pool.stats();
    let snapshot = PoolStats {
        frontend_active: frontend_budget.active(),
        backend_active: backend_stats.backend_active,
        backend_idle: backend_stats.backend_idle,
    };
    assert_eq!(snapshot.frontend_active, 1);
    assert_eq!(snapshot.backend_active, 1);
    assert_eq!(snapshot.backend_idle, 0);

    pool.release(lease.id, lease.stream, LeaseDisposition::ReturnToIdle)
        .await;
    let backend_stats = pool.stats();
    let snapshot = PoolStats {
        frontend_active: frontend_budget.active(),
        backend_active: backend_stats.backend_active,
        backend_idle: backend_stats.backend_idle,
    };
    assert_eq!(snapshot.backend_active, 0);
    assert_eq!(snapshot.backend_idle, 1);
}

// ---------------------------------------------------------------------
// R5: RAII capacity safety net.
// ---------------------------------------------------------------------

/// verify: pool::dropped_lease_without_explicit_release_does_not_leak_capacity_slot (R5)
#[tokio::test]
async fn dropped_lease_without_explicit_release_does_not_leak_capacity_slot() {
    let (port, _backend) = spawn_fake_backend_accept_and_hold().await;
    let pool = BackendPool::new(pool_config(port, 1));

    {
        let lease = pool.acquire_fresh().await.expect("acquire succeeds");
        assert_eq!(pool.stats().backend_active, 1);
        drop(lease); // No explicit release() call.
    }

    // Give the CapacityGuard's Drop a beat (it runs synchronously, but keep
    // this test robust to any future async cleanup).
    tokio::time::sleep(Duration::from_millis(20)).await;

    assert_eq!(
        pool.stats().backend_active,
        0,
        "dropping a lease without release() must still free its capacity slot"
    );

    // Prove the slot is genuinely usable again, not just zeroed in stats.
    let second = pool
        .acquire_fresh()
        .await
        .expect("capacity was actually freed");
    assert_eq!(pool.stats().backend_active, 1);
    drop(second);
}

/// verify: pool::reserve_admission_waits_before_opening_reserve_backend (R2)
#[tokio::test]
async fn reserve_admission_waits_before_opening_reserve_backend() {
    let mut config = pool_config(1, 0);
    config.acquire_timeout = Duration::from_millis(25);
    let pool = BackendPool::new_with_reserve(
        config,
        ReserveLeaseRuntimeConfig {
            endpoint: "primary".into(),
            pod: "pod-a".into(),
            policy: ReserveLeasePolicy {
                reserve_pool_timeout_seconds: 0,
                queue_wait_timeout_seconds: 0,
                reserve_idle_timeout_seconds: 1,
                lease_ttl_seconds: 10,
                request_chunk_size: 1,
            },
        },
    );
    assert!(matches!(
        pool.acquire().await,
        Err(PoolError::Saturated { .. })
    ));
    let stats = pool.stats();
    assert_eq!(stats.backend_active, 0);
    assert_eq!(stats.reserve_queued, 1);
    assert_eq!(
        stats.reserve_granted, 0,
        "a queued demand cannot create a physical backend without an allocator grant"
    );
}
// </HANDWRITE>
