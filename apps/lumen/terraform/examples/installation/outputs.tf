# What an installation publishes for the in-cluster controllers to consume.
# Names and public configuration only -- the controller fetches certificate
# material from CA Service itself, so nothing here changes when a leaf rotates.

output "issuing_ca_pool_id" {
  description = "Pool the certificate controller requests leaves from."
  value       = module.pki.issuing_ca_pool_id
}

output "trust_domain" {
  description = "SPIFFE trust domain for this environment."
  value       = module.pki.trust_domain
}

output "certificate_controller_principal" {
  description = "Federated principal the controller authenticates as."
  value       = module.pki.certificate_controller_principal
}

output "issuance_policy" {
  description = "Effective issuance constraints, for the acceptance harness to assert against."
  value       = module.pki.issuance_policy
}

output "cluster_name" {
  description = "The pre-existing cluster this installation targets."
  value       = data.google_container_cluster.lumen.name
}

output "capacity_profiles" {
  description = "Capacity contract handed to the #3066 module. Republished so the two authorities stay separately inspectable."
  value       = var.capacity_profiles
}
