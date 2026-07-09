// SPEC-MANAGED: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#unit-test
// <HANDWRITE gap="missing-generator:logic:pgpool-session-proxy" tracker="#1288" reason="Session-mode proxy needs generator primitives that do not exist yet.">
//! Offline (no live Postgres) coverage for the session-mode proxy, one test
//! function per TD Unit Test requirement (R1-R4), each driven against a
//! fake in-memory TCP backend rather than a real Postgres instance.

use std::future::Future;
use std::time::Duration;

use bytes::{Bytes, BytesMut};
use server_core::ConnectionBudget;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use pgpool::pool::{BackendPool, PoolConfig};
use pgpool::proxy::{
    run_session, BackendEndpointConfig, SessionHandler, SessionOutcome, SessionProxyConfig,
};
use pgpool::wire::{
    AuthenticationCleartextPassword, AuthenticationMd5Password, AuthenticationOk,
    AuthenticationSasl, AuthenticationSaslContinue, AuthenticationSaslFinal, BackendMessage,
    CommandComplete, DataRow, ErrorResponse, FieldDescription, FrameReader, FrontendMessage,
    PasswordMessage, Query, ReadyForQuery, Role, RowDescription, SaslInitialResponse, SaslResponse,
    StartupMessage, Terminate, TransactionStatus, WireCodecConfig, WireMessage,
};

fn test_wire_config() -> WireCodecConfig {
    WireCodecConfig::default()
}

fn test_config(backend_port: u16, max_frontend: usize) -> SessionProxyConfig {
    let backend = BackendEndpointConfig {
        host: "127.0.0.1".to_string(),
        port: backend_port,
    };
    // A generous fixed backend-pool capacity: these session-mode tests
    // exercise the session pipeline/frontend admission, not backend-pool
    // saturation (that lives in `tests/pool.rs`), so the pool must never be
    // the bottleneck here.
    let backend_pool = BackendPool::new(PoolConfig {
        endpoint: backend.clone(),
        max_backend_connections: 64,
        acquire_timeout: Duration::from_millis(500),
        backend_connect_timeout: Duration::from_millis(500),
        wire: test_wire_config(),
    });
    SessionProxyConfig {
        backend,
        frontend_budget: ConnectionBudget::new(max_frontend),
        backend_connect_timeout: Duration::from_millis(500),
        drain_timeout: Duration::from_millis(500),
        wire: test_wire_config(),
        backend_pool,
    }
}

async fn write_frontend(stream: &mut TcpStream, msg: &FrontendMessage) {
    let mut buf = BytesMut::new();
    msg.encode(&mut buf);
    stream.write_all(&buf).await.expect("write frontend frame");
}

async fn write_backend(stream: &mut TcpStream, msg: &BackendMessage) {
    let mut buf = BytesMut::new();
    msg.encode(&mut buf);
    stream.write_all(&buf).await.expect("write backend frame");
}

async fn read_frontend_frame(
    stream: &mut TcpStream,
    reader: &mut FrameReader,
) -> Option<FrontendMessage> {
    loop {
        match reader.next_frame().expect("frontend frame decodes") {
            Some(WireMessage::Frontend(msg)) => return Some(msg),
            Some(WireMessage::Backend(_)) => {
                unreachable!("frontend-role reader only emits Frontend frames")
            }
            None => {
                let mut buf = [0_u8; 4096];
                let n = stream.read(&mut buf).await.expect("read frontend bytes");
                if n == 0 {
                    return None;
                }
                reader.feed(&buf[..n]);
            }
        }
    }
}

/// Like [`read_frontend_frame`], but also accumulates every raw byte fed to
/// `reader` into `raw` while decoding the next frame. Only meaningful when
/// called as the very first read on a fresh connection (before any other
/// frame's bytes have been fed), so `raw` ends up holding exactly that one
/// frame's on-wire bytes for byte-identity comparisons.
async fn read_frontend_frame_capturing_raw(
    stream: &mut TcpStream,
    reader: &mut FrameReader,
    raw: &mut Vec<u8>,
) -> Option<FrontendMessage> {
    loop {
        match reader.next_frame().expect("frontend frame decodes") {
            Some(WireMessage::Frontend(msg)) => return Some(msg),
            Some(WireMessage::Backend(_)) => {
                unreachable!("frontend-role reader only emits Frontend frames")
            }
            None => {
                let mut buf = [0_u8; 4096];
                let n = stream.read(&mut buf).await.expect("read frontend bytes");
                if n == 0 {
                    return None;
                }
                raw.extend_from_slice(&buf[..n]);
                reader.feed(&buf[..n]);
            }
        }
    }
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

/// Extracts a tagged frame's raw payload bytes (skips the 1-byte tag + i32
/// length header) so SASL 'p'-tag variants can be compared against what a
/// generic `PasswordMessage` decode sees, mirroring `FrontendMessage::decode`'s
/// documented tag-'p' dispatch (see `tests/wire_codec.rs`).
fn tagged_payload(encode: impl FnOnce(&mut BytesMut)) -> Bytes {
    let mut buf = BytesMut::new();
    encode(&mut buf);
    Bytes::copy_from_slice(&buf[5..])
}

/// Spawns a one-shot fake Postgres backend on an ephemeral loopback port,
/// running `script` against the single connection it accepts.
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

/// Returns a connected `(proxy_side, client_side)` pair: `proxy_side` is fed
/// into `run_session` as the accepted client stream; `client_side` is the
/// test-driven peer that plays the real Postgres client's role.
async fn connected_pair() -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind pair listener");
    let addr = listener.local_addr().expect("pair addr");
    let connect = tokio::spawn(TcpStream::connect(addr));
    let (proxy_side, _) = listener.accept().await.expect("accept pair connection");
    let client_side = connect
        .await
        .expect("connect task joined")
        .expect("connect");
    (proxy_side, client_side)
}

