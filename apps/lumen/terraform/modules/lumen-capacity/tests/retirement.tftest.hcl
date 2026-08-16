# AC4: an in-use profile cannot be deleted by an ordinary input edit. It enters
# draining, and pool deletion requires an explicit retirement acknowledgement.

mock_provider "google" {}
mock_provider "kubernetes" {}

variables {
  project_id   = "lumen-capacity-fixture"
  region       = "asia-east1"
  cluster_name = "lumen-fixture-cluster"
}

run "removing_an_in_use_profile_without_acknowledgement_is_rejected" {
  command = plan

  variables {
    created_pools_in_use = ["n2-standard-8"]
    capacity_profiles    = {}
    draining_profiles    = {}
    retirement = {
      acknowledged = false
      reason       = ""
    }
  }

  expect_failures = [var.created_pools_in_use]
}

run "draining_an_in_use_profile_retains_the_pool" {
  command = plan

  variables {
    created_pools_in_use = ["n2-standard-8"]
    capacity_profiles    = {}
    draining_profiles = {
      "n2-standard-8" = { min_nodes = 0, max_nodes = 3 }
    }
  }

  assert {
    condition     = length(google_container_node_pool.pools) == 1
    error_message = "draining profile must retain the node pool"
  }
  assert {
    condition = contains([
      for entry in jsondecode(kubernetes_config_map.catalog.data["catalog.json"]).entries :
      entry.machine_type
    ], "n2-standard-8")
    error_message = "draining profile must be present in the decoded catalog"
  }
  assert {
    condition = one([
      for entry in jsondecode(kubernetes_config_map.catalog.data["catalog.json"]).entries :
      entry.lifecycle_state if entry.machine_type == "n2-standard-8"
    ]) == "draining"
    error_message = "draining profile must be marked draining in the decoded catalog"
  }
}

run "acknowledged_retirement_plans_deletion" {
  command = plan

  variables {
    created_pools_in_use = ["n2-standard-8"]
    capacity_profiles    = {}
    draining_profiles    = {}
    retirement = {
      acknowledged = true
      reason       = "decommissioning n2-standard-8 pool after workload migration"
    }
  }

  assert {
    condition     = length(google_container_node_pool.pools) == 0
    error_message = "acknowledged retirement must allow node pool deletion"
  }
}

run "retirement_without_sufficient_reason_is_rejected" {
  command = plan

  variables {
    retirement = {
      acknowledged = true
      reason       = "cleanup"
    }
  }

  expect_failures = [var.retirement]
}

run "unused_profile_may_be_removed_freely" {
  command = plan

  variables {
    created_pools_in_use = []
    capacity_profiles    = {}
    draining_profiles    = {}
  }

  assert {
    condition     = length(google_container_node_pool.pools) == 0
    error_message = "an unused profile may be removed without acknowledgement"
  }
}
