//! Schedule monitor for tracking periodic task triggers
//!
//! Tracks expected trigger times vs actual trigger times for periodic tasks.
//! Computes `expected_at` from cron/interval, records `actual_at`, classifies
//! fires as `on_time`/`late`/`missed`, and emits Prometheus metrics.
//!
//! Hooks into both trigger paths uniformly: push receiver calls
//! `monitor.record_trigger(task_name, actual_at)` on each callback; the
//! self-hosted tick loop calls the same method after enqueue.  A background
//! `check_missed` task detects fires whose `expected_at` has passed beyond
//! the per-task leeway without a corresponding recording.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use chrono::{DateTime, Utc};
use cron::Schedule;
use serde::Serialize;

use crate::TaskError;

#[cfg(feature = "metrics")]
use once_cell::sync::Lazy;
#[cfg(feature = "metrics")]
use prometheus::{HistogramOpts, HistogramVec, IntCounterVec, Opts};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Classification of a task fire relative to `expected_at` and leeway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireStatus {
    /// Trigger arrived within leeway of `expected_at`.
    OnTime,
    /// Trigger arrived but beyond leeway.
    Late,
    /// No trigger received — detected by background check.
    Missed,
}

impl FireStatus {
    /// Prometheus label value.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OnTime => "on_time",
            Self::Late => "late",
            Self::Missed => "missed",
        }
    }
}

/// Schedule type for monitoring: cron expression or fixed interval.
#[derive(Debug, Clone)]
pub enum TaskSchedule {
    /// Cron expression with pre-parsed schedule.
    ///
    /// Uses the `cron` crate format (6-field with seconds, or 7-field with
    /// year).  Standard 5-field unix-cron expressions should be prefixed with
    /// a seconds field (e.g. `"0 */5 * * * *"` for every 5 minutes).
    Cron {
        expression: String,
        parsed: Schedule,
    },
    /// Fixed interval between fires.
    Interval { duration: Duration },
}

impl TaskSchedule {
    /// Create a cron-based schedule from a cron expression string.
    pub fn cron(expression: &str) -> Result<Self, TaskError> {
        let parsed = Schedule::from_str(expression).map_err(|e| {
            TaskError::Configuration(format!("Invalid cron expression '{}': {}", expression, e))
        })?;
        Ok(Self::Cron {
            expression: expression.to_string(),
            parsed,
        })
    }

    /// Create an interval-based schedule.
    pub fn interval(duration: Duration) -> Self {
        Self::Interval { duration }
    }
}

/// Per-task monitoring state tracked by [`ScheduleMonitor`].
pub struct TaskMonitorEntry {
    /// Unique task identifier.
    pub task_name: String,
    /// Cron expression or interval duration.
    pub schedule: TaskSchedule,
    /// Threshold between `on_time` and `late`.
    pub leeway: Duration,
    /// Next expected trigger time.  `None` before first computation.
    pub expected_at: Option<DateTime<Utc>>,
    /// Most recent actual trigger timestamp.
    pub last_actual_at: Option<DateTime<Utc>>,
    /// Per-task webhook URL override.  Falls back to global config if `None`.
    pub webhook_url: Option<String>,
}

/// JSON payload POSTed to webhook URL on missed detection.
#[derive(Debug, Clone, Serialize)]
pub struct WebhookPayload {
    pub task_name: String,
    pub expected_at: String,
    pub detected_at: String,
    pub status: String,
}

// ---------------------------------------------------------------------------
// Prometheus metrics (feature-gated)
// ---------------------------------------------------------------------------

#[cfg(feature = "metrics")]
struct MonitorMetrics {
    fire_total: IntCounterVec,
    latency_seconds: HistogramVec,
}

#[cfg(feature = "metrics")]
static MONITOR_METRICS: Lazy<MonitorMetrics> = Lazy::new(|| {
    let fire_total = IntCounterVec::new(
        Opts::new(
            "scheduler_task_fire_total",
            "Total task fires by task and status",
        ),
        &["task_name", "status"],
    )
    .expect("scheduler_task_fire_total IntCounterVec");

    let latency_seconds = HistogramVec::new(
        HistogramOpts::new(
            "scheduler_task_latency_seconds",
            "Seconds between expected_at and actual_at",
        )
        .buckets(vec![
            0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0,
        ]),
        &["task_name"],
    )
    .expect("scheduler_task_latency_seconds HistogramVec");

    prometheus::register(Box::new(fire_total.clone())).expect("register scheduler_task_fire_total");
    prometheus::register(Box::new(latency_seconds.clone()))
        .expect("register scheduler_task_latency_seconds");

    MonitorMetrics {
        fire_total,
        latency_seconds,
    }
});

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Configuration for [`ScheduleMonitor`].
#[derive(Debug, Clone)]
pub struct ScheduleMonitorConfig {
    /// Default leeway for on_time vs late classification (default 30 s).
    pub default_leeway: Duration,
    /// Interval between background missed-check sweeps (default 60 s).
    pub check_interval: Duration,
    /// Global webhook URL for missed-schedule alerts.
    pub webhook_url: Option<String>,
    /// Timeout for webhook HTTP calls (default 10 s).
    pub webhook_timeout: Duration,
}