fn startup_message() -> StartupMessage {
    StartupMessage {
        protocol_major: 3,
        protocol_minor: 0,
        parameters: vec![("user".to_string(), "postgres".to_string())],
    }
}

// ---------------------------------------------------------------------
// R1: admission budget enforcement and permit release.
// ---------------------------------------------------------------------

/// verify: proxy::permit_released_on_every_session_exit_path (R1, regression)
#[tokio::test]
async fn permit_released_on_every_session_exit_path() {
    let budget = ConnectionBudget::new(1);

    // Path 1: backend unreachable -> permit acquired then released.
    let dead_listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind dead listener");
    let dead_port = dead_listener.local_addr().unwrap().port();
    drop(dead_listener);

    let mut config = test_config(dead_port, 1);
    config.frontend_budget = budget.clone();
    let (proxy_side, _client_side) = connected_pair().await;
    let outcome = run_session(proxy_side, &config).await;
    assert_eq!(outcome, SessionOutcome::RejectedBackendUnreachable);
    assert_eq!(
        budget.active(),
        0,
        "permit released after backend-unreachable rejection"
    );

    // Path 2: clean established session -> permit acquired then released.
    let (backend_port, backend_task) = spawn_fake_backend(|mut stream: TcpStream| async move {
        let wire = test_wire_config();
        let mut reader = FrameReader::new(Role::Frontend, &wire);
        assert!(matches!(
            read_frontend_frame(&mut stream, &mut reader).await,
            Some(FrontendMessage::Startup(_))
        ));
        write_backend(
            &mut stream,
            &BackendMessage::AuthenticationOk(AuthenticationOk),
        )
        .await;
        write_backend(
            &mut stream,
            &BackendMessage::ReadyForQuery(ReadyForQuery {
                status: TransactionStatus::Idle,
            }),
        )
        .await;
        loop {
            match read_frontend_frame(&mut stream, &mut reader).await {
                Some(FrontendMessage::Terminate(_)) | None => break,
                Some(_) => continue,
            }
        }
    })
    .await;

    let mut config = test_config(backend_port, 1);
    config.frontend_budget = budget.clone();
    let (proxy_side, mut client_side) = connected_pair().await;
    let session = tokio::spawn(async move { run_session(proxy_side, &config).await });

    write_frontend(
        &mut client_side,
        &FrontendMessage::Startup(startup_message()),
    )
    .await;
    let wire = test_wire_config();
    let mut reader = FrameReader::new(Role::Backend, &wire);
    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::AuthenticationOk(_))
    ));
    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::ReadyForQuery(_))
    ));
    write_frontend(&mut client_side, &FrontendMessage::Terminate(Terminate)).await;

    let outcome = session.await.expect("session task");
    assert_eq!(outcome, SessionOutcome::EstablishedClosedClean);
    assert_eq!(
        budget.active(),
        0,
        "permit released after clean established close"
    );
    backend_task.await.expect("backend task");
}

/// verify: proxy::rejects_new_session_with_error_response_when_budget_exhausted (R1, functional)
#[tokio::test]
async fn rejects_new_session_with_error_response_when_budget_exhausted() {
    let budget = ConnectionBudget::new(1);
    let _held_permit = budget.try_acquire().expect("pre-acquire the only permit");

    let mut config = test_config(0, 1);
    config.frontend_budget = budget;

    let (proxy_side, mut client_side) = connected_pair().await;
    let outcome = run_session(proxy_side, &config).await;
    assert_eq!(outcome, SessionOutcome::RejectedSaturated);

    let mut buf = Vec::new();
    client_side
        .read_to_end(&mut buf)
        .await
        .expect("read rejection bytes");
    let mut reader = FrameReader::new(Role::Backend, &test_wire_config());
    reader.feed(&buf);
    match reader
        .next_frame()
        .expect("frame decodes")
        .expect("frame present")
    {
        WireMessage::Backend(BackendMessage::ErrorResponse(err)) => {
            assert!(
                err.fields
                    .iter()
                    .any(|(code, value)| *code == b'C' && value == "53300"),
                "expected SQLSTATE 53300, got {:?}",
                err.fields
            );
        }
        other => panic!("expected ErrorResponse, got {other:?}"),
    }
}

