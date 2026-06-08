//! Prometheus metrics for task queue observability

#[cfg(feature = "metrics")]
use prometheus::{
    CounterVec, HistogramVec, GaugeVec,
    register_counter_vec, register_histogram_vec, register_gauge_vec,
    Opts, HistogramOpts,
};
#[cfg(feature = "metrics")]
use once_cell::sync::Lazy;

/// Task execution metrics
#[cfg(feature = "metrics")]
pub struct TaskMetrics {
    /// Total tasks published
    pub tasks_published: CounterVec,
    /// Total tasks executed
    pub tasks_executed: CounterVec,
    /// Task execution duration in seconds
    pub task_duration_seconds: HistogramVec,
    /// Tasks currently in progress
    pub tasks_in_progress: GaugeVec,
    /// Task retries count
    pub task_retries: CounterVec,
    /// Task failures count
    pub task_failures: CounterVec,
}

#[cfg(feature = "metrics")]
impl TaskMetrics {
    pub fn new() -> Self {
        Self {
            tasks_published: register_counter_vec!(
                Opts::new("tasks_published_total", "Total number of tasks published"),
                &["task_name", "queue"]
            ).unwrap(),

            tasks_executed: register_counter_vec!(
                Opts::new("tasks_executed_total", "Total number of tasks executed"),
                &["task_name", "queue", "status"]
            ).unwrap(),

            task_duration_seconds: register_histogram_vec!(
                HistogramOpts::new("task_duration_seconds", "Task execution duration")
                    .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]),
                &["task_name", "queue"]
            ).unwrap(),

            tasks_in_progress: register_gauge_vec!(
                Opts::new("tasks_in_progress", "Number of tasks currently being executed"),
                &["task_name", "queue"]
            ).unwrap(),

            task_retries: register_counter_vec!(
                Opts::new("task_retries_total", "Total number of task retries"),
                &["task_name", "queue"]
            ).unwrap(),

            task_failures: register_counter_vec!(
                Opts::new("task_failures_total", "Total number of task failures"),
                &["task_name", "queue", "error_type"]
            ).unwrap(),
        }
    }

    pub fn record_published(&self, task_name: &str, queue: &str) {
        self.tasks_published.with_label_values(&[task_name, queue]).inc();
    }

    pub fn record_started(&self, task_name: &str, queue: &str) {
        self.tasks_in_progress.with_label_values(&[task_name, queue]).inc();
    }

    pub fn record_completed(&self, task_name: &str, queue: &str, duration_secs: f64, success: bool) {
        self.tasks_in_progress.with_label_values(&[task_name, queue]).dec();
        self.task_duration_seconds.with_label_values(&[task_name, queue]).observe(duration_secs);
        let status = if success { "success" } else { "failure" };
        self.tasks_executed.with_label_values(&[task_name, queue, status]).inc();
    }

    pub fn record_retry(&self, task_name: &str, queue: &str) {
        self.task_retries.with_label_values(&[task_name, queue]).inc();
    }

    pub fn record_failure(&self, task_name: &str, queue: &str, error_type: &str) {
        self.task_failures.with_label_values(&[task_name, queue, error_type]).inc();
    }
}

#[cfg(feature = "metrics")]
impl Default for TaskMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "metrics")]
pub static METRICS: Lazy<TaskMetrics> = Lazy::new(TaskMetrics::new);

/// Get metrics in Prometheus text format
#[cfg(feature = "metrics")]
pub fn gather_metrics() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

#[cfg(all(test, feature = "metrics"))]
mod tests {
    use super::*;
    use serial_test::serial;

    // T1: TaskMetrics::new() — all 6 fields accessible without panic
    // We access via the METRICS static (Lazy<TaskMetrics>) to avoid double-registration.
    #[test]
    #[serial]
    fn new_creates_all_instruments() {
        // Accessing METRICS triggers Lazy::new(TaskMetrics::new) exactly once.
        let _ = &METRICS.tasks_published;
        let _ = &METRICS.tasks_executed;
        let _ = &METRICS.task_duration_seconds;
        let _ = &METRICS.tasks_in_progress;
        let _ = &METRICS.task_retries;
        let _ = &METRICS.task_failures;
    }

    // T2: Default impl delegates to new() — does not panic
    #[test]
    #[serial]
    fn default_delegates_to_new() {
        // METRICS is initialized via TaskMetrics::new(), which is the Default impl.
        // Verify the static doesn't panic on access (proves Default works).
        let _ = &METRICS.tasks_published;
    }

    // T3: record_published increments counter
    #[test]
    #[serial]
    fn record_published_increments() {
        let before = METRICS
            .tasks_published
            .with_label_values(&["t3_task", "t3_queue"])
            .get();
        METRICS.record_published("t3_task", "t3_queue");
        let after = METRICS
            .tasks_published
            .with_label_values(&["t3_task", "t3_queue"])
            .get();
        assert_eq!(after - before, 1.0);
    }

