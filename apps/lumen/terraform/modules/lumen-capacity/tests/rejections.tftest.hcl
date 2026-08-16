# AC3: every invalid input is refused at plan with an actionable error.

mock_provider "google" {}
mock_provider "kubernetes" {}

variables {
  project_id   = "lumen-capacity-fixture"
  region       = "asia-east1"
  cluster_name = "lumen-fixture-cluster"
}

run "service_tier_key_is_rejected" {
  command = plan

  variables {
    capacity_profiles = {
      "gold" = { max_nodes = 4 }
    }
  }

  expect_failures = [var.capacity_profiles]
}

run "zero_maximum_nodes_is_rejected" {
  command = plan

  variables {
    capacity_profiles = {
      "n2-standard-8" = { max_nodes = 0 }
    }
  }

  expect_failures = [var.capacity_profiles]
}

run "negative_minimum_nodes_is_rejected" {
  command = plan

  variables {
    capacity_profiles = {
      "n2-standard-8" = { min_nodes = -1, max_nodes = 4 }
    }
  }

  expect_failures = [var.capacity_profiles]
}

run "minimum_nodes_exceeding_maximum_is_rejected" {
  command = plan

  variables {
    capacity_profiles = {
      "n2-standard-8" = { min_nodes = 5, max_nodes = 4 }
    }
  }

  expect_failures = [var.capacity_profiles]
}

run "invalid_project_id_is_rejected" {
  command = plan

  variables {
    project_id = "INVALID_PROJECT!"
  }

  expect_failures = [var.project_id]
}

run "zonal_region_string_is_rejected" {
  command = plan

  variables {
    region = "asia-east1-a"
  }

  expect_failures = [var.region]
}

run "profile_in_both_active_and_draining_is_rejected" {
  command = plan

  variables {
    capacity_profiles = {
      "n2-standard-8" = { max_nodes = 3 }
    }
    draining_profiles = {
      "n2-standard-8" = { max_nodes = 3 }
    }
  }

  expect_failures = [var.draining_profiles]
}
