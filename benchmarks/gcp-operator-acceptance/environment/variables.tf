variable "project_id" {
  description = "GCP project used for the disposable acceptance run."
  type        = string

  validation {
    condition     = can(regex("^[a-z][a-z0-9-]{4,28}[a-z0-9]$", var.project_id))
    error_message = "project_id must be a valid GCP project id."
  }
}

variable "region" {
  description = "GCP region for GKE, GCS, and the pre-existing Artifact Registry repository."
  type        = string
  default     = "asia-east1"
}

variable "gke_zone" {
  description = "Zonal Standard GKE control-plane location. Kept separate from the regional GCS, Artifact Registry, and Cloud Build location."
  type        = string
  default     = "asia-east1-a"

  validation {
    condition     = can(regex("^[a-z]+-[a-z]+[0-9]-[a-z]$", var.gke_zone))
    error_message = "gke_zone must be a GKE zone such as asia-east1-a."
  }
}

variable "node_machine_type" {
  description = "Smallest general-purpose Standard GKE machine shape used by this disposable acceptance run."
  type        = string
  default     = "e2-standard-2"
}

variable "node_min_count" {
  description = "Minimum node count for the disposable Standard GKE pool."
  type        = number
  default     = 1

  validation {
    condition     = var.node_min_count >= 1
    error_message = "node_min_count must keep at least one node available."
  }
}

variable "node_max_count" {
  description = "Maximum node count for the disposable Standard GKE pool."
  type        = number
  default     = 2

  validation {
    condition     = var.node_max_count >= var.node_min_count && var.node_max_count <= 2
    error_message = "node_max_count must be between node_min_count and 2 to keep the acceptance run bounded."
  }
}

variable "run_id" {
  description = "Lowercase run tag used to isolate every disposable resource."
  type        = string

  validation {
    condition     = can(regex("^[a-z0-9][a-z0-9-]{0,17}$", var.run_id))
    error_message = "run_id must be 1-18 lowercase letters, digits, or hyphens."
  }
}

variable "artifact_registry_repository" {
  description = "Existing Docker Artifact Registry repository. Terraform reads but never manages or deletes it."
  type        = string
  default     = "courier"
}

variable "image_tag" {
  description = "Run-unique image tag; workloads use the resolved immutable digest."
  type        = string
}
