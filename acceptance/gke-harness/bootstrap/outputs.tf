output "state_bucket" {
  description = "GCS bucket holding terraform state; pass as -backend-config to the cluster module."
  value       = google_storage_bucket.tfstate.name
}

output "wif_provider" {
  description = "Set as GitHub repo variable GCP_WIF_PROVIDER."
  value       = google_iam_workload_identity_pool_provider.github.name
}

output "deployer_sa_email" {
  description = "Set as GitHub repo variable GCP_DEPLOYER_SA."
  value       = google_service_account.deployer.email
}

output "github_variable_commands" {
  description = "Run these once to wire the repository to this bootstrap."
  value = join("\n", [
    "gh variable set GCP_PROJECT_ID --body '${var.project_id}'",
    "gh variable set GCP_TFSTATE_BUCKET --body '${google_storage_bucket.tfstate.name}'",
    "gh variable set GCP_WIF_PROVIDER --body '${google_iam_workload_identity_pool_provider.github.name}'",
    "gh variable set GCP_DEPLOYER_SA --body '${google_service_account.deployer.email}'",
  ])
}
