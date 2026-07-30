"""Executable Guard security-policy selection and severity mapping."""

from __future__ import annotations

from enum import Enum

from report import Severity

__aw_artifact_id__ = "artifact:guard/security-policy-profile"
__aw_public_contract__ = True
__aw_public_behaviors__ = (
    "baseline_static_policy",
    "standalone_cli_distribution",
    "security_lint_policy",
    "stable_policy_selection",
)


class PolicyProfile(str, Enum):
    BASELINE_STATIC = "guard-baseline-static/1"
    SECURITY_LINT = "guard-security-lint/1"
    STRICT = "guard-strict/1"

    @classmethod
    def from_cli(cls, value: str) -> PolicyProfile:
        normalized = value.replace("_", "-")
        aliases = {
            "baseline-static": cls.BASELINE_STATIC,
            "security-lint": cls.SECURITY_LINT,
            "strict": cls.STRICT,
        }
        if normalized in aliases:
            return aliases[normalized]
        return cls(value)


class DiagnosticCategory(str, Enum):
    SECURITY = "security"
    STYLE = "style"
    LOGIC = "logic"
    SYNTAX = "syntax"


class DiagnosticSeverity(str, Enum):
    ERROR = "error"
    WARNING = "warning"
    INFORMATION = "info"
    HINT = "hint"


class PolicyDesign:
    """Fail-closed profile behavior used by the reference scanner."""

    SECURITY_IMPACTING_LINT = frozenset(
        {"DK002", "JS007", "JS008", "SQL-INJ", "TS102"}
    )

    @classmethod
    def included_rule(
        cls,
        profile: PolicyProfile,
        category: DiagnosticCategory | str,
        rule: str,
    ) -> bool:
        category_value = (
            category.value if isinstance(category, DiagnosticCategory) else category
        )
        if category_value == DiagnosticCategory.SECURITY.value:
            return True
        return (
            profile in {PolicyProfile.SECURITY_LINT, PolicyProfile.STRICT}
            and rule in cls.SECURITY_IMPACTING_LINT
        )

    @staticmethod
    def map_severity(
        profile: PolicyProfile,
        severity: DiagnosticSeverity,
    ) -> Severity:
        if profile is PolicyProfile.STRICT:
            return {
                DiagnosticSeverity.ERROR: Severity.HIGH,
                DiagnosticSeverity.WARNING: Severity.HIGH,
                DiagnosticSeverity.INFORMATION: Severity.MEDIUM,
                DiagnosticSeverity.HINT: Severity.LOW,
            }[severity]
        return {
            DiagnosticSeverity.ERROR: Severity.HIGH,
            DiagnosticSeverity.WARNING: Severity.MEDIUM,
            DiagnosticSeverity.INFORMATION: Severity.LOW,
            DiagnosticSeverity.HINT: Severity.INFO,
        }[severity]


def baseline_static_policy() -> bool:
    return PolicyDesign.included_rule(
        PolicyProfile.BASELINE_STATIC,
        DiagnosticCategory.SECURITY,
        "JS004",
    )


def standalone_cli_distribution() -> tuple[str, ...]:
    return ("scan", "report", "spec", "llm")


def security_lint_policy() -> bool:
    return PolicyDesign.included_rule(
        PolicyProfile.SECURITY_LINT,
        DiagnosticCategory.STYLE,
        "DK002",
    )


def stable_policy_selection() -> str:
    return PolicyProfile.SECURITY_LINT.value
