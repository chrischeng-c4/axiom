// HANDWRITE-BEGIN gap="missing-generator:unit-test:70a5ad2b" tracker="#1599" reason="Verify exact match, synthetic cancellation key, challenge exclusion, and concurrent capped trust startup."
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bytes::{Bytes, BytesMut};
use server_lifecycle::{BindConfig, ConnectionBudget};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use pgpool::pool::{BackendPool, PoolConfig, TransactionHandler, TransactionProxyConfig};
use pgpool::proxy::BackendEndpointConfig;
use pgpool::wire::{
    AuthenticationCleartextPassword, AuthenticationMd5Password, AuthenticationOk,
    AuthenticationSasl, AuthenticationSaslContinue, AuthenticationSaslFinal, BackendKeyData,
    BackendMessage, CommandComplete, FrameReader, FrontendMessage, ParameterStatus,
    PasswordMessage, Query, ReadyForQuery, Role, StartupMessage, Terminate, TransactionStatus,
    WireCodecConfig, WireMessage,
};

#[derive(Clone, Copy)]
enum BackendAuth {
    Trust,
    Cleartext,
    Md5,
    Sasl,
}

fn wire() -> WireCodecConfig {
    WireCodecConfig::default()
}

fn startup(database: &str) -> StartupMessage {
    StartupMessage {
        protocol_major: 3,
        protocol_minor: 0,
        parameters: vec![
            ("user".to_string(), "postgres".to_string()),
            ("database".to_string(), database.to_string()),
            ("client_encoding".to_string(), "UTF8".to_string()),
        ],
    }
}

fn pool_config(port: u16, cap: usize) -> PoolConfig {
    PoolConfig {
        endpoint: BackendEndpointConfig {
            host: "127.0.0.1".to_string(),
            port,
        },
        max_backend_connections: cap,
        acquire_timeout: Duration::from_secs(3),
        backend_connect_timeout: Duration::from_secs(1),
        wire: wire(),
    }
}

async fn write_backend(stream: &mut TcpStream, message: &BackendMessage) {
    let mut buf = BytesMut::new();
    message.encode(&mut buf);
    stream.write_all(&buf).await.expect("write backend frame");
}

async fn write_frontend(stream: &mut TcpStream, message: &FrontendMessage) {
    let mut buf = BytesMut::new();
    message.encode(&mut buf);
    stream.write_all(&buf).await.expect("write frontend frame");
}

async fn read_frontend(
    stream: &mut TcpStream,
    reader: &mut FrameReader,
) -> Option<FrontendMessage> {
    loop {
        match reader.next_frame().expect("frontend frame decodes") {
            Some(WireMessage::Frontend(message)) => return Some(message),
            Some(WireMessage::Backend(_)) => unreachable!("frontend reader cannot emit backend"),
            None => {
                let mut buf = [0_u8; 4096];
                let count = stream.read(&mut buf).await.expect("read frontend bytes");
                if count == 0 {
                    return None;
                }
                reader.feed(&buf[..count]);
            }
        }
    }
}

async fn read_backend(stream: &mut TcpStream, reader: &mut FrameReader) -> Option<BackendMessage> {
    loop {
        match reader.next_frame().expect("backend frame decodes") {
            Some(WireMessage::Backend(message)) => return Some(message),
            Some(WireMessage::Frontend(_)) => unreachable!("backend reader cannot emit frontend"),
            None => {
                let mut buf = [0_u8; 4096];
                let count = stream.read(&mut buf).await.expect("read backend bytes");
                if count == 0 {
                    return None;
                }
                reader.feed(&buf[..count]);
            }
        }
    }
}

