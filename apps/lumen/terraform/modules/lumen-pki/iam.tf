# Issuance authority for exactly one in-cluster identity.
#
# The grant is made to the cluster's federated principal directly -- no
# intermediate GCP service account, and therefore no long-lived credential to
# create, store, rotate, or leak (AC2). Workload Identity Federation resolves
# `ns/<namespace>/sa/<name>` from the projected token the controller already
# holds, so the trust path is: kube-apiserver vouches for the ServiceAccount,
# GCP honours that assertion, CA Service honours the two roles below. Nothing in
# that chain is a secret this module has to manage.

locals {
  # One principal, spelled once. Every binding derives from it, so widening the
  # grant means editing a subject here rather than adding a member somewhere.
  controller_principal = join("", [
    "principal://iam.googleapis.com/projects/${data.google_project.this.number}",
    "/locations/global/workloadIdentityPools/${var.workload_identity_pool}",
    "/subject/ns/${var.certificate_controller.namespace}",
    "/sa/${var.certificate_controller.service_account}",
  ])

  controller_roles = distinct(concat([
    # Ask for a leaf. Not manage, not revoke, not reconfigure the pool.
    "roles/privateca.certificateRequester",
    # Read the pool and its chain, so the controller can publish the trust
    # anchor into the cluster without being told what it is out of band.
    "roles/privateca.auditor",
  ], var.additional_controller_roles))
}

resource "google_privateca_ca_pool_iam_member" "controller" {
  for_each = toset(local.controller_roles)

  project  = var.project_id
  ca_pool  = local.issuing_pool_short
  location = local.issuing_pool_location
  role     = each.value
  member   = local.controller_principal
}
