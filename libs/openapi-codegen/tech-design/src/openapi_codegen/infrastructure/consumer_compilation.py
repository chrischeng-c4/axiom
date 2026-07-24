"""Cargo dependency boundary for every monorepo consumer."""

__aw_artifact_id__ = "artifact:integration/consumer-compilation"


class ConsumerDependencyBoundary:
    """Requires path plus compatible version without package aliasing."""

    pass


def verify_consumer_compilation(
    dependency_name: str,
    local_path: str,
    compatible_version: str,
) -> None:
    """Compile each required consumer against the renamed Rust crate."""

    pass
