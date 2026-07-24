"""Public verification matrix for generated client targets."""

__aw_artifact_id__ = "artifact:verification/target-matrix"


class RequiredTargetMatrix:
    """Names every Python, TypeScript, Rust, legacy, and determinism profile."""

    pass


def verify_required_target_matrix(
    python_profiles: tuple[str, ...],
    typescript_profiles: tuple[str, ...],
    rust_profiles: tuple[str, ...],
) -> None:
    """Reject missing, skipped, or renamed target-profile evidence."""

    pass
