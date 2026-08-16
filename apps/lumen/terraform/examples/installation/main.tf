# One installation apply, two independent authorities.
#
# This root exists to prove a composition property, not to be the only way to
# install Lumen: the PKI module can be applied on its own, the capacity module
# can be applied on its own, and putting them in one root does not merge their
# ownership. Nothing below reads a PKI output into a capacity input or the
# reverse -- if it did, resizing a machine pool would plan a change against the
# certificate authority.

data "google_container_cluster" "lumen" {
  project  = var.project_id
  name     = var.cluster_name
  location = var.region
}

module "pki" {
  source = "../../modules/lumen-pki"

  project_id             = var.project_id
  region                 = var.region
  trust_domain           = var.trust_domain
  workload_identity_pool = var.workload_identity_pool
  certificate_controller = var.certificate_controller

  max_leaf_lifetime_seconds = var.max_leaf_lifetime_seconds

  labels = {
    "lumen-cluster" = var.cluster_name
  }
}

data "google_client_config" "default" {}

module "capacity" {
  source            = "../../modules/lumen-capacity"
  project_id        = var.project_id
  region            = var.region
  cluster_name      = data.google_container_cluster.lumen.name
  capacity_profiles = var.capacity_profiles
}