// ---------------------------------------------------------------------
// R2: auth passthrough, bidirectional relay, non-retention, frame errors.
// ---------------------------------------------------------------------

async fn assert_password_challenge_relayed_verbatim(
    challenge: BackendMessage,
    client_payload: Bytes,
) {
    let (backend_port, backend_task) = spawn_fake_backend({
        let challenge = challenge.clone();
        let client_payload = client_payload.clone();
        move |mut stream: TcpStream| async move {
            let wire = test_wire_config();
            let mut reader = FrameReader::new(Role::Frontend, &wire);
            assert!(matches!(
                read_frontend_frame(&mut stream, &mut reader).await,
                Some(FrontendMessage::Startup(_))
            ));
            write_backend(&mut stream, &challenge).await;
            match read_frontend_frame(&mut stream, &mut reader).await {
                Some(FrontendMessage::Password(p)) => assert_eq!(p.payload, client_payload),
                other => panic!("expected relayed password reply, got {other:?}"),
            }
            write_backend(
                &mut stream,
                &BackendMessage::AuthenticationOk(AuthenticationOk),
            )
            .await;
            write_backend(
                &mut stream,
                &BackendMessage::ReadyForQuery(ReadyForQuery {
                    status: TransactionStatus::Idle,
                }),
            )
            .await;
            loop {
                match read_frontend_frame(&mut stream, &mut reader).await {
                    Some(FrontendMessage::Terminate(_)) | None => break,
                    Some(_) => continue,
                }
            }
        }
    })
    .await;

    let config = test_config(backend_port, 10);
    let (proxy_side, mut client_side) = connected_pair().await;
    let session = tokio::spawn(async move { run_session(proxy_side, &config).await });

    write_frontend(
        &mut client_side,
        &FrontendMessage::Startup(startup_message()),
    )
    .await;
    let wire = test_wire_config();
    let mut reader = FrameReader::new(Role::Backend, &wire);
    match read_backend_frame(&mut client_side, &mut reader).await {
        Some(msg) => assert_eq!(msg, challenge),
        None => panic!("expected auth challenge relayed to client"),
    }
    write_frontend(
        &mut client_side,
        &FrontendMessage::Password(PasswordMessage {
            payload: client_payload,
        }),
    )
    .await;

    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::AuthenticationOk(_))
    ));
    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::ReadyForQuery(_))
    ));

    write_frontend(&mut client_side, &FrontendMessage::Terminate(Terminate)).await;
    let outcome = session.await.expect("session task");
    assert_eq!(outcome, SessionOutcome::EstablishedClosedClean);
    backend_task.await.expect("backend task");
}

