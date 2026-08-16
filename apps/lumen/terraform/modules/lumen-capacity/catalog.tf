# In-cluster capacity catalog.
#
# Published as a Kubernetes ConfigMap in the Lumen system namespace. The
# operator reads this object directly via in-cluster RBAC to discover available
# machine types, their selectors, declared bounds, and lifecycle state.
#
# This catalog is complete and readable even when every pool holds zero nodes,
# decoupling capacity discovery from Node inventory or GCP API calls.

resource "kubernetes_config_map" "catalog" {
  metadata {
    name      = "${var.name_prefix}-capacity-catalog"
    namespace = var.namespace
    labels    = local.labels
  }

  data = {
    "catalog.json" = jsonencode({
      version = "1.0.0"
      entries = [
        for machine_type, profile in local.all_profiles : {
          machine_type = machine_type
          selector     = "${local.selector_key}=${machine_type}"
          stable_selector = {
            key   = local.selector_key
            value = machine_type
          }
          max_nodes       = profile.max_nodes
          min_nodes       = profile.min_nodes
          lifecycle_state = profile.lifecycle_state
          pool_group      = "lumen-data"
        }
      ]
    })
  }
}
