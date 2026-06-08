//! Integration tests for metrics module

#[cfg(feature = "metrics")]
#[cfg(test)]
mod tests {
    use cclab_queue::metrics::{METRICS, gather_metrics};

    #[test]
    fn test_metrics_recording() {
        // Record some test metrics
        METRICS.record_published("test_task", "default");
        METRICS.record_started("test_task", "default");
        METRICS.record_completed("test_task", "default", 0.5, true);
        METRICS.record_retry("test_task", "default");
        METRICS.record_failure("test_task", "default", "timeout");

        // Gather metrics
        let metrics_text = gather_metrics();

        // Verify metrics are present
        assert!(metrics_text.contains("tasks_published_total"));
        assert!(metrics_text.contains("tasks_executed_total"));
        assert!(metrics_text.contains("task_duration_seconds"));
        assert!(metrics_text.contains("tasks_in_progress"));
        assert!(metrics_text.contains("task_retries_total"));
        assert!(metrics_text.contains("task_failures_total"));

        // Verify labels
        assert!(metrics_text.contains("task_name=\"test_task\""));
        assert!(metrics_text.contains("queue=\"default\""));
    }

    #[test]
    fn test_metrics_multiple_tasks() {
        // Record metrics for multiple tasks
        for i in 0..10 {
            let task_name = format!("task_{}", i);
            METRICS.record_published(&task_name, "batch");
            METRICS.record_started(&task_name, "batch");
            METRICS.record_completed(&task_name, "batch", 0.1 * i as f64, i % 2 == 0);
        }

        let metrics_text = gather_metrics();

        // Should contain metrics for all tasks
        assert!(metrics_text.contains("queue=\"batch\""));
    }

    #[test]
    fn test_metrics_counter_increments() {
        let initial = gather_metrics();

        // Record some events
        METRICS.record_published("counter_test", "test_queue");
        METRICS.record_published("counter_test", "test_queue");

        let after = gather_metrics();

        // The metrics should have changed
        assert_ne!(initial, after);
        assert!(after.contains("counter_test"));
    }
}
