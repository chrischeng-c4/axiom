// SPEC-MANAGED: apps/pgpool/tech-design/semantic/pgpool-runtime-connection-limit-discovery.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-platform-discovery" tracker="#1570" reason="Live PostgreSQL system-view discovery needs an async adapter primitive.">
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointProvider {
    PlainPostgres,
    CloudSql,
    AlloyDb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointRole {
    Primary,
    ReadPool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteEndpoint {
    pub name: String,
    pub provider: EndpointProvider,
    pub role: EndpointRole,
    pub configured_ceiling: Option<u32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderAdvisory {
    /// Optional provider control-plane cap. It can only reduce runtime capacity.
    pub max_connections: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeConnectionFacts {
    pub max_connections: u32,
    pub total_connections: u32,
    pub pgpool_connections: u32,
}

impl RuntimeConnectionFacts {
    pub fn non_pgpool_connections(self) -> u32 {
        self.total_connections
            .saturating_sub(self.pgpool_connections)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionFacts {
    pub endpoint: RemoteEndpoint,
    pub runtime: RuntimeConnectionFacts,
    pub advisory: ProviderAdvisory,
    pub effective_max_connections: u32,
    pub non_pgpool_connections: u32,
}

#[derive(Debug, Error)]
pub enum ConnectionDiscoveryError {
    #[error("failed to connect to remote PostgreSQL endpoint: {0}")]
    Connect(#[source] tokio_postgres::Error),
    #[error("remote PostgreSQL discovery query failed: {0}")]
    Query(#[source] tokio_postgres::Error),
    #[error("remote max_connections value is not a positive integer: {0}")]
    InvalidMaxConnections(String),
    #[error("remote connection count is outside the supported u32 range: {0}")]
    InvalidConnectionCount(i64),
}

pub fn effective_connection_limit(
    runtime_max: u32,
    configured_ceiling: Option<u32>,
    advisory_max: Option<u32>,
) -> u32 {
    [Some(runtime_max), configured_ceiling, advisory_max]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(runtime_max)
}

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in discovery.rs is hand-written pending codegen support">
/// Query the live endpoint. Provider SDK metadata is accepted only as an
/// advisory cap and never substitutes for this runtime result.
pub async fn discover_connection_facts(
    endpoint: RemoteEndpoint,
    postgres: tokio_postgres::Config,
    advisory: ProviderAdvisory,
) -> Result<ConnectionFacts, ConnectionDiscoveryError> {
    let (client, connection) = postgres
        .connect(tokio_postgres::NoTls)
        .await
        .map_err(ConnectionDiscoveryError::Connect)?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            tracing::warn!(%error, "remote PostgreSQL discovery connection ended");
        }
    });

    let row = client
        .query_one(
            "SELECT current_setting('max_connections') AS max_connections, \
                    count(*)::bigint AS total_connections, \
                    count(*) FILTER (WHERE application_name LIKE 'pgpool%')::bigint \
                        AS pgpool_connections \
             FROM pg_stat_activity",
            &[],
        )
        .await
        .map_err(ConnectionDiscoveryError::Query)?;

    let runtime_max_raw: String = row.get("max_connections");
    let runtime_max = runtime_max_raw
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| ConnectionDiscoveryError::InvalidMaxConnections(runtime_max_raw))?;
    let total_connections = count_to_u32(row.get("total_connections"))?;
    let pgpool_connections = count_to_u32(row.get("pgpool_connections"))?;
    let runtime = RuntimeConnectionFacts {
        max_connections: runtime_max,
        total_connections,
        pgpool_connections,
    };
    let effective_max_connections = effective_connection_limit(
        runtime.max_connections,
        endpoint.configured_ceiling,
        advisory.max_connections,
    );

    Ok(ConnectionFacts {
        endpoint,
        runtime,
        advisory,
        effective_max_connections,
        non_pgpool_connections: runtime.non_pgpool_connections(),
    })
}
// </HANDWRITE>

fn count_to_u32(value: i64) -> Result<u32, ConnectionDiscoveryError> {
    value
        .try_into()
        .map_err(|_| ConnectionDiscoveryError::InvalidConnectionCount(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advisory_values_can_only_reduce_runtime_capacity() {
        assert_eq!(effective_connection_limit(500, None, None), 500);
        assert_eq!(effective_connection_limit(500, Some(400), None), 400);
        assert_eq!(effective_connection_limit(500, None, Some(600)), 500);
        assert_eq!(effective_connection_limit(500, Some(450), Some(420)), 420);
    }

    #[test]
    fn provider_and_endpoint_role_are_independent_budget_keys() {
        let primary = RemoteEndpoint {
            name: "alloy-primary".into(),
            provider: EndpointProvider::AlloyDb,
            role: EndpointRole::Primary,
            configured_ceiling: None,
        };
        let read_pool = RemoteEndpoint {
            name: "alloy-read-pool".into(),
            provider: EndpointProvider::AlloyDb,
            role: EndpointRole::ReadPool,
            configured_ceiling: None,
        };
        assert_ne!(primary, read_pool);
        assert_eq!(primary.provider, EndpointProvider::AlloyDb);
    }

    #[test]
    fn non_pgpool_usage_is_saturating() {
        let facts = RuntimeConnectionFacts {
            max_connections: 100,
            total_connections: 12,
            pgpool_connections: 5,
        };
        assert_eq!(facts.non_pgpool_connections(), 7);
        assert_eq!(
            RuntimeConnectionFacts {
                pgpool_connections: 20,
                ..facts
            }
            .non_pgpool_connections(),
            0
        );
    }
}
// </HANDWRITE>
