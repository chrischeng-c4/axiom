variable "project_id" {
  description = "Billed GCP project that owns the persistent acceptance cluster."
  type        = string
  validation {
    condition     = can(regex("^[a-z][a-z0-9-]{4,28}[a-z0-9]$", var.project_id))
    error_message = "project_id must be a real-looking GCP project ID."
  }
}

variable "region" {
  description = "Region for the tfstate bucket; matches the acceptance cluster's region."
  type        = string
  default     = "asia-east1"
}

variable "github_repository" {
  description = "owner/name of the GitHub repository allowed to assume the deployer SA via OIDC."
  type        = string
  default     = "chrischeng-c4/axiom"
  validation {
    condition     = can(regex("^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$", var.github_repository))
    error_message = "github_repository must be owner/name."
  }
}

variable "state_bucket_name" {
  description = "Globally unique GCS bucket for terraform state. Empty derives <project_id>-axiom-tfstate."
  type        = string
  default     = ""
}
