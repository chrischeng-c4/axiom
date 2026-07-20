variable "project_id" {
  type = string
}

variable "region" {
  type    = string
  default = "asia-east1"
}

variable "run_id" {
  type = string

  validation {
    condition     = can(regex("^[a-z0-9-]{1,18}$", var.run_id))
    error_message = "run_id must be 1-18 lowercase letters, digits, or hyphens."
  }
}

variable "registry" {
  description = "Artifact Registry host/repository created by the bootstrap state."
  type        = string
}

variable "image_tag" {
  description = "Immutable tag shared by the four images built from this checkout."
  type        = string
}

variable "replay_samples" {
  description = "Independent named subscriptions used for full backlog drain samples."
  type        = number
  default     = 5

  validation {
    condition     = var.replay_samples >= 3 && var.replay_samples <= 9
    error_message = "replay_samples must be between 3 and 9."
  }
}
