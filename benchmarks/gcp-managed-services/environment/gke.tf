resource "google_container_cluster" "benchmark" {
  project             = var.project_id
  name                = "${local.prefix}-gke"
  location            = var.region
  enable_autopilot    = true
  deletion_protection = false

  release_channel {
    channel = "REGULAR"
  }

  workload_identity_config {
    workload_pool = "${var.project_id}.svc.id.goog"
  }

  cluster_autoscaling {
    auto_provisioning_defaults {
      service_account = google_service_account.gke_nodes.email
    }
  }

  resource_labels = local.labels

  timeouts {
    create = "45m"
    delete = "45m"
  }

  depends_on = [
    google_project_iam_member.gke_node_baseline,
    google_project_iam_member.gke_node_image_pull,
  ]
}
