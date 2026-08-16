#!/usr/bin/env bash
set -euo pipefail

# HANDWRITE-BEGIN gap="missing-generator:unit-test:3109-pki-ownership" tracker="3109" reason="Terraform ownership boundaries are a property of which resource types a configuration declares; no generator primitive emits a configuration-shape oracle yet."
#
# What this oracle holds is the ownership boundary of the Lumen PKI module: the
# exact set of resource types it may declare, and the absence of any actuator
# that would have to run again when a certificate expires.
#
# It is written as an ALLOWLIST, not a list of forbidden things, and that is the
# whole design. A denylist can only refuse what someone thought to forbid --
# it would not have caught a certificate resource under a name nobody predicted,
# and every future provider release adds more names. Comparing the declared set
# against an expected set fails on anything new, including the two additions
# that matter most: a resource that generates leaf material (which would put
# certificate material into Terraform state, AC4) and a resource that renews it
# on a schedule (which would put routine rotation back inside `terraform apply`,
# R7). Adding either means editing the expected set below, in a diff whose
# entire content is the boundary being moved.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
PKI_MODULE="$REPO_ROOT/apps/lumen/terraform/modules/lumen-pki"
INSTALL_ROOT="$REPO_ROOT/apps/lumen/terraform/examples/installation"

fail() {
  echo "lumen-pki ownership oracle: $1" >&2
  exit 1
}

[[ -d "$PKI_MODULE" ]] || fail "the PKI module is missing at ${PKI_MODULE#"$REPO_ROOT"/}"
[[ -d "$INSTALL_ROOT" ]] || fail "the installation example is missing at ${INSTALL_ROOT#"$REPO_ROOT"/}"

# declared_types <dir> -- every "<kind> <type>" the configuration declares, one
# per line, sorted. Comment lines are excluded: the commented-out capacity
# module in the installation root is documentation of a seam, not a declaration,
# and an oracle that could not tell the two apart would either reject the
# comment or accept a real resource someone hid behind a `#`.
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
    echo "lumen-pki ownership oracle: $label declares a different resource set than its boundary allows" >&2
    echo "--- expected ---" >&2
    printf '%s\n' "$expected" >&2
    echo "--- actual ---" >&2
    printf '%s\n' "$actual" >&2
    echo "If this is an intentional widening, move the boundary here deliberately." >&2
    exit 1
  fi
}

# The module owns a trust substrate and the narrow authority to use it. Note
# what is absent and must stay absent: anything that issues, stores, or renews a
# leaf, and anything that owns a project, network, cluster, or node pool (R1).
expect_types "the PKI module" "$PKI_MODULE" \
  "data google_project" \
  "resource google_project_service" \
  "resource google_privateca_ca_pool" \
  "resource google_privateca_certificate_authority" \
  "resource google_privateca_ca_pool_iam_member"

# The installation root composes; it declares no resources of its own beyond
# reading the cluster it was pointed at.
expect_types "the installation root" "$INSTALL_ROOT" \
  "data google_client_config" \
  "data google_container_cluster"

# --- no actuator, anywhere ---------------------------------------------------
# R7's real requirement is not "no rotation code" but "re-running apply is never
# how a leaf gets renewed". A provisioner is the one construct that turns a
# Terraform run into an imperative step, so its absence is the property worth
# holding directly rather than inferring from resource types.
if rg -q --no-messages '^\s*provisioner\s' "$PKI_MODULE" "$INSTALL_ROOT" -g '*.tf'; then
  fail "a provisioner would make certificate lifecycle an apply-time side effect; rotation belongs to the in-cluster controller"
fi

# --- the module stays composable --------------------------------------------
# A `provider` block inside a module pins configuration the caller cannot
# override and makes the module impossible to drop into an installation root
# that already has its own (R1). The root is where it belongs, and it is there.
if rg -q --no-messages '^\s*provider\s+"' "$PKI_MODULE" -g '*.tf'; then
  fail "the PKI module must not configure a provider; the composing root owns that"
fi
rg -q --no-messages '^\s*provider\s+"google"' "$INSTALL_ROOT" -g '*.tf' ||
  fail "the installation root must configure the google provider"

# --- either child validates alone (AC5) --------------------------------------
# The composition example must not be the only way in. This is asserted as a
# structural property -- the module declares its own version constraints and
# provider requirements -- because check.sh separately runs `terraform validate`
# and `terraform test` against the module directory on its own, which is the
# executable half of the same claim.
for required in 'required_version' 'required_providers'; do
  rg -q --no-messages "$required" "$PKI_MODULE/versions.tf" ||
    fail "the PKI module must declare $required to be independently validatable"
done

# --- one trust domain, not one CA per namespace (R5) -------------------------
# Two pools exist by design (a protected root and a regional issuer). A third
# would mean the module had started scaling trust with tenancy.
pool_blocks="$(rg -c --no-messages '^resource "google_privateca_ca_pool"' "$PKI_MODULE" -g '*.tf' | cut -d: -f2 | paste -sd+ - | bc)"
[[ "$pool_blocks" == "2" ]] ||
  fail "expected exactly 2 CA pool declarations (root + regional issuer), found $pool_blocks"

echo "lumen-pki ownership oracle: ok"
# HANDWRITE-END