async fn authenticate_backend(stream: &mut TcpStream, auth: BackendAuth, connection_id: usize) {
    let mut reader = FrameReader::new(Role::Frontend, &wire());
    assert!(matches!(
        read_frontend(stream, &mut reader).await,
        Some(FrontendMessage::Startup(_))
    ));

    match auth {
        BackendAuth::Trust => {}
        BackendAuth::Cleartext => {
            write_backend(
                stream,
                &BackendMessage::AuthenticationCleartextPassword(AuthenticationCleartextPassword),
            )
            .await;
            assert!(read_frontend(stream, &mut reader).await.is_some());
        }
        BackendAuth::Md5 => {
            write_backend(
                stream,
                &BackendMessage::AuthenticationMd5Password(AuthenticationMd5Password {
                    salt: [1, 2, 3, 4],
                }),
            )
            .await;
            assert!(read_frontend(stream, &mut reader).await.is_some());
        }
        BackendAuth::Sasl => {
            write_backend(
                stream,
                &BackendMessage::AuthenticationSasl(AuthenticationSasl {
                    mechanisms: vec!["SCRAM-SHA-256".to_string()],
                }),
            )
            .await;
            assert!(read_frontend(stream, &mut reader).await.is_some());
            write_backend(
                stream,
                &BackendMessage::AuthenticationSaslContinue(AuthenticationSaslContinue {
                    payload: Bytes::from_static(b"r=nonce,s=salt,i=1"),
                }),
            )
            .await;
            assert!(read_frontend(stream, &mut reader).await.is_some());
            write_backend(
                stream,
                &BackendMessage::AuthenticationSaslFinal(AuthenticationSaslFinal {
                    payload: Bytes::from_static(b"v=proof"),
                }),
            )
            .await;
        }
    }

    write_backend(stream, &BackendMessage::AuthenticationOk(AuthenticationOk)).await;
    write_backend(
        stream,
        &BackendMessage::ParameterStatus(ParameterStatus {
            name: "client_encoding".to_string(),
            value: "UTF8".to_string(),
        }),
    )
    .await;
    write_backend(
        stream,
        &BackendMessage::BackendKeyData(BackendKeyData {
            process_id: connection_id as i32 + 100,
            secret_key: connection_id as i32 + 200,
        }),
    )
    .await;
    write_backend(
        stream,
        &BackendMessage::ReadyForQuery(ReadyForQuery {
            status: TransactionStatus::Idle,
        }),
    )
    .await;

    while let Some(message) = read_frontend(stream, &mut reader).await {
        match message {
            FrontendMessage::Query(query) => {
                let tag = if query.sql == "DISCARD ALL" {
                    "DISCARD ALL"
                } else {
                    "SELECT 1"
                };
                write_backend(
                    stream,
                    &BackendMessage::CommandComplete(CommandComplete {
                        tag: tag.to_string(),
                    }),
                )
                .await;
                write_backend(
                    stream,
                    &BackendMessage::ReadyForQuery(ReadyForQuery {
                        status: TransactionStatus::Idle,
                    }),
                )
                .await;
            }
            FrontendMessage::Terminate(_) => return,
            other => panic!("unexpected frontend frame after startup: {other:?}"),
        }
    }
}

async fn spawn_backend(auth: BackendAuth) -> (u16, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fake backend");
    let port = listener.local_addr().expect("fake backend address").port();
    let accepted = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&accepted);
    let server = tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.expect("accept fake backend");
            let connection_id = count.fetch_add(1, Ordering::SeqCst);
            tokio::spawn(async move {
                authenticate_backend(&mut stream, auth, connection_id).await;
            });
        }
    });
    (port, accepted, server)
}

async fn spawn_proxy(
    pool: BackendPool,
) -> (
    std::net::SocketAddr,
    tokio::task::JoinHandle<()>,
    tokio::sync::oneshot::Sender<()>,
) {
    let handler = TransactionHandler::new(TransactionProxyConfig {
        frontend_budget: ConnectionBudget::new(256),
        backend_pool: pool,
        wire: wire(),
        drain_timeout: Duration::from_secs(2),
    });
    let config = server_tcp::TcpServerConfig::new(BindConfig::localhost(0));
    let listener = server_tcp::bind(&config)
        .await
        .expect("bind transaction proxy");
    let address = listener.local_addr().expect("transaction proxy address");
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(server_tcp::serve(listener, config, handler, async move {
        let _ = shutdown_rx.await;
    }));
    (address, server, shutdown_tx)
}

