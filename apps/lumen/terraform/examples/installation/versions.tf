terraform {
  required_version = ">= 1.9.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 6.0"
    }
  }
}

# The provider is configured here and only here. Neither child module declares
# one, which is what lets an operator drop them into an existing installation
# root that already has its own provider, aliases, and credentials.
provider "google" {
  project = var.project_id
  region  = var.region
}
