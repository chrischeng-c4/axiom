"""Executable design for Guard CLI flag-to-evidence dispatch."""

from dataclasses import dataclass

__aw_artifact_id__ = "artifact:guard/design-cli-dispatch"


@dataclass(frozen=True)
class AdapterInvocation:
    tool: str
    leading_arguments: tuple[str, ...]


def adapter_invocation(tool: str, value: str) -> AdapterInvocation:
    if tool == "vat":
        return AdapterInvocation("vat", ("run", "--json", value))
    if tool == "rig":
        return AdapterInvocation("rig", ("run", "--scenario", value, "--compact"))
    if tool == "meter":
        return AdapterInvocation(
            "meter",
            (
                "run",
                "--target",
                value,
                "--skip-bench",
                "--skip-profile",
                "--compact",
            ),
        )
    raise ValueError(f"unsupported required adapter: {tool}")


def persisted_by_default(no_persist: bool) -> bool:
    return not no_persist
