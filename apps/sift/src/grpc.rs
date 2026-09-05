use std::{future::Future, sync::Arc};

use axum::http::{HeaderMap, HeaderValue};
use opentelemetry_proto::tonic::collector::{
    logs::v1::{
        logs_service_client::LogsServiceClient,
        logs_service_server::{LogsService, LogsServiceServer},
        ExportLogsServiceRequest, ExportLogsServiceResponse,
    },
    metrics::v1::{
        metrics_service_client::MetricsServiceClient,
        metrics_service_server::{MetricsService, MetricsServiceServer},
        ExportMetricsServiceRequest, ExportMetricsServiceResponse,
    },
    trace::v1::{
        trace_service_client::TraceServiceClient,
        trace_service_server::{TraceService, TraceServiceServer},
        ExportTraceServiceRequest, ExportTraceServiceResponse,
    },
};
use service_auth::{AuthError, Role};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{codec::CompressionEncoding, transport::Channel, Request, Response, Status};

use crate::{auth::SiftVerifier, ingest::otlp, ServiceState};

#[derive(Clone)]
struct SiftGrpcConsumer {
    state: Arc<ServiceState>,
}

#[derive(Clone)]
struct SiftGrpcAuthorizer {
    verifier: Arc<SiftVerifier>,
}

#[derive(Clone)]
struct OtlpGrpcProxy {
    channel: Channel,
    verifier: Arc<SiftVerifier>,
}

pub async fn serve<F>(
    listener: tokio::net::TcpListener,
    state: Arc<ServiceState>,
    verifier: Arc<SiftVerifier>,
    shutdown: F,
) -> anyhow::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    let maximum = state.admission.limits().max_decoded_body_bytes;
    transport_otlp::serve_grpc(
        listener,
        Arc::new(SiftGrpcConsumer { state }),
        Arc::new(SiftGrpcAuthorizer { verifier }),
        maximum,
        shutdown,
    )
    .await
}

/// Serve the public OTLP/gRPC port while keeping the durable append boundary
/// in the replicated store role.
pub async fn serve_proxy<F>(
    listener: tokio::net::TcpListener,
    store_endpoint: &str,
    verifier: Arc<SiftVerifier>,
    maximum_message_bytes: usize,
    shutdown: F,
) -> anyhow::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    let channel =
        tonic::transport::Endpoint::from_shared(store_endpoint.to_string())?.connect_lazy();
    let service = OtlpGrpcProxy { channel, verifier };
    tonic::transport::Server::builder()
        .add_service(
            LogsServiceServer::new(service.clone())
                .accept_compressed(CompressionEncoding::Gzip)
                .send_compressed(CompressionEncoding::Gzip)
                .max_decoding_message_size(maximum_message_bytes),
        )
        .add_service(
            MetricsServiceServer::new(service.clone())
                .accept_compressed(CompressionEncoding::Gzip)
                .send_compressed(CompressionEncoding::Gzip)
                .max_decoding_message_size(maximum_message_bytes),
        )
        .add_service(
            TraceServiceServer::new(service)
                .accept_compressed(CompressionEncoding::Gzip)
                .send_compressed(CompressionEncoding::Gzip)
                .max_decoding_message_size(maximum_message_bytes),
        )
        .serve_with_incoming_shutdown(TcpListenerStream::new(listener), shutdown)
        .await?;
    Ok(())
}

#[tonic::async_trait]
impl LogsService for OtlpGrpcProxy {
    async fn export(
        &self,
        mut request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        authorize(request.metadata(), &self.verifier).await?;
        strip_compression_metadata(request.metadata_mut());
        let response = LogsServiceClient::new(self.channel.clone())
            .send_compressed(CompressionEncoding::Gzip)
            .accept_compressed(CompressionEncoding::Gzip)
            .export(request)
            .await?;
        Ok(Response::new(response.into_inner()))
    }
}

#[tonic::async_trait]
impl MetricsService for OtlpGrpcProxy {
    async fn export(
        &self,
        mut request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        authorize(request.metadata(), &self.verifier).await?;
        strip_compression_metadata(request.metadata_mut());
        let response = MetricsServiceClient::new(self.channel.clone())
            .send_compressed(CompressionEncoding::Gzip)
            .accept_compressed(CompressionEncoding::Gzip)
            .export(request)
            .await?;
        Ok(Response::new(response.into_inner()))
    }
}