impl Default for ScheduleMonitorConfig {
    fn default() -> Self {
        Self {
            default_leeway: Duration::from_secs(30),
            check_interval: Duration::from_secs(60),
            webhook_url: None,
            webhook_timeout: Duration::from_secs(10),
        }
    }
}

// ---------------------------------------------------------------------------
// ScheduleMonitor
// ---------------------------------------------------------------------------

/// Tracks expected vs actual trigger times for periodic tasks.
///
/// Shared as `Arc<ScheduleMonitor>` across push receiver, periodic scheduler,
/// and background missed-check task.  Requires `Send + Sync`.
pub struct ScheduleMonitor {
    config: ScheduleMonitorConfig,
    tasks: Arc<RwLock<HashMap<String, TaskMonitorEntry>>>,
    http_client: reqwest::Client,
    shutdown_tx: tokio::sync::watch::Sender<bool>,
}

impl ScheduleMonitor {
    /// Create a new monitor.
    ///
    /// Registers Prometheus metrics on first instantiation (idempotent via
    /// `once_cell::Lazy`).
    pub fn new(config: ScheduleMonitorConfig) -> Result<Self, TaskError> {
        // Force metric initialisation so registration errors surface early.
        #[cfg(feature = "metrics")]
        {
            Lazy::force(&MONITOR_METRICS);
        }

        let http_client = reqwest::Client::builder()
            .timeout(config.webhook_timeout)
            .build()
            .map_err(|e| {
                TaskError::Configuration(format!("Failed to create HTTP client: {}", e))
            })?;

        let (shutdown_tx, _) = tokio::sync::watch::channel(false);

        Ok(Self {
            config,
            tasks: Arc::new(RwLock::new(HashMap::new())),
            http_client,
            shutdown_tx,
        })
    }

    // -- Registration -------------------------------------------------------

    /// Register a task for monitoring.
    ///
    /// Computes initial `expected_at` from the schedule.  Uses the per-task
    /// `leeway` if provided, otherwise `config.default_leeway`.
    pub fn register_task(
        &self,
        name: &str,
        schedule: TaskSchedule,
        leeway: Option<Duration>,
        webhook_url: Option<String>,
    ) -> Result<(), TaskError> {
        let now = Utc::now();
        let expected_at = Self::compute_next_expected(&schedule, now);
        let leeway = leeway.unwrap_or(self.config.default_leeway);

        let entry = TaskMonitorEntry {
            task_name: name.to_string(),
            schedule,
            leeway,
            expected_at,
            last_actual_at: None,
            webhook_url,
        };

        let mut tasks = self
            .tasks
            .write()
            .map_err(|e| TaskError::Internal(format!("Lock poisoned: {}", e)))?;
        tasks.insert(name.to_string(), entry);

        tracing::info!(
            task_name = %name,
            expected_at = ?expected_at,
            leeway_secs = leeway.as_secs(),
            "Registered task for schedule monitoring"
        );
        Ok(())
    }

    // -- Recording ----------------------------------------------------------

    /// Record when a task was actually triggered.
    ///
    /// Classifies fire status, emits Prometheus metrics, and advances
    /// `expected_at`.  Returns `None` for unregistered tasks (no-op, logged
    /// at `debug` level).
    pub fn record_trigger(
        &self,
        task_name: &str,
        actual_at: DateTime<Utc>,
    ) -> Result<Option<FireStatus>, TaskError> {
        let mut tasks = self
            .tasks
            .write()
            .map_err(|e| TaskError::Internal(format!("Lock poisoned: {}", e)))?;

        let entry = match tasks.get_mut(task_name) {
            Some(e) => e,
            None => {
                tracing::debug!(
                    task_name = %task_name,
                    "Trigger for unmonitored task, ignored"
                );
                return Ok(None);
            }
        };

        // Compute latency and classify
        let (status, latency_secs) = if let Some(expected_at) = entry.expected_at {
            let latency = (actual_at - expected_at).to_std().unwrap_or(Duration::ZERO);
            (
                Self::classify_fire(latency, entry.leeway),
                latency.as_secs_f64(),
            )
        } else {
            // No expected_at yet — first fire, treat as on_time.
            (FireStatus::OnTime, 0.0)
        };

        // Emit Prometheus metrics
        #[cfg(feature = "metrics")]
        {
            MONITOR_METRICS
                .fire_total
                .with_label_values(&[task_name, status.as_str()])
                .inc();
            MONITOR_METRICS
                .latency_seconds
                .with_label_values(&[task_name])
                .observe(latency_secs);
        }

        // Advance expected_at from the *previous* expected_at (not actual_at)
        // to avoid drift for interval-based schedules.
        let base = entry.expected_at.unwrap_or(actual_at);
        entry.last_actual_at = Some(actual_at);
        entry.expected_at = Self::compute_next_expected(&entry.schedule, base);

        tracing::debug!(
            task_name = %task_name,
            status = %status.as_str(),
            latency_secs = latency_secs,
            next_expected = ?entry.expected_at,
            "Recorded task trigger"
        );

        Ok(Some(status))
    }

    // -- Lifecycle ----------------------------------------------------------

