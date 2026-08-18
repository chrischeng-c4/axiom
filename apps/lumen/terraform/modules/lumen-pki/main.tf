# Lumen private-PKI substrate on GCP CA Service.
#
# What this module owns: the long-lived trust substrate -- a protected root, an
# in-region issuing CA, the issuance policy that bounds what a leaf may ever
# claim, and the narrow IAM that lets one in-cluster controller ask for leaves.
#
# What it deliberately does not own, and must never grow into: leaf issuance.
# There is no certificate resource in this module and no rotation actuator. A
# renewed leaf or a replaced Pod must not require `terraform apply` (R7); the
# controller talks to the CAS API directly with the identity granted in iam.tf.
# The one-line test for any future addition here is whether it would have to run
# again when a certificate expires. If yes, it belongs in the controller.

locals {
  create = var.ca_pool_mode == "create"

  root_pool_name    = "${var.name_prefix}-root"
  issuing_pool_name = "${var.name_prefix}-issuing"

  # The SAN policy, as CA Service evaluates it. Two shapes are issuable and
  # nothing else: a Kubernetes-internal DNS name, and a SPIFFE URI inside this
  # environment's single trust domain. `all()` is load-bearing -- `exists()`
  # would let a request smuggle an arbitrary name alongside a valid one.
  dns_clause = join(" || ", [
    for suffix in var.allowed_dns_suffixes : "san.value.endsWith(\"${suffix}\")"
  ])

  san_cel = join("", [
    "subject_alt_names.all(san, ",
    "(san.type == DNS && (${local.dns_clause}))",
    " || ",
    "(san.type == URI && san.value.startsWith(\"spiffe://${var.trust_domain}/ns/\"))",
    ")",
  ])

  # Issuing-pool identity, whichever way we got here. Downstream (iam.tf,
  # outputs.tf) reads only this, so the create/existing split stays in one file.
  issuing_pool_id = local.create ? (
    "projects/${var.project_id}/locations/${var.region}/caPools/${local.issuing_pool_name}"
    ) : (
    "projects/${var.project_id}/locations/${var.existing_ca_pool.location}/caPools/${var.existing_ca_pool.name}"
  )

  issuing_pool_location = local.create ? var.region : var.existing_ca_pool.location
  issuing_pool_short    = local.create ? local.issuing_pool_name : var.existing_ca_pool.name

  labels = merge({
    "lumen-component" = "private-pki"
  }, var.labels)
}

data "google_project" "this" {
  project_id = var.project_id
}

resource "google_project_service" "privateca" {
  count = var.enable_privateca_api ? 1 : 0

  project = var.project_id
  service = "privateca.googleapis.com"

  # Disabling CA Service on `terraform destroy` would take every unrelated CA in
  # the project down with it. Enabling an API is not an ownership claim.
  disable_on_destroy = false
}

# --- the protected root ------------------------------------------------------
# One root per environment trust domain (R5). It signs the issuing CA and
# nothing else, so it is deliberately boring: long lifetime, deletion protection
# on, and no issuance policy of its own beyond CA-signing.

resource "google_privateca_ca_pool" "root" {
  count = local.create ? 1 : 0

  project  = var.project_id
  name     = local.root_pool_name
  location = var.region
  tier     = "ENTERPRISE"
  labels   = local.labels

  publishing_options {
    publish_ca_cert = true
    publish_crl     = true
  }

  depends_on = [google_project_service.privateca]
}

resource "google_privateca_certificate_authority" "root" {
  count = local.create ? 1 : 0

  project                  = var.project_id
  pool                     = google_privateca_ca_pool.root[0].name
  location                 = var.region
  certificate_authority_id = "${var.name_prefix}-root-ca"
  type                     = "SELF_SIGNED"
  lifetime                 = "${var.root_ca_lifetime_years * 365 * 24 * 3600}s"

  # AC6, the enforcing half. Acknowledged retirement is the only input that
  # releases this, which is what makes "removing an in-use CA fails closed" a
  # property of the plan rather than a review convention.
  deletion_protection = !var.retirement.acknowledged

  key_spec {
    algorithm = "RSA_PKCS1_4096_SHA256"
  }

  config {
    subject_config {
      subject {
        organization = "lumen"
        common_name  = "${var.name_prefix} ${var.trust_domain} root"
      }
    }
    x509_config {
      ca_options {
        is_ca                  = true
        max_issuer_path_length = 1
      }
      key_usage {
        base_key_usage {
          cert_sign         = true
          crl_sign          = true
          digital_signature = true
        }
        extended_key_usage {}
      }
    }
  }
}

