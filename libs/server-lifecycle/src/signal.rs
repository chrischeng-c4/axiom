use std::time::Duration;

/// Resolve when SIGINT or, on Unix, SIGTERM arrives.
/// @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
pub async fn wait_shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        if let Ok(mut s) = signal(SignalKind::terminate()) {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("received SIGINT"),
        _ = terminate => tracing::info!("received SIGTERM"),
    }
}

/// Await shutdown, start draining, hold the grace window, then return.
pub async fn shutdown_with_drain(start_drain: impl FnOnce() + Send, grace: Duration) {
    wait_shutdown_signal().await;
    start_drain();
    tracing::info!(grace_secs = grace.as_secs(), "draining");
    tokio::time::sleep(grace).await;
    tracing::info!("grace expired; shutting down");
}
