# Regression fixture for #2457/#2456: the integrator's mainstream GKE auth
# stack is Secret Manager + SecretProviderClass `provider: gke` +
# `auth: required` + `tokensSecretProviderClass`/`tokensSecretCsiDriver`. This
# is a run-scoped Secret Manager secret holding a minimal lumen
# token-registry.json payload, read by a second small `lumen-authcsi` CR
# through the GKE-managed Secrets Store CSI driver
# (`secrets-store-gke.csi.k8s.io`) in verify-lumen.sh's auth+CSI leg. Requires
# `secretmanager.googleapis.com` enabled on the project (checked in run.sh's
# required_apis list, same as every other required API — the harness never
# enables APIs itself).
resource "google_secret_manager_secret" "lumen_authcsi_tokens" {
  project   = var.project_id
  secret_id = "${local.prefix}-lumen-tokens"
  labels    = local.labels

  replication {
    auto {}
  }
}

# Payload shape mirrors `apps/lumen/src/spec.rs::token_registry_schema()`:
# a JSON object keyed by the exact bearer token string. The token value here
# is deterministic from `run_id` (not randomly generated) so verify-lumen.sh
# can compute the identical string independently — `axo-<run_id>-lumen-authcsi-token`
# — without a Terraform output roundtrip. Single admin-on-`*` token is enough
# to prove the auth+CSI mount path end to end; it is destroyed with the rest
# of this run's resources and never used past this leg.
resource "google_secret_manager_secret_version" "lumen_authcsi_tokens" {
  secret = google_secret_manager_secret.lumen_authcsi_tokens.id
  secret_data = jsonencode({
    "${local.prefix}-lumen-authcsi-token" = {
      subject = "gke-authcsi-acceptance"
      roles   = { "*" = "admin" }
    }
  })
}

# The `lumen-authcsi` CR (verify-lumen.sh) sets no `spec.serviceAccountName`,
# so the operator auto-creates and owns a Workload Identity KSA named after
# the instance: ns/lumen/sa/lumen-authcsi. Grant that federated principal
# read on the secret directly (no GSA in the middle) — mirrors
# `lumen_restore_reader` above, which is the reference pattern for granting a
# WI principal without a GSA.
resource "google_secret_manager_secret_iam_member" "lumen_authcsi_reader" {
  project   = var.project_id
  secret_id = google_secret_manager_secret.lumen_authcsi_tokens.secret_id
  role      = "roles/secretmanager.secretAccessor"
  member    = "principal://iam.googleapis.com/projects/${data.google_project.current.number}/locations/global/workloadIdentityPools/${var.project_id}.svc.id.goog/subject/ns/lumen/sa/lumen-authcsi"
}
