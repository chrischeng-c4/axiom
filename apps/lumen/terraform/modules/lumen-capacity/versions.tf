terraform {
  # 1.9 is the floor for a reason, not a habit: several of this module's
  # security boundaries are expressed as variable validations that read *other*
  # variables (cross-variable validation landed in 1.9.0). On an older CLI those
  # rules would be a syntax error rather than a silently weaker gate.
  required_version = ">= 1.9.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 6.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.30"
    }
  }
}
