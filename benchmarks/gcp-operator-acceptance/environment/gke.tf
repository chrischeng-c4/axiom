resource "google_service_account" "gke_nodes" {
  project      = var.project_id
  account_id   = "${local.prefix}-node"
  display_name = "Disposable Axiom operator acceptance GKE nodes"
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

resource "google_container_cluster" "acceptance" {
  project                  = var.project_id
  name                     = "${local.prefix}-gke"
  location                 = var.gke_zone
  remove_default_node_pool = true
  initial_node_count       = 1
  deletion_protection      = false
  networking_mode          = "VPC_NATIVE"

  release_channel {
    channel = "REGULAR"
  }

  workload_identity_config {
    workload_pool = "${var.project_id}.svc.id.goog"
  }

  ip_allocation_policy {}

  resource_labels = local.labels

  timeouts {
    create = "30m"
    delete = "30m"
  }

  depends_on = [
    google_project_iam_member.gke_node_baseline,
    google_project_iam_member.gke_node_image_pull,
  ]
}

resource "google_container_node_pool" "acceptance" {
  project    = var.project_id
  name       = "${local.prefix}-pool"
  location   = var.gke_zone
  cluster    = google_container_cluster.acceptance.name
  node_count = var.node_min_count

  autoscaling {
    min_node_count = var.node_min_count
    max_node_count = var.node_max_count
  }

  management {
    auto_repair  = true
    auto_upgrade = true
  }

  node_config {
    machine_type    = var.node_machine_type
    service_account = google_service_account.gke_nodes.email
    oauth_scopes    = ["https://www.googleapis.com/auth/cloud-platform"]
    labels          = local.labels

    metadata = {
      disable-legacy-endpoints = "true"
    }

    workload_metadata_config {
      mode = "GKE_METADATA"
    }
  }

  depends_on = [
    google_project_iam_member.gke_node_baseline,
    google_project_iam_member.gke_node_image_pull,
  ]
}
