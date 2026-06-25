// HANDWRITE-BEGIN gap="missing-generator:source:61602ce5" tracker="pending-tracker" reason="Shared helper: multiplex a tonic gRPC service and an axum REST router on one TcpListener, routing by the application/grpc content-type, served via hyper-util auto Builder (h1 + h2)."
//! Serve a tonic gRPC service and an axum REST router on ONE port.
//!
//! The REST router owns its `/v1` / `/v2` routes; the tonic service is mounted
//! as the router's **fallback**. A gRPC request path is `/package.Service/Method`
//! (plus `content-type: application/grpc`), which never matches a REST route, so
//! it falls through to the gRPC service. `axum::serve` drives hyper's auto
//! connection builder, which negotiates HTTP/1 for the REST clients and HTTP/2
//! (h2c) for the gRPC clients on the same listener — so one
//! `CLOUD_TASKS_EMULATOR_HOST` / `CLOUD_SCHEDULER_EMULATOR_HOST` serves both
//! protocols.
//!
//! @spec projects/vat/tech-design/logic/built-in-cloud-tasks-cloud-scheduler-emulators.md#logic

use anyhow::{Context, Result};
use axum::Router;

/// Serve `rest` with `grpc` (a tonic-generated `*Server`) as its fallback on
/// `host_port` until the process is killed.
///
/// Any tonic server qualifies: it is a `Service<Request, Error = Infallible>`
/// whose response body is a tonic `BoxBody` (which axum renders via
/// `IntoResponse`).
pub async fn serve<S>(host_port: &str, rest: Router, grpc: S) -> Result<()>
where
    S: tonic::codegen::Service<axum::extract::Request, Error = std::convert::Infallible>
        + Clone
        + Send
        + Sync
        + 'static,
    S::Response: axum::response::IntoResponse,
    S::Future: Send + 'static,
{
    let app = rest.fallback_service(grpc);
    let listener = tokio::net::TcpListener::bind(host_port)
        .await
        .with_context(|| format!("bind grpc+rest emulator on {host_port}"))?;
    axum::serve(listener, app)
        .await
        .context("serve grpc+rest emulator")?;
    Ok(())
}
// HANDWRITE-END