async fn admit(
    proxy: std::net::SocketAddr,
    startup: StartupMessage,
    answer_challenges: bool,
) -> Result<(TcpStream, Vec<BackendMessage>), String> {
    let mut stream = TcpStream::connect(proxy)
        .await
        .map_err(|error| format!("connect proxy: {error}"))?;
    write_frontend(&mut stream, &FrontendMessage::Startup(startup)).await;

    let mut reader = FrameReader::new(Role::Backend, &wire());
    let mut messages = Vec::new();
    loop {
        let message = read_backend(&mut stream, &mut reader)
            .await
            .ok_or_else(|| "proxy closed during startup".to_string())?;
        match &message {
            BackendMessage::ErrorResponse(error) => {
                return Err(format!("proxy rejected startup: {error:?}"));
            }
            BackendMessage::AuthenticationCleartextPassword(_)
            | BackendMessage::AuthenticationMd5Password(_)
            | BackendMessage::AuthenticationSasl(_)
            | BackendMessage::AuthenticationSaslContinue(_) => {
                if !answer_challenges {
                    return Err("unexpected authentication challenge".to_string());
                }
                write_frontend(
                    &mut stream,
                    &FrontendMessage::Password(PasswordMessage {
                        payload: Bytes::from_static(b"opaque-client-response\0"),
                    }),
                )
                .await;
            }
            BackendMessage::ReadyForQuery(_) => {
                messages.push(message);
                return Ok((stream, messages));
            }
            _ => {}
        }
        messages.push(message);
    }
}

async fn simple_query(stream: &mut TcpStream) {
    write_frontend(
        stream,
        &FrontendMessage::Query(Query {
            sql: "SELECT 1".to_string(),
        }),
    )
    .await;
    let mut reader = FrameReader::new(Role::Backend, &wire());
    loop {
        let message = read_backend(stream, &mut reader)
            .await
            .expect("query response before proxy EOF");
        if matches!(message, BackendMessage::ReadyForQuery(_)) {
            return;
        }
    }
}

async fn write_pipelined_queries(stream: &mut TcpStream, sql: &[&str]) {
    let mut bytes = BytesMut::new();
    for statement in sql {
        FrontendMessage::Query(Query {
            sql: (*statement).to_string(),
        })
        .encode(&mut bytes);
    }
    stream
        .write_all(&bytes)
        .await
        .expect("write pipelined query frames");
}

async fn close_client(mut stream: TcpStream) {
    write_frontend(&mut stream, &FrontendMessage::Terminate(Terminate)).await;
    stream.shutdown().await.expect("shutdown client");
}

