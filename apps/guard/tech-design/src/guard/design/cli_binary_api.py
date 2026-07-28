"""Executable design for the standalone Guard process boundary."""

from dataclasses import dataclass

__aw_artifact_id__ = "artifact:guard/design-cli-binary-api"


@dataclass(frozen=True)
class ProcessProjection:
    stdout: str
    stderr: str
    exit_code_source: str


def standalone_process_projection() -> ProcessProjection:
    return ProcessProjection(
        stdout="one guard.report/1 JSON object",
        stderr="optional human summary only",
        exit_code_source="GuardReport.exit_code",
    )
