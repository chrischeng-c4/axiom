#!/usr/bin/env bash

# Shared helpers for project-root build.sh scripts used by build skills.
# Callers must run from the repository root and set -euo pipefail.

project_build_bump63() {
  local version="$1"
  local core="${version%%+*}"
  core="${core%%-*}"

  local major minor patch
  IFS='.' read -r major minor patch <<< "$core"
  if [[ -z "${major:-}" || -z "${minor:-}" || -z "${patch:-}" ]]; then
    echo "error: unsupported version '$version' (expected major.minor.patch)" >&2
    return 2
  fi

  local new_patch=$((patch + 1))
  local new_minor=$minor
  local new_major=$major
  if [[ "$new_patch" -gt 63 ]]; then
    new_patch=0
    new_minor=$((minor + 1))
  fi
  if [[ "$new_minor" -gt 63 ]]; then
    new_minor=0
    new_major=$((major + 1))
  fi

  printf '%s.%s.%s\n' "$new_major" "$new_minor" "$new_patch"
}

project_build_read_version() {
  local manifest="$1"
  awk -F'"' '/^[[:space:]]*version[[:space:]]*=[[:space:]]*"/ { print $2; exit }' "$manifest"
}

project_build_set_version() {
  local current="$1"
  local next="$2"
  shift 2

  local manifest
  for manifest in "$@"; do
    perl -0pi -e 's/^version = "\Q'"$current"'\E"/version = "'"$next"'"/m' "$manifest"
  done
}

project_build_remote_tag_exists() {
  local tag="$1"
  local output status

  if output="$(git ls-remote --exit-code --tags origin "refs/tags/${tag}" 2>&1)"; then
    return 0
  else
    status=$?
  fi
  if [[ "$status" -eq 2 ]]; then
    return 1
  fi

  if [[ "${PROJECT_BUILD_REQUIRE_REMOTE_TAG_CHECK:-0}" == "1" ]]; then
    echo "error: could not check remote tag ${tag}: ${output}" >&2
    return "$status"
  fi

  echo "warning: could not check remote tag ${tag}: ${output}" >&2
  return 1
}

project_build_tag_exists() {
  local tag="$1"

  if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
    return 0
  fi

  project_build_remote_tag_exists "$tag"
}

project_build_next_unique_version() {
  local project="$1"
  local current="$2"
  local candidate="${current%%+*}"
  candidate="${candidate%%-*}"

  while project_build_tag_exists "${project}@${candidate}"; do
    echo "Version ${candidate} already has tag ${project}@${candidate}; trying next." >&2
    candidate="$(project_build_bump63 "$candidate")"
  done

  printf '%s\n' "$candidate"
}

project_build_commit_debug_checkpoint() {
  local project="$1"

  if [[ -n "$(git status --porcelain)" ]]; then
    git status --short
    git add -A
    git commit -m "build(${project}): debug checkpoint"
  fi
}

PROJECT_BUILD_SNAPSHOT_DIR=""
PROJECT_BUILD_SNAPSHOT_FILES=()

project_build_snapshot_manifests() {
  PROJECT_BUILD_SNAPSHOT_DIR="$(mktemp -d "${TMPDIR:-/tmp}/project-build.XXXXXX")"
  PROJECT_BUILD_SNAPSHOT_FILES=()

  local manifest
  for manifest in "$@"; do
    if [[ -f "$manifest" ]]; then
      mkdir -p "${PROJECT_BUILD_SNAPSHOT_DIR}/$(dirname "$manifest")"
      cp "$manifest" "${PROJECT_BUILD_SNAPSHOT_DIR}/${manifest}"
      PROJECT_BUILD_SNAPSHOT_FILES+=("$manifest")
    fi
  done

  trap project_build_restore_manifests EXIT
}

project_build_restore_manifests() {
  if [[ -z "${PROJECT_BUILD_SNAPSHOT_DIR:-}" ]]; then
    return 0
  fi

  local manifest
  for manifest in "${PROJECT_BUILD_SNAPSHOT_FILES[@]}"; do
    cp "${PROJECT_BUILD_SNAPSHOT_DIR}/${manifest}" "$manifest"
  done
  rm -rf "$PROJECT_BUILD_SNAPSHOT_DIR"
  PROJECT_BUILD_SNAPSHOT_DIR=""
  PROJECT_BUILD_SNAPSHOT_FILES=()
  trap - EXIT
}

project_build_prepare_debug_version() {
  local project="$1"
  local current="$2"
  shift 2

  project_build_commit_debug_checkpoint "$project"

  local sha
  sha="$(git rev-parse --short=12 HEAD)"

  local base
  base="$(project_build_next_unique_version "$project" "$current")"

  PROJECT_BUILD_DEBUG_BASE_VERSION="$base"
  PROJECT_BUILD_DEBUG_VERSION="${base}+${sha}"
  PROJECT_BUILD_DEBUG_SHA="$sha"
  export PROJECT_BUILD_DEBUG_BASE_VERSION PROJECT_BUILD_DEBUG_VERSION PROJECT_BUILD_DEBUG_SHA

  project_build_snapshot_manifests "$@" Cargo.lock
  project_build_set_version "$current" "$PROJECT_BUILD_DEBUG_VERSION" "$@"

  echo "Debug version: ${PROJECT_BUILD_DEBUG_VERSION}"
}

project_build_prepare_release_version() {
  local project="$1"
  local current="$2"
  shift 2

  PROJECT_BUILD_RELEASE_VERSION="$(project_build_next_unique_version "$project" "$current")"
  PROJECT_BUILD_RELEASE_TAG="${project}@${PROJECT_BUILD_RELEASE_VERSION}"
  export PROJECT_BUILD_RELEASE_VERSION PROJECT_BUILD_RELEASE_TAG

  if [[ "$PROJECT_BUILD_RELEASE_VERSION" != "$current" ]]; then
    echo "Bumping version: $current -> $PROJECT_BUILD_RELEASE_VERSION"
    project_build_set_version "$current" "$PROJECT_BUILD_RELEASE_VERSION" "$@"
  else
    echo "Using current untagged version: $PROJECT_BUILD_RELEASE_VERSION"
  fi
}

project_build_print_release_next_steps() {
  local project="$1"
  local tag="$2"

  echo ""
  echo "Release prepared: ${tag} built + installed; release commit created."
  echo "NOT tagged and NOT pushed."
  echo "RELEASE_TAG=${tag}"
  echo ""
  echo "Complete the release with the build skill:"
  echo "  1. run git:land to land this release commit on main"
  echo "  2. git tag -a ${tag} -m \"Release ${tag}\""
  echo "  3. git push origin ${tag}"
  echo "  4. scripts/project-build-monitor-release.sh ${project} ${tag}"
  echo "Pushing ${tag} triggers the ${project}@* GitHub release workflow when one exists; monitor it before declaring the release complete."
}
