# Outputs for the Lumen capacity authority.
#
# Exposes selector and catalog identities without exposing internal pool names
# or sensitive cloud infrastructure details.

output "catalog_config_map_name" {
  description = "Name of the in-cluster ConfigMap publishing the capacity catalog."
  value       = kubernetes_config_map.catalog.metadata[0].name
}

output "catalog_config_map_namespace" {
  description = "Namespace of the in-cluster ConfigMap publishing the capacity catalog."
  value       = kubernetes_config_map.catalog.metadata[0].namespace
}

output "selectors" {
  description = "Stable node selectors mapping machine type to label selector map."
  value = {
    for machine_type, profile in local.all_profiles :
    machine_type => {
      (local.selector_key) = machine_type
    }
  }
}

output "tolerations" {
  description = "Tolerations required for workloads to land on Lumen shared data pools."
  value = {
    for machine_type, profile in local.all_profiles :
    machine_type => {
      key      = local.taint_key
      operator = "Equal"
      value    = machine_type
      effect   = "NoSchedule"
    }
  }
}

output "module_version" {
  description = "Version of this module's input/output contract."
  value       = "1.0.0"
}
