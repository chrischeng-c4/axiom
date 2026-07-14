// HANDWRITE-BEGIN gap="sift-ingest-admission-limits" tracker="1658" reason="Enforce compressed and decoded sizes, event count and size, per-project quota, concurrency, draining, and overload errors."
use std::{
    collections::HashMap,
    io::Read,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::http::{HeaderMap, StatusCode};
use bytes::Bytes;
use flate2::read::GzDecoder;

#[derive(Clone, Debug)]
pub struct IngestLimits {
    pub max_compressed_body_bytes: usize,
    pub max_decoded_body_bytes: usize,
    pub max_event_bytes: usize,
    pub max_events_per_batch: usize,
    pub max_concurrent_requests_per_project: usize,
    pub max_items_per_project_window: usize,
    pub quota_window_secs: u64,
}

impl Default for IngestLimits {
    fn default() -> Self {
        Self {
            max_compressed_body_bytes: 1_048_576,
            max_decoded_body_bytes: 8_388_608,
            max_event_bytes: 262_144,
            max_events_per_batch: 1_000,
            max_concurrent_requests_per_project: 8,
            max_items_per_project_window: 10_000,
            quota_window_secs: 60,
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
}

#[derive(Debug)]
struct ProjectAdmission {
    in_flight: usize,
    used_items: usize,
    window_started: Instant,
}

#[derive(Debug)]
pub struct AdmissionController {
    limits: IngestLimits,
    projects: Arc<Mutex<HashMap<String, ProjectAdmission>>>,
}

impl AdmissionController {
    pub fn new(limits: IngestLimits) -> anyhow::Result<Self> {
        limits.validate()?;
        Ok(Self {
            limits,
            projects: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn limits(&self) -> &IngestLimits {
        &self.limits
    }

    pub fn decode_body(
        &self,
        headers: &HeaderMap,
        body: Bytes,
    ) -> Result<Vec<u8>, AdmissionError> {
        if body.len() > self.limits.max_compressed_body_bytes {
            return Err(AdmissionError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "compressed_body_too_large",
                format!(
                    "compressed request body exceeds {} bytes",
                    self.limits.max_compressed_body_bytes
                ),
                false,
                None,
            ));
        }
        let encoding = headers
            .get("content-encoding")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("identity")
            .trim()
            .to_ascii_lowercase();
        let decoded = match encoding.as_str() {
            "" | "identity" => body.to_vec(),
            "gzip" => {
                let mut decoder = GzDecoder::new(body.as_ref());
                let mut decoded = Vec::new();
                decoder
                    .by_ref()
                    .take(self.limits.max_decoded_body_bytes as u64 + 1)
                    .read_to_end(&mut decoded)
                    .map_err(|error| {
                        AdmissionError::invalid("invalid_gzip", error.to_string())
                    })?;
                decoded
            }
            _ => {
                return Err(AdmissionError::new(
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    "unsupported_content_encoding",
                    "content-encoding must be identity or gzip",
                    false,
                    None,
                ))
            }
        };
        if decoded.len() > self.limits.max_decoded_body_bytes {
            return Err(AdmissionError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "decoded_body_too_large",
                format!(
                    "decoded request body exceeds {} bytes",
                    self.limits.max_decoded_body_bytes
                ),
                false,
                None,
            ));
        }
        Ok(decoded)
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
        if draining {
            return Err(AdmissionError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "service_draining",
                "Sift is draining and cannot accept new writes",
                true,
                Some(1),
            ));
        }
        self.validate_item_count(item_count)?;
        let now = Instant::now();
        let window = Duration::from_secs(self.limits.quota_window_secs);
        let mut projects = self.projects.lock().expect("admission lock poisoned");
        let state = projects.entry(project.to_string()).or_insert(ProjectAdmission {
            in_flight: 0,
            used_items: 0,
            window_started: now,
        });
        if now.duration_since(state.window_started) >= window {
            state.used_items = 0;
            state.window_started = now;
        }
        if state.in_flight >= self.limits.max_concurrent_requests_per_project {
            return Err(AdmissionError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "project_concurrency_exceeded",
                "project has too many concurrent ingest requests",
                true,
                Some(1),
            ));
        }
        if state.used_items.saturating_add(item_count)
            > self.limits.max_items_per_project_window
        {
            let elapsed = now.duration_since(state.window_started).as_secs();
            return Err(AdmissionError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "project_quota_exceeded",
                "project ingest quota exceeded for the current window",
                true,
                Some(self.limits.quota_window_secs.saturating_sub(elapsed).max(1)),
            ));
        }
        state.in_flight += 1;
        state.used_items += item_count;
        Ok(AdmissionPermit {
            project: project.to_string(),
            projects: self.projects.clone(),
        })
    }
}

pub struct AdmissionPermit {
    project: String,
    projects: Arc<Mutex<HashMap<String, ProjectAdmission>>>,
}

impl Drop for AdmissionPermit {
    fn drop(&mut self) {
        if let Some(state) = self
            .projects
            .lock()
            .expect("admission lock poisoned")
            .get_mut(&self.project)
        {
            state.in_flight = state.in_flight.saturating_sub(1);
        }
    }
}
// HANDWRITE-END
