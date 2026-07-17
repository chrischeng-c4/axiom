// SPEC-MANAGED: apps/pgpool/tech-design/semantic/pgpool-runtime-connection-limit-discovery.md#unit-test
// <HANDWRITE gap="missing-generator:unit-test:pgpool-platform-discovery" tracker="#1570" reason="Real PostgreSQL discovery fixture generation is not available.">
use pgpool::platform::{
    discover_connection_facts, EndpointProvider, EndpointRole, ProviderAdvisory, RemoteEndpoint,
};

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
    };
    let Ok(facts) = discover_connection_facts(endpoint, config, ProviderAdvisory::default()).await
    else {
        return;
    };
    assert!(facts.runtime.max_connections > 0);
    assert!(facts.runtime.total_connections >= 1);
    assert!(facts.runtime.pgpool_connections >= 1);
    assert!(facts.effective_max_connections <= facts.runtime.max_connections);
}
// </HANDWRITE>
