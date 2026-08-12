"""EC behavior case for #2334 -- CLI-spec-codegen record admission.

Every expected value below is an EC-owned literal transcribed from #2334:
R1/AC1 fixes the reusable Cargo gate and requires successful non-zero gate
work plus complete provenance; R2/AC2 requires the complete CLI/spec/codegen
observation set; R3/R4/AC3 permits only a verified pass or a bounded,
app-domain-only tracked skip; and AC4 requires both cleanup paths for every
declared resource category.  Runtime execution, Git, tracker lookup, and real
resource cleanup are intentionally absent because this case drives only the
pure design model.
"""

from __future__ import annotations

from lumen.cli_spec_codegen.admission import (
    decide_cleanup_record,
    decide_terminal_result,
    decide_verification_record,
)
from lumen.cli_spec_codegen.spec import (
    CleanupReceipt,
    CleanupRecord,
    FailureClassification,
    GateObservation,
    ResourceCategory,
    TerminalInput,
    VerificationEvidence,
    VerificationRecord,
)
from lumen.cli_spec_codegen.verdict import Rejection

MINIMUM_CHECKS = 10

CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX = (
    ("required_reusable_gate_command_is_admitted", "admitted"),
    ("admitted_record_retains_the_exact_required_command", "cargo test -p lumen --test cli_convention --test spec_cli --test spec_gen_e2e --test generated_clients_crud_e2e"),
    ("all_three_client_languages_are_retained", ("ts", "py", "rust")),
    ("admitted_record_retains_nonzero_generated_test_work", 9),
    ("admitted_record_retains_evidence_location", "external-contracts/evidence/2334.json"),
    ("complete_cleanup_record_is_admitted", "admitted"),
    ("cleanup_admission_retains_all_declared_categories", 3),
    ("verified_all_green_record_reaches_passed", "Passed"),
    ("app_domain_only_issue_backed_result_reaches_tracked_skip", "TrackedSkip"),
    ("tracked_skip_retains_the_exact_issue_reference", "#2340"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _complete_record(**overrides) -> VerificationRecord:
    values = {
        "command": "cargo test -p lumen --test cli_convention --test spec_cli --test spec_gen_e2e --test generated_clients_crud_e2e",
        "gates": (
            GateObservation(name="cli_convention", exit_code=0, work_count=3),
            GateObservation(name="spec_cli", exit_code=0, work_count=8),
            GateObservation(name="spec_gen_e2e", exit_code=0, work_count=4),
            GateObservation(name="generated_clients_crud_e2e", exit_code=0, work_count=5),
        ),
        "grammar_observed": True,
        "spec_formats_observed": True,
        "client_languages": ("ts", "py", "rust"),
        "deployment_renderers_observed": True,
        "cold_regeneration_observed": True,
        "executable_stdout_observed": True,
        "deterministic_generation_observed": True,
        "no_todo_or_invalid_shell_scaffold": True,
        "generated_test_work_count": 9,
        "evidence": VerificationEvidence(
            commit="abc1234",
            environment="local-isolated",
            output_summary="20 gate checks and 9 generated-client assertions passed",
            evidence_path="external-contracts/evidence/2334.json",
            duration_ms=1200,
            resource_summary="peak_rss_mb=96",
        ),
    }
    values.update(overrides)
    return VerificationRecord(**values)


def _complete_cleanup(**overrides) -> CleanupRecord:
    values = {
        "receipts": (
            CleanupReceipt(category=ResourceCategory.PROCESS, success_path_complete=True, failure_path_complete=True),
            CleanupReceipt(category=ResourceCategory.NAMESPACE, success_path_complete=True, failure_path_complete=True),
            CleanupReceipt(category=ResourceCategory.EVIDENCE, success_path_complete=True, failure_path_complete=True),
        )
    }
    values.update(overrides)
    return CleanupRecord(**values)


def verify_cli_spec_codegen_2334_behavior() -> dict:
    checks = []

    admitted = decide_verification_record(_complete_record())

    # 1. R1/AC1 -- only the issue's exact reusable command reaches admission.
    obs1 = _outcome(admitted)
    exp1 = CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1/AC1 -- admission preserves the exact command rather than a label.
    obs2 = admitted.record.command if not isinstance(admitted, Rejection) else "rejected"
    exp2 = CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R2/AC2 -- the admitted record carries all three required client languages.
    full_surface = decide_verification_record(_complete_record())
    obs3 = full_surface.record.client_languages if not isinstance(full_surface, Rejection) else ()
    exp3 = CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R2/AC2 -- generated tests prove non-zero work, not merely a flag.
    obs4 = full_surface.record.generated_test_work_count if not isinstance(full_surface, Rejection) else -1
    exp4 = CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R1/R2 -- evidence has a retained, reproducible location.
    obs5 = full_surface.record.evidence.evidence_path if not isinstance(full_surface, Rejection) else "rejected"
    exp5 = CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    cleanup = decide_cleanup_record(_complete_cleanup())

    # 6. AC4 -- all declared success and failure path receipts admit cleanup.
    obs6 = _outcome(cleanup)
    exp6 = CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. AC4 -- admission preserves the full declared resource-category set.
    obs7 = len(cleanup.record.receipts) if not isinstance(cleanup, Rejection) else -1
    exp7 = CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    passed = decide_terminal_result(
        TerminalInput(record=full_surface, classification=FailureClassification.ALL_GREEN, cleanup=cleanup)
    )

    # 8. R3/AC3 -- a verified all-green record reaches the passed vocabulary.
    obs8 = type(passed).__name__
    exp8 = CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    tracked = decide_terminal_result(
        TerminalInput(
            record=full_surface,
            classification=FailureClassification.APP_DOMAIN_ONLY,
            cleanup=cleanup,
            validated_issue_number="#2340",
            exact_reproduction="lumen spec gen --lang rust --out /tmp/lumen-client",
        )
    )

    # 9. R3/AC3 -- the bounded app-domain-only path has its own terminal type.
    obs9 = type(tracked).__name__
    exp9 = CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R3/AC3 -- that result retains the one exact issue reference.
    obs10 = tracked.issue_ref if not isinstance(tracked, Rejection) else "rejected"
    exp10 = CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "cli-spec-codegen-2334-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
