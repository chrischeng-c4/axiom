"""Executable design for Guard CLI registry ownership."""

from dataclasses import dataclass

__aw_artifact_id__ = "artifact:guard/design-cli-registry"


@dataclass(frozen=True)
class CliRegistration:
    module_name: str
    distributed_slice: str
    standalone_binary: str


def guard_registration() -> CliRegistration:
    return CliRegistration(
        module_name="guard",
        distributed_slice="CLI_MODULES",
        standalone_binary="guard",
    )
