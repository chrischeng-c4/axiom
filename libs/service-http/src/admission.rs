// HANDWRITE-BEGIN gap="missing-generator:logic:72398ba0" tracker="#1642" reason="Bounded deterministic opaque-key token buckets, redacted decision hooks, standard Retry-After rejection, and reusable axum middleware."
//! Bounded in-process request admission for HTTP services.
//!
//! Applications select endpoint classes and opaque request keys. This module
//! hashes a key before it reaches retained state, applies the configured token
//! bucket, and emits only class/outcome metadata to observers.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::ApiErr;

/// One endpoint-class token-bucket policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionPolicy {
    capacity: u32,
    refill_window: Duration,
    max_keys: usize,
}

impl AdmissionPolicy {
    pub fn new(
        capacity: u32,
        refill_window: Duration,
        max_keys: usize,
    ) -> Result<Self, AdmissionPolicyError> {
        if capacity == 0 {
            return Err(AdmissionPolicyError::ZeroCapacity);
        }
        if refill_window.is_zero() {
            return Err(AdmissionPolicyError::ZeroRefillWindow);
        }
        if max_keys == 0 {
            return Err(AdmissionPolicyError::ZeroMaxKeys);
        }
        Ok(Self {
            capacity,
            refill_window,
            max_keys,
        })
    }

    pub fn capacity(self) -> u32 {
        self.capacity
    }

    pub fn refill_window(self) -> Duration {
        self.refill_window
    }

    pub fn max_keys(self) -> usize {
        self.max_keys
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionPolicyError {
    ZeroCapacity,
    ZeroRefillWindow,
    ZeroMaxKeys,
}

impl fmt::Display for AdmissionPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ZeroCapacity => "admission capacity must be greater than zero",
            Self::ZeroRefillWindow => "admission refill window must be greater than zero",
            Self::ZeroMaxKeys => "admission max_keys must be greater than zero",
        };
        f.write_str(message)
    }
}

impl Error for AdmissionPolicyError {}

/// A request classification whose key is intentionally not `Debug` or
/// serializable. It exists only until [`AdmissionController::admit`] hashes it.
pub struct AdmissionInput {
    class: String,
    key: Vec<u8>,
}

impl AdmissionInput {
    pub fn new(class: impl Into<String>, key: impl AsRef<[u8]>) -> Self {
        Self {
            class: class.into(),
            key: key.as_ref().to_vec(),
        }
    }

