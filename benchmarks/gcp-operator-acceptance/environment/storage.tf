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
  display_name = "Disposable Lumen and Sift backup writer"
}

resource "google_storage_bucket_iam_member" "backup_writer" {
  bucket = google_storage_bucket.backups.name
  role   = "roles/storage.objectAdmin"
  member = "serviceAccount:${google_service_account.backup.email}"
}

resource "google_service_account_iam_member" "backup_workload_identity" {
  for_each = toset([
    "lumen/lumen-backup",
    "sift/sift-backup",
  ])

  service_account_id = google_service_account.backup.name
  role               = "roles/iam.workloadIdentityUser"
  member             = "serviceAccount:${var.project_id}.svc.id.goog[${each.value}]"

  depends_on = [google_container_cluster.acceptance]
}
