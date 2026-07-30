# Inputs for the Lumen private-PKI substrate.
#
# Everything security-critical that an operator could get wrong is a variable
# with a validation rule; everything security-critical that an operator has no
# business varying is not a variable at all (see main.tf's issuance policy).
# That split is deliberate: it is what lets `terraform test` prove the rejects
# with Terraform's own validation engine as the oracle, instead of a grep over
# the configuration text.

variable "project_id" {
  description = "Existing GCP project that owns the CA Service resources. This module never creates it."
  type        = string

  validation {
    condition     = can(regex("^[a-z][a-z0-9-]{4,28}[a-z0-9]$", var.project_id))
    error_message = "project_id must be a valid GCP project id."
  }
}

variable "region" {
  description = "Region holding the issuing capacity. One environment gets regional issuing capacity, not one CA per namespace."
  type        = string
  default     = "asia-east1"

  validation {
    # A zone here (asia-east1-a) would still create resources, just not where
    # the caller believes: CA Service locations are regional, and a mistyped
    # zonal string produces a second trust domain nobody is watching.
    condition     = can(regex("^[a-z]+-[a-z]+[0-9]$", var.region))
    error_message = "region must be a GCP region such as asia-east1, not a zone."
  }
}

variable "name_prefix" {
  description = "Prefix for every resource this module creates."
  type        = string
  default     = "lumen"

  validation {
    condition     = can(regex("^[a-z][a-z0-9-]{1,20}$", var.name_prefix))
    error_message = "name_prefix must be 2-21 lowercase letters, digits, or hyphens starting with a letter."
  }
}

variable "ca_pool_mode" {
  description = "'create' provisions a protected root plus an in-region issuing CA; 'existing' references an operator-approved pool and creates no hierarchy."
  type        = string
  default     = "create"

  validation {
    condition     = contains(["create", "existing"], var.ca_pool_mode)
    error_message = "ca_pool_mode must be either 'create' or 'existing'."
  }

  validation {
    # AC6. Terraform cannot see prior state from a variable, so the caller
    # declares whether a hierarchy this module created is still issuing. Once
    # it is, flipping to 'existing' -- which would orphan the created CAs and,
    # with deletion protection released, destroy them -- has to travel through
    # an explicit retirement acknowledgement rather than an input edit.
    condition = (
      var.ca_pool_mode != "existing"
      || !var.created_hierarchy_in_use
      || var.retirement.acknowledged
    )
    error_message = "the created hierarchy is still in use: retiring it requires retirement.acknowledged = true with a reason, not an edit to ca_pool_mode."
  }
}

variable "existing_ca_pool" {
  description = "Operator-approved pool to issue from when ca_pool_mode = 'existing'. Referenced, never recreated."
  type = object({
    name     = string
    location = string
  })
  default = null

  validation {
    condition     = var.ca_pool_mode != "existing" || var.existing_ca_pool != null
    error_message = "ca_pool_mode = 'existing' requires existing_ca_pool."
  }

  validation {
    condition     = var.ca_pool_mode != "create" || var.existing_ca_pool == null
    error_message = "existing_ca_pool is meaningless when ca_pool_mode = 'create'; drop it rather than leave two sources of truth."
  }
}

variable "workload_identity_pool" {
  description = "Cluster workload identity pool, conventionally <project>.svc.id.goog."
  type        = string

  validation {
    condition     = endswith(var.workload_identity_pool, ".svc.id.goog")
    error_message = "workload_identity_pool must be a GKE workload identity pool ending in .svc.id.goog."
  }
}

variable "certificate_controller" {
  description = "The one shared in-cluster certificate controller identity permitted to request leaves."
  type = object({
    namespace       = string
    service_account = string
  })

  validation {
    condition = alltrue([
      can(regex("^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$", var.certificate_controller.namespace)),
      can(regex("^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$", var.certificate_controller.service_account)),
    ])
    error_message = "certificate_controller.namespace and .service_account must be RFC 1123 label names."
  }

  validation {
    # R5: trusting the CA is not authorization, and issuance authority is not a
    # namespace-wide grant. A wildcard here would hand every workload in the
    # namespace the right to mint a Lumen-trusted identity.
    condition = !contains(
      ["*", "default", "system"],
      var.certificate_controller.service_account
    )
    error_message = "certificate_controller.service_account must name one dedicated controller ServiceAccount, never '*', 'default', or 'system'."
  }
}

variable "trust_domain" {
  description = "One SPIFFE trust domain per environment. Peer and serving identities are scoped inside it."
  type        = string

  validation {
    condition     = can(regex("^[a-z0-9]([a-z0-9.-]{0,253}[a-z0-9])?$", var.trust_domain))
    error_message = "trust_domain must be a DNS-shaped SPIFFE trust domain such as lumen-prod.svc.id.goog."
  }
}

