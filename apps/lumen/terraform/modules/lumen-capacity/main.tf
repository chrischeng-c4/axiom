# Lumen shared capacity substrate on GKE.
#
# What this module owns: shared autoscaled data node pools on an existing GKE
# cluster (one per direct GCE machine type) and the in-cluster capacity
# catalog that records machine types, selectors, maxima, and lifecycle state.
#
# What it deliberately does not own: cluster lifecycle, general system node
# pools, VPC/network creation, or runtime mutation permissions. The operator
# reads capacity from the in-cluster catalog and schedules stateful pods with
# nodeSelector and tolerations; it never holds GCP API permissions to mutate
# capacity at runtime.

locals {
  active_profiles = {
    for machine_type, profile in var.capacity_profiles :
    machine_type => {
      min_nodes       = coalesce(profile.min_nodes, 0)
      max_nodes       = profile.max_nodes
      lifecycle_state = "ready"
    }
  }

  draining_profiles = {
    for machine_type, profile in var.draining_profiles :
    machine_type => {
      min_nodes       = coalesce(profile.min_nodes, 0)
      max_nodes       = profile.max_nodes
      lifecycle_state = "draining"
    }
  }

  all_profiles = merge(local.active_profiles, local.draining_profiles)

  selector_key = "lumen.axiom.dev/capacity-profile"
  taint_key    = "lumen.axiom.dev/capacity-profile"

  labels = merge({
    "lumen-component" = "capacity"
  }, var.labels)
}

resource "google_container_node_pool" "pools" {
  for_each = local.all_profiles

  project            = var.project_id
  cluster            = var.cluster_name
  location           = var.region
  name               = "${var.name_prefix}-data-${each.key}"
  initial_node_count = each.value.min_nodes

  autoscaling {
    min_node_count = each.value.min_nodes
    max_node_count = each.value.max_nodes
  }

  management {
    auto_repair  = true
    auto_upgrade = true
  }

  node_config {
    machine_type = each.key
    oauth_scopes = ["https://www.googleapis.com/auth/cloud-platform"]

    labels = merge(local.labels, {
      (local.selector_key) = each.key
    })

    taint {
      key    = local.taint_key
      value  = each.key
      effect = "NO_SCHEDULE"
    }
  }
}
