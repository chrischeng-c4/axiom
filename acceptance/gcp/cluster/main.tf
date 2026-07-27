terraform {
  required_version = ">= 1.9.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 6.0"
    }
  }
}

variable "project_id" { type = string }
variable "region" { type = string }
variable "gke_zone" { type = string }
variable "cluster_name" { type = string }
variable "node_service_account_id" { type = string }

provider "google" {
  project = var.project_id
  region  = var.region
}

locals {
  # The dedicated data-plane pool's identity, in one place: the pool resource
  # below, the outputs the harness reads, and the placement leg's drift check
  # all derive from these four strings.
  data_plane_pool_name   = "data-plane-pool"
  data_plane_label_key   = "axiom.dev/pool"
  data_plane_label_value = "data-plane"
  data_plane_taint_key   = "axiom.dev/dedicated"
}

resource "google_service_account" "nodes" {
  project      = var.project_id
  account_id   = var.node_service_account_id
  display_name = "Axiom persistent operator acceptance GKE nodes"
}

resource "google_project_iam_member" "node_baseline" {
  project = var.project_id
  role    = "roles/container.defaultNodeServiceAccount"
  member  = "serviceAccount:${google_service_account.nodes.email}"
}

resource "google_project_iam_member" "node_image_pull" {
  project = var.project_id
  role    = "roles/artifactregistry.reader"
  member  = "serviceAccount:${google_service_account.nodes.email}"
}

resource "google_container_cluster" "acceptance" {
  project                  = var.project_id
  name                     = var.cluster_name
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
  # The #2457 auth+CSI leg needs the GKE Secret Manager add-on, which registers
  # the secrets-store-gke.csi.k8s.io CSIDriver. It used to be enabled by hand on
  # the long-lived cluster and was never written down here, so when the cluster
  # was recreated (run 0726090015) the add-on vanished with it and the next run
  # reported auth_csi_gke_leg="skipped_no_addon" -- coverage shrank without a
  # single failure. A cluster fact that only exists in the cluster is a fact
  # that dies the next time the cluster does.
  secret_manager_config {
    enabled = true
  }
  ip_allocation_policy {}
  resource_labels = { "axiom-owner" = "gcp-operator-acceptance" }

  depends_on = [
    google_project_iam_member.node_baseline,
    google_project_iam_member.node_image_pull,
  ]
}

resource "google_container_node_pool" "acceptance" {
  project    = var.project_id
  name       = "acceptance-pool"
  location   = var.gke_zone
  cluster    = google_container_cluster.acceptance.name
  node_count = 1

  autoscaling {
    min_node_count = 1
    max_node_count = 2
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
    workload_metadata_config {
      mode = "GKE_METADATA"
    }
  }
}

# A second, DEDICATED node pool that exists for one reason: `spec.placement`
# (nodeSelector + tolerations) is only meaningfully proven against a real pool
# boundary. On a single-pool cluster every pod lands on the same node whether
# the operator renders the field or silently drops it, so a one-pool run cannot
# tell a working feature from a broken one.
#
# Scale-to-zero (min 0) is what makes it free: the pool holds no node until the
# placement leg's pod is Pending against it, and the autoscaler removes the node
# again afterwards. GKE stores the labels and taint on the pool itself, so
# scale-from-zero simulation sees them without a node ever having existed.
#
# The taint is the load-bearing half. Without it, a Lumen that merely matched
# the label would schedule here even with `tolerations` dropped; with it, only a
# pod carrying BOTH halves of `spec.placement` can ever run on this node.
resource "google_container_node_pool" "data_plane" {
  project            = var.project_id
  name               = local.data_plane_pool_name
  location           = var.gke_zone
  cluster            = google_container_cluster.acceptance.name
  initial_node_count = 0

  autoscaling {
    min_node_count = 0
    max_node_count = 1
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
    labels          = { (local.data_plane_label_key) = local.data_plane_label_value }
    taint {
      key    = local.data_plane_taint_key
      value  = local.data_plane_label_value
      effect = "NO_SCHEDULE"
    }
    workload_metadata_config {
      mode = "GKE_METADATA"
    }
  }
}

output "cluster_name" {
  value = google_container_cluster.acceptance.name
}

# The placement leg re-derives these from the live GKE API and fails if they
# disagree, so this file stays the single source of truth even though the
# persistent cluster is reused (and therefore never re-applied) between runs.
output "data_plane_pool_name" {
  value = local.data_plane_pool_name
}

output "data_plane_node_selector" {
  value = { (local.data_plane_label_key) = local.data_plane_label_value }
}

output "data_plane_toleration" {
  value = {
    key      = local.data_plane_taint_key
    operator = "Equal"
    value    = local.data_plane_label_value
    effect   = "NoSchedule"
  }
}
