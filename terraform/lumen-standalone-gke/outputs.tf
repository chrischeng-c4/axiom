output "project_id" { value = var.project_id }
output "region" { value = var.region }
output "gke_zone" { value = var.gke_zone }
output "cluster_name" { value = google_container_cluster.standalone.name }
output "node_pool_name" { value = google_container_node_pool.standalone.name }
output "node_selector" { value = { "cloud.google.com/gke-nodepool" = google_container_node_pool.standalone.name } }
output "storage_class_name" { value = var.storage_class_name }
output "workload_identity_pool" { value = google_container_cluster.standalone.workload_identity_config[0].workload_pool }
output "node_service_account_email" { value = google_service_account.nodes.email }
output "run_id" { value = var.run_id }
