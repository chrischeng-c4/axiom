# AC2: the in-cluster capacity catalog is published as a ConfigMap, readable
# without GCP API access and complete when every pool holds zero nodes.

mock_provider "google" {}
mock_provider "kubernetes" {}

variables {
  project_id   = "lumen-capacity-fixture"
  region       = "asia-east1"
  cluster_name = "lumen-fixture-cluster"
  capacity_profiles = {
    "n2-standard-4" = { min_nodes = 0, max_nodes = 2 }
    "n2-standard-8" = { min_nodes = 0, max_nodes = 4 }
  }
}

run "catalog_is_present_when_pools_hold_zero_nodes" {
  command = plan

  assert {
    condition     = kubernetes_config_map.catalog.metadata[0].name == "lumen-capacity-catalog"
    error_message = "catalog ConfigMap must use lumen-capacity-catalog name"
  }
  assert {
    condition     = kubernetes_config_map.catalog.metadata[0].namespace == "lumen-system"
    error_message = "catalog ConfigMap must reside in lumen-system namespace"
  }
  assert {
    condition     = length(jsondecode(kubernetes_config_map.catalog.data["catalog.json"]).entries) == 2
    error_message = "catalog JSON must record 2 entries for 2 profiles"
  }
}

run "catalog_entries_carry_required_zero_node_fields" {
  command = plan

  assert {
    condition = contains([
      for entry in jsondecode(kubernetes_config_map.catalog.data["catalog.json"]).entries :
      entry.machine_type
    ], "n2-standard-8")
    error_message = "catalog entry must record direct machine type"
  }
  assert {
    condition = contains([
      for entry in jsondecode(kubernetes_config_map.catalog.data["catalog.json"]).entries :
      entry.selector
    ], "lumen.axiom.dev/capacity-profile=n2-standard-8")
    error_message = "catalog entry must record stable selector"
  }
  assert {
    condition = one([
      for entry in jsondecode(kubernetes_config_map.catalog.data["catalog.json"]).entries :
      entry.max_nodes if entry.machine_type == "n2-standard-8"
    ]) == 4
    error_message = "catalog entry must record declared maximum"
  }
  assert {
    condition = one([
      for entry in jsondecode(kubernetes_config_map.catalog.data["catalog.json"]).entries :
      entry.lifecycle_state if entry.machine_type == "n2-standard-8"
    ]) == "ready"
    error_message = "catalog entry must record ready lifecycle state for active profiles"
  }
  assert {
    condition = one([
      for entry in jsondecode(kubernetes_config_map.catalog.data["catalog.json"]).entries :
      entry.pool_group if entry.machine_type == "n2-standard-8"
    ]) == "lumen-data"
    error_message = "catalog entry must record lumen-data pool group"
  }
}

run "empty_capacity_profiles_produces_valid_empty_catalog" {
  command = plan

  variables {
    capacity_profiles = {}
  }

  assert {
    condition     = kubernetes_config_map.catalog.metadata[0].name == "lumen-capacity-catalog"
    error_message = "catalog ConfigMap must exist even with no profiles"
  }
  assert {
    condition     = length(jsondecode(kubernetes_config_map.catalog.data["catalog.json"]).entries) == 0
    error_message = "empty profiles must produce 0 catalog entries"
  }
  assert {
    condition     = length(google_container_node_pool.pools) == 0
    error_message = "empty profiles must plan 0 node pools"
  }
}

run "catalog_config_map_outputs_are_stable" {
  command = plan

  assert {
    condition     = output.catalog_config_map_name == "lumen-capacity-catalog"
    error_message = "catalog_config_map_name output must match"
  }
  assert {
    condition     = output.catalog_config_map_namespace == "lumen-system"
    error_message = "catalog_config_map_namespace output must match"
  }
}

run "draining_profile_is_present_in_catalog_with_draining_lifecycle_state" {
  command = plan

  variables {
    capacity_profiles = {
      "n2-standard-4" = { max_nodes = 2 }
    }
    draining_profiles = {
      "n2-standard-8" = { min_nodes = 0, max_nodes = 4 }
    }
  }

  assert {
    condition     = length(jsondecode(kubernetes_config_map.catalog.data["catalog.json"]).entries) == 2
    error_message = "catalog JSON must record entries for both active and draining profiles"
  }
  assert {
    condition = one([
      for entry in jsondecode(kubernetes_config_map.catalog.data["catalog.json"]).entries :
      entry.lifecycle_state if entry.machine_type == "n2-standard-8"
    ]) == "draining"
    error_message = "draining profile in draining_profiles must be recorded with draining lifecycle_state"
  }
  assert {
    condition = one([
      for entry in jsondecode(kubernetes_config_map.catalog.data["catalog.json"]).entries :
      entry.lifecycle_state if entry.machine_type == "n2-standard-4"
    ]) == "ready"
    error_message = "active profile in capacity_profiles must be recorded with ready lifecycle_state"
  }
}
