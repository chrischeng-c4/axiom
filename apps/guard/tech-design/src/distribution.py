"""Executable design for Guard packaging, installation, and EC inventory."""

from dataclasses import dataclass

__aw_artifact_id__ = "artifact:guard/design-distribution"


@dataclass(frozen=True)
class GuardProjectLayout:
    cargo_package: str
    binary_target: str
    capability_contract: str
    external_contract_root: str
    tech_design_root: str


@dataclass(frozen=True)
class BuildProfile:
    name: str
    cargo_arguments: tuple[str, ...]


@dataclass(frozen=True)
class InstallTransaction:
    release_prefix: str
    verifies_checksum: bool
    atomic_replace: bool


@dataclass(frozen=True)
class ExternalUseCase:
    capability_id: str
    use_case_id: str
    dimension: str


@dataclass(frozen=True)
class DomainArtifactSlice:
    """One shallow domain projected across EC, TD, and codebase artifacts."""

    domain: str
    external_contract_root: str
    tech_design_modules: tuple[str, ...]
    codebase_artifacts: tuple[str, ...]


class DistributionDesign:
    """One independently built Cargo package; no CLI registry adapter."""

    @staticmethod
    def project_layout() -> GuardProjectLayout:
        return GuardProjectLayout(
            cargo_package="guard",
            binary_target="guard",
            capability_contract="apps/guard/CAPABILITIES.md",
            external_contract_root="apps/guard/external-contracts",
            tech_design_root="apps/guard/tech-design",
        )

    @staticmethod
    def build_profiles() -> tuple[BuildProfile, ...]:
        return (
            BuildProfile("debug", ("build", "-p", "guard", "--bin", "guard")),
            BuildProfile(
                "release",
                ("build", "--release", "-p", "guard", "--bin", "guard"),
            ),
        )

    @staticmethod
    def install_transaction() -> InstallTransaction:
        return InstallTransaction("guard@", True, True)

    @staticmethod
    def domain_artifact_slices() -> tuple[DomainArtifactSlice, ...]:
        """Keep DDD boundaries visible without repeating the Guard root."""

        return (
            DomainArtifactSlice(
                "scan",
                "src/scan",
                ("src/scan.py",),
                ("src/scan.rs",),
            ),
            DomainArtifactSlice(
                "policy",
                "src/policy",
                ("src/policy.py",),
                ("src/policy.rs",),
            ),
            DomainArtifactSlice(
                "report",
                "src/report",
                ("src/report.py",),
                ("src/report.rs",),
            ),
            DomainArtifactSlice(
                "evidence",
                "src/evidence",
                ("src/evidence.py",),
                ("src/evidence.rs",),
            ),
            DomainArtifactSlice(
                "distribution",
                "src/distribution",
                ("src/distribution.py", "src/cli.py"),
                (
                    "Cargo.toml",
                    "src/main.rs",
                    "src/cli.rs",
                    "build.sh",
                    "install.sh",
                ),
            ),
        )

    @staticmethod
    def required_external_use_cases() -> tuple[ExternalUseCase, ...]:
        return (
            ExternalUseCase("security-ec-profile", "lifecycle-security-metric", "security"),
            ExternalUseCase("security-policy-profile", "baseline-static-policy", "security"),
            ExternalUseCase(
                "security-policy-profile",
                "standalone-cli-distribution",
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
            ExternalUseCase("security-policy-profile", "security-lint-policy", "security"),
            ExternalUseCase(
                "static-security-scan",
                "static-scan-clean-report",
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
