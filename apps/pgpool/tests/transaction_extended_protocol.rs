//! Regression coverage for transaction pooling's explicit extended-protocol
//! stopgap. It uses a real local Postgres for startup/auth and skips when that
//! service is unavailable, matching the app's integration-test convention.

use std::ffi::OsString;
use std::net::SocketAddr;
use std::time::Duration;

use pgpool::pool::{BackendPool, PoolConfig, TransactionHandler, TransactionProxyConfig};
use pgpool::proxy::BackendEndpointConfig;
use pgpool::wire::WireCodecConfig;
use server_lifecycle::{BindConfig, ConnectionBudget};

async fn real_backend_ready() -> Option<(SocketAddr, String)> {
    let addr: SocketAddr = "127.0.0.1:5432".parse().ok()?;
    let user = std::env::var("USER").unwrap_or_else(|_| "postgres".to_string());
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

fn transaction_handler(backend_addr: SocketAddr, engine: Option<&str>) -> TransactionHandler {
    let original: Option<OsString> = std::env::var_os("PGPOOL_TRANSACTION_ENGINE");
    // The engine is selected during handler construction. Restore the caller's
    // process environment immediately so this integration test never leaks
    // the legacy switch to other tests.
    unsafe {
        match engine {
            Some(engine) => std::env::set_var("PGPOOL_TRANSACTION_ENGINE", engine),
            None => std::env::remove_var("PGPOOL_TRANSACTION_ENGINE"),
        }
    }
    let handler = TransactionHandler::new(TransactionProxyConfig {
        frontend_budget: ConnectionBudget::new(10),
        backend_pool: BackendPool::new(PoolConfig {
            endpoint: BackendEndpointConfig {
                host: backend_addr.ip().to_string(),
                port: backend_addr.port(),
            },
            max_backend_connections: 4,
            acquire_timeout: Duration::from_secs(2),
            backend_connect_timeout: Duration::from_secs(2),
            wire: WireCodecConfig::default(),
        }),
        wire: WireCodecConfig::default(),
        drain_timeout: Duration::from_secs(2),
    });
    unsafe {
        match original {
            Some(value) => std::env::set_var("PGPOOL_TRANSACTION_ENGINE", value),
            None => std::env::remove_var("PGPOOL_TRANSACTION_ENGINE"),
        }
    }
    handler
}

async fn spawn_transaction_proxy(
    backend_addr: SocketAddr,
    engine: Option<&str>,
) -> (
    SocketAddr,
    tokio::task::JoinHandle<()>,
    tokio::sync::oneshot::Sender<()>,
) {
    let handler = transaction_handler(backend_addr, engine);
    let server_config = server_tcp::TcpServerConfig::new(BindConfig::localhost(0));
    let listener = server_tcp::bind(&server_config)
        .await
        .expect("bind transaction proxy listener");
    let proxy_addr = listener.local_addr().expect("proxy listener addr");
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(server_tcp::serve(
        listener,
        server_config,
        handler,
        async move {
            let _ = shutdown_rx.await;
        },
    ));
    (proxy_addr, server, shutdown_tx)
}

/// verify: transaction_extended_protocol::parse_is_rejected_without_hang (R1)
#[tokio::test]
async fn parse_is_rejected_without_hang() {
    let Some((backend_addr, user)) = real_backend_ready().await else {
        eprintln!("skipping parse_is_rejected_without_hang: no local Postgres for startup/auth");
        return;
    };

    for (label, engine) in [("legacy", Some("legacy")), ("reactor", None)] {
        let (proxy_addr, server, shutdown_tx) = spawn_transaction_proxy(backend_addr, engine).await;
        let dsn = format!(
            "host={} port={} user={} dbname=postgres connect_timeout=5",
            proxy_addr.ip(),
            proxy_addr.port(),
            user
        );
        let (client, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
            .await
            .unwrap_or_else(|error| panic!("{label} transaction client startup failed: {error}"));
        let connection_task = tokio::spawn(connection);

        let error = tokio::time::timeout(Duration::from_secs(2), client.query_one("SELECT 1", &[]))
            .await
            .unwrap_or_else(|_| panic!("{label} Parse must receive an error instead of hanging"))
            .expect_err(
                "transaction pooling must reject Parse until extended protocol is supported",
            );
        let db_error = error
            .as_db_error()
            .unwrap_or_else(|| panic!("{label} must receive a PostgreSQL ErrorResponse: {error}"));
        assert_eq!(db_error.code().code(), "0A000");
        assert_eq!(
            db_error.message(),
            "extended query protocol not yet supported in transaction pooling mode"
        );

        drop(client);
        let _ = tokio::time::timeout(Duration::from_secs(2), connection_task).await;
        let _ = shutdown_tx.send(());
        server.await.expect("transaction proxy server task joins");
    }
}
