resource "google_cloud_tasks_queue" "defer" {
  project  = var.project_id
  name     = "${local.prefix}-defer"
  location = var.region

  rate_limits {
    max_dispatches_per_second = 500
    max_concurrent_dispatches = 100
  }

  retry_config {
    max_attempts       = 3
    max_retry_duration = "60s"
    min_backoff        = "1s"
    max_backoff        = "2s"
    max_doublings      = 1
  }

  stackdriver_logging_config {
    sampling_ratio = 0
  }
}
