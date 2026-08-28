#!/usr/bin/env bash
set -euo pipefail

module_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/lumen-tf-check.XXXXXX")"
trap 'rm -rf "$tmp_dir"' EXIT
export TF_DATA_DIR="$tmp_dir/terraform-data"

terraform -chdir="$module_dir" fmt -check -recursive -diff
terraform -chdir="$module_dir" init -backend=false -input=false -lockfile=readonly -no-color >/dev/null
terraform -chdir="$module_dir" validate -no-color
terraform -chdir="$module_dir" test -no-color

check_contract() {
  local root="$1"
  local cluster_file="$root/main.tf"
  for required in \
    'remove_default_node_pool[[:space:]]*=[[:space:]]*true' \
    'initial_node_count[[:space:]]*=[[:space:]]*1' \
    'channel = "REGULAR"' \
    'workload_identity_config' \
    'ip_allocation_policy[[:space:]]*\{\}' \
    'gce_persistent_disk_csi_driver_config' \
    'enable_components = \["SYSTEM_COMPONENTS", "WORKLOADS"\]' \
    'machine_type[[:space:]]*=[[:space:]]*"e2-standard-2"' \
    'oauth_scopes[[:space:]]*=[[:space:]]*\["https://www.googleapis.com/auth/cloud-platform"\]' \
    'disable-legacy-endpoints = "true"' \
    'mode = "GKE_METADATA"'; do
    rg -q "$required" "$cluster_file" || { echo "missing contract: $required" >&2; return 1; }
  done
  if rg -q 'provider[[:space:]]+"kubernetes"|provider[[:space:]]+"tls"|google_gke_hub_|google_storage_bucket|google_container_registry|google_artifact_registry|network_policy|taint[[:space:]]*\{|credential|access_token|private_key|impersonat|workload_identity_user|roles/iam.workloadIdentityUser|shard|replica' "$root" --glob '*.tf'; then
    echo "forbidden provider, resource, credential, auth, or scope found" >&2
    return 1
  fi
  if [ "$(rg -o '^resource ' "$root" --glob '*.tf' | wc -l | tr -d ' ')" -ne 4 ]; then return 1; fi
  if [ "$(rg -o '^resource "google_container_cluster"' "$root" --glob '*.tf' | wc -l | tr -d ' ')" -ne 1 ]; then return 1; fi
  if [ "$(rg -o '^resource "google_container_node_pool"' "$root" --glob '*.tf' | wc -l | tr -d ' ')" -ne 1 ]; then return 1; fi
  if [ "$(rg -o '^resource "google_service_account"' "$root" --glob '*.tf' | wc -l | tr -d ' ')" -ne 1 ]; then return 1; fi
  if [ "$(rg -o '^resource "google_project_iam_member"' "$root" --glob '*.tf' | wc -l | tr -d ' ')" -ne 1 ]; then return 1; fi
  if [ "$(rg -o 'role[[:space:]]*=[[:space:]]*"roles/container.defaultNodeServiceAccount"' "$root" --glob '*.tf' | wc -l | tr -d ' ')" -ne 1 ]; then return 1; fi
  local expected_outputs='project_id region gke_zone cluster_name node_pool_name node_selector storage_class_name workload_identity_pool node_service_account_email run_id'
  if [ "$(rg -o '^output ' "$root" --glob '*.tf' | wc -l | tr -d ' ')" -ne 10 ]; then return 1; fi
  for output in $expected_outputs; do
    rg -q "^output \"$output\"" "$root" --glob '*.tf' || { echo "missing output: $output" >&2; return 1; }
  done
}

check_contract "$module_dir"

# Prove the rejection oracle without changing repository bytes.
mutation_dir="$tmp_dir/mutated-module"
mkdir -p "$mutation_dir"
cp "$module_dir"/*.tf "$mutation_dir/"
printf '\nresource "google_container_node_pool" "mutation" {}\n' >> "$mutation_dir/main.tf"
if check_contract "$mutation_dir"; then
  echo "negative mutation was not rejected" >&2
  exit 1
fi

echo "terraform standalone GKE checks passed (cloud-free)"
