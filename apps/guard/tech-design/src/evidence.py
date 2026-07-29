"""Executable external-evidence adapters for the Guard reference product."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from enum import Enum
import json
import os
from pathlib import Path
import subprocess
from typing import Any

from report import Finding, Location, Severity

__aw_artifact_id__ = "artifact:guard/dynamic-security-evidence"
__aw_public_contract__ = True
__aw_public_behaviors__ = (
    "meter_dos_resource_evidence_bridge",
    "rig_exploit_journey_bridge",
    "vat_isolated_security_runner",
    "dynamic_adapter_routing",
    "stable_evidence_folding",
)


class EvidenceStatus(str, Enum):
    CLEAN = "clean"
    FINDINGS = "findings"
    TOOL_ERROR = "tool_error"


@dataclass(frozen=True)
class EvidenceDecision:
    status: EvidenceStatus
    clean: bool
    finding_count: int


@dataclass(frozen=True)
class AdapterInvocation:
    tool: str
    leading_arguments: tuple[str, ...]


@dataclass(frozen=True)
class EvidenceCommand:
    tool: str
    label: str
    command: tuple[str, ...]
    cwd: Path | None = None
    env: dict[str, str] = field(default_factory=dict)

    @classmethod
    def argv(
        cls,
        tool: str,
        label: str,
        command: list[str] | tuple[str, ...],
    ) -> EvidenceCommand:
        return cls(tool, label, tuple(command))

    @classmethod
    def shell(cls, tool: str, label: str, command: str) -> EvidenceCommand:
        return cls(tool, label, ("sh", "-c", command))

    def with_cwd(self, cwd: Path) -> EvidenceCommand:
        return EvidenceCommand(self.tool, self.label, self.command, cwd, self.env)

    def with_env(self, key: str, value: str) -> EvidenceCommand:
        return EvidenceCommand(
            self.tool,
            self.label,
            self.command,
            self.cwd,
            {**self.env, key: value},
        )


@dataclass(frozen=True)
class ExternalEvidence:
    tool: str
    label: str
    command: tuple[str, ...]
    status: EvidenceStatus
    clean: bool
    exit_code: int | None
    finding_count: int
    report: dict[str, Any] | None
    stderr_tail: str
    cwd: str | None = None
    env: dict[str, str] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        value = asdict(self)
        value["command"] = list(self.command)
        value["status"] = self.status.value
        if self.cwd is None:
            value.pop("cwd")
        if not self.env:
            value.pop("env")
        if self.report is None:
            value.pop("report")
        if self.exit_code is None:
            value.pop("exit_code")
        if not self.stderr_tail:
            value.pop("stderr_tail")
        return value

    def to_guard_finding(self, target: str) -> Finding | None:
        if self.clean:
            return None
        command = " ".join(self.command)
        rule = f"{self.tool.upper()}-EVIDENCE"
        return Finding(
            id=f"evidence:{_squash(self.tool)}:{_squash(self.label)}",
            severity=Severity.HIGH,
            rule=rule,
            title=f"{self.tool} security evidence is not clean",
            detail=(
                f"`{command}` returned {self.exit_code!r} with "
                f"{self.finding_count} finding(s)"
            ),
            remediation=(
                f"Inspect the {self.tool} report, fix the finding, "
                f"then rerun `{command}`."
            ),
            location=Location(target, 0, 0, 0, 0),
            evidence={
                "source": self.tool,
                "label": self.label,
                "command": list(self.command),
                "cwd": self.cwd,
                "env": self.env,
                "exit_code": self.exit_code,
                "report": self.report,
                "stderr_tail": self.stderr_tail,
            },
        )


class EvidenceDesign:
    REQUIRED_TOOLS = ("vat", "rig", "meter")

    @staticmethod
    def normalize_result(
        process_success: bool,
        report_clean: bool | None,
        finding_count: int,
    ) -> EvidenceDecision:
        clean = process_success and report_clean is True and finding_count == 0
        return EvidenceDecision(
            EvidenceStatus.CLEAN if clean else EvidenceStatus.FINDINGS,
            clean,
            finding_count,
        )

    @staticmethod
    def adapter_invocation(tool: str, value: str) -> AdapterInvocation:
        if tool == "vat":
            return AdapterInvocation("vat", ("run", "--json", value))
        if tool == "rig":
            return AdapterInvocation(
                "rig",
                ("run", "--scenario", value, "--compact"),
            )
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


def run_evidence_commands(commands: list[EvidenceCommand]) -> list[ExternalEvidence]:
    return [_run_one(command) for command in commands]


def _run_one(command: EvidenceCommand) -> ExternalEvidence:
    if not command.command:
        return ExternalEvidence(
            command.tool,
            command.label,
            (),
            EvidenceStatus.TOOL_ERROR,
            False,
            None,
            0,
            None,
            "empty evidence command",
            str(command.cwd) if command.cwd else None,
            command.env,
        )
    try:
        result = subprocess.run(
            command.command,
            cwd=command.cwd,
            env=None if not command.env else {**os.environ, **command.env},
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError as error:
        return ExternalEvidence(
            command.tool,
            command.label,
            command.command,
            EvidenceStatus.TOOL_ERROR,
            False,
            None,
            0,
            None,
            str(error)[-2000:],
            str(command.cwd) if command.cwd else None,
            command.env,
        )
    report = _parse_json_payload(result.stdout)
    report_clean = _report_clean(report)
    finding_count = _finding_count(report)
    clean = result.returncode == 0 and report_clean is True and finding_count == 0
    status = EvidenceStatus.CLEAN if clean else EvidenceStatus.FINDINGS
    return ExternalEvidence(
        command.tool,
        command.label,
        command.command,
        status,
        clean,
        result.returncode,
        finding_count,
        _compact_report(report),
        result.stderr[-2000:],
        str(command.cwd) if command.cwd else None,
        command.env,
    )


def _parse_json_payload(stdout: str) -> dict[str, Any] | None:
    candidates = [stdout.strip(), *reversed(stdout.splitlines())]
    for candidate in candidates:
        if not candidate:
            continue
        try:
            value = json.loads(candidate)
        except json.JSONDecodeError:
            continue
        if isinstance(value, dict):
            return value
    return None


def _report_clean(report: dict[str, Any] | None) -> bool | None:
    if report is None:
        return False
    schema_version = report.get("schema_version")
    if not isinstance(schema_version, str) or not schema_version:
        return False
    projected: bool | None = None
    if isinstance(report.get("clean"), bool):
        projected = report["clean"]
    if projected is None:
        completion = report.get("completion")
        if isinstance(completion, dict) and isinstance(completion.get("clean"), bool):
            projected = completion["clean"]
    if projected is None:
        status = report.get("status")
        if isinstance(status, dict) and isinstance(status.get("state"), str):
            state = status["state"]
            projected = True if state == "clean" else False if state else None
    if projected is None and isinstance(report.get("ok"), bool):
        projected = report["ok"]
    if projected is None:
        return False
    counts = _finding_counts(report)
    if not counts or len(set(counts)) != 1:
        return False
    if projected and counts[0] != 0:
        return False
    return projected


def _finding_count(report: dict[str, Any] | None) -> int:
    counts = _finding_counts(report)
    return max(counts, default=0)


def _finding_counts(report: dict[str, Any] | None) -> list[int]:
    if report is None:
        return []
    counts: list[int] = []
    summary = report.get("summary")
    if isinstance(summary, dict):
        for field_name in ("security_findings", "total"):
            value = summary.get(field_name)
            if isinstance(value, int) and value >= 0:
                counts.append(value)
                break
    findings = report.get("findings")
    if isinstance(findings, list):
        counts.append(len(findings))
    return counts


def _compact_report(report: dict[str, Any] | None) -> dict[str, Any] | None:
    if report is None:
        return None
    findings = report.get("findings")
    return {
        "schema_version": report.get("schema_version"),
        "status": report.get("status"),
        "clean": report.get("clean"),
        "summary": report.get("summary"),
        "completion": report.get("completion"),
        "agent_prompt": report.get("agent_prompt"),
        "ok": report.get("ok"),
        "runner": report.get("runner"),
        "runners": report.get("runners"),
        "findings_preview": findings[:4] if isinstance(findings, list) else [],
    }


def _squash(value: str) -> str:
    return "".join(
        character if character.isalnum() or character in "-_" else "-"
        for character in value
    )


def meter_dos_resource_evidence_bridge() -> AdapterInvocation:
    return EvidenceDesign.adapter_invocation("meter", ".")


def rig_exploit_journey_bridge() -> AdapterInvocation:
    return EvidenceDesign.adapter_invocation("rig", "scenario.toml")


def vat_isolated_security_runner() -> AdapterInvocation:
    return EvidenceDesign.adapter_invocation("vat", "guard-security-smoke")


def dynamic_adapter_routing() -> tuple[AdapterInvocation, ...]:
    return (
        vat_isolated_security_runner(),
        rig_exploit_journey_bridge(),
        meter_dos_resource_evidence_bridge(),
    )


def stable_evidence_folding() -> EvidenceDecision:
    return EvidenceDesign.normalize_result(True, True, 0)
