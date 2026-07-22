resource "random_password" "receiver_secret" {
  length  = 32
  special = false
}

resource "google_cloud_run_v2_service" "receiver" {
  project             = var.project_id
  name                = "${local.prefix}-receiver"
  location            = var.region
  deletion_protection = false
  ingress             = "INGRESS_TRAFFIC_ALL"

  template {
    timeout = "30s"

    scaling {
      min_instance_count = 0
      max_instance_count = 1
    }

    containers {
      image = local.receiver_image

      ports {
        container_port = 8080
      }

      env {
        name  = "BENCH_SECRET"
        value = random_password.receiver_secret.result
      }

      resources {
        cpu_idle = true
        limits = {
          cpu    = "1"
          memory = "512Mi"
        }
      }
    }

    labels = local.labels
  }

  labels = local.labels
}

resource "google_cloud_run_v2_service_iam_member" "public_receiver" {
  project  = var.project_id
  location = google_cloud_run_v2_service.receiver.location
  name     = google_cloud_run_v2_service.receiver.name
  role     = "roles/run.invoker"
  member   = "allUsers"
}
