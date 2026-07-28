"""Executable design for Guard's Rust library module exports."""

__aw_artifact_id__ = "artifact:guard/design-library-module"


def exported_modules() -> tuple[str, ...]:
    return ("evidence", "report", "scan")


def exported_contract_types() -> tuple[str, ...]:
    return (
        "EvidenceCommand",
        "ExternalEvidence",
        "GuardReport",
        "PolicyProfile",
        "ScanOptions",
    )
