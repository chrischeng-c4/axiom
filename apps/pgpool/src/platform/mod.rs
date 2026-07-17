// SPEC-MANAGED: apps/pgpool/tech-design/semantic/pgpool-runtime-connection-limit-discovery.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-platform-discovery" tracker="#1570" reason="Provider adapter module generation is not available.">
//! Provider adapters above the PostgreSQL pooler core.

mod discovery;

pub use discovery::{
    discover_connection_facts, discovery_tls_mode, effective_connection_limit,
    ConnectionDiscoveryError, ConnectionFacts, DiscoveryTlsMode, EndpointProvider, EndpointRole,
    ProviderAdvisory, RemoteEndpoint, RuntimeConnectionFacts,
};
// </HANDWRITE>