    /// Spawn the background missed-check task.
    ///
    /// Returns a [`JoinHandle`](tokio::task::JoinHandle) that completes when
    /// [`stop`](Self::stop) is called.
    pub fn start(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let monitor = Arc::clone(self);
        tokio::spawn(async move {
            tracing::info!(
                check_interval_secs = monitor.config.check_interval.as_secs(),
                "Schedule monitor background check started"
            );

            let mut shutdown_rx = monitor.shutdown_tx.subscribe();

            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            tracing::info!("Schedule monitor shutting down");
                            break;
                        }
                    }
                    _ = tokio::time::sleep(monitor.config.check_interval) => {
                        monitor.check_missed().await;
                    }
                }
            }
        })
    }

    /// Signal the background task to shut down.
    pub fn stop(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    // -- Internal: missed detection -----------------------------------------

    /// Iterate registered tasks and detect fires whose `expected_at` has
    /// passed beyond leeway without a recorded trigger.
    async fn check_missed(&self) {
        let now = Utc::now();

        // Collect missed info under the write lock, then release before HTTP.
        let missed: Vec<(String, DateTime<Utc>, Option<String>)> = {
            let mut tasks = match self.tasks.write() {
                Ok(g) => g,
                Err(e) => {
                    tracing::error!("check_missed: lock poisoned: {}", e);
                    return;
                }
            };

            let mut out = Vec::new();

            for (name, entry) in tasks.iter_mut() {
                let expected_at = match entry.expected_at {
                    Some(ea) => ea,
                    None => continue,
                };

                let leeway_chrono = chrono::Duration::from_std(entry.leeway)
                    .unwrap_or_else(|_| chrono::Duration::seconds(entry.leeway.as_secs() as i64));
                let deadline = expected_at + leeway_chrono;

                if now <= deadline {
                    continue;
                }

                // Check if a trigger was recorded since expected_at
                let received = entry
                    .last_actual_at
                    .map(|a| a >= expected_at)
                    .unwrap_or(false);
                if received {
                    continue;
                }

                // --- Missed ---
                tracing::warn!(
                    task_name = %name,
                    expected_at = %expected_at,
                    detected_at = %now,
                    "Missed schedule detected"
                );

                #[cfg(feature = "metrics")]
                {
                    MONITOR_METRICS
                        .fire_total
                        .with_label_values(&[name, FireStatus::Missed.as_str()])
                        .inc();
                }

                let webhook_url = entry
                    .webhook_url
                    .clone()
                    .or_else(|| self.config.webhook_url.clone());

                out.push((name.clone(), expected_at, webhook_url));

                // Advance expected_at to next scheduled time
                entry.expected_at = Self::compute_next_expected(&entry.schedule, expected_at);
            }

            out
        }; // write lock dropped here

        // Fire webhooks (non-blocking, one spawn per missed entry)
        for (task_name, expected_at, webhook_url) in missed {
            if let Some(url) = webhook_url {
                let payload = WebhookPayload {
                    task_name,
                    expected_at: expected_at.to_rfc3339(),
                    detected_at: now.to_rfc3339(),
                    status: "missed".to_string(),
                };
                let client = self.http_client.clone();
                tokio::spawn(async move {
                    Self::send_webhook(&client, &url, &payload).await;
                });
            }
        }
    }

    // -- Internal: webhook --------------------------------------------------

    /// POST JSON payload to webhook URL.  Logs errors at `warn` level.
    async fn send_webhook(client: &reqwest::Client, url: &str, payload: &WebhookPayload) {
        match client.post(url).json(payload).send().await {
            Ok(resp) if resp.status().is_success() => {
                tracing::debug!(
                    url = %url,
                    task_name = %payload.task_name,
                    "Missed-schedule webhook sent"
                );
            }
            Ok(resp) => {
                tracing::warn!(
                    url = %url,
                    status = %resp.status(),
                    task_name = %payload.task_name,
                    "Webhook returned non-success status"
                );
            }
            Err(e) => {
                tracing::warn!(
                    url = %url,
                    error = %e,
                    task_name = %payload.task_name,
                    "Failed to send missed-schedule webhook"
                );
            }
        }
    }

    // -- Pure helpers -------------------------------------------------------

    /// Compute the next expected trigger time from a schedule.
    ///
    /// For cron schedules, returns the first upcoming time **after** `after`.
    /// For interval schedules, returns `after + duration`.
    pub fn compute_next_expected(
        schedule: &TaskSchedule,
        after: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        match schedule {
            TaskSchedule::Cron { parsed, .. } => parsed
                .after(&after)
                .next()
                .map(|dt| DateTime::from_naive_utc_and_offset(dt.naive_utc(), Utc)),
            TaskSchedule::Interval { duration } => {
                let d = chrono::Duration::from_std(*duration).ok()?;
                Some(after + d)
            }
        }
    }

    /// Classify a fire: `OnTime` if latency <= leeway, else `Late`.
    pub fn classify_fire(latency: Duration, leeway: Duration) -> FireStatus {
        if latency <= leeway {
            FireStatus::OnTime
        } else {
            FireStatus::Late
        }
    }
}

