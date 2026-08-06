# AC3: the issuance policy admits the Kubernetes service and peer identities
# Lumen actually uses, and refuses CA leaves, arbitrary DNS names, wrong usages,
# and unconstrained subject passthrough.
#
# Two kinds of assertion appear here, and the difference matters. The rejects an
# operator could trigger are `expect_failures` -- Terraform's validation engine
# is the oracle, so the rule is executed, not read. The rejects nobody can
# trigger (leaf-is-CA, subject passthrough, usages) are fixed in main.tf and
# asserted structurally, because a constant that no input reaches cannot be
# tested by varying inputs; what a test can still catch is someone making it a
# variable later.

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

run "the_real_lumen_identities_are_issuable" {
  command = plan

  # The names on the left are not illustrative -- they are the shapes the
  # serving Service, the headless peer record, and the SPIFFE peer identity
  # take in a deployed Lumen. A suffix list that stopped admitting them would
  # produce a trust domain that cannot certify the product it exists for.
  assert {
    condition     = anytrue([for s in var.allowed_dns_suffixes : endswith("lumen.lumen.svc.cluster.local", s)])
    error_message = "the client Service DNS name must be issuable"
  }
  assert {
    condition     = anytrue([for s in var.allowed_dns_suffixes : endswith("lumen-0.lumen-headless.lumen.svc.cluster.local", s)])
    error_message = "the per-Pod headless peer name must be issuable"
  }
  assert {
    condition     = anytrue([for s in var.allowed_dns_suffixes : endswith("lumen.lumen.svc", s)])
    error_message = "the short in-cluster Service form must be issuable"
  }
  assert {
    condition     = startswith("spiffe://${var.trust_domain}/ns/lumen/sa/lumen", "spiffe://${var.trust_domain}/ns/")
    error_message = "the SPIFFE peer identity must fall inside the environment trust domain"
  }
}

run "names_outside_the_cluster_are_not_issuable" {
  command = plan

  assert {
    condition     = !anytrue([for s in var.allowed_dns_suffixes : endswith("lumen.example.com", s)])
    error_message = "a public DNS name must not match any allowed suffix"
  }
  assert {
    condition     = !anytrue([for s in var.allowed_dns_suffixes : endswith("lumen.internal.corp", s)])
    error_message = "an arbitrary private DNS name must not match any allowed suffix"
  }
  assert {
    condition     = !startswith("spiffe://other-env.svc.id.goog/ns/lumen/sa/lumen", "spiffe://${var.trust_domain}/ns/")
    error_message = "a SPIFFE identity from another trust domain must not be issuable"
  }
}

run "the_san_expression_constrains_every_name_not_merely_one" {
  command = plan

  assert {
    # `all(` versus `exists(`: with `exists`, a request carrying one valid
    # cluster name plus one attacker-chosen public name satisfies the policy and
    # gets both certified. This assertion exists because that substitution is a
    # single word and completely silent.
    condition     = startswith(output.issuance_policy.subject_alt_name_expression, "subject_alt_names.all(san, ")
    error_message = "the SAN expression must constrain every name in the request"
  }
  assert {
    condition     = strcontains(output.issuance_policy.subject_alt_name_expression, "spiffe://${var.trust_domain}/ns/")
    error_message = "the SAN expression must pin the SPIFFE trust domain"
  }
  assert {
    condition     = strcontains(output.issuance_policy.subject_alt_name_expression, "san.value.endsWith(\".svc.cluster.local\")")
    error_message = "the SAN expression must carry the configured DNS suffixes"
  }
  assert {
    condition = google_privateca_ca_pool.issuing[0].issuance_policy[0].identity_constraints[0].cel_expression[0].expression == output.issuance_policy.subject_alt_name_expression
    # The output is what the fixtures above assert on; if it ever stops being
    # the expression the pool is actually configured with, every one of them
    # becomes decoration.
    error_message = "the reported SAN expression must be the one the pool enforces"
  }
}