async fn assert_scram_challenge_relayed_verbatim() {
    let sasl_initial_response = Bytes::from_static(b"clientnonce-payload");
    let server_first = Bytes::from_static(b"r=clientnonce+servernonce,s=salt,i=4096");
    let sasl_response_payload =
        Bytes::from_static(b"c=biws,r=clientnonce+servernonce,p=clientproof");
    let server_final = Bytes::from_static(b"v=serversignature");

    let expected_initial_payload = tagged_payload(|buf| {
        SaslInitialResponse {
            mechanism: "SCRAM-SHA-256".to_string(),
            response: Some(sasl_initial_response.clone()),
        }
        .encode(buf)
    });
    let expected_response_payload = tagged_payload(|buf| {
        SaslResponse {
            payload: sasl_response_payload.clone(),
        }
        .encode(buf)
    });

    let (backend_port, backend_task) = spawn_fake_backend({
        let server_first = server_first.clone();
        let server_final = server_final.clone();
        let expected_initial_payload = expected_initial_payload.clone();
        let expected_response_payload = expected_response_payload.clone();
        move |mut stream: TcpStream| async move {
            let wire = test_wire_config();
            let mut reader = FrameReader::new(Role::Frontend, &wire);
            assert!(matches!(
                read_frontend_frame(&mut stream, &mut reader).await,
                Some(FrontendMessage::Startup(_))
            ));

            write_backend(
                &mut stream,
                &BackendMessage::AuthenticationSasl(AuthenticationSasl {
                    mechanisms: vec!["SCRAM-SHA-256".to_string()],
                }),
            )
            .await;

            match read_frontend_frame(&mut stream, &mut reader).await {
                Some(FrontendMessage::Password(p)) => {
                    assert_eq!(p.payload, expected_initial_payload)
                }
                other => panic!("expected relayed SASL initial response, got {other:?}"),
            }

            write_backend(
                &mut stream,
                &BackendMessage::AuthenticationSaslContinue(AuthenticationSaslContinue {
                    payload: server_first,
                }),
            )
            .await;

            match read_frontend_frame(&mut stream, &mut reader).await {
                Some(FrontendMessage::Password(p)) => {
                    assert_eq!(p.payload, expected_response_payload)
                }
                other => panic!("expected relayed SASL response, got {other:?}"),
            }

            write_backend(
                &mut stream,
                &BackendMessage::AuthenticationSaslFinal(AuthenticationSaslFinal {
                    payload: server_final,
                }),
            )
            .await;
            write_backend(
                &mut stream,
                &BackendMessage::AuthenticationOk(AuthenticationOk),
            )
            .await;
            write_backend(
                &mut stream,
                &BackendMessage::ReadyForQuery(ReadyForQuery {
                    status: TransactionStatus::Idle,
                }),
            )
            .await;

            loop {
                match read_frontend_frame(&mut stream, &mut reader).await {
                    Some(FrontendMessage::Terminate(_)) | None => break,
                    Some(_) => continue,
                }
            }
        }
    })
    .await;

    let config = test_config(backend_port, 10);
    let (proxy_side, mut client_side) = connected_pair().await;
    let session = tokio::spawn(async move { run_session(proxy_side, &config).await });

    write_frontend(
        &mut client_side,
        &FrontendMessage::Startup(startup_message()),
    )
    .await;
    let wire = test_wire_config();
    let mut reader = FrameReader::new(Role::Backend, &wire);

    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::AuthenticationSasl(_))
    ));
    write_frontend(
        &mut client_side,
        &FrontendMessage::SaslInitialResponse(SaslInitialResponse {
            mechanism: "SCRAM-SHA-256".to_string(),
            response: Some(sasl_initial_response),
        }),
    )
    .await;

    match read_backend_frame(&mut client_side, &mut reader).await {
        Some(BackendMessage::AuthenticationSaslContinue(c)) => assert_eq!(c.payload, server_first),
        other => panic!("expected AuthenticationSaslContinue relayed to client, got {other:?}"),
    }
    write_frontend(
        &mut client_side,
        &FrontendMessage::SaslResponse(SaslResponse {
            payload: sasl_response_payload,
        }),
    )
    .await;

    match read_backend_frame(&mut client_side, &mut reader).await {
        Some(BackendMessage::AuthenticationSaslFinal(f)) => assert_eq!(f.payload, server_final),
        other => panic!("expected AuthenticationSaslFinal relayed to client, got {other:?}"),
    }
    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::AuthenticationOk(_))
    ));
    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::ReadyForQuery(_))
    ));

    write_frontend(&mut client_side, &FrontendMessage::Terminate(Terminate)).await;
    let outcome = session.await.expect("session task");
    assert_eq!(outcome, SessionOutcome::EstablishedClosedClean);
    backend_task.await.expect("backend task");
}

/// verify: proxy::auth_frames_relayed_verbatim_for_cleartext_md5_and_scram (R2, functional)
#[tokio::test]
async fn auth_frames_relayed_verbatim_for_cleartext_md5_and_scram() {
    assert_password_challenge_relayed_verbatim(
        BackendMessage::AuthenticationCleartextPassword(AuthenticationCleartextPassword),
        Bytes::from_static(b"trustno1\0"),
    )
    .await;

    assert_password_challenge_relayed_verbatim(
        BackendMessage::AuthenticationMd5Password(AuthenticationMd5Password { salt: [1, 2, 3, 4] }),
        Bytes::from_static(b"md5deadbeef00112233445566778899\0"),
    )
    .await;

    assert_scram_challenge_relayed_verbatim().await;
}