variable "allowed_dns_suffixes" {
  description = "Kubernetes-internal DNS suffixes leaves may carry. Public names are not expressible here."
  type        = list(string)
  default     = [".svc.cluster.local", ".svc"]

  validation {
    condition     = length(var.allowed_dns_suffixes) > 0
    error_message = "allowed_dns_suffixes must not be empty; an empty list would leave the SAN policy trivially unsatisfiable."
  }

  validation {
    # R4/AC3. The suffix list is the whole DNS half of the issuance policy, so
    # this is the rule that makes "arbitrary public DNS names are rejected" a
    # property of the module rather than a promise in a comment. `.com`,
    # `.example.com`, and a bare `lumen.dev` all fail here.
    condition = alltrue([
      for suffix in var.allowed_dns_suffixes :
      startswith(suffix, ".") && (
        endswith(suffix, ".cluster.local") || suffix == ".svc" || endswith(suffix, ".svc")
      )
    ])
    error_message = "allowed_dns_suffixes must be Kubernetes-internal suffixes beginning with '.' and ending in '.svc' or '.cluster.local'; public DNS names are not issuable from this trust domain."
  }
}

variable "max_leaf_lifetime_seconds" {
  description = "Ceiling the issuing pool enforces on every leaf. Short-lived by construction."
  type        = number
  default     = 86400

  validation {
    condition     = var.max_leaf_lifetime_seconds >= 300 && var.max_leaf_lifetime_seconds <= 604800
    error_message = "max_leaf_lifetime_seconds must be between 300 (5m) and 604800 (7d): shorter cannot survive a controller outage, longer stops being a short-lived credential."
  }
}

variable "root_ca_lifetime_years" {
  description = "Lifetime of the protected root, in years."
  type        = number
  default     = 10

  validation {
    condition     = var.root_ca_lifetime_years >= 2 && var.root_ca_lifetime_years <= 25
    error_message = "root_ca_lifetime_years must be between 2 and 25."
  }
}

variable "issuing_ca_lifetime_years" {
  description = "Lifetime of the in-region issuing CA, in years."
  type        = number
  default     = 3

  validation {
    condition     = var.issuing_ca_lifetime_years >= 1 && var.issuing_ca_lifetime_years <= 10
    error_message = "issuing_ca_lifetime_years must be between 1 and 10."
  }

  validation {
    condition     = var.issuing_ca_lifetime_years <= var.root_ca_lifetime_years
    error_message = "the issuing CA cannot outlive the root that signed it."
  }
}

variable "issuing_tier" {
  description = "CA Service tier for the issuing pool. DEVOPS suits high-volume short-lived leaves; ENTERPRISE retains issued certificates."
  type        = string
  default     = "DEVOPS"

  validation {
    condition     = contains(["DEVOPS", "ENTERPRISE"], var.issuing_tier)
    error_message = "issuing_tier must be DEVOPS or ENTERPRISE."
  }
}

variable "enable_privateca_api" {
  description = "Enable privateca.googleapis.com in the project. Set false when a platform team enables services centrally."
  type        = bool
  default     = true
}

variable "additional_controller_roles" {
  description = "Extra roles for the controller principal on the issuing pool. Bounded to request/read; the widening roles are not expressible."
  type        = list(string)
  default     = []

  validation {
    # R3 names what must never be granted (CA admin, project editor, key
    # creation, Compute, GKE mutation). Rather than denylist them -- a denylist
    # is only as good as its last update -- this is the allowlist, so a role
    # nobody thought about is refused by default.
    condition = alltrue([
      for role in var.additional_controller_roles :
      contains([
        "roles/privateca.certificateRequester",
        "roles/privateca.workloadCertificateRequester",
        "roles/privateca.auditor",
      ], role)
    ])
    error_message = "additional_controller_roles may only add certificate-request or read-only CA Service roles; CA admin, project editor, credential creation, Compute, and GKE roles are not grantable through this module."
  }
}

variable "created_hierarchy_in_use" {
  description = "True once a hierarchy created by this module is issuing. Guards the retirement handoff in ca_pool_mode's validation."
  type        = bool
  default     = false
}

variable "retirement" {
  description = "Explicit handoff for retiring a created hierarchy. Acknowledging it is what releases deletion protection."
  type = object({
    acknowledged = bool
    reason       = string
  })
  default = {
    acknowledged = false
    reason       = ""
  }

  validation {
    condition     = !var.retirement.acknowledged || length(trimspace(var.retirement.reason)) >= 10
    error_message = "retirement.acknowledged requires a reason of at least 10 characters; the audit trail is the point of the handoff."
  }
}

variable "labels" {
  description = "Labels applied to the CA pools this module creates."
  type        = map(string)
  default     = {}
}
