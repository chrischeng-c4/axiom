#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 --repo <owner/repo> --tag <lumen@semver> --commit <40-hex> --image <ghcr-image@sha256:64-hex> --release-state <draft|published>" >&2
  exit 1
}

REPO=""
TAG=""
COMMIT=""
IMAGE=""
RELEASE_STATE=""

while [[ $# -gt 0 ]]; do
  [[ $# -ge 2 ]] || usage
  case "$1" in
    --repo)
      [[ -z "$REPO" ]] || usage
      REPO="$2"
      ;;
    --tag)
      [[ -z "$TAG" ]] || usage
      TAG="$2"
      ;;
    --commit)
      [[ -z "$COMMIT" ]] || usage
      COMMIT="$2"
      ;;
    --image)
      [[ -z "$IMAGE" ]] || usage
      IMAGE="$2"
      ;;
    --release-state)
      [[ -z "$RELEASE_STATE" ]] || usage
      RELEASE_STATE="$2"
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      ;;
  esac
  shift 2
done

[[ -n "$REPO" && -n "$TAG" && -n "$COMMIT" && -n "$IMAGE" && -n "$RELEASE_STATE" ]] || usage

if [[ "$REPO" != "chrischeng-c4/axiom" ]]; then
  echo "unsupported repository: $REPO" >&2
  exit 1
fi

if [[ ! "$TAG" =~ ^lumen@[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "invalid tag shape: $TAG" >&2
  exit 1
fi

if [[ ! "$COMMIT" =~ ^[0-9a-f]{40}$ ]]; then
  echo "invalid commit shape: $COMMIT" >&2
  exit 1
fi

if [[ ! "$IMAGE" =~ ^ghcr\.io/chrischeng-c4/lumen@sha256:[0-9a-f]{64}$ ]]; then
  echo "invalid image shape: $IMAGE" >&2
  exit 1
fi

if [[ "$RELEASE_STATE" != "draft" && "$RELEASE_STATE" != "published" ]]; then
  echo "invalid release state: $RELEASE_STATE" >&2
  exit 1
fi

root_digest="${IMAGE#*@}"
source_ref="refs/tags/${TAG}"
expected_cert_id="https://github.com/${REPO}/.github/workflows/lumen-release.yml@${source_ref}"
expected_issuer="https://token.actions.githubusercontent.com"
expected_signer_workflow="${REPO}/.github/workflows/lumen-release.yml"

echo ">> verifying GitHub release identity and state"
release_json="$(gh release view "$TAG" --repo "$REPO" --json isDraft,tagName)"
release_tag="$(jq -er '.tagName | select(type == "string")' <<<"$release_json")"
is_draft="$(jq -er '.isDraft | select(type == "boolean") | tostring' <<<"$release_json")"

[[ "$release_tag" == "$TAG" ]] || { echo "release tag mismatch: $release_tag" >&2; exit 1; }
if [[ "$RELEASE_STATE" == "draft" ]]; then
  [[ "$is_draft" == "true" ]] || { echo "expected draft release" >&2; exit 1; }
else
  [[ "$is_draft" == "false" ]] || { echo "expected published release" >&2; exit 1; }
fi

tag_commit="$(gh api "repos/${REPO}/commits/${TAG}" --jq '.sha')"
[[ "$tag_commit" =~ ^[0-9a-f]{40}$ ]] || { echo "tag did not resolve to a commit" >&2; exit 1; }
[[ "$tag_commit" == "$COMMIT" ]] || { echo "tag commit mismatch" >&2; exit 1; }

echo ">> verifying keyless root signature"
cosign verify \
  --certificate-identity "$expected_cert_id" \
  --certificate-oidc-issuer "$expected_issuer" \
  "$IMAGE" >/dev/null

verify_attestation() {
  local label="$1"
  local subject="$2"
  local digest="$3"
  local predicate="$4"
  local result

  result="$(gh attestation verify "oci://${subject}" \
    --bundle-from-oci \
    --repo "$REPO" \
    --signer-workflow "$expected_signer_workflow" \
    --source-ref "$source_ref" \
    --source-digest "$COMMIT" \
    --cert-identity "$expected_cert_id" \
    --cert-oidc-issuer "$expected_issuer" \
    --predicate-type "$predicate" \
    --format json)"

  if ! jq -e --arg digest "${digest#sha256:}" --arg predicate "$predicate" '
    type == "array" and length > 0 and
    all(.[];
      .verificationResult.statement.predicateType == $predicate and
      (.verificationResult.statement.subject | type == "array" and length > 0) and
      all(.verificationResult.statement.subject[];
        (.digest | type == "object" and keys == ["sha256"]) and
        .digest.sha256 == $digest
      )
    )
  ' <<<"$result" >/dev/null; then
    echo "$label attestation subject or predicate mismatch" >&2
    exit 1
  fi
}

echo ">> verifying root SLSA v1 provenance"
verify_attestation \
  "root provenance" \
  "$IMAGE" \
  "$root_digest" \
  "https://slsa.dev/provenance/v1"

echo ">> inspecting the two-platform image index"
index_json="$(docker buildx imagetools inspect --raw "$IMAGE")"
if ! jq -e '
  (.manifests | type == "array" and length == 2) and
  all(.manifests[];
    .platform.os == "linux" and
    (.platform | has("variant") | not) and
    (.platform.architecture == "amd64" or .platform.architecture == "arm64") and
    (.digest | type == "string") and
    (.digest | test("^sha256:[0-9a-f]{64}$"))
  ) and
  ([.manifests[].platform.architecture] | sort == ["amd64", "arm64"]) and
  ([.manifests[].digest] | unique | length == 2)
' <<<"$index_json" >/dev/null; then
  echo "root index is not exactly linux/amd64 plus linux/arm64" >&2
  exit 1
fi

amd64_digest="$(jq -er '.manifests[] | select(.platform.architecture == "amd64") | .digest' <<<"$index_json")"
arm64_digest="$(jq -er '.manifests[] | select(.platform.architecture == "arm64") | .digest' <<<"$index_json")"
if [[ "$amd64_digest" == "$arm64_digest" || "$amd64_digest" == "$root_digest" || "$arm64_digest" == "$root_digest" ]]; then
  echo "root and child digests must be pairwise distinct" >&2
  exit 1
fi

image_repo="${IMAGE%@*}"
echo ">> verifying linux/amd64 SPDX 2.3 SBOM"
verify_attestation \
  "linux/amd64 SBOM" \
  "${image_repo}@${amd64_digest}" \
  "$amd64_digest" \
  "https://spdx.dev/Document/v2.3"

echo ">> verifying linux/arm64 SPDX 2.3 SBOM"
verify_attestation \
  "linux/arm64 SBOM" \
  "${image_repo}@${arm64_digest}" \
  "$arm64_digest" \
  "https://spdx.dev/Document/v2.3"

echo "Artifact verification PASS"
echo "  identity: $expected_cert_id"
echo "  root: $root_digest (keyless signature and SLSA v1 provenance)"
echo "  linux/amd64: $amd64_digest (SPDX 2.3 SBOM)"
echo "  linux/arm64: $arm64_digest (SPDX 2.3 SBOM)"
