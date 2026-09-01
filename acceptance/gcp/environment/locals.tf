locals {
  prefix = "axo-${var.run_id}"
  labels = {
    "axiom-owner"  = "gcp-operator-acceptance"
    "axiom-run-id" = var.run_id
  }

  registry       = "${var.region}-docker.pkg.dev/${var.project_id}/${var.artifact_registry_repository}"
  backup_enabled = var.acceptance_apps != "lumen-auth"
  images = var.acceptance_apps == "tape" ? {
    tape = "${local.registry}/tape:${var.image_tag}"
    } : var.acceptance_apps == "sift" ? {
    sift = "${local.registry}/sift:${var.image_tag}"
    rig  = "${local.registry}/rig:${var.image_tag}"
    } : var.acceptance_apps == "lumen-auth" ? {
    lumen = "${local.registry}/lumen:${var.image_tag}"
    } : {
    lumen = "${local.registry}/lumen:${var.image_tag}"
    sift  = "${local.registry}/sift:${var.image_tag}"
  }
}

data "google_artifact_registry_repository" "existing" {
  project       = var.project_id
  location      = var.region
  repository_id = var.artifact_registry_repository
}