async fn wait_for_idle(pool: &BackendPool) {
    for _ in 0..100 {
        let stats = pool.stats();
        if stats.backend_active == 0 && stats.backend_idle > 0 {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!(
        "pool did not return a fresh handshake backend to idle: {:?}",
        pool.stats()
    );
}

async fn stop_proxy(
    server: tokio::task::JoinHandle<()>,
    shutdown: tokio::sync::oneshot::Sender<()>,
) {
    let _ = shutdown.send(());
    server.await.expect("proxy server joins");
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="#1878" reason="unit-test section in trust_startup_replay.rs is hand-written pending codegen support">
async fn write_trust_startup_reply(stream: &mut TcpStream) {
    write_backend(stream, &BackendMessage::AuthenticationOk(AuthenticationOk)).await;
    write_backend(
        stream,
        &BackendMessage::ParameterStatus(ParameterStatus {
            name: "client_encoding".to_string(),
            value: "UTF8".to_string(),
        }),
    )
    .await;
    write_backend(
        stream,
        &BackendMessage::BackendKeyData(BackendKeyData {
            process_id: 100,
            secret_key: 200,
        }),
    )
    .await;
    write_backend(
        stream,
        &BackendMessage::ReadyForQuery(ReadyForQuery {
            status: TransactionStatus::Idle,
        }),
    )
    .await;
}

async fn reply_idle(stream: &mut TcpStream, tag: &str) {
    write_backend(
        stream,
        &BackendMessage::CommandComplete(CommandComplete {
            tag: tag.to_string(),
        }),
    )
    .await;
    write_backend(
        stream,
        &BackendMessage::ReadyForQuery(ReadyForQuery {
            status: TransactionStatus::Idle,
        }),
    )
    .await;
}

async fn expect_query(stream: &mut TcpStream, reader: &mut FrameReader, expected: &str) {
    assert!(matches!(
        read_frontend(stream, reader).await,
        Some(FrontendMessage::Query(Query { sql })) if sql == expected
    ));
}

async fn wait_for_active(pool: &BackendPool) {
    for _ in 0..100 {
        if pool.stats().backend_active == 1 {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("sole backend never became active: {:?}", pool.stats());
}

async fn run_saturated_pipeline(statements: &[&str]) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fake backend");
    let backend_port = listener.local_addr().expect("fake backend address").port();
    let statements: Vec<String> = statements
        .iter()
        .map(|statement| (*statement).to_string())
        .collect();
    let backend_statements = statements.clone();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let backend_server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept fake backend");
        let mut reader = FrameReader::new(Role::Frontend, &wire());

        assert!(matches!(
            read_frontend(&mut stream, &mut reader).await,
            Some(FrontendMessage::Startup(_))
        ));
        write_trust_startup_reply(&mut stream).await;
        expect_query(&mut stream, &mut reader, "DISCARD ALL").await;
        reply_idle(&mut stream, "DISCARD ALL").await;

        expect_query(&mut stream, &mut reader, "SELECT hold").await;
        let _ = release_rx.await;
        reply_idle(&mut stream, "hold").await;

        for statement in backend_statements {
            expect_query(&mut stream, &mut reader, "DISCARD ALL").await;
            reply_idle(&mut stream, "DISCARD ALL").await;
            expect_query(&mut stream, &mut reader, &statement).await;
            reply_idle(&mut stream, &statement).await;
        }
        expect_query(&mut stream, &mut reader, "DISCARD ALL").await;
        reply_idle(&mut stream, "DISCARD ALL").await;
    });

    let pool = BackendPool::new(pool_config(backend_port, 1));
    let (proxy, proxy_server, shutdown) = spawn_proxy(pool.clone()).await;
    let (mut holder, _) = admit(proxy, startup("pipeline"), false)
        .await
        .expect("holder trust startup admits");
    wait_for_idle(&pool).await;
    write_frontend(
        &mut holder,
        &FrontendMessage::Query(Query {
            sql: "SELECT hold".to_string(),
        }),
    )
    .await;
    wait_for_active(&pool).await;

    let (mut pipelined, _) = admit(proxy, startup("pipeline"), false)
        .await
        .expect("matching startup replays while the sole backend is active");
    let expected: Vec<&str> = statements.iter().map(String::as_str).collect();
    write_pipelined_queries(&mut pipelined, &expected).await;
    tokio::time::sleep(Duration::from_millis(25)).await;
    release_tx
        .send(())
        .expect("backend script is still holding the sole lease");

    let mut reader = FrameReader::new(Role::Backend, &wire());
    let mut completions = Vec::new();
    let mut ready_count = 0;
    while completions.len() < expected.len() || ready_count < expected.len() {
        match tokio::time::timeout(
            Duration::from_secs(2),
            read_backend(&mut pipelined, &mut reader),
        )
        .await
        .expect("pipelined response must not hang")
        .expect("pipelined client remains connected")
        {
            BackendMessage::CommandComplete(command) => completions.push(command.tag),
            BackendMessage::ReadyForQuery(_) => ready_count += 1,
            other => panic!("unexpected pipelined response: {other:?}"),
        }
    }
    assert_eq!(completions, statements);
    assert_eq!(ready_count, expected.len());

    close_client(pipelined).await;
    close_client(holder).await;
    stop_proxy(proxy_server, shutdown).await;
    drop(pool);
    backend_server.await.expect("backend script joins");
}

/// verify: trust_startup_replay::reactor_saturated_pipelined_queries_resume_without_new_socket_read (R1)
#[tokio::test]
async fn reactor_saturated_pipelined_queries_resume_without_new_socket_read() {
    run_saturated_pipeline(&["SELECT two_one", "SELECT two_two"]).await;
    run_saturated_pipeline(&["SELECT three_one", "SELECT three_two", "SELECT three_three"]).await;
}

/// verify: trust_startup_replay::reactor_pipelined_startup_and_first_query_complete (R1)
#[tokio::test]
async fn reactor_pipelined_startup_and_first_query_complete() {
    let (backend_port, _, backend_server) = spawn_backend(BackendAuth::Trust).await;
    let pool = BackendPool::new(pool_config(backend_port, 1));
    let (proxy, proxy_server, shutdown) = spawn_proxy(pool).await;
    let mut client = TcpStream::connect(proxy).await.expect("connect proxy");
    let mut bytes = BytesMut::new();
    FrontendMessage::Startup(startup("startup-pipeline")).encode(&mut bytes);
    FrontendMessage::Query(Query {
        sql: "SELECT startup_pipeline".to_string(),
    })
    .encode(&mut bytes);
    client
        .write_all(&bytes)
        .await
        .expect("write startup and first query in one segment");

    let mut reader = FrameReader::new(Role::Backend, &wire());
    let mut ready_count = 0;
    let mut commands = Vec::new();
    while ready_count < 2 || commands.is_empty() {
        match tokio::time::timeout(
            Duration::from_secs(2),
            read_backend(&mut client, &mut reader),
        )
        .await
        .expect("pipelined startup must not hang")
        .expect("client remains connected after startup pipeline")
        {
            BackendMessage::CommandComplete(command) => commands.push(command.tag),
            BackendMessage::ReadyForQuery(_) => ready_count += 1,
            _ => {}
        }
    }
    assert_eq!(commands, ["SELECT 1"]);
    assert_eq!(ready_count, 2);

    close_client(client).await;
    stop_proxy(proxy_server, shutdown).await;
    backend_server.abort();
}

/// A pipelined implicit next query remains in the client socket while the
/// first query's backend lease is released and reset. The old backend must
/// therefore observe `DISCARD ALL` before it can receive the second query.
///
/// verify: trust_startup_replay::backend_first_relay_keeps_pipelined_query_out_of_resetting_backend (P0 #1709)
#[tokio::test]
async fn backend_first_relay_keeps_pipelined_query_out_of_resetting_backend() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fake backend");
    let backend_port = listener.local_addr().expect("fake backend address").port();
    let backend_server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept fake backend");
        let mut reader = FrameReader::new(Role::Frontend, &wire());

        assert!(matches!(
            read_frontend(&mut stream, &mut reader).await,
            Some(FrontendMessage::Startup(_))
        ));
        write_backend(
            &mut stream,
            &BackendMessage::AuthenticationOk(AuthenticationOk),
        )
        .await;
        write_backend(
            &mut stream,
            &BackendMessage::ParameterStatus(ParameterStatus {
                name: "client_encoding".to_string(),
                value: "UTF8".to_string(),
            }),
        )
        .await;
        write_backend(
            &mut stream,
            &BackendMessage::BackendKeyData(BackendKeyData {
                process_id: 100,
                secret_key: 200,
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

        // Fresh startup is reset before the backend becomes idle.
        assert!(matches!(
            read_frontend(&mut stream, &mut reader).await,
            Some(FrontendMessage::Query(Query { sql })) if sql == "DISCARD ALL"
        ));
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

        assert!(matches!(
            read_frontend(&mut stream, &mut reader).await,
            Some(FrontendMessage::Query(Query { sql })) if sql == "SELECT first"
        ));

        // The second query was pipelined from the client, but cannot become
        // input to this backend before the first ReadyForQuery(Idle).
        assert!(
            tokio::time::timeout(
                Duration::from_millis(100),
                read_frontend(&mut stream, &mut reader),
            )
            .await
            .is_err(),
            "pipelined implicit query reached the active backend before its response"
        );

        write_backend(
            &mut stream,
            &BackendMessage::CommandComplete(CommandComplete {
                tag: "first".to_string(),
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

        // The previous lease is reset before the outer loop reads and
        // acquires for the pipelined second implicit query.
        assert!(matches!(
            read_frontend(&mut stream, &mut reader).await,
            Some(FrontendMessage::Query(Query { sql })) if sql == "DISCARD ALL"
        ));
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

        assert!(matches!(
            read_frontend(&mut stream, &mut reader).await,
            Some(FrontendMessage::Query(Query { sql })) if sql == "SELECT second"
        ));
        write_backend(
            &mut stream,
            &BackendMessage::CommandComplete(CommandComplete {
                tag: "second".to_string(),
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

        assert!(matches!(
            read_frontend(&mut stream, &mut reader).await,
            Some(FrontendMessage::Query(Query { sql })) if sql == "DISCARD ALL"
        ));
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

        assert!(
            read_frontend(&mut stream, &mut reader).await.is_none(),
            "client terminate is handled after the lease returns idle"
        );
    });

    let pool = BackendPool::new(pool_config(backend_port, 1));
    let (proxy, proxy_server, shutdown) = spawn_proxy(pool.clone()).await;
    let (mut client, _) = admit(proxy, startup("pipeline"), false)
        .await
        .expect("trust startup admits");
    wait_for_idle(&pool).await;

    write_pipelined_queries(&mut client, &["SELECT first", "SELECT second"]).await;
    let mut reader = FrameReader::new(Role::Backend, &wire());
    let mut completions = Vec::new();
    let mut ready_count = 0;
    while completions.len() < 2 || ready_count < 2 {
        match read_backend(&mut client, &mut reader)
            .await
            .expect("pipelined response before proxy EOF")
        {
            BackendMessage::CommandComplete(command) => completions.push(command.tag),
            BackendMessage::ReadyForQuery(_) => ready_count += 1,
            other => panic!("unexpected response to pipelined queries: {other:?}"),
        }
    }
    assert_eq!(completions, ["first", "second"]);
    wait_for_idle(&pool).await;

    close_client(client).await;
    stop_proxy(proxy_server, shutdown).await;
    drop(pool);
    backend_server.await.expect("backend script joins");
}
// </HANDWRITE>

fn backend_key(messages: &[BackendMessage]) -> BackendKeyData {
    messages
        .iter()
        .find_map(|message| match message {
            BackendMessage::BackendKeyData(key) => Some(*key),
            _ => None,
        })
        .expect("startup reply includes BackendKeyData")
}

/// verify: trust_startup_replay::exact_no_challenge_startup_replays_without_a_backend_lease (R1)
#[tokio::test]
async fn exact_no_challenge_startup_replays_without_a_backend_lease() {
    let (backend_port, accepted, backend_server) = spawn_backend(BackendAuth::Trust).await;
    let pool = BackendPool::new(pool_config(backend_port, 1));
    let (proxy, proxy_server, shutdown) = spawn_proxy(pool.clone()).await;

    let (first, first_messages) = admit(proxy, startup("first"), false)
        .await
        .expect("first trust startup admits");
    wait_for_idle(&pool).await;
    assert_eq!(accepted.load(Ordering::SeqCst), 1);
    assert_ne!(backend_key(&first_messages).process_id, 0);
    assert_eq!(pool.startup_replay_count(), 1);

    let (second, second_messages) = admit(proxy, startup("first"), false)
        .await
        .expect("matching startup replays");
    assert_eq!(
        accepted.load(Ordering::SeqCst),
        1,
        "cache hit must not dial a backend"
    );
    assert_eq!(backend_key(&second_messages).process_id, 0);
    assert_eq!(backend_key(&second_messages).secret_key, 0);
    assert_eq!(pool.stats().backend_active, 0, "replay must hold no lease");

    close_client(first).await;
    close_client(second).await;
    stop_proxy(proxy_server, shutdown).await;
    backend_server.abort();
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="pending-tracker" reason="unit-test section in trust_startup_replay.rs is hand-written pending codegen support">
/// verify: trust_startup_replay::startup_mismatch_and_auth_challenges_never_replay (R2)
#[tokio::test]
async fn startup_mismatch_and_auth_challenges_never_replay() {
    let (backend_port, accepted, backend_server) = spawn_backend(BackendAuth::Trust).await;
    let pool = BackendPool::new(pool_config(backend_port, 2));
    let (proxy, proxy_server, shutdown) = spawn_proxy(pool.clone()).await;
    let (first, _) = admit(proxy, startup("one"), false)
        .await
        .expect("first trust startup");
    wait_for_idle(&pool).await;
    let (second, _) = admit(proxy, startup("two"), false)
        .await
        .expect("different startup takes a fresh backend");
    wait_for_idle(&pool).await;
    assert_eq!(accepted.load(Ordering::SeqCst), 2);
    assert_eq!(pool.startup_replay_count(), 2);
    close_client(first).await;
    close_client(second).await;
    stop_proxy(proxy_server, shutdown).await;
    backend_server.abort();

    for auth in [BackendAuth::Cleartext, BackendAuth::Md5, BackendAuth::Sasl] {
        let (backend_port, accepted, backend_server) = spawn_backend(auth).await;
        let pool = BackendPool::new(pool_config(backend_port, 1));
        let (proxy, proxy_server, shutdown) = spawn_proxy(pool.clone()).await;
        let (client, _) = admit(proxy, startup("challenge"), true)
            .await
            .expect("challenge passthrough admits first client");
        wait_for_idle(&pool).await;
        assert_eq!(
            pool.startup_replay_count(),
            0,
            "challenge must not populate cache"
        );
        assert!(
            admit(proxy, startup("challenge"), true).await.is_err(),
            "without a replay, a capped fresh admission must be rejected"
        );
        assert_eq!(
            accepted.load(Ordering::SeqCst),
            1,
            "rejected admission cannot dial"
        );
        close_client(client).await;
        stop_proxy(proxy_server, shutdown).await;
        backend_server.abort();
    }
}
// </HANDWRITE>

/// verify: trust_startup_replay::capped_trust_clients_complete_without_startup_rejection (AC1)
#[tokio::test]
async fn capped_trust_clients_complete_without_startup_rejection() {
    let (backend_port, accepted, backend_server) = spawn_backend(BackendAuth::Trust).await;
    let pool = BackendPool::new(pool_config(backend_port, 16));
    let (proxy, proxy_server, shutdown) = spawn_proxy(pool.clone()).await;

    let mut starts = tokio::task::JoinSet::new();
    for _ in 0..64 {
        starts.spawn(admit(proxy, startup("pgpool_bench"), false));
    }
    let mut clients = Vec::new();
    while let Some(result) = starts.join_next().await {
        clients.push(
            result
                .expect("startup task joins")
                .expect("all capped trust clients admit"),
        );
    }

    wait_for_idle(&pool).await;
    assert!(
        accepted.load(Ordering::SeqCst) <= 16,
        "capped startup may race to at most one fresh backend per capacity slot"
    );
    assert_eq!(clients.len(), 64);

    for (mut client, _) in clients {
        simple_query(&mut client).await;
        close_client(client).await;
    }
    wait_for_idle(&pool).await;
    assert!(pool.stats().backend_idle <= 16);

    stop_proxy(proxy_server, shutdown).await;
    backend_server.abort();
}
// HANDWRITE-END
