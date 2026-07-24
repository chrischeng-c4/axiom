"""Repository-wide migration of active references to the new identity."""

__aw_artifact_id__ = "artifact:distribution/reference-sweep"


class ActiveReferenceMigration:
    """Separates active references from explicitly historical evidence."""

    pass


def migrate_active_references(
    superseded_distribution: str,
    superseded_crate: str,
    superseded_sidecar: str,
) -> None:
    """Rewrite every active source, manifest, test, config, and documentation use."""

    pass
