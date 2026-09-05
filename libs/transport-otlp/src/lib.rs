//! Official OTLP transport shell.
//!
//! Products implement [`OtlpConsumer`]. This crate owns media negotiation,
//! official protobuf decoding, bounded gzip, and partial-success encoding.

use std::io::Read;

use async_trait::async_trait;
use prost::Message;
use serde_json::{json, Map, Value};
use thiserror::Error;
use tonic::{Request, Response, Status};

pub mod proto {
    pub use opentelemetry_proto::tonic::{
        collector::{
            logs::v1::{
                ExportLogsPartialSuccess, ExportLogsServiceRequest, ExportLogsServiceResponse,
            },
            metrics::v1::{
                ExportMetricsPartialSuccess, ExportMetricsServiceRequest,
                ExportMetricsServiceResponse,
            },
            trace::v1::{
                ExportTracePartialSuccess, ExportTraceServiceRequest, ExportTraceServiceResponse,
            },
        },
        common::v1::{
            any_value, AnyValue, ArrayValue, InstrumentationScope, KeyValue, KeyValueList,
        },
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
        metrics::v1::{
            exemplar, metric, number_data_point, Exemplar, Gauge, Histogram, HistogramDataPoint,
            Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
        },
        resource::v1::Resource,
        trace::v1::{ResourceSpans, ScopeSpans, Span},
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OtlpSignal {
    Logs,
    Metrics,
    Traces,
}

impl OtlpSignal {
    pub fn rejected_json_field(self) -> &'static str {
        match self {
            Self::Logs => "rejectedLogRecords",
            Self::Metrics => "rejectedDataPoints",
            Self::Traces => "rejectedSpans",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OtlpMediaType {
    Json,
    Protobuf,
}

impl OtlpMediaType {
    pub fn parse(value: Option<&str>) -> Result<Self> {
        let value = value.unwrap_or("application/json");
        let media = value.split(';').next().unwrap_or(value).trim();
        match media {
            "application/json" => Ok(Self::Json),
            "application/x-protobuf" | "application/protobuf" => Ok(Self::Protobuf),
            other => Err(TransportError::UnsupportedMediaType {
                media_type: other.to_string(),
            }),
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            Self::Json => "application/json",
            Self::Protobuf => "application/x-protobuf",
        }
    }
}

#[derive(Clone, Debug)]
pub enum DecodedPayload {
    Logs(proto::ExportLogsServiceRequest),
    Metrics(proto::ExportMetricsServiceRequest),
    Traces(proto::ExportTraceServiceRequest),
    Json { signal: OtlpSignal, value: Value },
}

impl DecodedPayload {
    pub fn signal(&self) -> OtlpSignal {
        match self {
            Self::Logs(_) => OtlpSignal::Logs,
            Self::Metrics(_) => OtlpSignal::Metrics,
            Self::Traces(_) => OtlpSignal::Traces,
            Self::Json { signal, .. } => *signal,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PartialSuccess {
    pub rejected_items: usize,
    pub error_message: String,
}

impl PartialSuccess {
    pub fn new(rejected_items: usize, error_message: impl Into<String>) -> Self {
        Self {
            rejected_items,
            error_message: error_message.into(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.rejected_items == 0 && self.error_message.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedOtlpResponse {
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("unsupported OTLP content-type `{media_type}`")]
    UnsupportedMediaType { media_type: String },
    #[error("unsupported OTLP content-encoding `{encoding}`")]
    UnsupportedContentEncoding { encoding: String },
    #[error("OTLP decoded body exceeds {maximum_bytes} bytes")]
    DecodedBodyTooLarge { maximum_bytes: usize },
    #[error("decode OTLP {signal:?} JSON: {message}")]
    InvalidJson { signal: OtlpSignal, message: String },
    #[error("decode OTLP {signal:?} protobuf: {message}")]
    InvalidProtobuf { signal: OtlpSignal, message: String },
    #[error("decode OTLP gzip body: {message}")]
    InvalidGzip { message: String },
    #[error("encode OTLP response: {message}")]
    Encode { message: String },
    #[error("OTLP consumer failed: {message}")]
    Consumer { message: String },
}

pub type Result<T> = std::result::Result<T, TransportError>;

pub fn decode_payload(
    signal: OtlpSignal,
    media_type: OtlpMediaType,
    body: &[u8],
) -> Result<DecodedPayload> {
    match media_type {
        OtlpMediaType::Json => serde_json::from_slice(body)
            .map(|value| DecodedPayload::Json { signal, value })
            .map_err(|error| TransportError::InvalidJson {
                signal,
                message: error.to_string(),
            }),
        OtlpMediaType::Protobuf => match signal {
            OtlpSignal::Logs => proto::ExportLogsServiceRequest::decode(body)
                .map(DecodedPayload::Logs)
                .map_err(|error| TransportError::InvalidProtobuf {
                    signal,
                    message: error.to_string(),
                }),
            OtlpSignal::Metrics => proto::ExportMetricsServiceRequest::decode(body)
                .map(DecodedPayload::Metrics)
                .map_err(|error| TransportError::InvalidProtobuf {
                    signal,
                    message: error.to_string(),
                }),
            OtlpSignal::Traces => proto::ExportTraceServiceRequest::decode(body)
                .map(DecodedPayload::Traces)
                .map_err(|error| TransportError::InvalidProtobuf {
                    signal,
                    message: error.to_string(),
                }),
        },
    }
}

pub fn encode_response(
    signal: OtlpSignal,
    media_type: OtlpMediaType,
    partial: &PartialSuccess,
) -> Result<EncodedOtlpResponse> {
    let body = match media_type {
        OtlpMediaType::Json => {
            let value = if partial.is_empty() {
                json!({})
            } else {
                let mut fields = Map::new();
                fields.insert(
                    signal.rejected_json_field().to_string(),
                    json!(partial.rejected_items),
                );
                fields.insert("errorMessage".to_string(), json!(partial.error_message));
                json!({"partialSuccess": fields})
            };
            serde_json::to_vec(&value).map_err(|error| TransportError::Encode {
                message: error.to_string(),
            })?
        }
        OtlpMediaType::Protobuf => match signal {
            OtlpSignal::Logs => proto::ExportLogsServiceResponse {
                partial_success: (!partial.is_empty()).then_some(proto::ExportLogsPartialSuccess {
                    rejected_log_records: partial.rejected_items as i64,
                    error_message: partial.error_message.clone(),
                }),
            }
            .encode_to_vec(),
            OtlpSignal::Metrics => proto::ExportMetricsServiceResponse {
                partial_success: (!partial.is_empty()).then_some(
                    proto::ExportMetricsPartialSuccess {
                        rejected_data_points: partial.rejected_items as i64,
                        error_message: partial.error_message.clone(),
                    },
                ),
            }
            .encode_to_vec(),
            OtlpSignal::Traces => proto::ExportTraceServiceResponse {
                partial_success: (!partial.is_empty()).then_some(
                    proto::ExportTracePartialSuccess {
                        rejected_spans: partial.rejected_items as i64,
                        error_message: partial.error_message.clone(),
                    },
                ),
            }
            .encode_to_vec(),
        },
    };
    Ok(EncodedOtlpResponse {
        content_type: media_type.content_type(),
        body,
    })
}

pub fn decode_content_encoding(
    content_encoding: Option<&str>,
    body: &[u8],
    maximum_bytes: usize,
) -> Result<Vec<u8>> {
    match content_encoding
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        None | Some("identity") => {
            if body.len() > maximum_bytes {
                return Err(TransportError::DecodedBodyTooLarge { maximum_bytes });
            }
            Ok(body.to_vec())
        }
        Some("gzip") => {
            let mut output = Vec::new();
            flate2::read::GzDecoder::new(body)
                .take(maximum_bytes.saturating_add(1) as u64)
                .read_to_end(&mut output)
                .map_err(|error| TransportError::InvalidGzip {
                    message: error.to_string(),
                })?;
            if output.len() > maximum_bytes {
                return Err(TransportError::DecodedBodyTooLarge { maximum_bytes });
            }
            Ok(output)
        }
        Some(encoding) => Err(TransportError::UnsupportedContentEncoding {
            encoding: encoding.to_string(),
        }),
    }
}

#[async_trait]
pub trait OtlpConsumer: Send + Sync {
    async fn consume(&self, project: &str, payload: DecodedPayload) -> Result<PartialSuccess>;
}

pub async fn dispatch<C>(
    consumer: &C,
    project: &str,
    signal: OtlpSignal,
    media_type: OtlpMediaType,
    body: &[u8],
) -> Result<PartialSuccess>
where
    C: OtlpConsumer + ?Sized,
{
    let payload = decode_payload(signal, media_type, body)?;
    consumer.consume(project, payload).await
}

#[async_trait]
pub trait GrpcProjectAuthorizer: Send + Sync {
    async fn authorize(
        &self,
        metadata: &tonic::metadata::MetadataMap,
    ) -> std::result::Result<String, Status>;
}

struct GrpcService<C, A> {
    consumer: std::sync::Arc<C>,
    authorizer: std::sync::Arc<A>,
}

impl<C, A> Clone for GrpcService<C, A> {
    fn clone(&self) -> Self {
        Self {
            consumer: self.consumer.clone(),
            authorizer: self.authorizer.clone(),
        }
    }
}

impl<C, A> GrpcService<C, A>
where
    C: OtlpConsumer + 'static,
    A: GrpcProjectAuthorizer + 'static,
{
    async fn consume(
        &self,
        metadata: &tonic::metadata::MetadataMap,
        payload: DecodedPayload,
    ) -> std::result::Result<PartialSuccess, Status> {
        let project = self.authorizer.authorize(metadata).await?;
        self.consumer
            .consume(&project, payload)
            .await
            .map_err(status_from_error)
    }
}

#[tonic::async_trait]
impl<C, A> opentelemetry_proto::tonic::collector::logs::v1::logs_service_server::LogsService
    for GrpcService<C, A>
where
    C: OtlpConsumer + 'static,
    A: GrpcProjectAuthorizer + 'static,
{
    async fn export(
        &self,
        request: Request<proto::ExportLogsServiceRequest>,
    ) -> std::result::Result<Response<proto::ExportLogsServiceResponse>, Status> {
        let partial = self
            .consume(
                request.metadata(),
                DecodedPayload::Logs(request.get_ref().clone()),
            )
            .await?;
        Ok(Response::new(proto::ExportLogsServiceResponse {
            partial_success: (!partial.is_empty()).then_some(proto::ExportLogsPartialSuccess {
                rejected_log_records: partial.rejected_items as i64,
                error_message: partial.error_message,
            }),
        }))
    }
}

#[tonic::async_trait]
impl<C, A>
    opentelemetry_proto::tonic::collector::metrics::v1::metrics_service_server::MetricsService
    for GrpcService<C, A>
where
    C: OtlpConsumer + 'static,
    A: GrpcProjectAuthorizer + 'static,
{
    async fn export(
        &self,
        request: Request<proto::ExportMetricsServiceRequest>,
    ) -> std::result::Result<Response<proto::ExportMetricsServiceResponse>, Status> {
        let partial = self
            .consume(
                request.metadata(),
                DecodedPayload::Metrics(request.get_ref().clone()),
            )
            .await?;
        Ok(Response::new(proto::ExportMetricsServiceResponse {
            partial_success: (!partial.is_empty()).then_some(proto::ExportMetricsPartialSuccess {
                rejected_data_points: partial.rejected_items as i64,
                error_message: partial.error_message,
            }),
        }))
    }
}

#[tonic::async_trait]
impl<C, A> opentelemetry_proto::tonic::collector::trace::v1::trace_service_server::TraceService
    for GrpcService<C, A>
where
    C: OtlpConsumer + 'static,
    A: GrpcProjectAuthorizer + 'static,
{
    async fn export(
        &self,
        request: Request<proto::ExportTraceServiceRequest>,
    ) -> std::result::Result<Response<proto::ExportTraceServiceResponse>, Status> {
        let partial = self
            .consume(
                request.metadata(),
                DecodedPayload::Traces(request.get_ref().clone()),
            )
            .await?;
        Ok(Response::new(proto::ExportTraceServiceResponse {
            partial_success: (!partial.is_empty()).then_some(proto::ExportTracePartialSuccess {
                rejected_spans: partial.rejected_items as i64,
                error_message: partial.error_message,
            }),
        }))
    }
}

pub async fn serve_grpc<C, A, Shutdown>(
    listener: tokio::net::TcpListener,
    consumer: std::sync::Arc<C>,
    authorizer: std::sync::Arc<A>,
    maximum_message_bytes: usize,
    shutdown: Shutdown,
) -> anyhow::Result<()>
where
    C: OtlpConsumer + 'static,
    A: GrpcProjectAuthorizer + 'static,
    Shutdown: std::future::Future<Output = ()> + Send + 'static,
{
    use opentelemetry_proto::tonic::collector::{
        logs::v1::logs_service_server::LogsServiceServer,
        metrics::v1::metrics_service_server::MetricsServiceServer,
        trace::v1::trace_service_server::TraceServiceServer,
    };
    use tonic::codec::CompressionEncoding;

    let service = GrpcService {
        consumer,
        authorizer,
    };
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
        .serve_with_incoming_shutdown(
            tokio_stream::wrappers::TcpListenerStream::new(listener),
            shutdown,
        )
        .await?;
    Ok(())
}

fn status_from_error(error: TransportError) -> Status {
    match error {
        TransportError::UnsupportedMediaType { .. }
        | TransportError::UnsupportedContentEncoding { .. }
        | TransportError::InvalidJson { .. }
        | TransportError::InvalidProtobuf { .. }
        | TransportError::InvalidGzip { .. } => Status::invalid_argument(error.to_string()),
        TransportError::DecodedBodyTooLarge { .. } => Status::resource_exhausted(error.to_string()),
        TransportError::Encode { .. } => Status::internal(error.to_string()),
        TransportError::Consumer { .. } => Status::unavailable(error.to_string()),
    }
}
