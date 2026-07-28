"""Executable design for Guard's release installer."""

from dataclasses import dataclass

__aw_artifact_id__ = "artifact:guard/design-install-script"


@dataclass(frozen=True)
class InstallTransaction:
    release_prefix: str
    verifies_checksum: bool
    atomic_replace: bool


def install_transaction() -> InstallTransaction:
    return InstallTransaction(
        release_prefix="guard@",
        verifies_checksum=True,
        atomic_replace=True,
    )
