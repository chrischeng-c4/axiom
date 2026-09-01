// HANDWRITE-BEGIN gap="sift-ingest-admission-limits" tracker="1658" reason="Enforce compressed and decoded sizes, event count and size, per-project quota, concurrency, draining, and overload errors."
use std::time::Duration;

use axum::http::{HeaderMap, StatusCode};
use bytes::Bytes;

#[derive(Clone, Debug)]
pub struct IngestLimits {
    pub max_compressed_body_bytes: usize,
    pub max_decoded_body_bytes: usize,
    pub max_event_bytes: usize,
    pub max_events_per_batch: usize,
    pub max_concurrent_requests_per_project: usize,
    pub max_items_per_project_window: usize,
    pub quota_window_secs: u64,
    pub max_local_storage_bytes: u64,
    pub min_local_free_bytes: u64,
}

impl Default for IngestLimits {
    fn default() -> Self {
        Self {
            max_compressed_body_bytes: 1_048_576,
            max_decoded_body_bytes: 8_388_608,
            max_event_bytes: 262_144,
            max_events_per_batch: 1_000,
            max_concurrent_requests_per_project: 32,
            max_items_per_project_window: 720_000,
            quota_window_secs: 60,
            max_local_storage_bytes: 50 * 1024 * 1024 * 1024,
            min_local_free_bytes: 1024 * 1024 * 1024,
        }
    }
}

impl IngestLimits {
    pub fn from_env() -> anyhow::Result<Self> {
        let defaults = Self::default();
        Ok(Self {
            max_compressed_body_bytes: env_usize(
                "SIFT_MAX_COMPRESSED_BODY_BYTES",
                defaults.max_compressed_body_bytes,
            )?,
            max_decoded_body_bytes: env_usize(
                "SIFT_MAX_DECODED_BODY_BYTES",
                defaults.max_decoded_body_bytes,
            )?,
            max_event_bytes: env_usize("SIFT_MAX_EVENT_BYTES", defaults.max_event_bytes)?,
            max_events_per_batch: env_usize(
                "SIFT_MAX_EVENTS_PER_BATCH",
                defaults.max_events_per_batch,
            )?,
            max_concurrent_requests_per_project: env_usize(
                "SIFT_MAX_CONCURRENT_INGEST_PER_PROJECT",
                defaults.max_concurrent_requests_per_project,
            )?,
            max_items_per_project_window: env_usize(
                "SIFT_MAX_INGEST_ITEMS_PER_PROJECT_WINDOW",
                defaults.max_items_per_project_window,
            )?,
            quota_window_secs: env_u64(
                "SIFT_INGEST_QUOTA_WINDOW_SECS",
                defaults.quota_window_secs,
            )?,
            max_local_storage_bytes: env_u64(
                "SIFT_MAX_LOCAL_STORAGE_BYTES",
                defaults.max_local_storage_bytes,
            )?,
            min_local_free_bytes: env_u64(
                "SIFT_MIN_LOCAL_FREE_BYTES",
                defaults.min_local_free_bytes,
            )?,
        })
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.max_compressed_body_bytes == 0
            || self.max_decoded_body_bytes == 0
            || self.max_event_bytes == 0
            || self.max_events_per_batch == 0
            || self.max_concurrent_requests_per_project == 0
            || self.max_items_per_project_window == 0
            || self.quota_window_secs == 0
            || self.max_local_storage_bytes == 0
            || self.min_local_free_bytes == 0
        {
            anyhow::bail!("all ingest limits must be greater than zero");
        }
        if self.max_compressed_body_bytes > self.max_decoded_body_bytes {
            anyhow::bail!("compressed body limit must not exceed decoded body limit");
        }
        Ok(())
    }
}

fn env_usize(name: &str, default: usize) -> anyhow::Result<usize> {
    match std::env::var(name) {
        Ok(value) => Ok(value.parse()?),
        Err(std::env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(error.into()),
    }
}

fn env_u64(name: &str, default: u64) -> anyhow::Result<u64> {
    match std::env::var(name) {
        Ok(value) => Ok(value.parse()?),
        Err(std::env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(error.into()),
    }
}

#[derive(Clone, Debug)]
pub struct AdmissionError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
    pub retryable: bool,
    pub retry_after_secs: Option<u64>,
}

impl AdmissionError {
    fn new(
        status: StatusCode,
        code: &'static str,
        message: impl Into<String>,
        retryable: bool,
        retry_after_secs: Option<u64>,
    ) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            retryable,
            retry_after_secs,
        }
    }

    pub fn invalid(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message, false, None)
    }

    pub fn local_storage_backpressure(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "local_storage_backpressure",
            message,
            true,
            Some(5),
        )
    }
}

