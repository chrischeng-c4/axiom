#!/usr/bin/env bash
set -euo pipefail

# HANDWRITE-BEGIN gap="missing-generator:unit-test:3066-capacity-ownership" tracker="3066" reason="Terraform ownership boundaries are a property of which resource types a configuration declares; no generator primitive emits a configuration-shape oracle yet."
#
# What this oracle holds is the ownership boundary of the Lumen capacity module:
# the exact set of resource types it may declare, the absence of cloud mutation
# authority in the runtime, and the absence of any cluster or network creation.
#
# It is written as an ALLOWLIST, not a list of forbidden things. A denylist can
# only refuse what someone thought to forbid; comparing the declared set against
# an expected set fails on anything new, including accidental cluster creation
# or IAM bindings granting container mutation.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
CAPACITY_MODULE="$REPO_ROOT/apps/lumen/terraform/modules/lumen-capacity"
INSTALL_ROOT="$REPO_ROOT/apps/lumen/terraform/examples/installation"

fail() {
  echo "lumen-capacity ownership oracle: $1" >&2
  exit 1
}

[[ -d "$CAPACITY_MODULE" ]] || fail "the capacity module is missing at ${CAPACITY_MODULE#"$REPO_ROOT"/}"
[[ -d "$INSTALL_ROOT" ]] || fail "the installation example is missing at ${INSTALL_ROOT#"$REPO_ROOT"/}"

# declared_types <dir> -- every "<kind> <type>" the configuration declares, one
# per line, sorted.
declared_types() {
  local dir="$1"
  rg --no-filename --no-line-number '^\s*(resource|data)\s+"[a-z0-9_]+"' "$dir" \
    -g '*.tf' -r '$0' 2>/dev/null |
    sed -E 's/^[[:space:]]*(resource|data)[[:space:]]+"([a-z0-9_]+)".*/\1 \2/' |
    sort -u
}

expect_types() { # expect_types <label> <dir> <expected...>
  local label="$1" dir="$2"
  shift 2
  local expected actual
  expected="$(printf '%s\n' "$@" | sort -u)"
  actual="$(declared_types "$dir")"
  if [[ "$expected" != "$actual" ]]; then
    echo "lumen-capacity ownership oracle: $label declares a different resource set than its boundary allows" >&2
    echo "--- expected ---" >&2
    printf '%s\n' "$expected" >&2
    echo "--- actual ---" >&2
    printf '%s\n' "$actual" >&2
    echo "If this is an intentional widening, move the boundary here deliberately." >&2
    exit 1
  fi
}

# The module creates only shared data node pools and publishes the in-cluster catalog.
# Note what is absent: project, VPC, cluster, system pool, and IAM bindings.
expect_types "the capacity module" "$CAPACITY_MODULE" \
  "resource google_container_node_pool" \
  "resource kubernetes_config_map"

# The installation root composes; it declares no resources of its own beyond
# reading the cluster it was pointed at and reading client config for the kubernetes provider.
expect_types "the installation root" "$INSTALL_ROOT" \
  "data google_client_config" \
  "data google_container_cluster"

# --- no actuator, anywhere ---------------------------------------------------
if rg -q --no-messages '^\s*provisioner\s' "$CAPACITY_MODULE" "$INSTALL_ROOT" -g '*.tf'; then
  fail "a provisioner would make capacity lifecycle an apply-time side effect; lifecycle belongs to Terraform"
fi

# --- the module stays composable --------------------------------------------
if rg -q --no-messages '^\s*provider\s+"' "$CAPACITY_MODULE" -g '*.tf'; then
  fail "the capacity module must not configure a provider; the composing root owns that"
fi
rg -q --no-messages '^\s*provider\s+"google"' "$INSTALL_ROOT" -g '*.tf' ||
  fail "the installation root must configure the google provider"
rg -q --no-messages '^\s*provider\s+"kubernetes"' "$INSTALL_ROOT" -g '*.tf' ||
  fail "the installation root must configure the kubernetes provider"

# --- either child validates alone (AC5) --------------------------------------
for required in 'required_version' 'required_providers'; do
  rg -q --no-messages "$required" "$CAPACITY_MODULE/versions.tf" ||
    fail "the capacity module must declare $required to be independently validatable"
done

# --- cluster is referenced, not created --------------------------------------
if rg -q --no-messages 'resource\s+"google_container_cluster"' "$CAPACITY_MODULE" -g '*.tf'; then
  fail "the capacity module must reference an existing cluster, never declare one"
fi

echo "lumen-capacity ownership oracle: ok"
# HANDWRITE-END