impl Drop for ScheduleMonitor {
    fn drop(&mut self) {
        // Signal background task to exit on drop.
        let _ = self.shutdown_tx.send(true);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::time::Duration;

    /// Helper: create a monitor with default config.
    fn default_monitor() -> ScheduleMonitor {
        ScheduleMonitor::new(ScheduleMonitorConfig::default()).unwrap()
    }

    /// Helper: create a monitor with a specific default leeway.
    fn monitor_with_leeway(leeway_secs: u64) -> ScheduleMonitor {
        ScheduleMonitor::new(ScheduleMonitorConfig {
            default_leeway: Duration::from_secs(leeway_secs),
            ..Default::default()
        })
        .unwrap()
    }

    /// Helper: set the `expected_at` for a registered task (for deterministic tests).
    fn set_expected_at(monitor: &ScheduleMonitor, task_name: &str, expected: DateTime<Utc>) {
        let mut tasks = monitor.tasks.write().unwrap();
        if let Some(entry) = tasks.get_mut(task_name) {
            entry.expected_at = Some(expected);
        }
    }

    /// Helper: read `expected_at` for a registered task.
    fn get_expected_at(monitor: &ScheduleMonitor, task_name: &str) -> Option<DateTime<Utc>> {
        let tasks = monitor.tasks.read().unwrap();
        tasks.get(task_name).and_then(|e| e.expected_at)
    }

    /// Helper: read `last_actual_at` for a registered task.
    fn get_last_actual_at(monitor: &ScheduleMonitor, task_name: &str) -> Option<DateTime<Utc>> {
        let tasks = monitor.tasks.read().unwrap();
        tasks.get(task_name).and_then(|e| e.last_actual_at)
    }

    /// Helper: read leeway for a registered task.
    fn get_leeway(monitor: &ScheduleMonitor, task_name: &str) -> Option<Duration> {
        let tasks = monitor.tasks.read().unwrap();
        tasks.get(task_name).map(|e| e.leeway)
    }

    // =======================================================================
    // Pure helper: classify_fire (R2, R3)
    // =======================================================================

    #[test]
    fn classify_fire_on_time_when_latency_equals_leeway() {
        assert_eq!(
            ScheduleMonitor::classify_fire(Duration::from_secs(30), Duration::from_secs(30)),
            FireStatus::OnTime,
        );
    }

    #[test]
    fn classify_fire_on_time_when_latency_below_leeway() {
        assert_eq!(
            ScheduleMonitor::classify_fire(Duration::from_secs(5), Duration::from_secs(30)),
            FireStatus::OnTime,
        );
    }

    #[test]
    fn classify_fire_late_when_latency_exceeds_leeway() {
        assert_eq!(
            ScheduleMonitor::classify_fire(Duration::from_secs(31), Duration::from_secs(30)),
            FireStatus::Late,
        );
    }

    #[test]
    fn classify_fire_on_time_zero_latency() {
        assert_eq!(
            ScheduleMonitor::classify_fire(Duration::ZERO, Duration::from_secs(30)),
            FireStatus::OnTime,
        );
    }

    // =======================================================================
    // Pure helper: compute_next_expected (R1, S9)
    // =======================================================================

    #[test]
    fn compute_next_expected_interval() {
        let schedule = TaskSchedule::interval(Duration::from_secs(300));
        let base = Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 0).unwrap();
        let next = ScheduleMonitor::compute_next_expected(&schedule, base);
        assert_eq!(
            next,
            Some(Utc.with_ymd_and_hms(2026, 3, 28, 10, 5, 0).unwrap()),
        );
    }

    #[test]
    fn compute_next_expected_interval_successive() {
        // S9: second advance
        let schedule = TaskSchedule::interval(Duration::from_secs(300));
        let first = Utc.with_ymd_and_hms(2026, 3, 28, 10, 5, 0).unwrap();
        let next = ScheduleMonitor::compute_next_expected(&schedule, first);
        assert_eq!(
            next,
            Some(Utc.with_ymd_and_hms(2026, 3, 28, 10, 10, 0).unwrap()),
        );
    }

