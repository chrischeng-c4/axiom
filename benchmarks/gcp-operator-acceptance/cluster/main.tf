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
    # Tape's 3-replica acceptance needs one node per replica (the shared
    # StatefulSet render uses pod anti-affinity); 2 nodes made a 3-replica
    # topology unschedulable (run 0723080156). The autoscaler still idles
    # back down to min_node_count after each run.
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
    workload_metadata_config {
      mode = "GKE_METADATA"
    }
  }
}

output "cluster_name" {
  value = google_container_cluster.acceptance.name
}
