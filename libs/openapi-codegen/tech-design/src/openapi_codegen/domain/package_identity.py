"""Versioned identity boundary for the independently tagged Rust library."""

__aw_artifact_id__ = "artifact:distribution/package-identity"


class VersionedPackageIdentity:
    """Owns the Cargo package, Rust crate, generator, and sidecar identities."""

    pass


def define_versioned_identity(
    distribution_name: str,
    crate_name: str,
    version: str,
    release_tag: str,
) -> None:
    """Require one exact identity across metadata, source, and releases."""

    pass