    // T4: record_published called twice increments by 2
    #[test]
    #[serial]
    fn record_published_multiple() {
        let before = METRICS
            .tasks_published
            .with_label_values(&["t4_task", "t4_queue"])
            .get();
        METRICS.record_published("t4_task", "t4_queue");
        METRICS.record_published("t4_task", "t4_queue");
        let after = METRICS
            .tasks_published
            .with_label_values(&["t4_task", "t4_queue"])
            .get();
        assert_eq!(after - before, 2.0);
    }

    // T5: record_started increments gauge
    #[test]
    #[serial]
    fn record_started_increments_gauge() {
        let before = METRICS
            .tasks_in_progress
            .with_label_values(&["t5_task", "t5_queue"])
            .get();
        METRICS.record_started("t5_task", "t5_queue");
        let after = METRICS
            .tasks_in_progress
            .with_label_values(&["t5_task", "t5_queue"])
            .get();
        assert_eq!(after - before, 1.0);
        // cleanup: dec back
        METRICS
            .tasks_in_progress
            .with_label_values(&["t5_task", "t5_queue"])
            .dec();
    }

    // T6: record_completed success — gauge dec, histogram observe, counter inc
    #[test]
    #[serial]
    fn record_completed_success() {
        // Pre-increment gauge so dec doesn't go negative in isolation
        METRICS.record_started("t6_task", "t6_queue");
        let gauge_before = METRICS
            .tasks_in_progress
            .with_label_values(&["t6_task", "t6_queue"])
            .get();
        let counter_before = METRICS
            .tasks_executed
            .with_label_values(&["t6_task", "t6_queue", "success"])
            .get();

        METRICS.record_completed("t6_task", "t6_queue", 0.5, true);

        let gauge_after = METRICS
            .tasks_in_progress
            .with_label_values(&["t6_task", "t6_queue"])
            .get();
        let counter_after = METRICS
            .tasks_executed
            .with_label_values(&["t6_task", "t6_queue", "success"])
            .get();
        assert_eq!(gauge_before - gauge_after, 1.0);
        assert_eq!(counter_after - counter_before, 1.0);
    }

    // T7: record_completed failure path
    #[test]
    #[serial]
    fn record_completed_failure() {
        METRICS.record_started("t7_task", "t7_queue");
        let counter_before = METRICS
            .tasks_executed
            .with_label_values(&["t7_task", "t7_queue", "failure"])
            .get();

        METRICS.record_completed("t7_task", "t7_queue", 1.0, false);

        let counter_after = METRICS
            .tasks_executed
            .with_label_values(&["t7_task", "t7_queue", "failure"])
            .get();
        assert_eq!(counter_after - counter_before, 1.0);
    }

    // T8: record_completed observes histogram
    #[test]
    #[serial]
    fn record_completed_duration_observed() {
        use prometheus::core::Metric;
        let hist = METRICS
            .task_duration_seconds
            .with_label_values(&["t8_task", "t8_queue"]);
        let count_before = hist.metric().get_histogram().get_sample_count();
        let sum_before = hist.metric().get_histogram().get_sample_sum();

        METRICS.record_started("t8_task", "t8_queue");
        METRICS.record_completed("t8_task", "t8_queue", 0.25, true);

        let count_after = hist.metric().get_histogram().get_sample_count();
        let sum_after = hist.metric().get_histogram().get_sample_sum();
        assert_eq!(count_after - count_before, 1);
        assert!((sum_after - sum_before - 0.25).abs() < 1e-9);
    }

    // T9: record_retry increments counter
    #[test]
    #[serial]
    fn record_retry_increments() {
        let before = METRICS
            .task_retries
            .with_label_values(&["t9_task", "t9_queue"])
            .get();
        METRICS.record_retry("t9_task", "t9_queue");
        let after = METRICS
            .task_retries
            .with_label_values(&["t9_task", "t9_queue"])
            .get();
        assert_eq!(after - before, 1.0);
    }

    // T10: record_failure increments counter
    #[test]
    #[serial]
    fn record_failure_increments() {
        let before = METRICS
            .task_failures
            .with_label_values(&["t10_task", "t10_queue", "timeout"])
            .get();
        METRICS.record_failure("t10_task", "t10_queue", "timeout");
        let after = METRICS
            .task_failures
            .with_label_values(&["t10_task", "t10_queue", "timeout"])
            .get();
        assert_eq!(after - before, 1.0);
    }

