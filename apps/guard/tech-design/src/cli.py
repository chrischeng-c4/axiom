"""Executable design for Guard's standalone process and command surface."""

from dataclasses import dataclass

__aw_artifact_id__ = "artifact:guard/design-cli"


@dataclass(frozen=True)
class CliVerb:
    name: str
    emits_json: bool
    mutates_scan_state: bool


@dataclass(frozen=True)
class ProcessProjection:
    stdout: str
    stderr: str
    exit_code_source: str


class CliDesign:
    EXECUTION_ORDER = ("parse", "dispatch", "print_report", "exit_from_report")

    @staticmethod
    def verbs() -> tuple[CliVerb, ...]:
        return (
            CliVerb("scan", True, True),
            CliVerb("report", True, False),
            CliVerb("spec", True, False),
            CliVerb("llm", True, False),
        )

    @staticmethod
    def process_projection() -> ProcessProjection:
        return ProcessProjection(
            stdout="one guard.report/1 JSON object",
            stderr="optional human summary only",
            exit_code_source="GuardReport.exit_code",
        )

    @staticmethod
    def persisted_by_default(no_persist: bool) -> bool:
        return not no_persist