/// verify: proxy::bidirectional_relay_forwards_frames_until_client_terminate (R2, functional)
#[tokio::test]
async fn bidirectional_relay_forwards_frames_until_client_terminate() {
    let (backend_port, backend_task) = spawn_fake_backend(|mut stream: TcpStream| async move {
        let wire = test_wire_config();
        let mut reader = FrameReader::new(Role::Frontend, &wire);
        assert!(matches!(
            read_frontend_frame(&mut stream, &mut reader).await,
            Some(FrontendMessage::Startup(_))
        ));
        write_backend(
            &mut stream,
            &BackendMessage::AuthenticationOk(AuthenticationOk),
        )
        .await;
        write_backend(
            &mut stream,
            &BackendMessage::ReadyForQuery(ReadyForQuery {
                status: TransactionStatus::Idle,
            }),
        )
        .await;

        match read_frontend_frame(&mut stream, &mut reader).await {
            Some(FrontendMessage::Query(q)) => assert_eq!(q.sql, "SELECT 1"),
            other => panic!("expected relayed Query, got {other:?}"),
        }
        write_backend(
            &mut stream,
            &BackendMessage::RowDescription(RowDescription {
                fields: vec![FieldDescription {
                    name: "one".to_string(),
                    table_oid: 0,
                    column_attr: 0,
                    type_oid: 23,
                    type_size: 4,
                    type_modifier: -1,
                    format: 0,
                }],
            }),
        )
        .await;
        write_backend(
            &mut stream,
            &BackendMessage::DataRow(DataRow {
                columns: vec![Some(Bytes::from_static(b"1"))],
            }),
        )
        .await;
        write_backend(
            &mut stream,
            &BackendMessage::CommandComplete(CommandComplete {
                tag: "SELECT 1".to_string(),
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

        loop {
            match read_frontend_frame(&mut stream, &mut reader).await {
                Some(FrontendMessage::Terminate(_)) | None => break,
                Some(_) => continue,
            }
        }
    })
    .await;

    let config = test_config(backend_port, 10);
    let (proxy_side, mut client_side) = connected_pair().await;
    let session = tokio::spawn(async move { run_session(proxy_side, &config).await });

    write_frontend(
        &mut client_side,
        &FrontendMessage::Startup(startup_message()),
    )
    .await;
    let wire = test_wire_config();
    let mut reader = FrameReader::new(Role::Backend, &wire);
    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::AuthenticationOk(_))
    ));
    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::ReadyForQuery(_))
    ));

    write_frontend(
        &mut client_side,
        &FrontendMessage::Query(Query {
            sql: "SELECT 1".to_string(),
        }),
    )
    .await;

    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::RowDescription(_))
    ));
    match read_backend_frame(&mut client_side, &mut reader).await {
        Some(BackendMessage::DataRow(row)) => {
            assert_eq!(row.columns, vec![Some(Bytes::from_static(b"1"))])
        }
        other => panic!("expected relayed DataRow, got {other:?}"),
    }
    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::CommandComplete(_))
    ));
    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::ReadyForQuery(_))
    ));

    write_frontend(&mut client_side, &FrontendMessage::Terminate(Terminate)).await;
    let outcome = session.await.expect("session task");
    assert_eq!(outcome, SessionOutcome::EstablishedClosedClean);
    backend_task.await.expect("backend task");
}

async fn run_password_session(budget: ConnectionBudget, password: Bytes) -> SessionOutcome {
    let (backend_port, backend_task) = spawn_fake_backend({
        let password = password.clone();
        move |mut stream: TcpStream| async move {
            let wire = test_wire_config();
            let mut reader = FrameReader::new(Role::Frontend, &wire);
            assert!(matches!(
                read_frontend_frame(&mut stream, &mut reader).await,
                Some(FrontendMessage::Startup(_))
            ));
            write_backend(
                &mut stream,
                &BackendMessage::AuthenticationCleartextPassword(AuthenticationCleartextPassword),
            )
            .await;
            match read_frontend_frame(&mut stream, &mut reader).await {
                Some(FrontendMessage::Password(p)) => {
                    assert_eq!(
                        p.payload, password,
                        "backend must see only this session's password"
                    )
                }
                other => panic!("expected password reply, got {other:?}"),
            }
            write_backend(
                &mut stream,
                &BackendMessage::AuthenticationOk(AuthenticationOk),
            )
            .await;
            write_backend(
                &mut stream,
                &BackendMessage::ReadyForQuery(ReadyForQuery {
                    status: TransactionStatus::Idle,
                }),
            )
            .await;
            loop {
                match read_frontend_frame(&mut stream, &mut reader).await {
                    Some(FrontendMessage::Terminate(_)) | None => break,
                    Some(_) => continue,
                }
            }
        }
    })
    .await;

    let mut config = test_config(backend_port, 10);
    config.frontend_budget = budget;
    let (proxy_side, mut client_side) = connected_pair().await;
    let session = tokio::spawn(async move { run_session(proxy_side, &config).await });

    write_frontend(
        &mut client_side,
        &FrontendMessage::Startup(startup_message()),
    )
    .await;
    let wire = test_wire_config();
    let mut reader = FrameReader::new(Role::Backend, &wire);
    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::AuthenticationCleartextPassword(_))
    ));
    write_frontend(
        &mut client_side,
        &FrontendMessage::Password(PasswordMessage { payload: password }),
    )
    .await;
    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::AuthenticationOk(_))
    ));
    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::ReadyForQuery(_))
    ));
    write_frontend(&mut client_side, &FrontendMessage::Terminate(Terminate)).await;

    let outcome = session.await.expect("session task");
    backend_task.await.expect("backend task");
    outcome
}

