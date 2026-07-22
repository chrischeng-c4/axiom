locals {
  prefix = "axiom-bench-${var.run_id}"
}

# These APIs were deliberately disabled before the benchmark was introduced.
# Owning them here means `terraform destroy` turns them back off after all
# resources in the environment state have been removed.
resource "google_project_service" "temporary" {
  for_each = toset([
    "cloudtasks.googleapis.com",
    "sts.googleapis.com",
  ])

  project            = var.project_id
  service            = each.value
  disable_on_destroy = true
}

resource "google_project_service" "container" {
  project            = var.project_id
  service            = "container.googleapis.com"
  disable_on_destroy = true
}

# GKE enables the Filestore API, and Service Usage reports File as a dependent
# of Container. Model that direction so creation is Container -> File and
# teardown is File -> Container.
resource "google_project_service" "file" {
  project            = var.project_id
  service            = "file.googleapis.com"
  disable_on_destroy = true

  depends_on = [google_project_service.container]
}

resource "google_artifact_registry_repository" "images" {
  project       = var.project_id
  location      = var.region
  repository_id = local.prefix
  description   = "Disposable Tape/Defer managed-service benchmark images"
  format        = "DOCKER"

  labels = {
    purpose = "axiom-managed-bench"
    run_id  = var.run_id
  }

}
