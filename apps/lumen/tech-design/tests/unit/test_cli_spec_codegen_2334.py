"""Unit tests for lumen.cli_spec_codegen verification design model (#2334)."""
from __future__ import annotations

import unittest
from typing import Final

from lumen.cli_spec_codegen.admission import (
    REQUIRED_CARGO_COMMAND,
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
from lumen.cli_spec_codegen.verdict import (
    AdmittedCleanupRecord,
    AdmittedVerificationRecord,
    Open,
    Passed,
    Rejection,
    RejectionReason,
    TrackedSkip,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/test-cli-spec-codegen-2334"


def _make_valid_record(**overrides) -> VerificationRecord:
    defaults = {
        "command": REQUIRED_CARGO_COMMAND,
        "gates": (
            GateObservation(name="cli_convention", exit_code=0, work_count=3),
            GateObservation(name="spec_cli", exit_code=0, work_count=8),
            GateObservation(name="spec_gen_e2e", exit_code=0, work_count=4),
            GateObservation(name="generated_clients_crud_e2e", exit_code=0, work_count=5),
        ),
        "grammar_observed": True,
        "spec_formats_observed": True,
        "client_languages": ("ts", "py", "rust", "go"),
        "deployment_renderers_observed": True,
        "cold_regeneration_observed": True,
        "executable_stdout_observed": True,
        "deterministic_generation_observed": True,
        "no_todo_or_invalid_shell_scaffold": True,
        "generated_test_work_count": 15,
        "evidence": VerificationEvidence(
            commit="def4567",
            environment="ci-runner-99",
            output_summary="Custom run summary",
            evidence_path="evidence/custom_run.json",
            duration_ms=950,
            resource_summary="rss=64MB",
        ),
    }
    defaults.update(overrides)
    return VerificationRecord(**defaults)


def _make_valid_cleanup(**overrides) -> CleanupRecord:
    defaults = {
        "receipts": (
            CleanupReceipt(category=ResourceCategory.PROCESS, success_path_complete=True, failure_path_complete=True),
            CleanupReceipt(category=ResourceCategory.NAMESPACE, success_path_complete=True, failure_path_complete=True),
            CleanupReceipt(category=ResourceCategory.EVIDENCE, success_path_complete=True, failure_path_complete=True),
        )
    }
    defaults.update(overrides)
    return CleanupRecord(**defaults)


class TestCliSpecCodegen2334(unittest.TestCase):
    def test_valid_record_admission(self) -> None:
        rec = _make_valid_record()
        res = decide_verification_record(rec)
        self.assertIsInstance(res, AdmittedVerificationRecord)
        assert isinstance(res, AdmittedVerificationRecord)
        self.assertEqual(res.record.command, REQUIRED_CARGO_COMMAND)
        self.assertEqual(res.record.generated_test_work_count, 15)

    def test_custom_command_mismatch(self) -> None:
        rec = _make_valid_record(command="cargo test -p lumen --test wrong_gate")
        res = decide_verification_record(rec)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, RejectionReason.REQUIRED_COMMAND_MISMATCH)
        self.assertEqual(res.field_path, "command")

    def test_omitted_required_gate(self) -> None:
        rec = _make_valid_record(gates=(
            GateObservation(name="cli_convention", exit_code=0, work_count=3),
            GateObservation(name="spec_cli", exit_code=0, work_count=8),
        ))
        res = decide_verification_record(rec)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, RejectionReason.REQUIRED_GATE_MISSING)
        self.assertEqual(res.field_path, "gates")

    def test_gate_exit_code_nonzero(self) -> None:
        rec = _make_valid_record(gates=(
            GateObservation(name="cli_convention", exit_code=0, work_count=3),
            GateObservation(name="spec_cli", exit_code=101, work_count=8),
            GateObservation(name="spec_gen_e2e", exit_code=0, work_count=4),
            GateObservation(name="generated_clients_crud_e2e", exit_code=0, work_count=5),
        ))
        res = decide_verification_record(rec)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, RejectionReason.GATE_EXIT_NONZERO)
        self.assertEqual(res.field_path, "gates")

    def test_gate_work_count_negative_or_zero(self) -> None:
        rec = _make_valid_record(gates=(
            GateObservation(name="cli_convention", exit_code=0, work_count=0),
            GateObservation(name="spec_cli", exit_code=0, work_count=8),
            GateObservation(name="spec_gen_e2e", exit_code=0, work_count=4),
            GateObservation(name="generated_clients_crud_e2e", exit_code=0, work_count=5),
        ))
        res = decide_verification_record(rec)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, RejectionReason.GATE_WORK_ZERO)
        self.assertEqual(res.field_path, "gates")

    def test_missing_evidence_fields(self) -> None:
        env_missing = _make_valid_record(evidence=VerificationEvidence(commit="abc", environment=None, output_summary="ok", evidence_path="path"))
        r1 = decide_verification_record(env_missing)
        self.assertIsInstance(r1, Rejection)
        assert isinstance(r1, Rejection)
        self.assertEqual(r1.reason, RejectionReason.MISSING_ENVIRONMENT)
        self.assertEqual(r1.field_path, "evidence.environment")

        out_missing = _make_valid_record(evidence=VerificationEvidence(commit="abc", environment="local", output_summary=None, evidence_path="path"))
        r2 = decide_verification_record(out_missing)
        self.assertIsInstance(r2, Rejection)
        assert isinstance(r2, Rejection)
        self.assertEqual(r2.reason, RejectionReason.MISSING_OUTPUT_SUMMARY)
        self.assertEqual(r2.field_path, "evidence.output_summary")

        path_missing = _make_valid_record(evidence=VerificationEvidence(commit="abc", environment="local", output_summary="ok", evidence_path=None))
        r3 = decide_verification_record(path_missing)
        self.assertIsInstance(r3, Rejection)
        assert isinstance(r3, Rejection)
        self.assertEqual(r3.reason, RejectionReason.MISSING_EVIDENCE_PATH)
        self.assertEqual(r3.field_path, "evidence.evidence_path")

    def test_incomplete_client_languages(self) -> None:
        rec = _make_valid_record(client_languages=("ts", "cpp"))
        res = decide_verification_record(rec)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, RejectionReason.CLIENT_LANGUAGES_INCOMPLETE)
        self.assertEqual(res.field_path, "client_languages")

    def test_valid_cleanup_admission(self) -> None:
        cleanup = _make_valid_cleanup()
        res = decide_cleanup_record(cleanup)
        self.assertIsInstance(res, AdmittedCleanupRecord)
        assert isinstance(res, AdmittedCleanupRecord)
        self.assertEqual(len(res.record.receipts), 3)

    def test_cleanup_incomplete_receipt(self) -> None:
        cleanup = _make_valid_cleanup(receipts=(
            CleanupReceipt(category=ResourceCategory.PROCESS, success_path_complete=True, failure_path_complete=True),
            CleanupReceipt(category=ResourceCategory.NAMESPACE, success_path_complete=True, failure_path_complete=False),
            CleanupReceipt(category=ResourceCategory.EVIDENCE, success_path_complete=True, failure_path_complete=True),
        ))
        res = decide_cleanup_record(cleanup)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, RejectionReason.CLEANUP_INCOMPLETE)
        self.assertEqual(res.field_path, "namespace")

    def test_cleanup_missing_declared_category(self) -> None:
        cleanup = _make_valid_cleanup(receipts=(
            CleanupReceipt(category=ResourceCategory.PROCESS, success_path_complete=True, failure_path_complete=True),
            CleanupReceipt(category=ResourceCategory.EVIDENCE, success_path_complete=True, failure_path_complete=True),
        ))
        res = decide_cleanup_record(cleanup)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, RejectionReason.CLEANUP_INCOMPLETE)
        self.assertEqual(res.field_path, "namespace")

    def test_terminal_result_all_green_passes(self) -> None:
        rec = _make_valid_record()
        cleanup = _make_valid_cleanup()
        inp = TerminalInput(record=rec, classification=FailureClassification.ALL_GREEN, cleanup=cleanup)
        res = decide_terminal_result(inp)
        self.assertIsInstance(res, Passed)
        assert isinstance(res, Passed)
        self.assertEqual(res.record.command, REQUIRED_CARGO_COMMAND)

    def test_terminal_result_tracked_skip_valid(self) -> None:
        rec = _make_valid_record()
        cleanup = _make_valid_cleanup()
        inp = TerminalInput(
            record=rec,
            classification=FailureClassification.APP_DOMAIN_ONLY,
            cleanup=cleanup,
            validated_issue_number="#9876",
            exact_reproduction="lumen execute --repro=test",
        )
        res = decide_terminal_result(inp)
        self.assertIsInstance(res, TrackedSkip)
        assert isinstance(res, TrackedSkip)
        self.assertEqual(res.issue_ref, "#9876")
        self.assertEqual(res.reproduction, "lumen execute --repro=test")

    def test_terminal_result_tracked_skip_missing_or_nonpositive_issue(self) -> None:
        rec = _make_valid_record()
        cleanup = _make_valid_cleanup()
        
        # Missing issue
        inp_missing = TerminalInput(
            record=rec,
            classification=FailureClassification.APP_DOMAIN_ONLY,
            cleanup=cleanup,
            validated_issue_number=None,
            exact_reproduction="repro string",
        )
        res_m = decide_terminal_result(inp_missing)
        self.assertIsInstance(res_m, Rejection)
        assert isinstance(res_m, Rejection)
        self.assertEqual(res_m.reason, RejectionReason.VALIDATED_ISSUE_NUMBER_MISSING)
        self.assertEqual(res_m.field_path, "validated_issue_number")

        # Nonpositive / invalid issue
        for bad_issue in ["9876", "#0", "#abc", ""]:
            inp = TerminalInput(
                record=rec,
                classification=FailureClassification.APP_DOMAIN_ONLY,
                cleanup=cleanup,
                validated_issue_number=bad_issue,
                exact_reproduction="repro string",
            )
            res = decide_terminal_result(inp)
            self.assertIsInstance(res, Rejection)
            assert isinstance(res, Rejection)
            self.assertEqual(res.reason, RejectionReason.VALIDATED_ISSUE_NUMBER_NONPOSITIVE)
            self.assertEqual(res.field_path, "validated_issue_number")

    def test_terminal_result_tracked_skip_missing_repro(self) -> None:
        rec = _make_valid_record()
        cleanup = _make_valid_cleanup()
        inp = TerminalInput(
            record=rec,
            classification=FailureClassification.APP_DOMAIN_ONLY,
            cleanup=cleanup,
            validated_issue_number="#9876",
            exact_reproduction=None,
        )
        res = decide_terminal_result(inp)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, RejectionReason.EXACT_REPRODUCTION_MISSING)
        self.assertEqual(res.field_path, "exact_reproduction")

    def test_terminal_result_open_classifications(self) -> None:
        rec = _make_valid_record()
        cleanup = _make_valid_cleanup()
        for classification in [FailureClassification.SHARED, FailureClassification.NON_DOMAIN, FailureClassification.MIXED]:
            inp = TerminalInput(record=rec, classification=classification, cleanup=cleanup)
            res = decide_terminal_result(inp)
            self.assertIsInstance(res, Open)
            assert isinstance(res, Open)
            self.assertEqual(res.classification, classification)


if __name__ == "__main__":
    unittest.main()
