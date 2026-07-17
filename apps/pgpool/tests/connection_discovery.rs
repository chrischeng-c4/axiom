// SPEC-MANAGED: apps/pgpool/tech-design/semantic/pgpool-runtime-connection-limit-discovery.md#unit-test
// <HANDWRITE gap="missing-generator:unit-test:pgpool-platform-discovery" tracker="#1570" reason="Real PostgreSQL discovery fixture generation is not available.">
use pgpool::platform::{
    discover_connection_facts, discovery_tls_mode, DiscoveryTlsMode, EndpointProvider,
    EndpointRole, ProviderAdvisory, RemoteEndpoint,
};

#[test]
fn managed_provider_selects_tls_discovery() {
    assert_eq!(
        discovery_tls_mode(EndpointProvider::PlainPostgres),
        DiscoveryTlsMode::NoTls
    );
    for provider in [EndpointProvider::CloudSql, EndpointProvider::AlloyDb] {
        assert_eq!(
            discovery_tls_mode(provider),
            DiscoveryTlsMode::SystemRoots,
            "managed provider {provider:?} must never take the plaintext discovery path"
        );
    }
}

#[tokio::test]
async fn discovers_runtime_limit_from_real_postgres_when_available() {
    let user = std::env::var("USER").unwrap_or_else(|_| "postgres".into());
    let config = format!(
        "host=127.0.0.1 port=5432 user={user} dbname=postgres connect_timeout=2 application_name=pgpool-discovery-test"
    )
    .parse::<tokio_postgres::Config>()
    .expect("valid postgres config");
    let endpoint = RemoteEndpoint {
        name: "local".into(),
        provider: EndpointProvider::PlainPostgres,
        role: EndpointRole::Primary,
        configured_ceiling: None,
        tls_ca_pem: None,
    };
    let Ok(facts) = discover_connection_facts(endpoint, config, ProviderAdvisory::default()).await
    else {
        return;
    };
    assert!(facts.runtime.max_connections > 0);
    assert!(facts.runtime.total_connections >= 1);
    assert!(facts.runtime.pgpool_connections >= 1);
    assert!(facts.effective_max_connections <= facts.runtime.max_connections);
    assert!(
        facts.effective_max_connections + facts.runtime.superuser_reserved_connections
            <= facts.runtime.max_connections
    );
}

#[tokio::test]
async fn pgpool_backend_connections_are_not_foreign_usage() {
    let user = std::env::var("USER").unwrap_or_else(|_| "postgres".into());
    let discovery_config = format!(
        "host=127.0.0.1 port=5432 user={user} dbname=postgres connect_timeout=2 application_name=pgpool-discovery-self-test"
    )
    .parse::<tokio_postgres::Config>()
    .expect("valid postgres config");
    let endpoint = RemoteEndpoint {
        name: "local".into(),
        provider: EndpointProvider::PlainPostgres,
        role: EndpointRole::Primary,
        configured_ceiling: None,
        tls_ca_pem: None,
    };
    let Ok(baseline) = discover_connection_facts(
        endpoint.clone(),
        discovery_config.clone(),
        ProviderAdvisory::default(),
    )
    .await
    else {
        return;
    };

    let backend_config = format!(
        "host=127.0.0.1 port=5432 user={user} dbname=postgres connect_timeout=2 application_name=pgpool-held-backend-test"
    )
    .parse::<tokio_postgres::Config>()
    .expect("valid postgres config");
    let Ok((_held_backend, connection)) = backend_config.connect(tokio_postgres::NoTls).await
    else {
        return;
    };
    let _connection = tokio::spawn(async move {
        let _ = connection.await;
    });

    let mut facts = baseline.clone();
    for _ in 0..20 {
        facts = discover_connection_facts(
            endpoint.clone(),
            discovery_config.clone(),
            ProviderAdvisory::default(),
        )
        .await
        .expect("discovery remains available while a pgpool backend is held");
        if facts.runtime.pgpool_connections >= baseline.runtime.pgpool_connections + 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(
        facts.runtime.pgpool_connections >= baseline.runtime.pgpool_connections + 1,
        "held pgpool backend must be classified as pgpool usage: baseline={baseline:?}, after={facts:?}"
    );
    assert_eq!(
        facts.non_pgpool_connections,
        facts
            .runtime
            .total_connections
            .saturating_sub(facts.runtime.pgpool_connections)
    );
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="pending-tracker" reason="Prove configured-CA CloudSql Rustls discovery against an externally supplied TLS-only PostgreSQL endpoint.">
/// Proves the managed-provider Rustls path against the TLS-required endpoint
/// started by `tls_required_discovery.sh`. Keeping its environment opt-in lets
/// the ordinary integration suite remain usable without Docker.
#[tokio::test]
async fn cloudsql_discovery_succeeds_against_tls_required_postgres() {
    let Ok(port) = std::env::var("PGPOOL_TLS_DISCOVERY_PORT") else {
        eprintln!(
            "skipping cloudsql_discovery_succeeds_against_tls_required_postgres: \
             run `sh apps/pgpool/tests/tls_required_discovery.sh`"
        );
        return;
    };
    let port: u16 = port
        .parse()
        .expect("PGPOOL_TLS_DISCOVERY_PORT must be a valid u16");
    let ca_path = std::env::var("PGPOOL_TLS_DISCOVERY_CA")
        .expect("PGPOOL_TLS_DISCOVERY_CA is required with PGPOOL_TLS_DISCOVERY_PORT");
    let tls_ca_pem = std::fs::read(&ca_path)
        .unwrap_or_else(|error| panic!("read TLS discovery CA {ca_path}: {error}"));
    let config = format!(
        "host=localhost port={port} user=postgres dbname=postgres \
         connect_timeout=5 application_name=pgpool-tls-discovery-proof"
    )
    .parse::<tokio_postgres::Config>()
    .expect("valid TLS discovery PostgreSQL config");
    let endpoint = RemoteEndpoint {
        name: "tls-required-cloudsql".into(),
        provider: EndpointProvider::CloudSql,
        role: EndpointRole::Primary,
        configured_ceiling: None,
        tls_ca_pem: Some(tls_ca_pem),
    };

    let facts = discover_connection_facts(endpoint, config, ProviderAdvisory::default())
        .await
        .expect("Cloud SQL discovery must trust the configured CA and query TLS-only PostgreSQL");

    assert!(facts.runtime.max_connections > 0);
    assert!(facts.runtime.total_connections >= 1);
    assert!(facts.runtime.pgpool_connections >= 1);
    assert_eq!(facts.endpoint.provider, EndpointProvider::CloudSql);
}
// </HANDWRITE>
// </HANDWRITE>
