"""Data structures for Lumen CLI-spec-codegen design verification (#2334)."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Any, Final

__aw_artifact_id__: Final[str] = "artifact:lumen/cli-spec-codegen-2334-spec"


class ResourceCategory(Enum):
    PROCESS = "process"
    NAMESPACE = "namespace"
    VOLUME = "volume"
    CLUSTER = "cluster"
    CLOUD_RESOURCE = "cloud_resource"
    CREDENTIAL = "credential"
    EVIDENCE = "evidence"


@dataclass(frozen=True)
class CleanupReceipt:
    category: ResourceCategory
    success_path_complete: bool
    failure_path_complete: bool


@dataclass(frozen=True)
class CleanupRecord:
    receipts: tuple[CleanupReceipt, ...]


@dataclass(frozen=True)
class GateObservation:
    name: str
    exit_code: int
    work_count: int


@dataclass(frozen=True)
class VerificationEvidence:
    commit: str | None
    environment: str
    output_summary: str
    evidence_path: str
    duration_ms: int | None = None
    resource_summary: str | None = None


@dataclass(frozen=True)
class VerificationRecord:
    command: str
    gates: tuple[GateObservation, ...]
    grammar_observed: bool
    spec_formats_observed: bool
    client_languages: tuple[str, ...]
    deployment_renderers_observed: bool
    cold_regeneration_observed: bool
    executable_stdout_observed: bool
    deterministic_generation_observed: bool
    no_todo_or_invalid_shell_scaffold: bool
    generated_test_work_count: int
    evidence: VerificationEvidence


class FailureClassification(Enum):
    ALL_GREEN = "all_green"
    APP_DOMAIN_ONLY = "app_domain_only"
    SHARED = "shared"
    NON_DOMAIN = "non_domain"
    MIXED = "mixed"


@dataclass(frozen=True)
class TerminalInput:
    record: Any
    classification: FailureClassification
    cleanup: Any
    validated_issue_number: str | None = None
    exact_reproduction: str | None = None
    authoritative_domain_items_accepted: bool = False
