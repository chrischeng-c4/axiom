# The installation root's inputs: what an operator supplies once, for a cluster
# that already exists.
#
# Two independent authorities are named here on purpose -- trust (PKI) and
# capacity (machine pools). They are kept as separate input groups feeding
# separate modules rather than one merged "platform" object, because they change
# on completely different clocks and by completely different people: capacity
# moves when a Lumen instance is resized, trust moves almost never and under
# review. A single combined variable would make every capacity edit a diff
# against the certificate authority.

variable "project_id" {
  description = "Existing GCP project. This root creates neither the project, the VPC, the cluster, nor the DNS zone."
  type        = string

  validation {
    condition     = can(regex("^[a-z][a-z0-9-]{4,28}[a-z0-9]$", var.project_id))
    error_message = "project_id must be a valid GCP project id."
  }
}

variable "region" {
  description = "Region for CA Service issuing capacity and the shared machine pools."
  type        = string
  default     = "asia-east1"
}

variable "cluster_name" {
  description = "Existing GKE Standard cluster Lumen runs on. Referenced, never created."
  type        = string
}

# --- trust inputs (consumed by modules/lumen-pki) ----------------------------

variable "trust_domain" {
  description = "One SPIFFE trust domain for this environment."
  type        = string
}

variable "workload_identity_pool" {
  description = "Cluster workload identity pool, conventionally <project>.svc.id.goog."
  type        = string
}

variable "certificate_controller" {
  description = "The single in-cluster controller identity permitted to request leaves."
  type = object({
    namespace       = string
    service_account = string
  })
  default = {
    namespace       = "lumen-system"
    service_account = "lumen-cert-controller"
  }
}

variable "max_leaf_lifetime_seconds" {
  description = "Ceiling on issued leaf lifetime."
  type        = number
  default     = 86400
}

# --- capacity inputs (the #3066 module's contract) ---------------------------
#
# Declared here, validated here, and passed straight through. #3066 owns the
# node pools and the in-cluster capacity catalog; this root owns only the fact
# that both authorities are configured by one apply. Keeping the shape honest
# now is what makes adding the module block a one-line change rather than a
# renegotiation.

variable "capacity_profiles" {
  description = "Shared Lumen data-plane capacity, keyed by direct GCE machine type. Every profile declares an explicit maximum; there is no unbounded mode."
  type = map(object({
    min_nodes = optional(number, 0)
    max_nodes = number
  }))
  default = {}

  validation {
    condition = alltrue([
      for machine_type, profile in var.capacity_profiles :
      can(regex("^[a-z][a-z0-9]*-[a-z0-9-]+$", machine_type))
    ])
    error_message = "capacity_profiles must be keyed by a direct GCE machine type such as n2-standard-8; service tiers stay internal."
  }

  validation {
    condition = alltrue([
      for profile in values(var.capacity_profiles) :
      profile.max_nodes >= 1 && profile.min_nodes >= 0 && profile.min_nodes <= profile.max_nodes
    ])
    error_message = "every capacity profile needs an explicit max_nodes of at least 1 and a min_nodes between 0 and that maximum."
  }
}