#[derive(Debug)]
pub struct AdmissionController {
    limits: IngestLimits,
    projects: service_http::WeightedAdmission<String>,
}

impl AdmissionController {
    pub fn new(limits: IngestLimits) -> anyhow::Result<Self> {
        limits.validate()?;
        let policy = service_http::WeightedAdmissionConfig::new(
            limits.max_concurrent_requests_per_project,
            limits.max_items_per_project_window,
            Duration::from_secs(limits.quota_window_secs),
            65_536,
        )?;
        Ok(Self {
            limits,
            projects: service_http::WeightedAdmission::new(policy),
        })
    }

    pub fn limits(&self) -> &IngestLimits {
        &self.limits
    }

    pub fn decode_body(&self, headers: &HeaderMap, body: Bytes) -> Result<Vec<u8>, AdmissionError> {
        let limits = service_http::ContentDecodeLimits::new(
            self.limits.max_compressed_body_bytes,
            self.limits.max_decoded_body_bytes,
        )
        .expect("validated Sift ingest limits are positive");
        service_http::decode_request_body(headers, body.as_ref(), limits).map_err(|error| {
            use service_http::ContentDecodeErrorKind as Kind;
            let (status, code) = match error.kind() {
                Kind::CompressedBodyTooLarge => {
                    (StatusCode::PAYLOAD_TOO_LARGE, "compressed_body_too_large")
                }
                Kind::DecodedBodyTooLarge => {
                    (StatusCode::PAYLOAD_TOO_LARGE, "decoded_body_too_large")
                }
                Kind::InvalidGzip => (StatusCode::BAD_REQUEST, "invalid_gzip"),
                Kind::UnsupportedContentEncoding => (
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    "unsupported_content_encoding",
                ),
            };
            AdmissionError::new(status, code, error.to_string(), false, None)
        })
    }

    pub fn validate_item_count(&self, item_count: usize) -> Result<(), AdmissionError> {
        if item_count == 0 {
            return Err(AdmissionError::invalid(
                "empty_batch",
                "ingest request must contain at least one item",
            ));
        }
        if item_count > self.limits.max_events_per_batch {
            return Err(AdmissionError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "batch_too_large",
                format!(
                    "batch contains {item_count} items; maximum is {}",
                    self.limits.max_events_per_batch
                ),
                false,
                None,
            ));
        }
        Ok(())
    }

    pub fn validate_event_bytes(&self, bytes: usize) -> Result<(), AdmissionError> {
        if bytes > self.limits.max_event_bytes {
            return Err(AdmissionError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "event_too_large",
                format!(
                    "event is {bytes} bytes; maximum is {}",
                    self.limits.max_event_bytes
                ),
                false,
                None,
            ));
        }
        Ok(())
    }

    pub fn acquire(
        &self,
        project: &str,
        item_count: usize,
        draining: bool,
    ) -> Result<AdmissionPermit, AdmissionError> {
        if project.trim().is_empty() {
            return Err(AdmissionError::invalid(
                "missing_project",
                "x-sift-project or an event project is required",
            ));
        }
        self.validate_item_count(item_count)?;
        self.projects
            .acquire(project.to_string(), item_count, draining)
            .map_err(|error| match error {
                service_http::WeightedAdmissionError::Draining => AdmissionError::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "service_draining",
                    "Sift is draining and cannot accept new writes",
                    true,
                    Some(1),
                ),
                service_http::WeightedAdmissionError::ConcurrencyExceeded { .. } => {
                    AdmissionError::new(
                        StatusCode::TOO_MANY_REQUESTS,
                        "project_concurrency_exceeded",
                        "project has too many concurrent ingest requests",
                        true,
                        Some(1),
                    )
                }
                service_http::WeightedAdmissionError::QuotaExceeded { retry_after, .. } => {
                    AdmissionError::new(
                        StatusCode::TOO_MANY_REQUESTS,
                        "project_quota_exceeded",
                        "project ingest quota exceeded for the current window",
                        true,
                        Some(
                            retry_after
                                .as_secs()
                                .saturating_add(u64::from(retry_after.subsec_nanos() > 0))
                                .max(1),
                        ),
                    )
                }
                service_http::WeightedAdmissionError::KeyLimitExceeded => AdmissionError::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "admission_capacity_exceeded",
                    "too many projects are active in the ingest admission window",
                    true,
                    Some(1),
                ),
                service_http::WeightedAdmissionError::ZeroWeight => {
                    AdmissionError::invalid("empty_batch", "ingest request must not be empty")
                }
            })
    }
}

pub type AdmissionPermit = service_http::ConcurrencyLease<String>;
// HANDWRITE-END
