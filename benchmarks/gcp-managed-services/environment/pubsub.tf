resource "google_pubsub_topic" "tape" {
  project = var.project_id
  name    = "${local.prefix}-tape"
  labels  = local.labels

  message_storage_policy {
    allowed_persistence_regions = [var.region]
    enforce_in_transit          = true
  }
}

resource "google_pubsub_subscription" "tape_samples" {
  count = var.replay_samples

  project                    = var.project_id
  name                       = "${local.prefix}-tape-${count.index}"
  topic                      = google_pubsub_topic.tape.id
  ack_deadline_seconds       = 60
  message_retention_duration = "86400s"
  retain_acked_messages      = true
  labels                     = local.labels

  retry_policy {
    minimum_backoff = "10s"
    maximum_backoff = "10s"
  }
}
