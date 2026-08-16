# AC1: one autoscaled shared node pool per allowed direct GCE machine type,
# carrying the Lumen scheduling label and NoSchedule taint.

mock_provider "google" {}
mock_provider "kubernetes" {}

variables {
  project_id   = "lumen-capacity-fixture"
  region       = "asia-east1"
  cluster_name = "lumen-fixture-cluster"
  capacity_profiles = {
    "n2-standard-4" = { min_nodes = 0, max_nodes = 2 }
    "n2-standard-8" = { max_nodes = 5 }
    "n2-highmem-16" = { min_nodes = 1, max_nodes = 3 }
  }
}

run "three_profiles_plan_three_shared_autoscaled_pools" {
  command = plan

  assert {
    condition     = length(google_container_node_pool.pools) == 3
    error_message = "expected exactly 3 node pools for 3 profiles"
  }
  assert {
    condition     = google_container_node_pool.pools["n2-standard-4"].autoscaling[0].min_node_count == 0
    error_message = "n2-standard-4 pool min_node_count must be 0"
  }
  assert {
    condition     = google_container_node_pool.pools["n2-standard-4"].autoscaling[0].max_node_count == 2
    error_message = "n2-standard-4 pool max_node_count must be 2"
  }
  assert {
    condition     = google_container_node_pool.pools["n2-standard-8"].autoscaling[0].min_node_count == 0
    error_message = "omitted min_nodes must default to 0"
  }
  assert {
    condition     = google_container_node_pool.pools["n2-standard-8"].autoscaling[0].max_node_count == 5
    error_message = "n2-standard-8 pool max_node_count must be 5"
  }
  assert {
    condition     = google_container_node_pool.pools["n2-highmem-16"].autoscaling[0].min_node_count == 1
    error_message = "n2-highmem-16 pool min_node_count must be 1"
  }
  assert {
    condition     = google_container_node_pool.pools["n2-highmem-16"].autoscaling[0].max_node_count == 3
    error_message = "n2-highmem-16 pool max_node_count must be 3"
  }
}

run "pools_carry_lumen_scheduling_label_and_noschedule_taint" {
  command = plan

  assert {
    condition     = google_container_node_pool.pools["n2-standard-8"].node_config[0].labels["lumen.axiom.dev/capacity-profile"] == "n2-standard-8"
    error_message = "pool node template must carry the scheduling label with direct machine type"
  }
  assert {
    condition     = google_container_node_pool.pools["n2-standard-8"].node_config[0].taint[0].key == "lumen.axiom.dev/capacity-profile"
    error_message = "pool node template must carry the Lumen taint key"
  }
  assert {
    condition     = google_container_node_pool.pools["n2-standard-8"].node_config[0].taint[0].value == "n2-standard-8"
    error_message = "pool node template must carry the matching taint value"
  }
  assert {
    condition     = google_container_node_pool.pools["n2-standard-8"].node_config[0].taint[0].effect == "NO_SCHEDULE"
    error_message = "pool node template must enforce NO_SCHEDULE"
  }
}

run "multiple_consumers_on_same_machine_type_share_one_pool_and_selector" {
  command = plan

  assert {
    condition     = output.selectors["n2-standard-8"]["lumen.axiom.dev/capacity-profile"] == "n2-standard-8"
    error_message = "selector must map to stable label key and machine type"
  }
  assert {
    condition     = length(keys(output.selectors)) == 3
    error_message = "selectors output must cover all declared profiles"
  }
  assert {
    condition     = output.tolerations["n2-standard-8"].key == "lumen.axiom.dev/capacity-profile"
    error_message = "toleration must match taint key"
  }
}

run "single_profile_plans_single_shared_pool" {
  command = plan

  variables {
    capacity_profiles = {
      "n2-standard-8" = { min_nodes = 0, max_nodes = 4 }
    }
  }

  assert {
    condition     = length(google_container_node_pool.pools) == 1
    error_message = "single profile must plan exactly one pool"
  }
  assert {
    condition     = google_container_node_pool.pools["n2-standard-8"].node_config[0].machine_type == "n2-standard-8"
    error_message = "pool machine_type must match profile key"
  }
}
