"""Executable design of Guard's dimension-complete external contracts."""

from dataclasses import dataclass

__aw_artifact_id__ = "artifact:guard/design-external-security-contracts"


@dataclass(frozen=True)
class ExternalUseCase:
    capability_id: str
    use_case_id: str
    dimension: str


def required_external_use_cases() -> tuple[ExternalUseCase, ...]:
    return (
        ExternalUseCase(
            "security-ec-profile",
            "aw-health-security-metric",
            "security",
        ),
        ExternalUseCase(
            "security-policy-profile",
            "baseline-static-policy",
            "security",
        ),
        ExternalUseCase(
            "security-policy-profile",
            "cli-module-registration",
            "behavior",
        ),
        ExternalUseCase(
            "static-security-scan",
            "compass-backed-diagnostic-scan",
            "security",
        ),
        ExternalUseCase(
            "security-ec-profile",
            "ec-security-evidence-command",
            "security",
        ),
        ExternalUseCase(
            "dynamic-security-evidence",
            "meter-dos-resource-evidence-bridge",
            "security",
        ),
        ExternalUseCase(
            "dynamic-security-evidence",
            "rig-exploit-journey-bridge",
            "security",
        ),
        ExternalUseCase(
            "security-policy-profile",
            "security-lint-policy",
            "security",
        ),
        ExternalUseCase(
            "static-security-scan",
            "json-report-envelope",
            "security",
        ),
        ExternalUseCase(
            "dynamic-security-evidence",
            "vat-isolated-security-runner",
            "security",
        ),
        ExternalUseCase(
            "security-ec-profile",
            "security-report-consumer-contract",
            "behavior",
        ),
        ExternalUseCase(
            "static-security-scan",
            "scan-command-report-projection",
            "behavior",
        ),
        ExternalUseCase(
            "dynamic-security-evidence",
            "dynamic-adapter-routing",
            "behavior",
        ),
        ExternalUseCase(
            "security-ec-profile",
            "stable-security-metric-projection",
            "stability",
        ),
        ExternalUseCase(
            "security-policy-profile",
            "stable-policy-selection",
            "stability",
        ),
        ExternalUseCase(
            "static-security-scan",
            "stable-static-finding-normalization",
            "stability",
        ),
        ExternalUseCase(
            "dynamic-security-evidence",
            "stable-evidence-folding",
            "stability",
        ),
    )
