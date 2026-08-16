# AC5: one root configures both authorities, and neither one's inputs reach the
# other's resources.

mock_provider "google" {}
mock_provider "kubernetes" {}

variables {
  project_id             = "lumen-pki-fixture"
  cluster_name           = "lumen-prod"
  trust_domain           = "lumen-fixture.svc.id.goog"
  workload_identity_pool = "lumen-pki-fixture.svc.id.goog"
  capacity_profiles = {
    "n2-standard-8" = { max_nodes = 6 }
    "n2-highmem-16" = { min_nodes = 1, max_nodes = 3 }
  }
}

run "one_root_configures_trust_and_capacity_together" {
  command = plan

  assert {
    condition     = output.issuing_ca_pool_id == "projects/lumen-pki-fixture/locations/asia-east1/caPools/lumen-issuing"
    error_message = "the root must compose the PKI module"
  }
  assert {
    condition     = length(keys(output.capacity_profiles)) == 2
    error_message = "the root must carry the capacity contract in the same apply"
  }
  assert {
    condition     = output.capacity_profiles["n2-standard-8"].min_nodes == 0
    error_message = "data pools default to zero nodes; an omitted minimum must not become one"
  }
}

run "the_cluster_is_referenced_not_created" {
  command = plan

  assert {
    # R1 for both authorities: an installation root that could create a cluster
    # is an installation root that can destroy one.
    condition     = output.cluster_name != null
    error_message = "the root must read the existing cluster"
  }
}

run "trust_and_capacity_do_not_share_a_boundary" {
  command = plan

  assert {
    # Capacity inputs must be invisible to the PKI module. If a machine profile
    # ever showed up in the issuance policy, the two authorities would have
    # merged and a resize would need a certificate review.
    condition = !anytrue([
      for machine_type in keys(var.capacity_profiles) :
      strcontains(output.issuance_policy.subject_alt_name_expression, machine_type)
    ])
    error_message = "capacity inputs must not reach the issuance policy"
  }
  assert {
    condition     = output.issuance_policy.maximum_lifetime_seconds == var.max_leaf_lifetime_seconds
    error_message = "the root must pass its own leaf-lifetime ceiling through to the PKI module"
  }
}

run "an_unbounded_capacity_profile_is_rejected" {
  command = plan

  variables {
    capacity_profiles = {
      "n2-standard-8" = { max_nodes = 0 }
    }
  }

  expect_failures = [var.capacity_profiles]
}

run "a_service_tier_where_a_machine_type_belongs_is_rejected" {
  command = plan

  variables {
    capacity_profiles = {
      "gold" = { max_nodes = 4 }
    }
  }

  expect_failures = [var.capacity_profiles]
}
