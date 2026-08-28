locals {
  run_hash             = substr(sha256(var.run_id), 0, 10)
  cluster_name         = "lumen-sa-${local.run_hash}"
  node_pool_name       = "lumen-np-${local.run_hash}"
  node_service_account = "lumen-nodes-${local.run_hash}"
  owner_label          = "lumen-standalone-${local.run_hash}"
}

resource "google_service_account" "nodes" {
  project      = var.project_id
  account_id   = local.node_service_account
  display_name = "Lumen standalone acceptance nodes ${local.run_hash}"
}

resource "google_project_iam_member" "node_baseline" {
  project = var.project_id
  role    = "roles/container.defaultNodeServiceAccount"
  member  = "serviceAccount:${google_service_account.nodes.email}"
}

resource "google_container_cluster" "standalone" {
  project                  = var.project_id
  name                     = local.cluster_name
  location                 = var.gke_zone
  remove_default_node_pool = true
  initial_node_count       = 1
  deletion_protection      = false
  networking_mode          = "VPC_NATIVE"
  datapath_provider        = "ADVANCED_DATAPATH"
  resource_labels          = { "lumen-owner" = local.owner_label }

  release_channel { channel = "REGULAR" }
  workload_identity_config { workload_pool = "${var.project_id}.svc.id.goog" }
  ip_allocation_policy {}
  logging_config { enable_components = ["SYSTEM_COMPONENTS", "WORKLOADS"] }
  monitoring_config { enable_components = ["SYSTEM_COMPONENTS"] }

  addons_config {
    gce_persistent_disk_csi_driver_config { enabled = true }
  }

  depends_on = [google_project_iam_member.node_baseline]
}

resource "google_container_node_pool" "standalone" {
  project    = var.project_id
  name       = local.node_pool_name
  location   = var.gke_zone
  cluster    = google_container_cluster.standalone.name
  node_count = 1

  autoscaling {
    min_node_count = 1
    max_node_count = 3
  }
  management {
    auto_repair  = true
    auto_upgrade = true
  }

  node_config {
    machine_type    = "e2-standard-2"
    service_account = google_service_account.nodes.email
    oauth_scopes    = ["https://www.googleapis.com/auth/cloud-platform"]
    metadata        = { disable-legacy-endpoints = "true" }
    labels          = { "lumen-owner" = local.owner_label }
    workload_metadata_config { mode = "GKE_METADATA" }
  }
}