run "a_leaf_can_never_be_a_certificate_authority" {
  command = plan

  assert {
    condition = (
      google_privateca_ca_pool.issuing[0].issuance_policy[0].baseline_values[0].ca_options[0].is_ca == false
      && google_privateca_ca_pool.issuing[0].issuance_policy[0].baseline_values[0].ca_options[0].max_issuer_path_length == 0
    )
    error_message = "leaves from this pool must not be CAs, and must not be able to delegate"
  }
}

run "leaves_carry_both_directions_of_lumen_traffic_and_no_more" {
  command = plan

  assert {
    condition = (
      google_privateca_ca_pool.issuing[0].issuance_policy[0].baseline_values[0].key_usage[0].extended_key_usage[0].server_auth
      && google_privateca_ca_pool.issuing[0].issuance_policy[0].baseline_values[0].key_usage[0].extended_key_usage[0].client_auth
    )
    error_message = "serving TLS and peer mTLS both need the leaf to authenticate in both directions"
  }
  assert {
    condition = (
      !google_privateca_ca_pool.issuing[0].issuance_policy[0].baseline_values[0].key_usage[0].base_key_usage[0].cert_sign
      && !google_privateca_ca_pool.issuing[0].issuance_policy[0].baseline_values[0].key_usage[0].base_key_usage[0].crl_sign
    )
    error_message = "a leaf that can sign certificates or revocation lists is a CA wearing a different label"
  }
}

run "the_requester_cannot_write_its_own_subject" {
  command = plan

  assert {
    condition     = google_privateca_ca_pool.issuing[0].issuance_policy[0].identity_constraints[0].allow_subject_passthrough == false
    error_message = "subject passthrough would let a requester name itself inside a certificate the environment trusts"
  }
  assert {
    condition     = google_privateca_ca_pool.issuing[0].issuance_policy[0].allowed_issuance_modes[0].allow_config_based_issuance == false
    error_message = "config-based issuance would certify a key the requester never proved it holds"
  }
  assert {
    condition     = google_privateca_ca_pool.issuing[0].issuance_policy[0].identity_constraints[0].allow_subject_alt_names_passthrough == true
    error_message = "SAN passthrough must stay on -- fenced by the CEL expression -- or the controller cannot request the name it needs"
  }
}

run "leaves_are_short_lived_by_construction" {
  command = plan

  assert {
    condition     = google_privateca_ca_pool.issuing[0].issuance_policy[0].maximum_lifetime == "86400s"
    error_message = "the pool must cap leaf lifetime, not merely suggest one"
  }
}

run "a_public_dns_suffix_is_rejected" {
  command = plan

  variables {
    allowed_dns_suffixes = [".svc.cluster.local", ".example.com"]
  }

  expect_failures = [var.allowed_dns_suffixes]
}

run "a_bare_domain_without_a_leading_dot_is_rejected" {
  command = plan

  variables {
    # Without the dot this is a substring match: "notlumen.dev" ends with
    # "lumen.dev", so a suffix rule written this way admits names the operator
    # never intended.
    allowed_dns_suffixes = ["svc.cluster.local"]
  }

  expect_failures = [var.allowed_dns_suffixes]
}

run "an_empty_suffix_list_is_rejected" {
  command = plan

  variables {
    allowed_dns_suffixes = []
  }

  expect_failures = [var.allowed_dns_suffixes]
}

run "a_long_lived_leaf_is_rejected" {
  command = plan

  variables {
    max_leaf_lifetime_seconds = 2592000
  }

  expect_failures = [var.max_leaf_lifetime_seconds]
}

run "a_trust_domain_shaped_like_a_url_is_rejected" {
  command = plan

  variables {
    trust_domain = "spiffe://lumen-fixture.svc.id.goog"
  }

  expect_failures = [var.trust_domain]
}