    #[test]
    fn compute_next_expected_cron_every_5_minutes() {
        // cron crate uses 6-field format: sec min hour day month weekday
        let schedule = TaskSchedule::cron("0 */5 * * * *").unwrap();
        let base = Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 0).unwrap();
        let next = ScheduleMonitor::compute_next_expected(&schedule, base);
        // Next 5-min boundary after 10:00:00 is 10:05:00
        assert_eq!(
            next,
            Some(Utc.with_ymd_and_hms(2026, 3, 28, 10, 5, 0).unwrap()),
        );
    }

    #[test]
    fn compute_next_expected_cron_daily_at_2am() {
        // "0 0 2 * * *" = every day at 02:00:00
        let schedule = TaskSchedule::cron("0 0 2 * * *").unwrap();
        let base = Utc.with_ymd_and_hms(2026, 3, 28, 2, 0, 0).unwrap();
        let next = ScheduleMonitor::compute_next_expected(&schedule, base);
        // Next daily 02:00 after 2026-03-28T02:00:00 is 2026-03-29T02:00:00
        assert_eq!(
            next,
            Some(Utc.with_ymd_and_hms(2026, 3, 29, 2, 0, 0).unwrap()),
        );
    }

    // =======================================================================
    // TaskSchedule construction
    // =======================================================================

    #[test]
    fn task_schedule_cron_valid() {
        let s = TaskSchedule::cron("0 0 2 * * *");
        assert!(s.is_ok());
    }

    #[test]
    fn task_schedule_cron_invalid() {
        let s = TaskSchedule::cron("not-a-cron");
        assert!(s.is_err());
    }

    #[test]
    fn task_schedule_interval_construction() {
        let s = TaskSchedule::interval(Duration::from_secs(60));
        match s {
            TaskSchedule::Interval { duration } => assert_eq!(duration, Duration::from_secs(60)),
            _ => panic!("Expected Interval variant"),
        }
    }

    // =======================================================================
    // FireStatus::as_str
    // =======================================================================

    #[test]
    fn fire_status_as_str_values() {
        assert_eq!(FireStatus::OnTime.as_str(), "on_time");
        assert_eq!(FireStatus::Late.as_str(), "late");
        assert_eq!(FireStatus::Missed.as_str(), "missed");
    }

    // =======================================================================
    // ScheduleMonitorConfig defaults
    // =======================================================================

    #[test]
    fn config_defaults() {
        let cfg = ScheduleMonitorConfig::default();
        assert_eq!(cfg.default_leeway, Duration::from_secs(30));
        assert_eq!(cfg.check_interval, Duration::from_secs(60));
        assert!(cfg.webhook_url.is_none());
        assert_eq!(cfg.webhook_timeout, Duration::from_secs(10));
    }

    // =======================================================================
    // ScheduleMonitor::new (R7)
    // =======================================================================

    #[test]
    fn monitor_new_succeeds() {
        let m = ScheduleMonitor::new(ScheduleMonitorConfig::default());
        assert!(m.is_ok());
    }

    #[test]
    fn monitor_is_send_sync() {
        // R7: ScheduleMonitor must be Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ScheduleMonitor>();
    }

    // =======================================================================
    // S1: Register cron task, record on-time trigger (R1, R2, R3, R4)
    // =======================================================================

    #[test]
    fn s1_register_cron_task_sets_expected_at() {
        let monitor = monitor_with_leeway(30);
        // "0 0 2 * * *" = every day at 02:00:00
        let schedule = TaskSchedule::cron("0 0 2 * * *").unwrap();
        monitor
            .register_task("daily-cleanup", schedule, None, None)
            .unwrap();

        // expected_at should be set to a future time
        let expected = get_expected_at(&monitor, "daily-cleanup");
        assert!(
            expected.is_some(),
            "expected_at should be set after registration"
        );
        assert!(
            expected.unwrap() > Utc::now(),
            "expected_at should be in the future"
        );
    }

    #[test]
    fn s1_register_cron_task_default_leeway() {
        let monitor = monitor_with_leeway(30);
        let schedule = TaskSchedule::cron("0 0 2 * * *").unwrap();
        monitor
            .register_task("daily-cleanup", schedule, None, None)
            .unwrap();

        let leeway = get_leeway(&monitor, "daily-cleanup");
        assert_eq!(leeway, Some(Duration::from_secs(30)));
    }

    #[test]
    fn s1_record_on_time_trigger() {
        let monitor = monitor_with_leeway(30);
        let schedule = TaskSchedule::interval(Duration::from_secs(3600));
        monitor
            .register_task("daily-cleanup", schedule, None, None)
            .unwrap();

        // Set a known expected_at
        let expected = Utc.with_ymd_and_hms(2026, 3, 28, 2, 0, 0).unwrap();
        set_expected_at(&monitor, "daily-cleanup", expected);

        // Trigger 5 seconds after expected — within 30s leeway → on_time
        let actual = Utc.with_ymd_and_hms(2026, 3, 28, 2, 0, 5).unwrap();
        let result = monitor.record_trigger("daily-cleanup", actual).unwrap();
        assert_eq!(result, Some(FireStatus::OnTime));
    }

    #[test]
    fn s1_record_trigger_advances_expected_at() {
        let monitor = monitor_with_leeway(30);
        let schedule = TaskSchedule::interval(Duration::from_secs(3600));
        monitor
            .register_task("hourly-task", schedule, None, None)
            .unwrap();

        let expected = Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 0).unwrap();
        set_expected_at(&monitor, "hourly-task", expected);

        let actual = Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 5).unwrap();
        monitor.record_trigger("hourly-task", actual).unwrap();

        // For interval schedule, next expected = previous expected + interval
        let new_expected = get_expected_at(&monitor, "hourly-task");
        assert_eq!(
            new_expected,
            Some(Utc.with_ymd_and_hms(2026, 3, 28, 11, 0, 0).unwrap()),
        );
    }

    #[test]
    fn s1_record_trigger_updates_last_actual_at() {
        let monitor = default_monitor();
        let schedule = TaskSchedule::interval(Duration::from_secs(300));
        monitor
            .register_task("task-a", schedule, None, None)
            .unwrap();

        let t = Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 0).unwrap();
        set_expected_at(&monitor, "task-a", t);

        let actual = Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 2).unwrap();
        monitor.record_trigger("task-a", actual).unwrap();
        assert_eq!(get_last_actual_at(&monitor, "task-a"), Some(actual));
    }

    // =======================================================================
    // S2: Record a late trigger (R2, R3, R4)
    // =======================================================================

    #[test]
    fn s2_record_late_trigger() {
        let monitor = default_monitor();
        // Interval schedule, 3600s, leeway = 60s
        let schedule = TaskSchedule::interval(Duration::from_secs(3600));
        monitor
            .register_task("hourly-sync", schedule, Some(Duration::from_secs(60)), None)
            .unwrap();

        let expected = Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 0).unwrap();
        set_expected_at(&monitor, "hourly-sync", expected);

        // 150 seconds after expected — beyond 60s leeway → late
        let actual = Utc.with_ymd_and_hms(2026, 3, 28, 10, 2, 30).unwrap();
        let result = monitor.record_trigger("hourly-sync", actual).unwrap();
        assert_eq!(result, Some(FireStatus::Late));
    }

    #[test]
    fn s2_late_trigger_advances_expected_at() {
        let monitor = default_monitor();
        let schedule = TaskSchedule::interval(Duration::from_secs(3600));
        monitor
            .register_task("hourly-sync", schedule, Some(Duration::from_secs(60)), None)
            .unwrap();

        let expected = Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 0).unwrap();
        set_expected_at(&monitor, "hourly-sync", expected);

        let actual = Utc.with_ymd_and_hms(2026, 3, 28, 10, 2, 30).unwrap();
        monitor.record_trigger("hourly-sync", actual).unwrap();

        // Next expected should be based on *previous* expected_at, not actual
        // expected + 3600s = 11:00:00
        let next = get_expected_at(&monitor, "hourly-sync");
        assert_eq!(
            next,
            Some(Utc.with_ymd_and_hms(2026, 3, 28, 11, 0, 0).unwrap()),
        );
    }

    // =======================================================================
    // S3: Background check detects missed schedule (R5, R4, R6) — tested
    // via check_missed()
    // =======================================================================

    #[tokio::test]
    async fn s3_check_missed_detects_missed_schedule() {
        let monitor = monitor_with_leeway(30);
        let schedule = TaskSchedule::interval(Duration::from_secs(86400));
        monitor
            .register_task("daily-cleanup", schedule, None, None)
            .unwrap();

        // Set expected_at far in the PAST so check_missed sees it as missed.
        // Must be more than leeway (30s) before now.
        let expected = Utc.with_ymd_and_hms(2020, 1, 1, 2, 0, 0).unwrap();
        set_expected_at(&monitor, "daily-cleanup", expected);

        // Call check_missed directly (accessible within module tests)
        monitor.check_missed().await;

        // After check_missed, expected_at should have advanced
        let new_expected = get_expected_at(&monitor, "daily-cleanup");
        assert!(
            new_expected.is_some(),
            "expected_at should advance after missed detection"
        );
        assert!(
            new_expected.unwrap() > expected,
            "expected_at should advance past the old value"
        );
    }

    #[tokio::test]
    async fn s3_check_missed_does_not_flag_on_time_triggers() {
        let monitor = monitor_with_leeway(30);
        let schedule = TaskSchedule::interval(Duration::from_secs(3600));
        monitor
            .register_task("hourly-task", schedule, None, None)
            .unwrap();

        // Set expected_at far in the past…
        let expected = Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 0).unwrap();
        set_expected_at(&monitor, "hourly-task", expected);

        // …but record a trigger AFTER expected_at so it looks like it fired
        {
            let mut tasks = monitor.tasks.write().unwrap();
            let entry = tasks.get_mut("hourly-task").unwrap();
            entry.last_actual_at = Some(Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 5).unwrap());
        }

        // check_missed should NOT advance expected_at (trigger was received)
        let before = get_expected_at(&monitor, "hourly-task");
        monitor.check_missed().await;
        let after = get_expected_at(&monitor, "hourly-task");
        assert_eq!(
            before, after,
            "expected_at should not change when trigger was recorded"
        );
    }

    // =======================================================================
    // S4: Per-task leeway overrides default (R3)
    // =======================================================================

    #[test]
    fn s4_per_task_leeway_override() {
        let monitor = monitor_with_leeway(30);
        let schedule = TaskSchedule::cron("0 */5 * * * *").unwrap();
        monitor
            .register_task(
                "critical-job",
                schedule,
                Some(Duration::from_secs(10)),
                None,
            )
            .unwrap();

        // Leeway should be 10s, not 30s default
        assert_eq!(
            get_leeway(&monitor, "critical-job"),
            Some(Duration::from_secs(10)),
        );
    }

    #[test]
    fn s4_trigger_late_with_custom_leeway() {
        let monitor = monitor_with_leeway(30);
        let schedule = TaskSchedule::interval(Duration::from_secs(300));
        monitor
            .register_task(
                "critical-job",
                schedule,
                Some(Duration::from_secs(10)),
                None,
            )
            .unwrap();

        let expected = Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 0).unwrap();
        set_expected_at(&monitor, "critical-job", expected);

        // 15 seconds after expected — beyond 10s custom leeway → late
        let actual = Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 15).unwrap();
        let result = monitor.record_trigger("critical-job", actual).unwrap();
        assert_eq!(result, Some(FireStatus::Late));
    }

    #[test]
    fn s4_trigger_on_time_with_default_leeway_but_late_with_custom() {
        // 15s latency: on_time with 30s default, but late with 10s custom
        let monitor = monitor_with_leeway(30);
        let schedule = TaskSchedule::interval(Duration::from_secs(300));

        // Task with default leeway (30s)
        monitor
            .register_task("default-leeway-task", schedule.clone(), None, None)
            .unwrap();
        // Task with custom leeway (10s)
        monitor
            .register_task(
                "custom-leeway-task",
                schedule,
                Some(Duration::from_secs(10)),
                None,
            )
            .unwrap();

        let expected = Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 0).unwrap();
        set_expected_at(&monitor, "default-leeway-task", expected);
        set_expected_at(&monitor, "custom-leeway-task", expected);

        let actual = Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 15).unwrap();

        let r1 = monitor
            .record_trigger("default-leeway-task", actual)
            .unwrap();
        let r2 = monitor
            .record_trigger("custom-leeway-task", actual)
            .unwrap();

        assert_eq!(
            r1,
            Some(FireStatus::OnTime),
            "30s leeway: 15s should be on_time"
        );
        assert_eq!(r2, Some(FireStatus::Late), "10s leeway: 15s should be late");
    }

    // =======================================================================
    // S7: Record trigger for unregistered task is no-op (R2)
    // =======================================================================

    #[test]
    fn s7_unregistered_task_returns_none() {
        let monitor = default_monitor();
        let result = monitor.record_trigger("unknown-task", Utc::now()).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn s7_unregistered_task_does_not_affect_registered() {
        let monitor = default_monitor();
        let schedule = TaskSchedule::interval(Duration::from_secs(60));
        monitor
            .register_task("real-task", schedule, None, None)
            .unwrap();

        let before = get_expected_at(&monitor, "real-task");
        // Recording an unregistered task should be no-op
        let _ = monitor.record_trigger("unknown-task", Utc::now());
        let after = get_expected_at(&monitor, "real-task");
        assert_eq!(before, after);
    }

    // =======================================================================
    // S8: Monitor lifecycle start and stop (R8)
    // =======================================================================

    #[tokio::test]
    async fn s8_start_and_stop_lifecycle() {
        let monitor = Arc::new(
            ScheduleMonitor::new(ScheduleMonitorConfig {
                check_interval: Duration::from_millis(50),
                ..Default::default()
            })
            .unwrap(),
        );

        let handle = monitor.start();
        // Let the background task run a few cycles
        tokio::time::sleep(Duration::from_millis(150)).await;

        monitor.stop();
        // The join handle should resolve after stop
        let result = tokio::time::timeout(Duration::from_secs(2), handle).await;
        assert!(
            result.is_ok(),
            "Background task should exit after stop() within timeout"
        );
    }

    #[tokio::test]
    async fn s8_stop_then_drop() {
        // Verify stop() followed by drop works cleanly without panics.
        // Note: dropping Arc<ScheduleMonitor> alone doesn't trigger Drop
        // while the background task holds a clone, so stop() must be called
        // explicitly before the task's Arc clone is released.
        let monitor = Arc::new(
            ScheduleMonitor::new(ScheduleMonitorConfig {
                check_interval: Duration::from_millis(50),
                ..Default::default()
            })
            .unwrap(),
        );

        let handle = monitor.start();
        // Yield to let the background task start and subscribe to the
        // shutdown channel before we send the signal.
        tokio::task::yield_now().await;
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Explicit stop signals shutdown
        monitor.stop();

        let result = tokio::time::timeout(Duration::from_secs(2), handle).await;
        assert!(result.is_ok(), "Background task should exit after stop()");

        // Drop after background task has exited — should not panic
        drop(monitor);
    }

    // =======================================================================
    // S9: Interval-based task computes expected_at correctly (R1)
    // =======================================================================

    #[test]
    fn s9_interval_first_trigger_advances_expected() {
        let monitor = default_monitor();
        let schedule = TaskSchedule::interval(Duration::from_secs(300));
        monitor
            .register_task("every-5m", schedule, None, None)
            .unwrap();

        let first_trigger = Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 0).unwrap();
        set_expected_at(&monitor, "every-5m", first_trigger);

        monitor.record_trigger("every-5m", first_trigger).unwrap();
        let next = get_expected_at(&monitor, "every-5m");
        assert_eq!(
            next,
            Some(Utc.with_ymd_and_hms(2026, 3, 28, 10, 5, 0).unwrap()),
        );
    }

    #[test]
    fn s9_interval_second_trigger_advances_again() {
        let monitor = default_monitor();
        let schedule = TaskSchedule::interval(Duration::from_secs(300));
        monitor
            .register_task("every-5m", schedule, None, None)
            .unwrap();

        // First trigger at 10:00:00
        let first = Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 0).unwrap();
        set_expected_at(&monitor, "every-5m", first);
        monitor.record_trigger("every-5m", first).unwrap();

        // Second trigger at 10:05:02 (2s late but within default 30s leeway)
        let second = Utc.with_ymd_and_hms(2026, 3, 28, 10, 5, 2).unwrap();
        let result = monitor.record_trigger("every-5m", second).unwrap();
        assert_eq!(result, Some(FireStatus::OnTime));

        // expected_at should advance based on PREVIOUS expected (10:05:00), not actual
        let next = get_expected_at(&monitor, "every-5m");
        assert_eq!(
            next,
            Some(Utc.with_ymd_and_hms(2026, 3, 28, 10, 10, 0).unwrap()),
        );
    }

    // =======================================================================
    // S6: Concurrent access / thread safety (R7)
    // =======================================================================

    #[test]
    fn s6_concurrent_register_and_record() {
        use std::sync::Arc;
        use std::thread;

        let monitor = Arc::new(default_monitor());

        // Register tasks from multiple threads
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let m = Arc::clone(&monitor);
                thread::spawn(move || {
                    let name = format!("task-{}", i);
                    let schedule = TaskSchedule::interval(Duration::from_secs(60));
                    m.register_task(&name, schedule, None, None).unwrap();
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        // All tasks should be registered
        let tasks = monitor.tasks.read().unwrap();
        assert_eq!(tasks.len(), 10);
    }

    #[test]
    fn s6_concurrent_record_trigger() {
        use std::sync::Arc;
        use std::thread;

        let monitor = Arc::new(default_monitor());

        // Register tasks
        for i in 0..5 {
            let name = format!("concurrent-{}", i);
            let schedule = TaskSchedule::interval(Duration::from_secs(3600));
            monitor.register_task(&name, schedule, None, None).unwrap();
        }

        // Record triggers concurrently
        let handles: Vec<_> = (0..5)
            .map(|i| {
                let m = Arc::clone(&monitor);
                thread::spawn(move || {
                    let name = format!("concurrent-{}", i);
                    let result = m.record_trigger(&name, Utc::now());
                    assert!(result.is_ok());
                    assert!(result.unwrap().is_some());
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }
    }

    // =======================================================================
    // WebhookPayload serialization
    // =======================================================================

    #[test]
    fn webhook_payload_serializes_correctly() {
        let payload = WebhookPayload {
            task_name: "daily-cleanup".to_string(),
            expected_at: "2026-03-28T02:00:00+00:00".to_string(),
            detected_at: "2026-03-28T02:01:00+00:00".to_string(),
            status: "missed".to_string(),
        };

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["task_name"], "daily-cleanup");
        assert_eq!(json["expected_at"], "2026-03-28T02:00:00+00:00");
        assert_eq!(json["detected_at"], "2026-03-28T02:01:00+00:00");
        assert_eq!(json["status"], "missed");
    }

    // =======================================================================
    // Edge cases
    // =======================================================================

    #[test]
    fn record_trigger_before_first_expected_at() {
        // When expected_at is None (shouldn't normally happen but defensive)
        let monitor = default_monitor();
        let schedule = TaskSchedule::interval(Duration::from_secs(60));
        monitor
            .register_task("new-task", schedule, None, None)
            .unwrap();

        // Force expected_at to None
        {
            let mut tasks = monitor.tasks.write().unwrap();
            tasks.get_mut("new-task").unwrap().expected_at = None;
        }

        // Should still work — treated as on_time with 0 latency
        let result = monitor.record_trigger("new-task", Utc::now()).unwrap();
        assert_eq!(result, Some(FireStatus::OnTime));
    }

    #[test]
    fn register_task_with_webhook_url() {
        let monitor = default_monitor();
        let schedule = TaskSchedule::interval(Duration::from_secs(60));
        monitor
            .register_task(
                "webhook-task",
                schedule,
                None,
                Some("https://hooks.example.com/alert".to_string()),
            )
            .unwrap();

        let tasks = monitor.tasks.read().unwrap();
        let entry = tasks.get("webhook-task").unwrap();
        assert_eq!(
            entry.webhook_url,
            Some("https://hooks.example.com/alert".to_string()),
        );
    }

    #[test]
    fn register_task_overwrites_existing() {
        let monitor = default_monitor();
        let schedule1 = TaskSchedule::interval(Duration::from_secs(60));
        monitor
            .register_task("dup-task", schedule1, Some(Duration::from_secs(10)), None)
            .unwrap();

        let schedule2 = TaskSchedule::interval(Duration::from_secs(120));
        monitor
            .register_task("dup-task", schedule2, Some(Duration::from_secs(20)), None)
            .unwrap();

        // Should have the second registration's values
        assert_eq!(
            get_leeway(&monitor, "dup-task"),
            Some(Duration::from_secs(20)),
        );
    }

    #[tokio::test]
    async fn check_missed_no_tasks_is_noop() {
        let monitor = default_monitor();
        // Should not panic with no registered tasks
        monitor.check_missed().await;
    }

    #[tokio::test]
    async fn check_missed_future_expected_at_not_flagged() {
        let monitor = default_monitor();
        let schedule = TaskSchedule::interval(Duration::from_secs(3600));
        monitor
            .register_task("future-task", schedule, None, None)
            .unwrap();

        // expected_at is already in the future (set by register_task),
        // so check_missed should not flag it
        let before = get_expected_at(&monitor, "future-task");
        monitor.check_missed().await;
        let after = get_expected_at(&monitor, "future-task");
        assert_eq!(
            before, after,
            "Future expected_at should not be flagged as missed"
        );
    }
}
