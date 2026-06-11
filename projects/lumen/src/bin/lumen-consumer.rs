// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-src-bin-lumen-consumer-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! Placeholder consumer binary. lumen does not bundle an event-pipeline
//! subscriber — real adapters (AlloyDB CDC, Postgres logical replication,
//! Kafka, application-direct) live under `examples/`. This binary boots
//! just enough to anchor the K8s manifest and report shard routing.

use anyhow::{Context, Result};
use std::env;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let shard_count: u32 = env::var("SHARD_COUNT")
        .context("SHARD_COUNT not set")?
        .parse()
        .context("SHARD_COUNT must be a u32")?;
    let lumen_host =
        env::var("LUMEN_HOST").unwrap_or_else(|_| "lumen.lumen.svc.cluster.local".into());

    let router = lumen::consumer::ShardRouter {
        shard_count,
        lumen_host,
    };
    tracing::info!(
        shard_count = router.shard_count,
        host = %router.lumen_host,
        "consumer placeholder running — replace with a real adapter from examples/"
    );
    std::future::pending::<()>().await;
    Ok(())
}

// </HANDWRITE>
