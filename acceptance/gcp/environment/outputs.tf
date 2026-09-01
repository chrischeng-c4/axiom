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
  value = local.backup_enabled ? google_storage_bucket.backups[0].name : null
}

output "backup_gsa_email" {
  value = local.backup_enabled ? google_service_account.backup[0].email : null
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

output "sift_node_pool" {
  value = var.acceptance_apps == "sift" ? google_container_node_pool.sift_mvp[0].name : null
}
