terraform {
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

# The provider is configured here and only here. Neither child module declares
# one, which is what lets an operator drop them into an existing installation
# root that already has its own provider, aliases, and credentials.
provider "google" {
  project = var.project_id
  region  = var.region
}

provider "kubernetes" {
  host                   = "https://${data.google_container_cluster.lumen.endpoint}"
  token                  = data.google_client_config.default.access_token
  cluster_ca_certificate = base64decode(data.google_container_cluster.lumen.master_auth[0].cluster_ca_certificate)
}
