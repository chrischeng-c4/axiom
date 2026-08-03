use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

use server_lifecycle::DrainController;
use service_http::{
    serve, server_timing_middleware, shutdown_with_drain, standard_probe_routes, trace_layer,
    ReadinessHook,
};

#[derive(utoipa::OpenApi)]
#[openapi(info(title = "minimal_service", description = "minimal service example"))]
struct ApiDoc;

fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi as _;
    ApiDoc::openapi()
}

fn assert_readiness_hook<T: ReadinessHook>(_: &T) {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let readiness = Arc::new(DrainController::new());
    assert_readiness_hook(&*readiness);

    let app = standard_probe_routes(readiness.clone(), None, openapi)
        .layer(trace_layer())
        .layer(axum::middleware::from_fn(server_timing_middleware));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    println!("LISTENING {addr}");
    std::io::stdout().flush()?;

    let readiness_drain = readiness.clone();
    serve(
        listener,
        app,
        shutdown_with_drain(
            move || readiness_drain.start_drain(),
            Duration::from_secs(2),
        ),
    )
    .await;

    println!("SHUTDOWN complete");
    std::io::stdout().flush()?;

    Ok(())
}