/// verify: proxy::password_and_sasl_payload_bytes_are_never_retained (R2, regression)
///
/// Runs two independent password-auth sessions sharing one budget with
/// distinct password bytes; each fake backend's own `assert_eq!` proves the
/// *second* session's backend never observes the *first* session's
/// password bytes, i.e. nothing is cached/retained across sessions.
#[tokio::test]
async fn password_and_sasl_payload_bytes_are_never_retained() {
    let budget = ConnectionBudget::new(10);

    let first = run_password_session(budget.clone(), Bytes::from_static(b"first-secret\0")).await;
    assert_eq!(first, SessionOutcome::EstablishedClosedClean);

    let second = run_password_session(budget.clone(), Bytes::from_static(b"second-secret\0")).await;
    assert_eq!(second, SessionOutcome::EstablishedClosedClean);

    assert_eq!(budget.active(), 0, "permits released after both sessions");
}

/// verify: proxy::frame_error_on_either_leg_ends_session_without_forwarding_bad_bytes (R2, regression)
#[tokio::test]
async fn frame_error_on_either_leg_ends_session_without_forwarding_bad_bytes() {
    let (backend_port, backend_task) = spawn_fake_backend(|mut stream: TcpStream| async move {
        let wire = test_wire_config();
        let mut reader = FrameReader::new(Role::Frontend, &wire);
        assert!(matches!(
            read_frontend_frame(&mut stream, &mut reader).await,
            Some(FrontendMessage::Startup(_))
        ));
        write_backend(
            &mut stream,
            &BackendMessage::AuthenticationOk(AuthenticationOk),
        )
        .await;
        write_backend(
            &mut stream,
            &BackendMessage::ReadyForQuery(ReadyForQuery {
                status: TransactionStatus::Idle,
            }),
        )
        .await;

        // The malformed frame must never reach the backend: the proxy's leg
        // ends on FrameError before any forward. Assert clean EOF, not the
        // injected garbage bytes.
        let mut buf = [0_u8; 16];
        let n = stream.read(&mut buf).await.expect("read after ready");
        assert_eq!(
            n, 0,
            "backend must observe EOF, not the malformed frame's bytes"
        );
    })
    .await;

    let config = test_config(backend_port, 10);
    let (proxy_side, mut client_side) = connected_pair().await;
    let session = tokio::spawn(async move { run_session(proxy_side, &config).await });

    write_frontend(
        &mut client_side,
        &FrontendMessage::Startup(startup_message()),
    )
    .await;
    let wire = test_wire_config();
    let mut reader = FrameReader::new(Role::Backend, &wire);
    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::AuthenticationOk(_))
    ));
    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::ReadyForQuery(_))
    ));

    // An unknown tagged frame (tag 0xFF, declared length 4 = header only, no
    // body): no frontend message in this codec's scope uses this tag.
    client_side
        .write_all(&[0xFF, 0, 0, 0, 4])
        .await
        .expect("write malformed frame");

    let outcome = session.await.expect("session task");
    assert_eq!(outcome, SessionOutcome::EstablishedClosedError);
    backend_task.await.expect("backend task");
}

/// verify: proxy::startup_message_relayed_byte_identical_to_fake_backend (R2, functional)
#[tokio::test]
async fn startup_message_relayed_byte_identical_to_fake_backend() {
    let startup = StartupMessage {
        protocol_major: 3,
        protocol_minor: 0,
        parameters: vec![
            ("user".to_string(), "postgres".to_string()),
            ("database".to_string(), "app".to_string()),
            ("application_name".to_string(), "pgpool-test".to_string()),
        ],
    };
    let mut expected_bytes = BytesMut::new();
    startup.encode(&mut expected_bytes);
    let expected_bytes = expected_bytes.freeze();

    let (backend_port, backend_task) = spawn_fake_backend({
        let expected_bytes = expected_bytes.clone();
        move |mut stream: TcpStream| async move {
            // Use the same FrameReader for the whole connection (a fresh
            // reader would re-arm `awaiting_untagged_startup` and misparse
            // the next tagged frame as another untagged packet). Capture the
            // raw bytes fed to it while decoding the first frame so the
            // startup packet's bytes can be compared byte-for-byte.
            let wire = test_wire_config();
            let mut reader = FrameReader::new(Role::Frontend, &wire);
            let mut raw = Vec::new();
            let startup =
                read_frontend_frame_capturing_raw(&mut stream, &mut reader, &mut raw).await;
            assert!(matches!(startup, Some(FrontendMessage::Startup(_))));
            assert_eq!(
                raw,
                expected_bytes.to_vec(),
                "startup relayed byte-identical"
            );

            write_backend(
                &mut stream,
                &BackendMessage::AuthenticationOk(AuthenticationOk),
            )
            .await;
            write_backend(
                &mut stream,
                &BackendMessage::ReadyForQuery(ReadyForQuery {
                    status: TransactionStatus::Idle,
                }),
            )
            .await;

            loop {
                match read_frontend_frame(&mut stream, &mut reader).await {
                    Some(FrontendMessage::Terminate(_)) | None => break,
                    Some(_) => continue,
                }
            }
        }
    })
    .await;

    let config = test_config(backend_port, 10);
    let (proxy_side, mut client_side) = connected_pair().await;
    let session = tokio::spawn(async move { run_session(proxy_side, &config).await });

    write_frontend(&mut client_side, &FrontendMessage::Startup(startup)).await;
    let wire = test_wire_config();
    let mut reader = FrameReader::new(Role::Backend, &wire);
    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::AuthenticationOk(_))
    ));
    assert!(matches!(
        read_backend_frame(&mut client_side, &mut reader).await,
        Some(BackendMessage::ReadyForQuery(_))
    ));

    write_frontend(&mut client_side, &FrontendMessage::Terminate(Terminate)).await;
    let outcome = session.await.expect("session task");
    assert_eq!(outcome, SessionOutcome::EstablishedClosedClean);
    backend_task.await.expect("backend task");
}

