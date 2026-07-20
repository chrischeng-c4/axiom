variable "project_id" {
  description = "Existing billed GCP project used only for the bounded benchmark run."
  type        = string
}

variable "region" {
  description = "One region shared by GKE, Pub/Sub persistence, Cloud Tasks, and Cloud Run."
  type        = string
  default     = "asia-east1"
}

variable "run_id" {
  description = "Short lowercase run identifier used to make every disposable resource unique."
  type        = string

  validation {
    condition     = can(regex("^[a-z0-9-]{1,18}$", var.run_id))
    error_message = "run_id must be 1-18 lowercase letters, digits, or hyphens."
  }
}
