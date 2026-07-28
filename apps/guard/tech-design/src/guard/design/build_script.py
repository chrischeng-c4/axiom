"""Executable design for Guard's project-local build helper."""

from dataclasses import dataclass

__aw_artifact_id__ = "artifact:guard/design-build-script"


@dataclass(frozen=True)
class BuildProfile:
    name: str
    cargo_arguments: tuple[str, ...]
    installs_binary: bool


def supported_build_profiles() -> tuple[BuildProfile, ...]:
    return (
        BuildProfile("debug", ("build", "-p", "guard-cli", "--bin", "guard"), True),
        BuildProfile(
            "release",
            ("build", "--release", "-p", "guard-cli", "--bin", "guard"),
            True,
        ),
    )
