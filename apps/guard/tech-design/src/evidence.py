"""Guard external-evidence contract, routing, and normalization."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

__aw_artifact_id__ = "artifact:guard/dynamic-security-evidence"
__aw_public_contract__ = True


class EvidenceStatus(Enum):
    CLEAN = "clean"
    FINDINGS = "findings"


@dataclass(frozen=True)
class EvidenceDecision:
    status: EvidenceStatus
    clean: bool
    finding_count: int


@dataclass(frozen=True)
class AdapterInvocation:
    tool: str
    leading_arguments: tuple[str, ...]


class EvidenceDesign:
    REQUIRED_TOOLS = ("vat", "rig", "meter")

    @staticmethod
    def normalize_result(
        process_success: bool,
        report_clean: bool | None,
        finding_count: int,
    ) -> EvidenceDecision:
        clean = process_success and (
            report_clean if report_clean is not None else True
        )
        status = EvidenceStatus.CLEAN if clean else EvidenceStatus.FINDINGS
        return EvidenceDecision(status, clean, finding_count)

    @staticmethod
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


def meter_dos_resource_evidence_bridge() -> str:
    return "meter-target executes Meter and folds resource evidence into Guard"


def rig_exploit_journey_bridge() -> str:
    return "rig-scenario executes Rig and folds journey evidence into Guard"


def vat_isolated_security_runner() -> str:
    return "vat-runner executes VAT and folds isolated evidence into Guard"


def dynamic_adapter_routing() -> str:
    return "public adapter inputs map to exact argv and folded evidence"


def stable_evidence_folding() -> str:
    return "equivalent adapter runs preserve command grammar and folded results"
