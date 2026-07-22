output "cluster_name" {
  value = data.google_container_cluster.acceptance.name
}

output "region" {
  value = var.region
}

output "gke_zone" {
  value = var.gke_zone
}

output "backup_bucket" {
  value = google_storage_bucket.backups.name
}

output "backup_gsa_email" {
  value = google_service_account.backup.email
}

output "registry" {
  value = local.registry
}

output "tagged_images" {
  value = local.images
}

output "artifact_registry_repository" {
  value = data.google_artifact_registry_repository.existing.name
}
