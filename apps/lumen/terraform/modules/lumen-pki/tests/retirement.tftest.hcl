# AC6: an in-use created hierarchy cannot be removed by editing inputs. It goes
# through an explicit retirement acknowledgement, and that acknowledgement is
# also the only thing that releases deletion protection.
#
# The failure this guards against is quiet: flipping ca_pool_mode to "existing"
# looks like a configuration change and plans like a deletion. Everything Lumen
# trusts in that environment stops being verifiable at the moment it applies.

mock_provider "google" {}

variables {
  project_id             = "lumen-pki-fixture"
  workload_identity_pool = "lumen-pki-fixture.svc.id.goog"
  trust_domain           = "lumen-fixture.svc.id.goog"
  certificate_controller = {
    namespace       = "lumen-system"
    service_account = "lumen-cert-controller"
  }
}

run "abandoning_an_in_use_hierarchy_by_input_edit_is_rejected" {
  command = plan

  variables {
    created_hierarchy_in_use = true
    ca_pool_mode             = "existing"
    existing_ca_pool = {
      name     = "platform-approved-pool"
      location = "asia-east1"
    }
  }

  expect_failures = [var.ca_pool_mode]
}

run "an_unused_hierarchy_may_be_swapped_freely" {
  command = plan

  variables {
    created_hierarchy_in_use = false
    ca_pool_mode             = "existing"
    existing_ca_pool = {
      name     = "platform-approved-pool"
      location = "asia-east1"
    }
  }

  assert {
    condition     = output.manages_hierarchy == false
    error_message = "before anything issues from it, changing the trust source is an ordinary edit"
  }
}

run "acknowledged_retirement_is_the_handoff" {
  command = plan

  variables {
    created_hierarchy_in_use = true
    ca_pool_mode             = "existing"
    existing_ca_pool = {
      name     = "platform-approved-pool"
      location = "asia-east1"
    }
    retirement = {
      acknowledged = true
      reason       = "migrating to the shared platform CA pool, tracked in the change record"
    }
  }

  assert {
    condition     = output.manages_hierarchy == false
    error_message = "an acknowledged retirement must let the swap proceed"
  }
}

run "retirement_without_a_reason_is_rejected" {
  command = plan

  variables {
    retirement = {
      acknowledged = true
      reason       = "cleanup"
    }
  }

  expect_failures = [var.retirement]
}

run "acknowledged_retirement_releases_deletion_protection" {
  command = plan

  variables {
    retirement = {
      acknowledged = true
      reason       = "decommissioning the staging trust domain after the migration"
    }
  }

  assert {
    condition = (
      google_privateca_certificate_authority.root[0].deletion_protection == false
      && google_privateca_certificate_authority.issuing[0].deletion_protection == false
    )
    error_message = "retirement must be what releases deletion protection, so the two cannot drift apart"
  }
  assert {
    condition     = output.deletion_protected == false
    error_message = "the composing root must be able to see that protection has been released"
  }
}

run "deletion_protection_is_on_whenever_retirement_is_not_acknowledged" {
  command = plan

  variables {
    created_hierarchy_in_use = true
  }

  assert {
    condition     = output.deletion_protected == true
    error_message = "an in-use hierarchy must refuse destruction"
  }
}
