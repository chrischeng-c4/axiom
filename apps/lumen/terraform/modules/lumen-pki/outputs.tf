# What crosses the module boundary: names, issuer configuration, and public
# trust material. Nothing here is a credential.
#
# The controller does not receive certificate material through Terraform at all
# -- it fetches the chain from CA Service using the identity granted in iam.tf.
# That is what keeps routine renewal outside `terraform apply` (R7): these
# outputs are stable across every rotation, so nothing downstream has a reason
# to re-read them when a leaf expires.

output "issuing_ca_pool_id" {
  description = "Fully-qualified issuing pool the controller requests leaves from."
  value       = local.issuing_pool_id
}

output "issuing_ca_pool_name" {
  description = "Short name of the issuing pool."
  value       = local.issuing_pool_short
}

output "issuing_ca_pool_location" {
  description = "Region holding the issuing capacity."
  value       = local.issuing_pool_location
}

output "trust_domain" {
  description = "The single SPIFFE trust domain leaves are scoped inside."
  value       = var.trust_domain
}

output "certificate_controller_principal" {
  description = "Federated principal granted issuance authority. Resolvable from the controller's projected token; not a credential."
  value       = local.controller_principal
}

output "certificate_controller_roles" {
  description = "Every role bound to the controller principal on the issuing pool."
  value       = local.controller_roles
}

output "manages_hierarchy" {
  description = "True when this module created the trust hierarchy; false when it references an operator-approved pool."
  value       = local.create
}

output "root_ca_id" {
  description = "Created root CA, or null in referenced-pool mode."
  value       = local.create ? google_privateca_certificate_authority.root[0].id : null
}

output "issuing_ca_id" {
  description = "Created issuing CA, or null in referenced-pool mode."
  value       = local.create ? google_privateca_certificate_authority.issuing[0].id : null
}

output "trust_anchor_pem" {
  description = "Public CA chain for the created hierarchy. Public material by definition: it is what verifiers are supposed to already have."
  value       = local.create ? google_privateca_certificate_authority.root[0].pem_ca_certificates : null
}

output "deletion_protected" {
  description = "Whether the created CAs currently refuse destruction. False only after an acknowledged retirement."
  value       = local.create ? !var.retirement.acknowledged : null
}

# The issuance boundary, in a shape a test can assert on. These are not knobs --
# every field below is fixed in main.tf or derived from a validated input -- but
# a boundary nobody can observe is a boundary nobody notices losing, and the
# accept/reject fixtures in tests/ read exactly this.
output "issuance_policy" {
  description = "Effective issuance constraints of the created issuing pool, or null when referencing an operator-approved pool."
  value = local.create ? {
    maximum_lifetime_seconds            = var.max_leaf_lifetime_seconds
    leaf_is_ca                          = false
    max_issuer_path_length              = 0
    extended_key_usage                  = ["server_auth", "client_auth"]
    allow_subject_passthrough           = false
    allow_subject_alt_names_passthrough = true
    allow_config_based_issuance         = false
    subject_alt_name_expression         = local.san_cel
    allowed_dns_suffixes                = var.allowed_dns_suffixes
  } : null
}

output "module_version" {
  description = "Version of this module's input/output contract. Consumers pin it by git ref; this is what they assert against."
  value       = "1.0.0"
}