    pub fn class(&self) -> &str {
        &self.class
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionOutcome {
    Bypass,
    Allow,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AdmissionEvent {
    pub class: String,
    pub outcome: AdmissionOutcome,
    pub retry_after_ms: Option<u64>,
}

pub trait AdmissionObserver: Send + Sync {
    fn record(&self, event: &AdmissionEvent);
}

#[derive(Debug, Default)]
pub struct NoopAdmissionObserver;

impl AdmissionObserver for NoopAdmissionObserver {
    fn record(&self, _event: &AdmissionEvent) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionDecision {
    pub outcome: AdmissionOutcome,
    pub retry_after: Option<Duration>,
}

impl AdmissionDecision {
    pub fn is_allowed(&self) -> bool {
        self.outcome != AdmissionOutcome::Deny
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BucketKey {
    class: String,
    fingerprint: [u8; 32],
}

#[derive(Debug)]
struct Bucket {
    // Credits are measured in nanosecond-token units. One token costs one
    // refill-window worth of units; elapsed_ns * capacity refills exactly.
    credits: u128,
    last_ns: u128,
    last_seen: u64,
}

#[derive(Default)]
struct AdmissionState {
    buckets: HashMap<BucketKey, Bucket>,
    sequence: u64,
}

struct AdmissionInner {
    policies: HashMap<String, AdmissionPolicy>,
    state: Mutex<AdmissionState>,
    observer: Arc<dyn AdmissionObserver>,
    epoch: Instant,
}

// <HANDWRITE gap="missing-generator:service-http-admission-config" tracker="#1823" reason="Typed prefix-scoped admission configuration is hand-written pending codegen support.">
/// Cloneable shared admission controller. An empty policy set is disabled.
#[derive(Clone)]
pub struct AdmissionController(Arc<AdmissionInner>);

/// Validated, opt-in admission settings shared by service adopters.
///
/// The parser owns the common `<SVC>_ADMISSION_*` environment grammar while
/// callers retain their public route-class names when building a controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionConfig {
    read_capacity: Option<u32>,
    write_capacity: Option<u32>,
    admin_capacity: Option<u32>,
    refill_window: Duration,
    max_keys: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionConfigError {
    InvalidValue { key: String, value: String },
    OrphanedCommonSetting { key: String },
    InvalidPolicy { key: String, message: String },
}

impl fmt::Display for AdmissionConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidValue { key, value } => {
                write!(f, "{key} must be a positive integer, got {value:?}")
            }
            Self::OrphanedCommonSetting { key } => {
                write!(f, "{key} requires at least one admission class capacity")
            }
            Self::InvalidPolicy { key, message } => write!(f, "{key}: {message}"),
        }
    }
}

impl Error for AdmissionConfigError {}

impl AdmissionConfig {
    pub const DEFAULT_REFILL_SECS: u64 = 60;
    pub const DEFAULT_MAX_KEYS: usize = 1024;

    /// Parse configuration from process environment.
    pub fn from_env(prefix: &str) -> Result<Self, AdmissionConfigError> {
        Self::from_lookup(prefix, |key| std::env::var(key).ok())
    }

    /// Parse configuration from an injected lookup seam. This keeps tests and
    /// embedding callers independent of process-global environment mutation.
    pub fn from_lookup<F>(prefix: &str, lookup: F) -> Result<Self, AdmissionConfigError>
    where
        F: Fn(&str) -> Option<String>,
    {
        let key = |suffix: &str| format!("{prefix}_ADMISSION_{suffix}");
        let read_key = key("READ_CAPACITY");
        let write_key = key("WRITE_CAPACITY");
        let admin_key = key("ADMIN_CAPACITY");
        let refill_key = key("REFILL_SECS");
        let max_keys_key = key("MAX_KEYS");
        let parse = |key: &str| match lookup(key) {
            None => Ok(None),
            Some(value) => value
                .parse::<u32>()
                .ok()
                .filter(|value| *value > 0)
                .map(Some)
                .ok_or_else(|| AdmissionConfigError::InvalidValue {
                    key: key.to_owned(),
                    value,
                }),
        };
        let read_capacity = parse(&read_key)?;
        let write_capacity = parse(&write_key)?;
        let admin_capacity = parse(&admin_key)?;
        let enabled =
            read_capacity.is_some() || write_capacity.is_some() || admin_capacity.is_some();
        let common_present = lookup(&refill_key).is_some() || lookup(&max_keys_key).is_some();
        if !enabled && common_present {
            return Err(AdmissionConfigError::OrphanedCommonSetting {
                key: if lookup(&refill_key).is_some() {
                    refill_key
                } else {
                    max_keys_key
                },
            });
        }
        let refill_secs = parse(&key("REFILL_SECS"))?
            .map(u64::from)
            .unwrap_or(Self::DEFAULT_REFILL_SECS);
        let max_keys = parse(&key("MAX_KEYS"))?
            .map(|value| value as usize)
            .unwrap_or(Self::DEFAULT_MAX_KEYS);
        let refill_window = Duration::from_secs(refill_secs);
        for (class_key, capacity) in [
            (&read_key, read_capacity),
            (&write_key, write_capacity),
            (&admin_key, admin_capacity),
        ] {
            if let Some(capacity) = capacity {
                AdmissionPolicy::new(capacity, refill_window, max_keys).map_err(|error| {
                    AdmissionConfigError::InvalidPolicy {
                        key: class_key.clone(),
                        message: error.to_string(),
                    }
                })?;
            }
        }
        Ok(Self {
            read_capacity,
            write_capacity,
            admin_capacity,
            refill_window,
            max_keys,
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.read_capacity.is_some()
            || self.write_capacity.is_some()
            || self.admin_capacity.is_some()
    }

    /// Build a controller using the consuming service's route-class names.
    pub fn controller(
        &self,
        read_class: impl Into<String>,
        write_class: impl Into<String>,
        admin_class: impl Into<String>,
    ) -> Option<AdmissionController> {
        let classes = [
            (read_class.into(), self.read_capacity),
            (write_class.into(), self.write_capacity),
            (admin_class.into(), self.admin_capacity),
        ];
        let policies = classes.into_iter().filter_map(|(class, capacity)| {
            capacity.map(|capacity| {
                let policy = AdmissionPolicy::new(capacity, self.refill_window, self.max_keys)
                    .expect("AdmissionConfig validates policies at construction");
                (class, policy)
            })
        });
        self.is_enabled()
            .then(|| AdmissionController::new(policies))
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use std::collections::BTreeMap;

    fn config(entries: &[(&str, &str)]) -> Result<AdmissionConfig, AdmissionConfigError> {
        let values = entries
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect::<BTreeMap<_, _>>();
        AdmissionConfig::from_lookup("TAPE", |key| values.get(key).cloned())
    }

    #[test]
    fn config_without_capacities_is_disabled() {
        let config = config(&[]).unwrap();
        assert!(!config.is_enabled());
        assert!(config
            .controller("tape.read", "tape.write", "tape.admin")
            .is_none());
    }

    #[test]
    fn config_builds_multi_class_controller() {
        let config = config(&[
            ("TAPE_ADMISSION_READ_CAPACITY", "2"),
            ("TAPE_ADMISSION_WRITE_CAPACITY", "1"),
            ("TAPE_ADMISSION_ADMIN_CAPACITY", "3"),
            ("TAPE_ADMISSION_REFILL_SECS", "10"),
            ("TAPE_ADMISSION_MAX_KEYS", "4"),
        ])
        .unwrap();
        let controller = config
            .controller("tape.read", "tape.write", "tape.admin")
            .unwrap();
        let input = AdmissionInput::new("tape.read", b"principal");
        assert_eq!(
            controller.admit_at(&input, Duration::ZERO).outcome,
            AdmissionOutcome::Allow
        );
        assert_eq!(
            controller.admit_at(&input, Duration::ZERO).outcome,
            AdmissionOutcome::Allow
        );
        assert_eq!(
            controller.admit_at(&input, Duration::ZERO).outcome,
            AdmissionOutcome::Deny
        );
        assert_eq!(controller.tracked_keys("tape.read"), 1);
    }

    #[test]
    fn config_rejects_invalid_or_orphaned_common_values() {
        let error = config(&[("TAPE_ADMISSION_READ_CAPACITY", "many")]).unwrap_err();
        assert!(error.to_string().contains("TAPE_ADMISSION_READ_CAPACITY"));
        let error = config(&[("TAPE_ADMISSION_REFILL_SECS", "10")]).unwrap_err();
        assert!(error.to_string().contains("TAPE_ADMISSION_REFILL_SECS"));
    }
}
// </HANDWRITE>

impl fmt::Debug for AdmissionController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AdmissionController")
            .field("classes", &self.0.policies.keys().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}

impl AdmissionController {
    pub fn new<I, S>(policies: I) -> Self
    where
        I: IntoIterator<Item = (S, AdmissionPolicy)>,
        S: Into<String>,
    {
        Self::with_observer(policies, Arc::new(NoopAdmissionObserver))
    }

    pub fn with_observer<I, S>(policies: I, observer: Arc<dyn AdmissionObserver>) -> Self
    where
        I: IntoIterator<Item = (S, AdmissionPolicy)>,
        S: Into<String>,
    {
        Self(Arc::new(AdmissionInner {
            policies: policies
                .into_iter()
                .map(|(class, policy)| (class.into(), policy))
                .collect(),
            state: Mutex::new(AdmissionState::default()),
            observer,
            epoch: Instant::now(),
        }))
    }

    pub fn is_enabled(&self) -> bool {
        !self.0.policies.is_empty()
    }

    pub fn admit(&self, input: &AdmissionInput) -> AdmissionDecision {
        self.admit_at(input, self.0.epoch.elapsed())
    }

    /// Deterministic clock seam for tests and simulations.
    pub fn admit_at(&self, input: &AdmissionInput, now: Duration) -> AdmissionDecision {
        let Some(policy) = self.0.policies.get(input.class()).copied() else {
            return self.finish(input.class(), AdmissionOutcome::Bypass, None);
        };

        let fingerprint: [u8; 32] = Sha256::digest(&input.key).into();
        let key = BucketKey {
            class: input.class.clone(),
            fingerprint,
        };
        let window_ns = policy.refill_window.as_nanos();
        let max_credits = window_ns * u128::from(policy.capacity);
        let now_ns = now.as_nanos();

        let (outcome, retry_after) = {
            let mut state = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
            state.sequence = state.sequence.wrapping_add(1);
            let sequence = state.sequence;

            if !state.buckets.contains_key(&key) {
                let class = input.class();
                let class_count = state
                    .buckets
                    .keys()
                    .filter(|existing| existing.class == class)
                    .count();
                if class_count >= policy.max_keys {
                    if let Some(evict) = state
                        .buckets
                        .iter()
                        .filter(|(existing, _)| existing.class == class)
                        .min_by_key(|(_, bucket)| bucket.last_seen)
                        .map(|(existing, _)| existing.clone())
                    {
                        state.buckets.remove(&evict);
                    }
                }
                state.buckets.insert(
                    key.clone(),
                    Bucket {
                        credits: max_credits,
                        last_ns: now_ns,
                        last_seen: sequence,
                    },
                );
            }

            let bucket = state.buckets.get_mut(&key).expect("bucket inserted");
            let elapsed = now_ns.saturating_sub(bucket.last_ns);
            bucket.credits = bucket
                .credits
                .saturating_add(elapsed.saturating_mul(u128::from(policy.capacity)))
                .min(max_credits);
            bucket.last_ns = now_ns;
            bucket.last_seen = sequence;

            if bucket.credits >= window_ns {
                bucket.credits -= window_ns;
                (AdmissionOutcome::Allow, None)
            } else {
                let missing = window_ns - bucket.credits;
                let capacity = u128::from(policy.capacity);
                let wait_ns = missing.div_ceil(capacity).max(1);
                let wait_ns = u64::try_from(wait_ns).unwrap_or(u64::MAX);
                (AdmissionOutcome::Deny, Some(Duration::from_nanos(wait_ns)))
            }
        };

        self.finish(input.class(), outcome, retry_after)
    }

    pub fn tracked_keys(&self, class: &str) -> usize {
        self.0
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .buckets
            .keys()
            .filter(|key| key.class == class)
            .count()
    }

    fn finish(
        &self,
        class: &str,
        outcome: AdmissionOutcome,
        retry_after: Option<Duration>,
    ) -> AdmissionDecision {
        self.0.observer.record(&AdmissionEvent {
            class: class.to_owned(),
            outcome,
            retry_after_ms: retry_after
                .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)),
        });
        AdmissionDecision {
            outcome,
            retry_after,
        }
    }
}

type Classifier = dyn Fn(&Request) -> Option<AdmissionInput> + Send + Sync;

/// State consumed by [`admission_middleware`]. Apps own the classifier;
/// enforcement remains shared.
#[derive(Clone)]
pub struct AdmissionMiddleware {
    controller: AdmissionController,
    classifier: Arc<Classifier>,
}

impl AdmissionMiddleware {
    pub fn new<F>(controller: AdmissionController, classifier: F) -> Self
    where
        F: Fn(&Request) -> Option<AdmissionInput> + Send + Sync + 'static,
    {
        Self {
            controller,
            classifier: Arc::new(classifier),
        }
    }
}

pub async fn admission_middleware(
    State(state): State<AdmissionMiddleware>,
    request: Request,
    next: Next,
) -> Response {
    let Some(input) = (state.classifier)(&request) else {
        return next.run(request).await;
    };
    let decision = state.controller.admit(&input);
    if decision.is_allowed() {
        return next.run(request).await;
    }

    let retry_after = decision.retry_after.unwrap_or(Duration::from_secs(1));
    let retry_seconds = retry_after
        .as_secs()
        .saturating_add(u64::from(retry_after.subsec_nanos() > 0));
    let mut response = ApiErr::new(
        StatusCode::TOO_MANY_REQUESTS,
        "rate_limited",
        "request admission limit exceeded",
    )
    .into_response();
    response.headers_mut().insert(
        header::RETRY_AFTER,
        HeaderValue::from_str(&retry_seconds.max(1).to_string())
            .expect("retry-after seconds are valid header text"),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    use axum::body::Body;
    use axum::routing::get;
    use axum::Router;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[derive(Default)]
    struct RecordingObserver(Mutex<Vec<AdmissionEvent>>);

    impl AdmissionObserver for RecordingObserver {
        fn record(&self, event: &AdmissionEvent) {
            self.0.lock().unwrap().push(event.clone());
        }
    }

    fn policy(capacity: u32, max_keys: usize) -> AdmissionPolicy {
        AdmissionPolicy::new(capacity, Duration::from_secs(10), max_keys).unwrap()
    }

    #[test]
    fn allow_deny_and_refill_are_deterministic() {
        let controller = AdmissionController::new([("read", policy(2, 8))]);
        let input = AdmissionInput::new("read", b"opaque-secret");
        assert_eq!(
            controller.admit_at(&input, Duration::ZERO).outcome,
            AdmissionOutcome::Allow
        );
        assert_eq!(
            controller.admit_at(&input, Duration::ZERO).outcome,
            AdmissionOutcome::Allow
        );
        let denied = controller.admit_at(&input, Duration::ZERO);
        assert_eq!(denied.outcome, AdmissionOutcome::Deny);
        assert_eq!(denied.retry_after, Some(Duration::from_secs(5)));
        assert_eq!(
            controller.admit_at(&input, Duration::from_secs(5)).outcome,
            AdmissionOutcome::Allow
        );
    }

    #[test]
    fn state_is_bounded_and_observer_schema_is_key_free() {
        let observer = Arc::new(RecordingObserver::default());
        let controller =
            AdmissionController::with_observer([("write", policy(1, 2))], observer.clone());
        for key in ["secret-a", "secret-b", "secret-c"] {
            controller.admit_at(
                &AdmissionInput::new("write", key.as_bytes()),
                Duration::ZERO,
            );
        }
        assert_eq!(controller.tracked_keys("write"), 2);
        let json = serde_json::to_string(&*observer.0.lock().unwrap()).unwrap();
        assert!(!json.contains("secret-a"));
        assert!(!json.contains("secret-b"));
        assert!(!json.contains("secret-c"));
        assert!(!json.contains("fingerprint"));
        assert!(!json.contains("credential"));
    }

    #[test]
    fn unconfigured_class_bypasses_without_allocating_state() {
        let controller = AdmissionController::new([("read", policy(1, 1))]);
        let decision = controller.admit_at(
            &AdmissionInput::new("unconfigured", b"anything"),
            Duration::ZERO,
        );
        assert_eq!(decision.outcome, AdmissionOutcome::Bypass);
        assert_eq!(controller.tracked_keys("unconfigured"), 0);
    }

    #[tokio::test]
    async fn middleware_returns_standard_429_and_retry_after() {
        let controller = AdmissionController::new([("read", policy(1, 1))]);
        let middleware = AdmissionMiddleware::new(controller, |_| {
            Some(AdmissionInput::new("read", b"anonymous"))
        });
        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .route_layer(axum::middleware::from_fn_with_state(
                middleware,
                admission_middleware,
            ));
        let request = || Request::builder().uri("/").body(Body::empty()).unwrap();
        assert_eq!(
            app.clone().oneshot(request()).await.unwrap().status(),
            StatusCode::OK
        );
        let denied = app.oneshot(request()).await.unwrap();
        assert_eq!(denied.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(denied.headers()[header::RETRY_AFTER], "10");
        let body = denied.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "rate_limited");
    }
}
// HANDWRITE-END
