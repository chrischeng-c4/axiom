"""Executable reference implementation of ``guard.report/1``.

The Python TD is usable product code: it owns report reduction, persistence,
and the public JSON projection. The Rust CB may optimize the implementation,
but must preserve these results.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from enum import Enum
import json
from pathlib import Path
from typing import Any

__aw_artifact_id__ = "artifact:guard/security-ec-profile"
__aw_public_contract__ = True
__aw_public_behaviors__ = (
    "lifecycle_security_metric",
    "ec_security_evidence_command",
    "security_report_consumer_contract",
    "stable_security_metric_projection",
)

SCHEMA_VERSION = "guard.report/1"
TOOL_VERSION = "0.1.0-td"


class Severity(str, Enum):
    CRITICAL = "critical"
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"
    INFO = "info"

    @property
    def rank(self) -> int:
        return list(Severity).index(self)

    @property
    def actionable(self) -> bool:
        return self is not Severity.INFO


class ReportState(str, Enum):
    CLEAN = "clean"
    FINDINGS = "findings"
    TOOL_ERROR = "tool_error"


@dataclass(frozen=True)
class ReportDecision:
    state: ReportState
    exit_code: int
    completion_clean: bool


@dataclass(frozen=True)
class Location:
    path: str
    start_line: int
    start_col: int
    end_line: int
    end_col: int


@dataclass(frozen=True)
class Finding:
    id: str
    severity: Severity
    rule: str
    title: str
    detail: str
    remediation: str
    location: Location
    evidence: dict[str, Any]

    def to_dict(self) -> dict[str, Any]:
        value = asdict(self)
        value["severity"] = self.severity.value
        return value

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> Finding:
        return cls(
            id=str(value["id"]),
            severity=Severity(str(value["severity"])),
            rule=str(value["rule"]),
            title=str(value["title"]),
            detail=str(value["detail"]),
            remediation=str(value["remediation"]),
            location=Location(**value["location"]),
            evidence=dict(value.get("evidence", {})),
        )


@dataclass(frozen=True)
class Summary:
    files_scanned: int = 0
    diagnostics_scanned: int = 0
    security_findings: int = 0
    evidence_count: int = 0
    evidence_failed: int = 0
    critical: int = 0
    high: int = 0
    medium: int = 0
    low: int = 0
    info: int = 0
    sample: tuple[str, ...] = ()
    truncated: bool = False

    @classmethod
    def build(
        cls,
        files_scanned: int,
        diagnostics_scanned: int,
        findings: list[Finding],
        evidence: list[dict[str, Any]],
    ) -> Summary:
        counts = {severity.value: 0 for severity in Severity}
        for finding in findings:
            counts[finding.severity.value] += 1
        return cls(
            files_scanned=files_scanned,
            diagnostics_scanned=diagnostics_scanned,
            security_findings=len(findings),
            evidence_count=len(evidence),
            evidence_failed=sum(not bool(item.get("clean")) for item in evidence),
            critical=counts["critical"],
            high=counts["high"],
            medium=counts["medium"],
            low=counts["low"],
            info=counts["info"],
            sample=tuple(finding.id for finding in findings[:8]),
        )

    def to_dict(self) -> dict[str, Any]:
        value = asdict(self)
        value["sample"] = list(self.sample)
        return value


@dataclass(frozen=True)
class Completion:
    clean: bool
    criteria: tuple[str, ...]
    missing: tuple[str, ...]

    def to_dict(self) -> dict[str, Any]:
        value = asdict(self)
        value["criteria"] = list(self.criteria)
        value["missing"] = list(self.missing)
        return value


@dataclass(frozen=True)
class IntegrationMap:
    static_engine: str = "guard-python-reference"
    isolated_runner: str = "vat"
    dynamic_journeys: str = "rig"
    resource_evidence: str = "meter"
    benchmark_budget: str = "legacy arena (optional)"


@dataclass(frozen=True)
class GuardReport:
    verb: str
    target: str
    policy_profile: str
    state: ReportState
    exit_code: int
    summary: Summary
    findings: tuple[Finding, ...]
    evidence: tuple[dict[str, Any], ...]
    completion: Completion
    agent_prompt: str
    tool_error_code: int | None = None
    schema_version: str = SCHEMA_VERSION
    tool_version: str = TOOL_VERSION
    integrations: IntegrationMap = field(default_factory=IntegrationMap)

    @classmethod
    def from_scan(
        cls,
        target: str,
        policy_profile: str,
        files_scanned: int,
        diagnostics_scanned: int,
        findings: list[Finding],
        evidence: list[dict[str, Any]],
    ) -> GuardReport:
        ordered = sorted(findings, key=_finding_sort_key)
        state = (
            ReportState.FINDINGS
            if any(finding.severity.actionable for finding in ordered)
            else ReportState.CLEAN
        )
        summary = Summary.build(
            files_scanned,
            diagnostics_scanned,
            ordered,
            evidence,
        )
        missing = tuple(
            message
            for tool, message in (
                ("vat", "vat isolated security runner evidence is not configured"),
                ("rig", "rig exploit/e2e journey evidence is not configured"),
                ("meter", "meter DoS/resource evidence is not configured"),
            )
            if not any(item.get("tool") == tool for item in evidence)
        )
        clean = state is ReportState.CLEAN
        prompt = (
            "guard scan is clean for the configured security evidence"
            if clean
            else (
                f"guard found {summary.security_findings} security finding(s); "
                "inspect summary.sample, findings, and evidence"
            )
        )
        return cls(
            verb="scan",
            target=target,
            policy_profile=policy_profile,
            state=state,
            exit_code=0 if clean else 1,
            summary=summary,
            findings=tuple(ordered),
            evidence=tuple(evidence),
            completion=Completion(
                clean,
                (
                    "security diagnostics were scanned",
                    "findings were normalized into guard.report/1",
                    "vat/rig/meter evidence adapters are available",
                ),
                missing,
            ),
            agent_prompt=prompt,
        )

    @classmethod
    def stub(cls, verb: str, prompt: str) -> GuardReport:
        return cls(
            verb=verb,
            target="-",
            policy_profile="guard-baseline-static/1",
            state=ReportState.CLEAN,
            exit_code=0,
            summary=Summary(),
            findings=(),
            evidence=(),
            completion=Completion(
                True,
                ("offline self-description emitted",),
                (),
            ),
            agent_prompt=prompt,
        )

    @classmethod
    def tool_error(
        cls,
        verb: str,
        target: str,
        code: int,
        message: str,
    ) -> GuardReport:
        return cls(
            verb=verb,
            target=target,
            policy_profile="guard-baseline-static/1",
            state=ReportState.TOOL_ERROR,
            exit_code=code,
            summary=Summary(),
            findings=(),
            evidence=(),
            completion=Completion(False, (), (message,)),
            agent_prompt=f"guard {verb} could not run: {message}",
            tool_error_code=code,
        )

    def to_dict(self) -> dict[str, Any]:
        status: dict[str, Any] = {"state": self.state.value}
        if self.tool_error_code is not None:
            status["code"] = self.tool_error_code
        value: dict[str, Any] = {
            "schema_version": self.schema_version,
            "tool_version": self.tool_version,
            "verb": self.verb,
            "target": self.target,
            "policy_profile": self.policy_profile,
            "status": status,
            "exit_code": self.exit_code,
            "summary": self.summary.to_dict(),
            "findings": [finding.to_dict() for finding in self.findings],
            "completion": self.completion.to_dict(),
            "integrations": asdict(self.integrations),
            "agent_prompt": self.agent_prompt,
        }
        if self.evidence:
            value["evidence"] = list(self.evidence)
        return value

    def to_json(self, compact: bool = False) -> str:
        return json.dumps(
            self.to_dict(),
            sort_keys=True,
            separators=(",", ":") if compact else None,
            indent=None if compact else 2,
        )

    def persist(self, directory: Path) -> None:
        report_dir = directory / ".guard"
        report_dir.mkdir(parents=True, exist_ok=True)
        (report_dir / "last-report.json").write_text(
            self.to_json(),
            encoding="utf-8",
        )

    @classmethod
    def read_last(cls, directory: Path) -> GuardReport:
        value = json.loads(
            (directory / ".guard" / "last-report.json").read_text(encoding="utf-8")
        )
        status = value["status"]
        return cls(
            verb=value["verb"],
            target=value["target"],
            policy_profile=value["policy_profile"],
            state=ReportState(status["state"]),
            exit_code=int(value["exit_code"]),
            summary=Summary(
                **{
                    **value["summary"],
                    "sample": tuple(value["summary"].get("sample", ())),
                }
            ),
            findings=tuple(Finding.from_dict(item) for item in value["findings"]),
            evidence=tuple(value.get("evidence", ())),
            completion=Completion(
                bool(value["completion"]["clean"]),
                tuple(value["completion"]["criteria"]),
                tuple(value["completion"]["missing"]),
            ),
            agent_prompt=value["agent_prompt"],
            tool_error_code=status.get("code"),
            schema_version=value["schema_version"],
            tool_version=value["tool_version"],
            integrations=IntegrationMap(**value["integrations"]),
        )


class ReportDesign:
    REQUIRED_FIELDS = tuple(GuardReport.stub("spec", "contract").to_dict())

    @staticmethod
    def reduce_state(
        actionable_findings: int,
        tool_error_code: int | None,
    ) -> ReportDecision:
        if tool_error_code is not None:
            return ReportDecision(ReportState.TOOL_ERROR, tool_error_code, False)
        if actionable_findings > 0:
            return ReportDecision(ReportState.FINDINGS, 1, False)
        return ReportDecision(ReportState.CLEAN, 0, True)


def _finding_sort_key(finding: Finding) -> tuple[int, str]:
    return finding.severity.rank, finding.id


def lifecycle_security_metric() -> tuple[str, int, bool]:
    report = GuardReport.from_scan(".", "guard-baseline-static/1", 0, 0, [], [])
    return report.state.value, report.exit_code, report.completion.clean


def ec_security_evidence_command() -> ReportDecision:
    return ReportDesign.reduce_state(actionable_findings=1, tool_error_code=None)


def security_report_consumer_contract() -> tuple[str, ...]:
    return ReportDesign.REQUIRED_FIELDS


def stable_security_metric_projection() -> tuple[str, str, int]:
    report = GuardReport.from_scan(".", "guard-baseline-static/1", 0, 0, [], [])
    return report.schema_version, report.state.value, report.exit_code
