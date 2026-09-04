data "google_container_cluster" "acceptance" {
  project  = var.project_id
  name     = var.cluster_name
  location = var.gke_zone
}

locals {
  acceptance_node_service_accounts = [
    for pool in data.google_container_cluster.acceptance.node_pool :
    pool.node_config[0].service_account
    if pool.name == "acceptance-pool"
  ]
}

# Sift MVP owns a fixed, run-scoped three-node pool. Required pod
# anti-affinity can therefore place all three voters at the same time.
resource "google_container_node_pool" "sift_mvp" {
  count      = var.acceptance_apps == "sift" ? 1 : 0
  project    = var.project_id
  name       = "axo-${var.run_id}-sift"
  location   = var.gke_zone
  cluster    = data.google_container_cluster.acceptance.name
  node_count = 3

  management {
    auto_repair  = true
    auto_upgrade = true
  }

  # Provider 6.50 waits up to 30 minutes for one delete attempt. Cleanup
  # retries Terraform three times. Bound each attempt so one stale GKE
  # operation cannot consume most of the 90-minute acceptance window.
  timeouts {
    delete = "10m"
  }

  node_config {
    machine_type = "e2-standard-4"
    # Keep node boot disks out of SSD_TOTAL_GB. The Sift data PVCs use the
    # cluster's SSD-backed default StorageClass and need that quota instead.
    disk_type       = "pd-standard"
    service_account = one(local.acceptance_node_service_accounts)
    oauth_scopes    = ["https://www.googleapis.com/auth/cloud-platform"]
    metadata        = { disable-legacy-endpoints = "true" }
    labels = {
      "axiom-owner"  = "gcp-operator-acceptance"
      "axiom-run-id" = var.run_id
      "sift-mvp"     = "true"
    }
    workload_metadata_config {
      mode = "GKE_METADATA"
    }
  }
}
