"""Exact externally observable identities for the versioned Git boundary."""

from dataclasses import dataclass


@dataclass(frozen=True)
class GitVersionBoundary:
    package: str
    crate: str
    version: str
    publish: bool
    tag: str
    manifest: str


CONTRACT = GitVersionBoundary(
    package="openapi-codegen",
    crate="openapi_codegen",
    version="0.5.0",
    publish=False,
    tag="openapi-codegen@0.5.0",
    manifest=".openapi-codegen.json",
)

EXPECTED_CHECK_IDS = (
    "cargo-metadata-package",
    "manifest-package-version-publish",
    "rust-crate-name",
    "sidecar-filename",
    "generator-identity",
    "release-tag",
)

LEGACY_PACKAGE = "cclab" + "-openapi-codegen"
LEGACY_CRATE = "cclab" + "_openapi_codegen"
LEGACY_MANIFEST = ".cclab" + "-openapi-codegen.json"

LEGACY_IDENTITIES = (LEGACY_PACKAGE, LEGACY_CRATE, LEGACY_MANIFEST)
