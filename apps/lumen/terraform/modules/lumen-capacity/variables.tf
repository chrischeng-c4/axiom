# Inputs for the Lumen shared capacity module.
#
# Everything security-critical that an operator could get wrong is a variable
# with a validation rule; everything security-critical that an operator has no
# business varying is fixed in the implementation. That split is deliberate: it
# is what lets `terraform test` prove the rejects with Terraform's own
# validation engine as the oracle.

variable "project_id" {
  description = "Existing GCP project that owns the GKE cluster. This module never creates it."
  type        = string

  validation {
    condition     = can(regex("^[a-z][a-z0-9-]{4,28}[a-z0-9]$", var.project_id))
    error_message = "project_id must be a valid GCP project id."
  }
}

variable "region" {
  description = "Region holding the shared machine pools."
  type        = string
  default     = "asia-east1"

  validation {
    condition     = can(regex("^[a-z]+-[a-z]+[0-9]$", var.region))
    error_message = "region must be a GCP region such as asia-east1, not a zone."
  }
}

variable "cluster_name" {
  description = "Existing GKE Standard cluster Lumen runs on. Referenced, never created."
  type        = string

  validation {
    condition     = length(trimspace(var.cluster_name)) > 0
    error_message = "cluster_name must be a non-empty string."
  }
}

variable "name_prefix" {
  description = "Prefix for resources this module creates."
  type        = string
  default     = "lumen"

  validation {
    condition     = can(regex("^[a-z][a-z0-9-]{1,20}$", var.name_prefix))
    error_message = "name_prefix must be 2-21 lowercase letters, digits, or hyphens starting with a letter."
  }
}

variable "namespace" {
  description = "Kubernetes namespace where the in-cluster capacity catalog is published."
  type        = string
  default     = "lumen-system"

  validation {
    condition     = can(regex("^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$", var.namespace))
    error_message = "namespace must be a valid RFC 1123 label name."
  }
}

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

variable "draining_profiles" {
  description = "Profiles currently in draining state. Pools are retained and marked draining in the catalog."
  type = map(object({
    min_nodes = optional(number, 0)
    max_nodes = number
  }))
  default = {}

  validation {
    condition = alltrue([
      for machine_type, profile in var.draining_profiles :
      can(regex("^[a-z][a-z0-9]*-[a-z0-9-]+$", machine_type))
    ])
    error_message = "draining_profiles must be keyed by a direct GCE machine type."
  }

  validation {
    condition = alltrue([
      for profile in values(var.draining_profiles) :
      profile.max_nodes >= 1 && profile.min_nodes >= 0 && profile.min_nodes <= profile.max_nodes
    ])
    error_message = "every draining profile needs an explicit max_nodes of at least 1 and a min_nodes between 0 and that maximum."
  }

  validation {
    condition = alltrue([
      for machine_type in keys(var.draining_profiles) :
      !contains(keys(var.capacity_profiles), machine_type)
    ])
    error_message = "a profile cannot be simultaneously active in capacity_profiles and draining in draining_profiles."
  }
}

variable "created_pools_in_use" {
  description = "Machine types for which created pools are currently in use. Guards against accidental deletion during profile removal."
  type        = list(string)
  default     = []

  validation {
    condition = (
      length(var.created_pools_in_use) == 0
      || alltrue([
        for m in var.created_pools_in_use :
        contains(keys(var.capacity_profiles), m) || contains(keys(var.draining_profiles), m)
      ])
      || var.retirement.acknowledged
    )
    error_message = "capacity pools currently in use cannot be removed without entering draining or setting retirement.acknowledged = true with a reason."
  }
}

variable "retirement" {
  description = "Explicit handoff for retiring capacity profiles. Acknowledging it is what permits pool deletion."
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
  description = "Labels applied to the resources this module creates."
  type        = map(string)
  default     = {}
}
