// SPEC-MANAGED: projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-streaming-subscrib.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:695511a3" tracker="pending-tracker" reason="relay-server binary entrypoint: load config, build the app, serve h2c."
//! `relay-server` — serve the relay HTTP/2 (h2c) transport for one shard.

use relay::server::{router, AppState};
use relay::server_config::RelayServerConfig;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // v1: defaults. Config loading (env / file) is a later concern.
    let config = RelayServerConfig::default();
    let bind = config.bind.clone();

    let app = router(AppState::new(config));
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    eprintln!("relay-server listening on {} (h2c)", listener.local_addr()?);
    axum::serve(listener, app).await
}
// HANDWRITE-END
