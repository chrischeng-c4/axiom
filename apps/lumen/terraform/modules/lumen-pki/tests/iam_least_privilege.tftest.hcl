# AC2: the controller gets certificate-request and chain-read on one pool,
# through Workload Identity, with no long-lived credential anywhere in the plan.

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

run "the_grant_is_two_roles_on_one_pool" {
  command = plan

  assert {
    condition = toset(output.certificate_controller_roles) == toset([
      "roles/privateca.certificateRequester",
      "roles/privateca.auditor",
    ])
    error_message = "the controller must hold exactly certificate-request and read-only roles"
  }
  assert {
    condition     = length(google_privateca_ca_pool_iam_member.controller) == 2
    error_message = "one binding per role, and no third binding nobody declared"
  }
  assert {
    condition = alltrue([
      for binding in values(google_privateca_ca_pool_iam_member.controller) :
      binding.ca_pool == output.issuing_ca_pool_name
    ])
    error_message = "issuance authority must be scoped to the issuing pool, never granted at project level"
  }
}

run "authority_reaches_one_named_service_account" {
  command = plan

  assert {
    condition     = strcontains(output.certificate_controller_principal, "/subject/ns/lumen-system/sa/lumen-cert-controller")
    error_message = "the grant must name the controller's namespace and ServiceAccount exactly"
  }
  assert {
    # The federated principal is the whole reason there is no credential to
    # manage: GCP resolves it from the projected token kube-apiserver already
    # issues. An impersonation path would reintroduce a service account whose
    # keys someone eventually creates.
    condition     = startswith(output.certificate_controller_principal, "principal://iam.googleapis.com/projects/")
    error_message = "the controller must be bound as a federated principal, not through service-account impersonation"
  }
  assert {
    condition     = strcontains(output.certificate_controller_principal, "/workloadIdentityPools/lumen-pki-fixture.svc.id.goog/")
    error_message = "the binding must be scoped to this cluster's workload identity pool"
  }
}

run "a_namespace_wide_or_default_identity_is_rejected" {
  command = plan

  variables {
    certificate_controller = {
      namespace       = "lumen-system"
      service_account = "*"
    }
  }

  expect_failures = [var.certificate_controller]
}

run "the_default_service_account_is_rejected" {
  command = plan

  variables {
    certificate_controller = {
      namespace       = "lumen-system"
      service_account = "default"
    }
  }

  expect_failures = [var.certificate_controller]
}

run "ca_administration_is_not_grantable" {
  command = plan

  variables {
    additional_controller_roles = ["roles/privateca.caManager"]
  }

  expect_failures = [var.additional_controller_roles]
}

run "project_editor_is_not_grantable" {
  command = plan

  variables {
    additional_controller_roles = ["roles/editor"]
  }

  expect_failures = [var.additional_controller_roles]
}

run "credential_creation_is_not_grantable" {
  command = plan

  variables {
    additional_controller_roles = ["roles/iam.serviceAccountKeyAdmin"]
  }

  expect_failures = [var.additional_controller_roles]
}

run "gke_mutation_is_not_grantable" {
  command = plan

  variables {
    additional_controller_roles = ["roles/container.admin"]
  }

  expect_failures = [var.additional_controller_roles]
}

run "the_permitted_widening_is_the_workload_certificate_requester" {
  command = plan

  variables {
    additional_controller_roles = ["roles/privateca.workloadCertificateRequester"]
  }

  assert {
    condition     = length(google_privateca_ca_pool_iam_member.controller) == 3
    error_message = "the bounded seam must actually work; a validation that rejects everything is not least privilege, it is a wall"
  }
}

run "enabling_the_api_is_not_an_ownership_claim" {
  command = plan

  assert {
    condition     = google_project_service.privateca[0].disable_on_destroy == false
    error_message = "destroying this module must not disable CA Service for every other user of the project"
  }
}

run "central_service_enablement_can_be_deferred_to_the_platform" {
  command = plan

  variables {
    enable_privateca_api = false
  }

  assert {
    condition     = length(google_project_service.privateca) == 0
    error_message = "a platform team that enables services centrally must be able to opt out"
  }
}
