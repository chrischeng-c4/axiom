output "cluster_name" {
  value = google_container_cluster.benchmark.name
}

output "receiver_url" {
  value = google_cloud_run_v2_service.receiver.uri
}

output "receiver_secret" {
  value     = random_password.receiver_secret.result
  sensitive = true
}

output "benchmark_service_account" {
  value = google_service_account.benchmark_client.email
}

output "pubsub_topic" {
  value = google_pubsub_topic.tape.name
}

output "pubsub_subscriptions" {
  value = [for subscription in google_pubsub_subscription.tape_samples : subscription.name]
}

output "cloud_tasks_queue" {
  value = google_cloud_tasks_queue.defer.name
}

output "images" {
  value = {
    tape   = local.tape_image
    defer  = local.defer_image
    relay  = local.relay_image
    client = local.client_image
  }
}