// ---------------------------------------------------------------------
// R3: backend reachability and startup/auth rejection mapping.
// ---------------------------------------------------------------------

/// verify: proxy::backend_startup_error_response_forwarded_verbatim_to_client (R3, functional)
#[tokio::test]
async fn backend_startup_error_response_forwarded_verbatim_to_client() {
    let (backend_port, backend_task) = spawn_fake_backend(|mut stream: TcpStream| async move {
        let wire = test_wire_config();
        let mut reader = FrameReader::new(Role::Frontend, &wire);
        assert!(matches!(
            read_frontend_frame(&mut stream, &mut reader).await,
            Some(FrontendMessage::Startup(_))
        ));
        write_backend(
            &mut stream,
            &BackendMessage::ErrorResponse(ErrorResponse {
                fields: vec![
                    (b'S', "FATAL".to_string()),
                    (b'C', "28000".to_string()),
                    (b'M', "role \"nope\" does not exist".to_string()),
                ],
            }),
        )
        .await;
    })
    .await;

    let config = test_config(backend_port, 10);
    let (proxy_side, mut client_side) = connected_pair().await;
    let session = tokio::spawn(async move { run_session(proxy_side, &config).await });

    write_frontend(
        &mut client_side,
        &FrontendMessage::Startup(startup_message()),
    )
    .await;
    let wire = test_wire_config();
    let mut reader = FrameReader::new(Role::Backend, &wire);
    match read_backend_frame(&mut client_side, &mut reader).await {
        Some(BackendMessage::ErrorResponse(err)) => {
            assert!(err
                .fields
                .iter()
                .any(|(code, value)| *code == b'C' && value == "28000"));
        }
        other => panic!("expected backend ErrorResponse relayed verbatim, got {other:?}"),
    }

    let outcome = session.await.expect("session task");
    assert_eq!(outcome, SessionOutcome::RejectedAuthFailed);
    backend_task.await.expect("backend task");
}

/// verify: proxy::rejects_session_with_error_response_when_backend_unreachable (R3, functional)
#[tokio::test]
async fn rejects_session_with_error_response_when_backend_unreachable() {
    let dead_listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind dead listener");
    let dead_port = dead_listener.local_addr().unwrap().port();
    drop(dead_listener);

    let config = test_config(dead_port, 10);
    let (proxy_side, mut client_side) = connected_pair().await;
    let outcome = run_session(proxy_side, &config).await;
    assert_eq!(outcome, SessionOutcome::RejectedBackendUnreachable);

    let mut buf = Vec::new();
    client_side
        .read_to_end(&mut buf)
        .await
        .expect("read rejection bytes");
    let mut reader = FrameReader::new(Role::Backend, &test_wire_config());
    reader.feed(&buf);
    match reader
        .next_frame()
        .expect("frame decodes")
        .expect("frame present")
    {
        WireMessage::Backend(BackendMessage::ErrorResponse(err)) => {
            assert!(err
                .fields
                .iter()
                .any(|(code, value)| *code == b'C' && value == "08006"));
        }
        other => panic!("expected ErrorResponse with 08006, got {other:?}"),
    }
}

// ---------------------------------------------------------------------
// R4: drain interaction.
// ---------------------------------------------------------------------

