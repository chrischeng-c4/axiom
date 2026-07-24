resource "google_storage_bucket" "backups" {
  project                     = var.project_id
  name                        = "${var.project_id}-${local.prefix}-backup"
  location                    = var.region
  force_destroy               = true
  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"
  labels                      = local.labels

  lifecycle_rule {
    condition {
      age = 1
    }
    action {
      type = "Delete"
    }
  }
}

resource "google_service_account" "backup" {
  project      = var.project_id
  account_id   = "${local.prefix}-backup"
  display_name = "Disposable acceptance backup writer"
}

resource "google_storage_bucket_iam_member" "backup_writer" {
  bucket = google_storage_bucket.backups.name
  role   = "roles/storage.objectAdmin"
  member = "serviceAccount:${google_service_account.backup.email}"
}

resource "google_service_account_iam_member" "backup_workload_identity" {
  for_each = toset(concat(
    ["lumen/lumen-backup"],
    var.lumen_only ? [] : ["sift/sift-backup"],
  ))

  service_account_id = google_service_account.backup.name
  role               = "roles/iam.workloadIdentityUser"
  member             = "serviceAccount:${var.project_id}.svc.id.goog[${each.value}]"

  depends_on = [data.google_container_cluster.acceptance]
}

# The cold-restore leg (#2492) seeds a fresh-PVC `lumen-restore` instance
# straight from a backup object at pod startup (bootstrap seedUri). Unlike the
# backup Job — which writes through the `lumen-backup` KSA bound to the backup
# GSA above — the serving pod reads GCS through its OWN auto-created Workload
# Identity KSA (ns/lumen/sa/lumen-restore), which carries no GSA binding and so
# hits HTTP 403 without an explicit grant. Grant that federated principal read
# on the backup bucket. This mirrors the deployer responsibility any real
# cold-restore integrator carries: the serving ServiceAccount that seeds from
# GCS needs objectViewer on the seed bucket.
data "google_project" "current" {
  project_id = var.project_id
}

resource "google_storage_bucket_iam_member" "lumen_restore_reader" {
  bucket = google_storage_bucket.backups.name
  role   = "roles/storage.objectViewer"
  member = "principal://iam.googleapis.com/projects/${data.google_project.current.number}/locations/global/workloadIdentityPools/${var.project_id}.svc.id.goog/subject/ns/lumen/sa/lumen-restore"
}
