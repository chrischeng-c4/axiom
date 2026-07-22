output "registry" {
  description = "Artifact Registry host/repository prefix used by Cloud Build."
  value       = "${var.region}-docker.pkg.dev/${var.project_id}/${google_artifact_registry_repository.images.repository_id}"
}

output "temporary_services" {
  description = "APIs that this state must disable again during teardown."
  value = sort(concat(
    keys(google_project_service.temporary),
    [google_project_service.container.service, google_project_service.file.service],
  ))
}
