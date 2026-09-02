#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCEPTANCE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

for script in "$SCRIPT_DIR"/*.sh "$ACCEPTANCE_ROOT"/tests/*.sh; do
  bash -n "$script"
  [[ -x "$script" ]] || {
    echo "acceptance script is not executable: $script" >&2
    exit 1
  }
done
jq empty "$ACCEPTANCE_ROOT/evidence/schema.json"
python3 "$SCRIPT_DIR/validate-sift-mvp-evidence.py" \
  --schema "$ACCEPTANCE_ROOT/evidence/schema.json" --schema-only

# The Lumen CRD defaults `spec.auth` to `required`, and a lumen built without
# `delegated-auth` refuses to start in that mode rather than serve
# unauthenticated. That is the right call, but it means an acceptance image
# missing the feature turns every auth-mode instance into a crash loop whose
# only symptom is a pod that never goes Ready -- twenty minutes and a cloud
# build away from the one line that explains it. Check the feature here, where
# it costs nothing.
#
# Every Dockerfile that builds the lumen binary, not a named one: there are two
# build paths (both images together, or one at a time when the other was
# supplied by digest), and the first version of this check named only the path
# that was not being taken. A grep for the build line finds whichever file
# grows next.
lumen_build_files="$(grep -rl -- '-p lumen --bin lumen' "$ACCEPTANCE_ROOT/images")"
[[ -n "$lumen_build_files" ]] || {
  echo "no acceptance image builds the lumen binary; this check is now looking \
in the wrong place" >&2
  exit 1
}
while IFS= read -r dockerfile; do
  grep -q -- '--features "[^"]*delegated-auth' "$dockerfile" || {
    echo "$dockerfile must build lumen with the delegated-auth feature; \
spec.auth defaults to required and the binary refuses to start without it" >&2
    exit 1
  }
done <<<"$lumen_build_files"
for terraform_dir in environment cluster; do
  terraform -chdir="$ACCEPTANCE_ROOT/$terraform_dir" fmt -check -recursive
done
# The Lumen PKI module (#3109) is a product artifact, not part of this
# disposable harness -- an operator composes it into their own installation
# root, and the final GKE gate applies it against the test environment. It is
# checked from here because this is where the repo's Terraform gate lives, and a
# module with no gate is a module that drifts.
LUMEN_TERRAFORM="$(cd "$ACCEPTANCE_ROOT/../../apps/lumen/terraform" && pwd)"
terraform -chdir="$LUMEN_TERRAFORM" fmt -check -recursive
bash "$ACCEPTANCE_ROOT/tests/acceptance_mode_selection.sh"
bash "$ACCEPTANCE_ROOT/tests/manifest_render_matrix.sh"
bash "$ACCEPTANCE_ROOT/tests/lumen_pki_ownership.sh"
bash "$ACCEPTANCE_ROOT/tests/lumen_capacity_ownership.sh"

# Validate in a disposable copy so provider initialization never writes a
# lock/cache artifact into the source tree.
validate_root="$(mktemp -d "${TMPDIR:-/tmp}/axiom-gcp-operator-validate.XXXXXX")"
cleanup_validate() {
  find "$validate_root" -type f -delete
  find "$validate_root" -depth -type d -empty -delete
}
trap cleanup_validate EXIT INT TERM
for terraform_dir in environment cluster; do
  mkdir -p "$validate_root/$terraform_dir"
  cp "$ACCEPTANCE_ROOT/$terraform_dir"/*.tf "$validate_root/$terraform_dir/"
  TF_DATA_DIR="$validate_root/.terraform-$terraform_dir" terraform \
    -chdir="$validate_root/$terraform_dir" init -backend=false -input=false
  TF_DATA_DIR="$validate_root/.terraform-$terraform_dir" terraform \
    -chdir="$validate_root/$terraform_dir" validate
done

# The PKI tree is copied with its structure intact rather than by *.tf glob: the
# installation example reaches the module through a relative source, and its
# accept/reject fixtures live in tests/ subdirectories. A flat copy would
# silently validate a root with no module and run no fixtures at all.
#
# Any working state an operator's own `terraform init` left behind is excluded,
# not carried along -- it is a few hundred megabytes of provider cache, and
# copying it would make this gate slower than the cloud run it precedes.
mkdir -p "$validate_root/lumen-terraform"
tar -C "$LUMEN_TERRAFORM" -cf - \
  --exclude='.terraform' --exclude='.terraform.lock.hcl' . |
  tar -C "$validate_root/lumen-terraform" -xf -
for pki_dir in modules/lumen-pki modules/lumen-capacity examples/installation; do
  pki_slug="${pki_dir//\//-}"
  TF_DATA_DIR="$validate_root/.terraform-$pki_slug" terraform \
    -chdir="$validate_root/lumen-terraform/$pki_dir" init -backend=false -input=false
  TF_DATA_DIR="$validate_root/.terraform-$pki_slug" terraform \
    -chdir="$validate_root/lumen-terraform/$pki_dir" validate
  # `terraform test` is the executable half of the issuance-policy, IAM,
  # hierarchy, and retirement contracts. It runs against a mocked provider, so
  # it needs no GCP project, no credential, and creates nothing billable --
  # which is why these rejects can be a per-commit gate instead of something
  # only a cloud run would notice.
  TF_DATA_DIR="$validate_root/.terraform-$pki_slug" terraform \
    -chdir="$validate_root/lumen-terraform/$pki_dir" test
done

# A gate that says nothing when it passes is indistinguishable from one that
# did not run -- and this gate was silently red for several commits before
# anyone noticed. Say so out loud, and name what comes next.
echo "static acceptance gate: ok"
echo "next: PROJECT_ID=<project> acceptance/gcp/scripts/run.sh"
