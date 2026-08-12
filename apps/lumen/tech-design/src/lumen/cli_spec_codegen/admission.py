"""Admission deciders for CLI-spec-codegen verification (#2334)."""
from __future__ import annotations

from typing import Final

from lumen.cli_spec_codegen.spec import (
    CleanupRecord,
    FailureClassification,
    ResourceCategory,
    TerminalInput,
    VerificationRecord,
)
from lumen.cli_spec_codegen.verdict import (
    AdmittedCleanupRecord,
    AdmittedVerificationRecord,
    Open,
    Passed,
    Rejection,
    RejectionReason,
    TrackedSkip,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/cli-spec-codegen-2334-admission"

REQUIRED_CARGO_COMMAND: Final[str] = (
    "cargo test -p lumen --test cli_convention --test spec_cli --test spec_gen_e2e --test generated_clients_crud_e2e"
)

REQUIRED_GATES: Final[tuple[str, ...]] = (
    "cli_convention",
    "spec_cli",
    "spec_gen_e2e",
    "generated_clients_crud_e2e",
)

REQUIRED_CLIENT_LANGUAGES: Final[set[str]] = {"ts", "py", "rust"}

REQUIRED_CLEANUP_CATEGORIES: Final[tuple[ResourceCategory, ...]] = (
    ResourceCategory.PROCESS,
    ResourceCategory.NAMESPACE,
    ResourceCategory.EVIDENCE,
)


def decide_verification_record(
    record: VerificationRecord,
) -> AdmittedVerificationRecord | Rejection:
    """Decide admission of a verification record per R1/R2/AC1/AC2."""
    if not isinstance(record, VerificationRecord):
        return Rejection(reason=RejectionReason.INVALID_RECORD, field_path="record")

    if record.command != REQUIRED_CARGO_COMMAND:
        return Rejection(
            reason=RejectionReason.REQUIRED_COMMAND_MISMATCH,
            field_path="command",
        )

    if not record.gates:
        return Rejection(reason=RejectionReason.GATE_WORK_ZERO, field_path="gates")

    for gate in record.gates:
        if gate.exit_code != 0:
            return Rejection(
                reason=RejectionReason.GATE_EXIT_NONZERO,
                field_path="gates",
            )
        if gate.work_count <= 0:
            return Rejection(
                reason=RejectionReason.GATE_WORK_ZERO,
                field_path="gates",
            )

    observed_gate_names = {gate.name for gate in record.gates}
    for required_gate in REQUIRED_GATES:
        if required_gate not in observed_gate_names:
            return Rejection(
                reason=RejectionReason.REQUIRED_GATE_MISSING,
                field_path="gates",
            )

    if record.evidence is None or record.evidence.commit is None:
        return Rejection(
            reason=RejectionReason.MISSING_COMMIT,
            field_path="evidence.commit",
        )
    if record.evidence.environment is None:
        return Rejection(
            reason=RejectionReason.MISSING_ENVIRONMENT,
            field_path="evidence.environment",
        )
    if record.evidence.output_summary is None:
        return Rejection(
            reason=RejectionReason.MISSING_OUTPUT_SUMMARY,
            field_path="evidence.output_summary",
        )
    if record.evidence.evidence_path is None:
        return Rejection(
            reason=RejectionReason.MISSING_EVIDENCE_PATH,
            field_path="evidence.evidence_path",
        )

    if not record.grammar_observed:
        return Rejection(
            reason=RejectionReason.GRAMMAR_NOT_OBSERVED,
            field_path="grammar_observed",
        )

    if not record.spec_formats_observed:
        return Rejection(
            reason=RejectionReason.SPEC_FORMATS_NOT_OBSERVED,
            field_path="spec_formats_observed",
        )

    if not REQUIRED_CLIENT_LANGUAGES.issubset(set(record.client_languages)):
        return Rejection(
            reason=RejectionReason.CLIENT_LANGUAGES_INCOMPLETE,
            field_path="client_languages",
        )

    if not record.deployment_renderers_observed:
        return Rejection(
            reason=RejectionReason.RENDERERS_NOT_OBSERVED,
            field_path="deployment_renderers_observed",
        )

    if not record.cold_regeneration_observed:
        return Rejection(
            reason=RejectionReason.COLD_REGENERATION_NOT_OBSERVED,
            field_path="cold_regeneration_observed",
        )

    if not record.executable_stdout_observed:
        return Rejection(
            reason=RejectionReason.EXECUTABLE_STDOUT_NOT_OBSERVED,
            field_path="executable_stdout_observed",
        )

    if not record.deterministic_generation_observed:
        return Rejection(
            reason=RejectionReason.DETERMINISTIC_GENERATION_NOT_OBSERVED,
            field_path="deterministic_generation_observed",
        )

    if not record.no_todo_or_invalid_shell_scaffold:
        return Rejection(
            reason=RejectionReason.TODO_OR_INVALID_SHELL_SCAFFOLD,
            field_path="no_todo_or_invalid_shell_scaffold",
        )

    if record.generated_test_work_count <= 0:
        return Rejection(
            reason=RejectionReason.GENERATED_TEST_WORK_ZERO,
            field_path="generated_test_work_count",
        )

    return AdmittedVerificationRecord(record=record)


def decide_cleanup_record(
    cleanup: CleanupRecord,
) -> AdmittedCleanupRecord | Rejection:
    """Decide admission of a cleanup record per AC4."""
    if not isinstance(cleanup, CleanupRecord):
        return Rejection(reason=RejectionReason.INVALID_CLEANUP, field_path="cleanup")

    if not cleanup.receipts:
        return Rejection(reason=RejectionReason.CLEANUP_INCOMPLETE, field_path="receipts")

    for receipt in cleanup.receipts:
        if not receipt.success_path_complete or not receipt.failure_path_complete:
            return Rejection(
                reason=RejectionReason.CLEANUP_INCOMPLETE,
                field_path=receipt.category.value,
            )

    present_categories = {receipt.category for receipt in cleanup.receipts}
    for req_cat in REQUIRED_CLEANUP_CATEGORIES:
        if req_cat not in present_categories:
            return Rejection(
                reason=RejectionReason.CLEANUP_INCOMPLETE,
                field_path=req_cat.value,
            )

    return AdmittedCleanupRecord(record=cleanup)


def decide_terminal_result(
    terminal_input: TerminalInput,
) -> Passed | TrackedSkip | Open | Rejection:
    """Decide terminal verification result per R3/R4/AC3."""
    if not isinstance(terminal_input, TerminalInput):
        return Rejection(reason=RejectionReason.INVALID_RECORD, field_path="terminal_input")

    # Validate verification record
    raw_record = terminal_input.record
    if isinstance(raw_record, AdmittedVerificationRecord):
        admitted_verification = raw_record
    elif isinstance(raw_record, VerificationRecord):
        res = decide_verification_record(raw_record)
        if isinstance(res, Rejection):
            return res
        admitted_verification = res
    elif isinstance(raw_record, Rejection):
        return raw_record
    else:
        return Rejection(reason=RejectionReason.INVALID_RECORD, field_path="record")

    # Validate cleanup record
    raw_cleanup = terminal_input.cleanup
    if isinstance(raw_cleanup, AdmittedCleanupRecord):
        admitted_cleanup = raw_cleanup
    elif isinstance(raw_cleanup, CleanupRecord):
        c_res = decide_cleanup_record(raw_cleanup)
        if isinstance(c_res, Rejection):
            return c_res
        admitted_cleanup = c_res
    elif isinstance(raw_cleanup, Rejection):
        return raw_cleanup
    else:
        return Rejection(reason=RejectionReason.INVALID_CLEANUP, field_path="cleanup")

    classification = terminal_input.classification
    if classification == FailureClassification.ALL_GREEN:
        return Passed(
            record=admitted_verification.record,
            cleanup=admitted_cleanup.record,
        )

    if classification == FailureClassification.APP_DOMAIN_ONLY:
        issue_ref = terminal_input.validated_issue_number
        if issue_ref is None:
            return Rejection(
                reason=RejectionReason.VALIDATED_ISSUE_NUMBER_MISSING,
                field_path="validated_issue_number",
            )
        if not issue_ref.startswith("#") or not issue_ref[1:].isdigit() or int(issue_ref[1:]) <= 0:
            return Rejection(
                reason=RejectionReason.VALIDATED_ISSUE_NUMBER_NONPOSITIVE,
                field_path="validated_issue_number",
            )
        if terminal_input.exact_reproduction is None or not terminal_input.exact_reproduction.strip():
            return Rejection(
                reason=RejectionReason.EXACT_REPRODUCTION_MISSING,
                field_path="exact_reproduction",
            )
        return TrackedSkip(
            issue_ref=issue_ref,
            reproduction=terminal_input.exact_reproduction,
            record=admitted_verification.record,
            cleanup=admitted_cleanup.record,
        )

    if classification in (
        FailureClassification.SHARED,
        FailureClassification.NON_DOMAIN,
        FailureClassification.MIXED,
    ):
        return Open(
            classification=classification,
            record=admitted_verification.record,
            cleanup=admitted_cleanup.record,
            issue_ref=terminal_input.validated_issue_number,
            reproduction=terminal_input.exact_reproduction,
        )

    return Rejection(reason=RejectionReason.CLASSIFICATION_OPEN, field_path="classification")
