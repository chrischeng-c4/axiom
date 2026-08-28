mock_provider "google" {}

run "stateful_cluster_contract" {
  command = plan

  variables {
    project_id = "axiom-test-12345"
    region     = "us-central1"
    gke_zone   = "us-central1-a"
    run_id     = "acceptance-20260828-01"
  }

  assert {
    condition     = google_container_cluster.standalone.location == "us-central1-a"
    error_message = "cluster must be zonal"
  }
  assert {
    condition     = google_container_cluster.standalone.networking_mode == "VPC_NATIVE"
    error_message = "cluster must use VPC-native networking"
  }
  assert {
    condition     = google_container_cluster.standalone.datapath_provider == "ADVANCED_DATAPATH"
    error_message = "cluster must use Dataplane V2"
  }
  assert {
    condition     = google_container_cluster.standalone.deletion_protection == false
    error_message = "cluster must be disposable"
  }
  assert {
    condition     = google_container_cluster.standalone.remove_default_node_pool == true && google_container_cluster.standalone.initial_node_count == 1
    error_message = "cluster must start with one removable default node"
  }
  assert {
    condition     = google_container_cluster.standalone.release_channel[0].channel == "REGULAR"
    error_message = "cluster must use the Regular release channel"
  }
  assert {
    condition     = google_container_cluster.standalone.workload_identity_config[0].workload_pool == "axiom-test-12345.svc.id.goog"
    error_message = "cluster must enable Workload Identity"
  }
  assert {
    condition     = length(google_container_cluster.standalone.logging_config[0].enable_components) == 2 && contains(google_container_cluster.standalone.logging_config[0].enable_components, "SYSTEM_COMPONENTS") && contains(google_container_cluster.standalone.logging_config[0].enable_components, "WORKLOADS")
    error_message = "cluster must enable system and workload logs"
  }
  assert {
    condition     = google_container_cluster.standalone.addons_config[0].gce_persistent_disk_csi_driver_config[0].enabled == true
    error_message = "cluster must enable the PD CSI driver"
  }
  assert {
    condition     = google_container_node_pool.standalone.node_config[0].machine_type == "e2-standard-2"
    error_message = "node pool must use e2-standard-2"
  }
  assert {
    condition     = google_container_node_pool.standalone.autoscaling[0].min_node_count == 1 && google_container_node_pool.standalone.autoscaling[0].max_node_count == 3
    error_message = "node pool autoscaling must be 1..3"
  }
  assert {
    condition     = google_container_node_pool.standalone.node_count == 1
    error_message = "there must be exactly one node"
  }
  assert {
    condition     = contains(google_container_node_pool.standalone.node_config[0].oauth_scopes, "https://www.googleapis.com/auth/cloud-platform")
    error_message = "nodes must use the cloud-platform scope"
  }
  assert {
    condition     = google_container_node_pool.standalone.node_config[0].metadata["disable-legacy-endpoints"] == "true"
    error_message = "legacy metadata endpoint must be disabled"
  }
  assert {
    condition     = google_container_node_pool.standalone.node_config[0].workload_metadata_config[0].mode == "GKE_METADATA"
    error_message = "nodes must use GKE metadata"
  }
  assert {
    condition     = google_container_cluster.standalone.resource_labels["lumen-owner"] == local.owner_label
    error_message = "cluster ownership label must be run scoped"
  }
  assert {
    condition     = google_container_node_pool.standalone.node_config[0].labels["lumen-owner"] == local.owner_label
    error_message = "node ownership label must be run scoped"
  }
  assert {
    condition     = output.node_selector["cloud.google.com/gke-nodepool"] == google_container_node_pool.standalone.name
    error_message = "node_selector output must identify the sole node pool"
  }
}

run "reject_project_placeholder" {
  command = plan
  variables {
    project_id = "replace-with-real-project-id"
    region     = "us-central1"
    gke_zone   = "us-central1-a"
    run_id     = "acceptance-20260828-01"
  }
  expect_failures = [var.project_id]
}
