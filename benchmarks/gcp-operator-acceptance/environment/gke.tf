data "google_container_cluster" "acceptance" {
  project  = var.project_id
  name     = var.cluster_name
  location = var.gke_zone
}
