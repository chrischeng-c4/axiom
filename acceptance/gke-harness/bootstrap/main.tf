# One-time bootstrap for the GKE acceptance harness: the terraform state
# bucket and the keyless GitHub Actions -> GCP path (OIDC / Workload Identity
# Federation). Applied ONCE by a human with local state; everything downstream
# (cluster module, CI) consumes the outputs. See ../README.md for the ordering.
locals {
  state_bucket = var.state_bucket_name != "" ? var.state_bucket_name : "${var.project_id}-axiom-tfstate"
}

# WIF token exchange needs STS + IAM Credentials. disable_on_destroy = false:
# this bootstrap is persistent infrastructure, and turning project APIs off is
# never a side effect a teardown of *this* module should have.
resource "google_project_service" "wif" {
  for_each = toset([
    "sts.googleapis.com",
    "iamcredentials.googleapis.com",
  ])

  project            = var.project_id
  service            = each.value
  disable_on_destroy = false
}

# Terraform state for the persistent acceptance cluster. The previous home was
# /tmp on one workstation, and acceptance/gcp/scripts/destroy-cluster.sh
# documents exactly how that goes wrong; versioned GCS is the fix.
resource "google_storage_bucket" "tfstate" {
  project                     = var.project_id
  name                        = local.state_bucket
  location                    = var.region
  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"

  versioning {
    enabled = true
  }

  labels = { "axiom-owner" = "gke-harness-bootstrap" }
}

resource "google_iam_workload_identity_pool" "github" {
  project                   = var.project_id
  workload_identity_pool_id = "github-actions"
  display_name              = "GitHub Actions"
  description               = "OIDC trust for ${var.github_repository} workflows"
}

# The attribute condition is the security boundary: only tokens minted for
# this exact repository can exchange through this provider, so a fork or an
# unrelated repo in the same org gets nothing.
resource "google_iam_workload_identity_pool_provider" "github" {
  project                            = var.project_id
  workload_identity_pool_id          = google_iam_workload_identity_pool.github.workload_identity_pool_id
  workload_identity_pool_provider_id = "github-oidc"
  display_name                       = "GitHub OIDC"
  attribute_condition                = "assertion.repository == \"${var.github_repository}\""

  attribute_mapping = {
    "google.subject"       = "assertion.sub"
    "attribute.repository" = "assertion.repository"
  }

  oidc {
    issuer_uri = "https://token.actions.githubusercontent.com"
  }

  depends_on = [google_project_service.wif]
}

resource "google_service_account" "deployer" {
  project      = var.project_id
  account_id   = "axiom-gke-deployer"
  display_name = "GitHub Actions GKE acceptance deployer"
}

# container.admin: create/resize/get-credentials on the acceptance cluster.
# The cluster module also creates the node service account and its IAM
# bindings, so the deployer needs to manage SAs and project IAM for those two
# roles as well when the cluster is first applied from CI.
resource "google_project_iam_member" "deployer_container" {
  project = var.project_id
  role    = "roles/container.admin"
  member  = "serviceAccount:${google_service_account.deployer.email}"
}

resource "google_project_iam_member" "deployer_sa_admin" {
  project = var.project_id
  role    = "roles/iam.serviceAccountAdmin"
  member  = "serviceAccount:${google_service_account.deployer.email}"
}

resource "google_storage_bucket_iam_member" "deployer_state" {
  bucket = google_storage_bucket.tfstate.name
  role   = "roles/storage.objectAdmin"
  member = "serviceAccount:${google_service_account.deployer.email}"
}

# Lets workflows from the trusted repository impersonate the deployer SA.
resource "google_service_account_iam_member" "deployer_wif" {
  service_account_id = google_service_account.deployer.name
  role               = "roles/iam.workloadIdentityUser"
  member             = "principalSet://iam.googleapis.com/${google_iam_workload_identity_pool.github.name}/attribute.repository/${var.github_repository}"
}