# --- the in-region issuing CA ------------------------------------------------
# Regional issuing capacity for the whole environment, not one CA per Lumen
# namespace. The issuance policy below is the security boundary: it is what
# makes a leaf from this pool unable to be a CA, unable to carry a name outside
# the cluster, and unable to outlive its rotation window.

resource "google_privateca_ca_pool" "issuing" {
  count = local.create ? 1 : 0

  project  = var.project_id
  name     = local.issuing_pool_name
  location = var.region
  tier     = var.issuing_tier
  labels   = local.labels

  publishing_options {
    publish_ca_cert = true
    publish_crl     = var.issuing_tier == "ENTERPRISE"
  }

  issuance_policy {
    maximum_lifetime = "${var.max_leaf_lifetime_seconds}s"

    allowed_issuance_modes {
      # CSR-based only. Config-based issuance would let the caller hand CA
      # Service a subject instead of proving possession of the key being
      # certified, which is the whole reason the controller holds material this
      # module never sees.
      allow_csr_based_issuance    = true
      allow_config_based_issuance = false
    }

    allowed_key_types {
      elliptic_curve {
        signature_algorithm = "ECDSA_P256"
      }
    }
    allowed_key_types {
      rsa {
        min_modulus_size = 2048
      }
    }

    identity_constraints {
      # Not variables, on purpose. Subject passthrough would let a requester
      # write its own organization/common name into a certificate the whole
      # environment trusts; SAN passthrough is required (the controller must be
      # able to ask for the Service DNS name it needs) but is fenced by the CEL
      # expression, which is the constrained form of the same capability.
      allow_subject_passthrough           = false
      allow_subject_alt_names_passthrough = true

      cel_expression {
        title       = "lumen-cluster-identities"
        description = "Kubernetes-internal DNS names and SPIFFE URIs inside this environment's trust domain only."
        expression  = local.san_cel
      }
    }

    baseline_values {
      ca_options {
        is_ca = false
        # A leaf that cannot sign cannot delegate. Combined with is_ca = false
        # this is what stops a compromised controller from minting a parallel CA
        # that the same trust anchor would honour.
        max_issuer_path_length = 0
      }
      key_usage {
        base_key_usage {
          digital_signature = true
          key_encipherment  = true
          # Spelled out rather than omitted. Both default to unset, which CA
          # Service reads as false -- but "absent" and "false" are not the same
          # thing to a reviewer, and these are the two bits that would turn a
          # leaf into a CA in everything but name.
          cert_sign = false
          crl_sign  = false
        }
        extended_key_usage {
          # Both, because one leaf profile serves both directions of Lumen's
          # traffic: a Raft peer is a server to its callers and a client to its
          # peers on the same connection pair.
          server_auth = true
          client_auth = true
        }
      }
    }
  }

  depends_on = [google_project_service.privateca]
}

resource "google_privateca_certificate_authority" "issuing" {
  count = local.create ? 1 : 0

  project                  = var.project_id
  pool                     = google_privateca_ca_pool.issuing[0].name
  location                 = var.region
  certificate_authority_id = "${var.name_prefix}-issuing-ca"
  type                     = "SUBORDINATE"
  lifetime                 = "${var.issuing_ca_lifetime_years * 365 * 24 * 3600}s"

  deletion_protection = !var.retirement.acknowledged

  subordinate_config {
    certificate_authority = google_privateca_certificate_authority.root[0].id
  }

  key_spec {
    algorithm = "RSA_PKCS1_4096_SHA256"
  }

  config {
    subject_config {
      subject {
        organization = "lumen"
        common_name  = "${var.name_prefix} ${var.trust_domain} issuing"
      }
    }
    x509_config {
      ca_options {
        is_ca                  = true
        max_issuer_path_length = 0
      }
      key_usage {
        base_key_usage {
          cert_sign         = true
          crl_sign          = true
          digital_signature = true
        }
        extended_key_usage {}
      }
    }
  }
}

# --- the referenced pool -----------------------------------------------------
# `existing` mode has no resources at all: every block above is `count = 0`, and
# the pool is addressed by the id composed in `local.issuing_pool_id`. That
# absence is the guarantee -- there is no code path in this mode that can
# propose creating, mutating, or destroying the operator's pool, only the two
# IAM bindings in iam.tf, which fail closed at apply if the pool does not exist.
#
# The provider ships no `google_privateca_ca_pool` data source, so a plan-time
# existence check is not available to buy here; `terraform plan` in this mode
# proposing zero CA resources is the property the fixtures assert instead.
