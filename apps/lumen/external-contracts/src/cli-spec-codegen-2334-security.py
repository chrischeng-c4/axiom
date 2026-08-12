"""EC security case for #2334 -- fail-closed CLI-spec-codegen decisions.

Every expected value below is an EC-owned literal transcribed from #2334.
R1/AC1 rejects a changed command, failed/zero-work gates, and incomplete
provenance; R2/AC2 rejects absent required observations and passing TODO or
invalid-shell scaffolds; R3/R4/AC3 keeps shared, non-domain, and mixed failures
open; and AC4 rejects a missing success- or failure-path cleanup receipt.  The
case observes reason vocabulary and named fields, with a neighbouring admitted
input for each refusal family, rather than accepting design-computed booleans.
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

MINIMUM_CHECKS = 40

CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX = (
    ("changed_reusable_command_is_rejected", "required_command_mismatch"),
    ("changed_command_refusal_names_command", "command"),
    ("exact_command_neighbour_is_admitted", "admitted"),
    ("nonzero_gate_exit_is_rejected", "gate_exit_nonzero"),
    ("nonzero_exit_refusal_names_gates", "gates"),
    ("zero_gate_work_is_rejected", "gate_work_zero"),
    ("zero_work_refusal_names_gates", "gates"),
    ("missing_commit_is_rejected", "missing_commit"),
    ("missing_commit_refusal_names_commit", "evidence.commit"),
    ("missing_grammar_observation_is_rejected", "grammar_not_observed"),
    ("missing_grammar_refusal_names_grammar_observed", "grammar_observed"),
    ("todo_scaffold_is_rejected", "todo_or_invalid_shell_scaffold"),
    ("todo_scaffold_refusal_names_scaffold_observation", "no_todo_or_invalid_shell_scaffold"),
    ("shared_failure_remains_open", "Open"),
    ("non_domain_failure_remains_open", "Open"),
    ("mixed_failure_remains_open_until_shared_slice_reruns", "Open"),
    ("public_spec_formats_are_required", "spec_formats_not_observed"),
    ("public_spec_formats_refusal_names_observation", "spec_formats_observed"),
    ("all_three_generated_client_languages_are_required", "client_languages_incomplete"),
    ("client_language_refusal_names_client_languages", "client_languages"),
    ("deployment_renderers_are_required", "renderers_not_observed"),
    ("renderer_refusal_names_observation", "deployment_renderers_observed"),
    ("cold_regeneration_is_required", "cold_regeneration_not_observed"),
    ("cold_regeneration_refusal_names_observation", "cold_regeneration_observed"),
    ("executable_stdout_is_required", "executable_stdout_not_observed"),
    ("stdout_refusal_names_observation", "executable_stdout_observed"),
    ("deterministic_generation_is_required", "deterministic_generation_not_observed"),
    ("deterministic_generation_refusal_names_observation", "deterministic_generation_observed"),
    ("nonzero_generated_test_work_is_required", "generated_test_work_zero"),
    ("generated_test_work_refusal_names_count", "generated_test_work_count"),
    ("incomplete_cleanup_is_rejected", "CLEANUP_INCOMPLETE"),
    ("incomplete_cleanup_refusal_names_category", "process"),
    ("omitted_required_gate_is_rejected_and_names_gates", ("required_gate_missing", "gates")),
    ("missing_environment_is_rejected_and_names_field", ("missing_environment", "evidence.environment")),
    ("missing_output_summary_is_rejected_and_names_field", ("missing_output_summary", "evidence.output_summary")),
    ("missing_evidence_path_is_rejected_and_names_field", ("missing_evidence_path", "evidence.evidence_path")),
    ("absent_validated_issue_is_rejected_and_names_field", ("validated_issue_number_missing", "validated_issue_number")),
    ("nonpositive_validated_issue_is_rejected_and_names_field", ("validated_issue_number_nonpositive", "validated_issue_number")),
    ("absent_exact_reproduction_is_rejected_and_names_field", ("exact_reproduction_missing", "exact_reproduction")),
    ("declared_category_without_receipt_is_rejected_and_named", ("CLEANUP_INCOMPLETE", "namespace")),
)


def _reason(verdict) -> str:
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


def verify_cli_spec_codegen_2334_security() -> dict:
    checks = []

    changed_command = decide_verification_record(_complete_record(command="cargo test -p lumen --test spec_cli"))
    # 1. R1/AC1 -- a subset of the required gate is not the reusable gate.
    obs1 = _reason(changed_command)
    exp1 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[0][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    # 2. R1/AC1 -- the mismatch names the immutable command field.
    obs2 = changed_command.field_path if isinstance(changed_command, Rejection) else ""
    exp2 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[1][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    exact_command = decide_verification_record(_complete_record())
    # 3. R1/AC1 -- the exact command is the neighbouring admitted input.
    obs3 = _reason(exact_command)
    exp3 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[2][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    failed_gate = decide_verification_record(_complete_record(gates=(GateObservation(name="cli_convention", exit_code=1, work_count=3),)))
    # 4. R1/AC1 -- a named gate with a nonzero exit cannot be recorded passed.
    obs4 = _reason(failed_gate)
    exp4 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[3][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    # 5. R1/AC1 -- the failure identifies the supplied gate collection.
    obs5 = failed_gate.field_path if isinstance(failed_gate, Rejection) else ""
    exp5 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[4][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    zero_work = decide_verification_record(_complete_record(gates=(GateObservation(name="cli_convention", exit_code=0, work_count=0),)))
    # 6. R1/AC1 -- an exit-zero no-op does not become verification work.
    obs6 = _reason(zero_work)
    exp6 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[5][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    # 7. R1/AC1 -- the zero-work refusal identifies the gate observation.
    obs7 = zero_work.field_path if isinstance(zero_work, Rejection) else ""
    exp7 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[6][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    missing_commit = decide_verification_record(_complete_record(evidence=VerificationEvidence(commit=None, environment="local-isolated", output_summary="ok", evidence_path="external-contracts/evidence/2334.json", duration_ms=1200, resource_summary="peak_rss_mb=96")))
    # 8. R1/AC1 -- a pass claim without commit provenance fails closed.
    obs8 = _reason(missing_commit)
    exp8 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[7][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    # 9. R1/AC1 -- the provenance refusal identifies the missing commit.
    obs9 = missing_commit.field_path if isinstance(missing_commit, Rejection) else ""
    exp9 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[8][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    missing_grammar = decide_verification_record(_complete_record(grammar_observed=False))
    # 10. R2/AC2 -- canonical llm/upgrade/issue grammar must be observed.
    obs10 = _reason(missing_grammar)
    exp10 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[9][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    # 11. R2/AC2 -- the rejection names the absent grammar observation.
    obs11 = missing_grammar.field_path if isinstance(missing_grammar, Rejection) else ""
    exp11 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[10][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    todo_scaffold = decide_verification_record(_complete_record(no_todo_or_invalid_shell_scaffold=False))
    # 12. R2/AC2 -- a TODO or invalid shell scaffold cannot pass by construction.
    obs12 = _reason(todo_scaffold)
    exp12 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[11][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    # 13. R2/AC2 -- it names the concrete scaffold observation that was false.
    obs13 = todo_scaffold.field_path if isinstance(todo_scaffold, Rejection) else ""
    exp13 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[12][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    cleanup = decide_cleanup_record(_complete_cleanup())
    # 14. R3/AC3 -- shared failures stay open until repair and rerun.
    shared = decide_terminal_result(TerminalInput(record=exact_command, classification=FailureClassification.SHARED, cleanup=cleanup))
    obs14 = type(shared).__name__
    exp14 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[13][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    # 15. R3/AC3 -- non-domain failures have the same nonterminal boundary.
    non_domain = decide_terminal_result(TerminalInput(record=exact_command, classification=FailureClassification.NON_DOMAIN, cleanup=cleanup))
    obs15 = type(non_domain).__name__
    exp15 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[14][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    # 16. R4/AC3 -- a mixed result remains open until its shared slice reruns.
    mixed = decide_terminal_result(TerminalInput(record=exact_command, classification=FailureClassification.MIXED, cleanup=cleanup, validated_issue_number="#2340", exact_reproduction="lumen spec gen --lang rust --out /tmp/lumen-client", authoritative_domain_items_accepted=True))
    obs16 = type(mixed).__name__
    exp16 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[15][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    missing_formats = decide_verification_record(_complete_record(spec_formats_observed=False))
    # 17. R2/AC2 -- public spec formats are an explicit required observation.
    obs17 = _reason(missing_formats)
    exp17 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[16][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    obs18 = missing_formats.field_path if isinstance(missing_formats, Rejection) else ""
    exp18 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[17][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    missing_language = decide_verification_record(_complete_record(client_languages=("ts", "py")))
    # 19. R2/AC2 -- omitting Rust cannot masquerade as three-language codegen.
    obs19 = _reason(missing_language)
    exp19 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[18][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})
    obs20 = missing_language.field_path if isinstance(missing_language, Rejection) else ""
    exp20 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[19][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})

    missing_renderer = decide_verification_record(_complete_record(deployment_renderers_observed=False))
    # 21. R2/AC2 -- renderers are not a render-only substitute for the gate.
    obs21 = _reason(missing_renderer)
    exp21 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[20][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})
    obs22 = missing_renderer.field_path if isinstance(missing_renderer, Rejection) else ""
    exp22 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[21][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})

    cold_regeneration = decide_verification_record(_complete_record(cold_regeneration_observed=False))
    # 23. R2/AC2 -- artifacts must be observed cold-regenerated.
    obs23 = _reason(cold_regeneration)
    exp23 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[22][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})
    obs24 = cold_regeneration.field_path if isinstance(cold_regeneration, Rejection) else ""
    exp24 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[23][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[23][0], "expected": exp24, "observed": obs24, "passed": obs24 == exp24})

    missing_stdout = decide_verification_record(_complete_record(executable_stdout_observed=False))
    # 25. R2/AC2 -- an executable next or terminal stdout observation is required.
    obs25 = _reason(missing_stdout)
    exp25 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[24][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[24][0], "expected": exp25, "observed": obs25, "passed": obs25 == exp25})
    obs26 = missing_stdout.field_path if isinstance(missing_stdout, Rejection) else ""
    exp26 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[25][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[25][0], "expected": exp26, "observed": obs26, "passed": obs26 == exp26})

    nondeterministic = decide_verification_record(_complete_record(deterministic_generation_observed=False))
    # 27. R2/AC2 -- generated artifacts must be deterministic.
    obs27 = _reason(nondeterministic)
    exp27 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[26][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[26][0], "expected": exp27, "observed": obs27, "passed": obs27 == exp27})
    obs28 = nondeterministic.field_path if isinstance(nondeterministic, Rejection) else ""
    exp28 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[27][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[27][0], "expected": exp28, "observed": obs28, "passed": obs28 == exp28})

    no_generated_work = decide_verification_record(_complete_record(generated_test_work_count=0))
    # 29. R2/AC2 -- generated tests must report non-zero work.
    obs29 = _reason(no_generated_work)
    exp29 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[28][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[28][0], "expected": exp29, "observed": obs29, "passed": obs29 == exp29})
    obs30 = no_generated_work.field_path if isinstance(no_generated_work, Rejection) else ""
    exp30 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[29][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[29][0], "expected": exp30, "observed": obs30, "passed": obs30 == exp30})

    incomplete_cleanup = decide_cleanup_record(_complete_cleanup(receipts=(CleanupReceipt(category=ResourceCategory.PROCESS, success_path_complete=True, failure_path_complete=False),)))
    # 31. AC4 -- failure-path cleanup is mandatory, not an optional receipt.
    obs31 = _reason(incomplete_cleanup)
    exp31 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[30][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[30][0], "expected": exp31, "observed": obs31, "passed": obs31 == exp31})
    # 32. AC4 -- the cleanup rejection identifies the uncleaned category.
    obs32 = incomplete_cleanup.field_path if isinstance(incomplete_cleanup, Rejection) else ""
    exp32 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[31][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[31][0], "expected": exp32, "observed": obs32, "passed": obs32 == exp32})

    omitted_gate = decide_verification_record(
        _complete_record(gates=(
            GateObservation(name="cli_convention", exit_code=0, work_count=3),
            GateObservation(name="spec_cli", exit_code=0, work_count=8),
            GateObservation(name="spec_gen_e2e", exit_code=0, work_count=4),
        ))
    )
    # 33. R1/AC1 -- every gate named by the immutable command requires an observation.
    obs33 = (_reason(omitted_gate), omitted_gate.field_path if isinstance(omitted_gate, Rejection) else "")
    exp33 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[32][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[32][0], "expected": exp33, "observed": obs33, "passed": obs33 == exp33})

    missing_environment = decide_verification_record(_complete_record(evidence=VerificationEvidence(commit="abc1234", environment=None, output_summary="ok", evidence_path="external-contracts/evidence/2334.json", duration_ms=1200, resource_summary="peak_rss_mb=96")))
    # 34. R1/AC1 -- evidence must record the execution environment.
    obs34 = (_reason(missing_environment), missing_environment.field_path if isinstance(missing_environment, Rejection) else "")
    exp34 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[33][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[33][0], "expected": exp34, "observed": obs34, "passed": obs34 == exp34})

    missing_output_summary = decide_verification_record(_complete_record(evidence=VerificationEvidence(commit="abc1234", environment="local-isolated", output_summary=None, evidence_path="external-contracts/evidence/2334.json", duration_ms=1200, resource_summary="peak_rss_mb=96")))
    # 35. R1/AC1 -- evidence must retain an observable output summary.
    obs35 = (_reason(missing_output_summary), missing_output_summary.field_path if isinstance(missing_output_summary, Rejection) else "")
    exp35 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[34][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[34][0], "expected": exp35, "observed": obs35, "passed": obs35 == exp35})

    missing_evidence_path = decide_verification_record(_complete_record(evidence=VerificationEvidence(commit="abc1234", environment="local-isolated", output_summary="ok", evidence_path=None, duration_ms=1200, resource_summary="peak_rss_mb=96")))
    # 36. R1/AC1 -- evidence without a retained location fails closed.
    obs36 = (_reason(missing_evidence_path), missing_evidence_path.field_path if isinstance(missing_evidence_path, Rejection) else "")
    exp36 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[35][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[35][0], "expected": exp36, "observed": obs36, "passed": obs36 == exp36})

    absent_issue = decide_terminal_result(TerminalInput(record=exact_command, classification=FailureClassification.APP_DOMAIN_ONLY, cleanup=cleanup, exact_reproduction="lumen spec gen --lang rust --out /tmp/lumen-client"))
    # 37. R3/AC3 -- app-domain-only classification alone cannot reach tracked_skip.
    obs37 = (_reason(absent_issue), absent_issue.field_path if isinstance(absent_issue, Rejection) else "")
    exp37 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[36][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[36][0], "expected": exp37, "observed": obs37, "passed": obs37 == exp37})

    nonpositive_issue = decide_terminal_result(TerminalInput(record=exact_command, classification=FailureClassification.APP_DOMAIN_ONLY, cleanup=cleanup, validated_issue_number="#0", exact_reproduction="lumen spec gen --lang rust --out /tmp/lumen-client"))
    # 38. R3/AC3 -- a non-positive issue reference cannot authorize tracked_skip.
    obs38 = (_reason(nonpositive_issue), nonpositive_issue.field_path if isinstance(nonpositive_issue, Rejection) else "")
    exp38 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[37][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[37][0], "expected": exp38, "observed": obs38, "passed": obs38 == exp38})

    absent_reproduction = decide_terminal_result(TerminalInput(record=exact_command, classification=FailureClassification.APP_DOMAIN_ONLY, cleanup=cleanup, validated_issue_number="#2340"))
    # 39. R3/AC3 -- a bounded issue needs exact reproduction before tracked_skip.
    obs39 = (_reason(absent_reproduction), absent_reproduction.field_path if isinstance(absent_reproduction, Rejection) else "")
    exp39 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[38][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[38][0], "expected": exp39, "observed": obs39, "passed": obs39 == exp39})

    missing_category_receipt = decide_cleanup_record(_complete_cleanup(receipts=(
        CleanupReceipt(category=ResourceCategory.PROCESS, success_path_complete=True, failure_path_complete=True),
        CleanupReceipt(category=ResourceCategory.EVIDENCE, success_path_complete=True, failure_path_complete=True),
    )))
    # 40. AC4 -- every declared resource category must have a receipt.
    obs40 = (_reason(missing_category_receipt), missing_category_receipt.field_path if isinstance(missing_category_receipt, Rejection) else "")
    exp40 = CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[39][1]
    checks.append({"name": CLI_SPEC_CODEGEN_2334_SECURITY_MATRIX[39][0], "expected": exp40, "observed": obs40, "passed": obs40 == exp40})

    return {
        "case_id": "cli-spec-codegen-2334-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