#[tonic::async_trait]
impl TraceService for OtlpGrpcProxy {
    async fn export(
        &self,
        mut request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        authorize(request.metadata(), &self.verifier).await?;
        strip_compression_metadata(request.metadata_mut());
        let response = TraceServiceClient::new(self.channel.clone())
            .send_compressed(CompressionEncoding::Gzip)
            .accept_compressed(CompressionEncoding::Gzip)
            .export(request)
            .await?;
        Ok(Response::new(response.into_inner()))
    }
}

fn strip_compression_metadata(metadata: &mut tonic::metadata::MetadataMap) {
    metadata.remove("grpc-encoding");
    metadata.remove("grpc-accept-encoding");
}

#[tonic::async_trait]
impl transport_otlp::GrpcProjectAuthorizer for SiftGrpcAuthorizer {
    async fn authorize(&self, metadata: &tonic::metadata::MetadataMap) -> Result<String, Status> {
        authorize(metadata, &self.verifier).await
    }
}

#[tonic::async_trait]
impl transport_otlp::OtlpConsumer for SiftGrpcConsumer {
    async fn consume(
        &self,
        project: &str,
        payload: transport_otlp::DecodedPayload,
    ) -> transport_otlp::Result<transport_otlp::PartialSuccess> {
        let decoded = otlp::normalize_payload(payload, project).map_err(|error| {
            transport_otlp::TransportError::Consumer {
                message: error.to_string(),
            }
        })?;
        let _permit = self
            .state
            .admission
            .acquire(project, decoded.item_count(), self.state.is_draining())
            .map_err(|error| transport_otlp::TransportError::Consumer {
                message: error.message,
            })?;
        let mut rejected = 0usize;
        let mut messages = Vec::new();
        let mut accepted = Vec::new();
        let mut accepted_bytes = 0usize;
        for item in decoded.items {
            let event = match item {
                Ok(event) => event,
                Err(error) => {
                    rejected += 1;
                    push_message(&mut messages, error.message);
                    continue;
                }
            };
            if event.project != project {
                rejected += 1;
                push_message(
                    &mut messages,
                    format!(
                        "event project `{}` does not match admitted project `{project}`",
                        event.project
                    ),
                );
                continue;
            }
            let event_bytes = serde_json::to_vec(&event)
                .map(|bytes| bytes.len())
                .unwrap_or(usize::MAX);
            if let Err(error) = self.state.admission.validate_event_bytes(event_bytes) {
                rejected += 1;
                push_message(&mut messages, error.message);
                continue;
            }
            if let Some(message) = crate::retention_rejection(&event) {
                rejected += 1;
                push_message(&mut messages, message);
                continue;
            }
            accepted_bytes = accepted_bytes.saturating_add(event_bytes);
            accepted.push(event);
        }
        self.state
            .ensure_local_capacity(accepted_bytes)
            .map_err(|error| transport_otlp::TransportError::Consumer {
                message: format!("{}; retry after 5 seconds", error.message),
            })?;
        let accepted_count = accepted.len();
        if let Err(error) = self.state.append_events(accepted).await {
            rejected += accepted_count;
            push_message(
                &mut messages,
                format!("durable batch append failed: {error}"),
            );
        }
        Ok(transport_otlp::PartialSuccess::new(
            rejected,
            messages.join("; "),
        ))
    }
}

async fn authorize(
    metadata: &tonic::metadata::MetadataMap,
    verifier: &SiftVerifier,
) -> Result<String, Status> {
    let project = metadata
        .get("x-sift-project")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| Status::invalid_argument("x-sift-project metadata is required"))?
        .to_string();
    let mut headers = HeaderMap::new();
    if let Some(authorization) = metadata
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        headers.insert(
            "authorization",
            HeaderValue::from_str(authorization)
                .map_err(|_| Status::unauthenticated("authorization metadata is invalid"))?,
        );
    }
    let principal = verifier
        .authenticate_project(&headers, &project, Role::Write)
        .await
        .map_err(auth_status)?;
    crate::authorize_project(Some(&principal), &project)
        .map_err(|error| Status::permission_denied(error.message))?;
    Ok(project)
}

fn push_message(messages: &mut Vec<String>, message: String) {
    if messages.len() < 8 {
        messages.push(message);
    }
}

fn auth_status(error: AuthError) -> Status {
    match error {
        AuthError::Unauthenticated => Status::unauthenticated("valid bearer token required"),
        AuthError::Forbidden(message) => Status::permission_denied(message),
        AuthError::Unavailable(message) => Status::unavailable(message),
    }
}