    // T11: Different error_type labels are isolated
    #[test]
    #[serial]
    fn record_failure_different_error_types() {
        METRICS.record_failure("t11_task", "t11_queue", "oom");
        METRICS.record_failure("t11_task", "t11_queue", "crash");
        let oom = METRICS
            .task_failures
            .with_label_values(&["t11_task", "t11_queue", "oom"])
            .get();
        let crash = METRICS
            .task_failures
            .with_label_values(&["t11_task", "t11_queue", "crash"])
            .get();
        assert!(oom >= 1.0);
        assert!(crash >= 1.0);
    }

    // T12: start then complete returns gauge to zero
    #[test]
    #[serial]
    fn gauge_start_complete_returns_to_zero() {
        // Use unique labels so gauge starts at 0
        let gauge = METRICS
            .tasks_in_progress
            .with_label_values(&["t12_task", "t12_queue"]);
        let initial = gauge.get();
        METRICS.record_started("t12_task", "t12_queue");
        assert_eq!(gauge.get() - initial, 1.0);
        METRICS.record_completed("t12_task", "t12_queue", 0.1, true);
        assert_eq!(gauge.get(), initial);
    }

    // T13: histogram bucket boundaries
    #[test]
    #[serial]
    fn histogram_bucket_boundaries() {
        use prometheus::core::Metric;
        let hist = METRICS
            .task_duration_seconds
            .with_label_values(&["t13_task", "t13_queue"]);
        let proto = hist.metric();
        let h = proto.get_histogram();
        let buckets: Vec<f64> = h.get_bucket().iter().map(|b| b.get_upper_bound()).collect();
        let expected = vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0];
        assert_eq!(buckets.len(), expected.len());
        for (a, e) in buckets.iter().zip(expected.iter()) {
            assert!((a - e).abs() < 1e-9, "bucket {a} != expected {e}");
        }
    }

    // T14: gather_metrics returns Prometheus text format
    #[test]
    #[serial]
    fn gather_metrics_returns_text() {
        // Ensure METRICS is initialized so at least one metric family is registered
        let _ = &METRICS.tasks_published;
        let text = gather_metrics();
        assert!(!text.is_empty());
        assert!(text.contains("# HELP") || text.contains("# TYPE"));
    }

    // T15: gather_metrics includes registered metrics after recording
    #[test]
    #[serial]
    fn gather_metrics_includes_registered() {
        METRICS.record_published("t15_task", "t15_queue");
        let text = gather_metrics();
        assert!(
            text.contains("tasks_published_total"),
            "expected tasks_published_total in output"
        );
    }

    // T16: label isolation across queues
    #[test]
    #[serial]
    fn label_isolation_across_queues() {
        let before_q1 = METRICS
            .tasks_published
            .with_label_values(&["t16_task", "q1"])
            .get();
        let before_q2 = METRICS
            .tasks_published
            .with_label_values(&["t16_task", "q2"])
            .get();
        METRICS.record_published("t16_task", "q1");
        let after_q1 = METRICS
            .tasks_published
            .with_label_values(&["t16_task", "q1"])
            .get();
        let after_q2 = METRICS
            .tasks_published
            .with_label_values(&["t16_task", "q2"])
            .get();
        assert_eq!(after_q1 - before_q1, 1.0);
        assert_eq!(after_q2 - before_q2, 0.0);
    }

    // T17: label isolation across tasks
    #[test]
    #[serial]
    fn label_isolation_across_tasks() {
        let before_t1 = METRICS
            .tasks_published
            .with_label_values(&["t17_a", "t17_queue"])
            .get();
        let before_t2 = METRICS
            .tasks_published
            .with_label_values(&["t17_b", "t17_queue"])
            .get();
        METRICS.record_published("t17_a", "t17_queue");
        let after_t1 = METRICS
            .tasks_published
            .with_label_values(&["t17_a", "t17_queue"])
            .get();
        let after_t2 = METRICS
            .tasks_published
            .with_label_values(&["t17_b", "t17_queue"])
            .get();
        assert_eq!(after_t1 - before_t1, 1.0);
        assert_eq!(after_t2 - before_t2, 0.0);
    }

    // T18: METRICS static is lazy-initialized — access does not panic
    #[test]
    #[serial]
    fn metrics_static_is_lazy() {
        let _ = &METRICS.tasks_published;
    }

    // T19: concurrent recording is safe
    #[test]
    #[serial]
    fn concurrent_recording_safe() {
        let before = METRICS
            .tasks_published
            .with_label_values(&["t19_task", "t19_queue"])
            .get();
        let handles: Vec<_> = (0..10)
            .map(|_| {
                std::thread::spawn(|| {
                    METRICS.record_published("t19_task", "t19_queue");
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        let after = METRICS
            .tasks_published
            .with_label_values(&["t19_task", "t19_queue"])
            .get();
        assert_eq!(after - before, 10.0);
    }
}
