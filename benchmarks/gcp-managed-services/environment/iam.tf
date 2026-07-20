resource "google_service_account" "gke_nodes" {
  project      = var.project_id
  account_id   = "${local.prefix}-node"
  display_name = "Disposable Axiom benchmark GKE nodes"
}

resource "google_project_iam_member" "gke_node_baseline" {
  project = var.project_id
  role    = "roles/container.defaultNodeServiceAccount"
  member  = "serviceAccount:${google_service_account.gke_nodes.email}"
}

resource "google_project_iam_member" "gke_node_image_pull" {
  project = var.project_id
  role    = "roles/artifactregistry.reader"
  member  = "serviceAccount:${google_service_account.gke_nodes.email}"
}

resource "google_service_account" "benchmark_client" {
  project      = var.project_id
  account_id   = "${local.prefix}-client"
  display_name = "Disposable Tape/PubSub and Defer/Cloud Tasks benchmark client"
}

resource "google_project_iam_member" "benchmark_roles" {
  for_each = toset([
    "roles/cloudtasks.enqueuer",
    "roles/cloudtasks.viewer",
    "roles/pubsub.publisher",
    "roles/pubsub.subscriber",
  ])

  project = var.project_id
  role    = each.value
  member  = "serviceAccount:${google_service_account.benchmark_client.email}"
}

resource "google_service_account_iam_member" "benchmark_workload_identity" {
  service_account_id = google_service_account.benchmark_client.name
  role               = "roles/iam.workloadIdentityUser"
  member             = "serviceAccount:${var.project_id}.svc.id.goog[axiom-bench/bench-client]"

  # The workload identity pool is materialized only after the Autopilot
  # cluster has finished creating. Without this edge Terraform can race the
  # IAM binding against pool creation.
  depends_on = [google_container_cluster.benchmark]
}
