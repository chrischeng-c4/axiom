# AC1: one protected private hierarchy and one regional issuer, or a referenced
# operator-approved pool that this module never proposes to create.
#
# `mock_provider` is what makes these fixtures a gate rather than a ritual: they
# run offline, with no GCP project, credential, or billable resource, so the
# ownership boundary is checked on every commit instead of once per cloud run.

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

run "managed_mode_plans_one_root_and_one_regional_issuer" {
  command = plan

  assert {
    condition     = length(google_privateca_certificate_authority.root) == 1
    error_message = "managed mode must plan exactly one root CA"
  }
  assert {
    condition     = length(google_privateca_certificate_authority.issuing) == 1
    error_message = "managed mode must plan exactly one issuing CA"
  }
  assert {
    condition     = google_privateca_certificate_authority.root[0].type == "SELF_SIGNED"
    error_message = "the root must be self-signed; a subordinate root would put the trust anchor outside this environment"
  }
  assert {
    condition     = google_privateca_certificate_authority.issuing[0].type == "SUBORDINATE"
    error_message = "the issuer must chain to the root, not be a second trust anchor"
  }
  assert {
    # R5: regional issuing capacity for the environment. Two pools exist (root
    # and issuing) and both sit in the same region -- what must NOT appear is a
    # pool per namespace, which is why the count is asserted rather than the
    # names.
    condition = (
      google_privateca_ca_pool.root[0].location == var.region
      && google_privateca_ca_pool.issuing[0].location == var.region
    )
    error_message = "root and issuing capacity must both be in the module's region"
  }
  assert {
    condition     = output.manages_hierarchy == true
    error_message = "managed mode must report that it owns the hierarchy"
  }
}

run "the_root_is_protected_by_default" {
  command = plan

  assert {
    condition = (
      google_privateca_certificate_authority.root[0].deletion_protection
      && google_privateca_certificate_authority.issuing[0].deletion_protection
    )
    error_message = "a created hierarchy must be deletion-protected unless retirement is acknowledged"
  }
  assert {
    condition     = output.deletion_protected == true
    error_message = "deletion protection must be observable to the composing root"
  }
}

run "referenced_mode_creates_nothing" {
  command = plan

  variables {
    ca_pool_mode = "existing"
    existing_ca_pool = {
      name     = "platform-approved-pool"
      location = "asia-east1"
    }
  }

  assert {
    # The whole point of this mode: no CA resource is in the plan at all, so
    # there is no way for it to recreate, mutate, or destroy the operator's
    # trust hierarchy.
    condition = (
      length(google_privateca_certificate_authority.root) == 0
      && length(google_privateca_certificate_authority.issuing) == 0
      && length(google_privateca_ca_pool.root) == 0
      && length(google_privateca_ca_pool.issuing) == 0
    )
    error_message = "referenced mode must plan zero CA Service resources"
  }
  assert {
    condition     = output.issuing_ca_pool_id == "projects/lumen-pki-fixture/locations/asia-east1/caPools/platform-approved-pool"
    error_message = "referenced mode must address the operator's pool by its own name and location"
  }
  assert {
    condition     = output.root_ca_id == null && output.trust_anchor_pem == null
    error_message = "referenced mode owns no trust anchor and must not claim one"
  }
}

run "referenced_mode_still_binds_the_controller" {
  command = plan

  variables {
    ca_pool_mode = "existing"
    existing_ca_pool = {
      name     = "platform-approved-pool"
      location = "asia-east1"
    }
  }

  assert {
    condition     = length(google_privateca_ca_pool_iam_member.controller) == 2
    error_message = "issuance authority is the one thing this mode still grants; without it the controller cannot request a leaf"
  }
}

run "a_zone_where_a_region_belongs_is_rejected" {
  command = plan

  variables {
    region = "asia-east1-a"
  }

  expect_failures = [var.region]
}

run "referenced_mode_without_a_pool_is_rejected" {
  command = plan

  variables {
    ca_pool_mode     = "existing"
    existing_ca_pool = null
  }

  expect_failures = [var.existing_ca_pool]
}

run "naming_a_pool_while_creating_one_is_rejected" {
  command = plan

  variables {
    ca_pool_mode = "create"
    existing_ca_pool = {
      name     = "platform-approved-pool"
      location = "asia-east1"
    }
  }

  expect_failures = [var.existing_ca_pool]
}

run "an_issuer_outliving_its_root_is_rejected" {
  command = plan

  variables {
    root_ca_lifetime_years    = 3
    issuing_ca_lifetime_years = 5
  }

  expect_failures = [var.issuing_ca_lifetime_years]
}