/// verify: proxy::draining_stops_new_admissions_while_in_flight_session_completes (R4, functional)
#[tokio::test]
async fn draining_stops_new_admissions_while_in_flight_session_completes() {
    let (backend_port, backend_task) = spawn_fake_backend(|mut stream: TcpStream| async move {
        let wire = test_wire_config();
        let mut reader = FrameReader::new(Role::Frontend, &wire);
        assert!(matches!(
            read_frontend_frame(&mut stream, &mut reader).await,
            Some(FrontendMessage::Startup(_))
        ));
        write_backend(
            &mut stream,
            &BackendMessage::AuthenticationOk(AuthenticationOk),
        )
        .await;
        write_backend(
            &mut stream,
            &BackendMessage::ReadyForQuery(ReadyForQuery {
                status: TransactionStatus::Idle,
            }),
        )
        .await;
        loop {
            match read_frontend_frame(&mut stream, &mut reader).await {
                Some(FrontendMessage::Terminate(_)) | None => break,
                Some(_) => continue,
            }
        }
    })
    .await;

    let proxy_config = test_config(backend_port, 10);
    let handler = SessionHandler::new(proxy_config);
    let server_config = tcp_server::TcpServerConfig::new(server_core::BindConfig::localhost(0))
        .with_drain_timeout(Duration::from_secs(2));
    let listener = tcp_server::bind(&server_config)
        .await
        .expect("bind proxy listener");
    let addr = listener.local_addr().expect("proxy listener addr");
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let server = tokio::spawn(tcp_server::serve(
        listener,
        server_config,
        handler,
        async move {
            let _ = shutdown_rx.await;
        },
    ));

    let mut client = TcpStream::connect(addr)
        .await
        .expect("connect in-flight client");
    write_frontend(&mut client, &FrontendMessage::Startup(startup_message())).await;
    let wire = test_wire_config();
    let mut reader = FrameReader::new(Role::Backend, &wire);
    assert!(matches!(
        read_backend_frame(&mut client, &mut reader).await,
        Some(BackendMessage::AuthenticationOk(_))
    ));
    assert!(matches!(
        read_backend_frame(&mut client, &mut reader).await,
        Some(BackendMessage::ReadyForQuery(_))
    ));

    // Trigger drain: the accept loop stops immediately, but this in-flight
    // session is unaffected. Draining takes effect asynchronously (the
    // accept loop must be scheduled and the listener dropped), so retry a
    // fresh connection attempt until it is refused rather than racing a
    // single attempt against that handoff.
    let _ = shutdown_tx.send(());
    let refused = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if TcpStream::connect(addr).await.is_err() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await;
    assert!(
        refused.is_ok(),
        "new connection must eventually be refused while draining"
    );

    // The in-flight session, unaffected by draining, still completes
    // normally.
    write_frontend(&mut client, &FrontendMessage::Terminate(Terminate)).await;

    server.await.expect("server task");
    backend_task.await.expect("backend task");
}

/// verify: proxy::drain_timeout_elapses_and_abandons_still_running_session (R4, regression)
#[tokio::test]
async fn drain_timeout_elapses_and_abandons_still_running_session() {
    let (backend_port, _backend_task) = spawn_fake_backend(|mut stream: TcpStream| async move {
        let wire = test_wire_config();
        let mut reader = FrameReader::new(Role::Frontend, &wire);
        assert!(matches!(
            read_frontend_frame(&mut stream, &mut reader).await,
            Some(FrontendMessage::Startup(_))
        ));
        write_backend(
            &mut stream,
            &BackendMessage::AuthenticationOk(AuthenticationOk),
        )
        .await;
        write_backend(
            &mut stream,
            &BackendMessage::ReadyForQuery(ReadyForQuery {
                status: TransactionStatus::Idle,
            }),
        )
        .await;
        // Never replies again and never closes: simulates a session that
        // outlives the drain grace window.
        std::future::pending::<()>().await;
    })
    .await;

    let proxy_config = test_config(backend_port, 10);
    let handler = SessionHandler::new(proxy_config);
    let server_config = tcp_server::TcpServerConfig::new(server_core::BindConfig::localhost(0))
        .with_drain_timeout(Duration::from_millis(200));
    let listener = tcp_server::bind(&server_config)
        .await
        .expect("bind proxy listener");
    let addr = listener.local_addr().expect("proxy listener addr");
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let server = tokio::spawn(tcp_server::serve(
        listener,
        server_config,
        handler,
        async move {
            let _ = shutdown_rx.await;
        },
    ));

    let mut client = TcpStream::connect(addr)
        .await
        .expect("connect in-flight client");
    write_frontend(&mut client, &FrontendMessage::Startup(startup_message())).await;
    let wire = test_wire_config();
    let mut reader = FrameReader::new(Role::Backend, &wire);
    assert!(matches!(
        read_backend_frame(&mut client, &mut reader).await,
        Some(BackendMessage::AuthenticationOk(_))
    ));
    assert!(matches!(
        read_backend_frame(&mut client, &mut reader).await,
        Some(BackendMessage::ReadyForQuery(_))
    ));

    let _ = shutdown_tx.send(());
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .expect("drain loop must return once drain_timeout elapses, abandoning the stuck session")
        .expect("server task");
}
// </HANDWRITE>
