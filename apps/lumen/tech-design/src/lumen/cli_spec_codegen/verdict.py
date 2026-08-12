"""Verdict types and rejection reason vocabulary for CLI-spec-codegen verification (#2334)."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final

from lumen.cli_spec_codegen.spec import (
    CleanupRecord,
    FailureClassification,
    VerificationRecord,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/cli-spec-codegen-2334-verdict"


class RejectionReason(Enum):
    REQUIRED_COMMAND_MISMATCH = "required_command_mismatch"
    REQUIRED_GATE_MISSING = "required_gate_missing"
    GATE_EXIT_NONZERO = "gate_exit_nonzero"
    GATE_WORK_ZERO = "gate_work_zero"
    MISSING_COMMIT = "missing_commit"
    MISSING_ENVIRONMENT = "missing_environment"
    MISSING_OUTPUT_SUMMARY = "missing_output_summary"
    MISSING_EVIDENCE_PATH = "missing_evidence_path"
    GRAMMAR_NOT_OBSERVED = "grammar_not_observed"
    TODO_OR_INVALID_SHELL_SCAFFOLD = "todo_or_invalid_shell_scaffold"
    SPEC_FORMATS_NOT_OBSERVED = "spec_formats_not_observed"
    CLIENT_LANGUAGES_INCOMPLETE = "client_languages_incomplete"
    RENDERERS_NOT_OBSERVED = "renderers_not_observed"
    COLD_REGENERATION_NOT_OBSERVED = "cold_regeneration_not_observed"
    EXECUTABLE_STDOUT_NOT_OBSERVED = "executable_stdout_not_observed"
    DETERMINISTIC_GENERATION_NOT_OBSERVED = "deterministic_generation_not_observed"
    GENERATED_TEST_WORK_ZERO = "generated_test_work_zero"
    CLEANUP_INCOMPLETE = "CLEANUP_INCOMPLETE"
    VALIDATED_ISSUE_NUMBER_MISSING = "validated_issue_number_missing"
    VALIDATED_ISSUE_NUMBER_NONPOSITIVE = "validated_issue_number_nonpositive"
    EXACT_REPRODUCTION_MISSING = "exact_reproduction_missing"
    INVALID_RECORD = "invalid_record"
    INVALID_CLEANUP = "invalid_cleanup"
    CLASSIFICATION_OPEN = "classification_open"


@dataclass(frozen=True)
class Rejection:
    reason: RejectionReason
    field_path: str = ""


@dataclass(frozen=True)
class AdmittedVerificationRecord:
    record: VerificationRecord


@dataclass(frozen=True)
class AdmittedCleanupRecord:
    record: CleanupRecord


@dataclass(frozen=True)
class Passed:
    record: VerificationRecord
    cleanup: CleanupRecord


@dataclass(frozen=True)
class TrackedSkip:
    issue_ref: str
    reproduction: str
    record: VerificationRecord
    cleanup: CleanupRecord


@dataclass(frozen=True)
class Open:
    classification: FailureClassification
    record: VerificationRecord
    cleanup: CleanupRecord
    issue_ref: str | None = None
    reproduction: str | None = None
