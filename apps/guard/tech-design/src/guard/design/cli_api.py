"""Executable design for Guard's agent-facing CLI."""

from dataclasses import dataclass

__aw_artifact_id__ = "artifact:guard/design-cli-api"


@dataclass(frozen=True)
class CliVerb:
    name: str
    emits_json: bool
    mutates_scan_state: bool


def guard_cli_verbs() -> tuple[CliVerb, ...]:
    return (
        CliVerb("scan", True, True),
        CliVerb("report", True, False),
        CliVerb("spec", True, False),
        CliVerb("llm", True, False),
    )
