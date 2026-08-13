"""Unit tests for #2377 operational log integration design model."""
from __future__ import annotations

import pathlib
import sys
import unittest

SRC_DIR = str(pathlib.Path(__file__).parents[2] / "src")
if SRC_DIR not in sys.path:
    sys.path.insert(0, SRC_DIR)

from lumen.operational_log_integration.admission import (
    classify_failure,
    decide_coverage,
    decide_gate_record,
    decide_mixed_failure,
    decide_terminal_result,
)
from lumen.operational_log_integration.spec import (
    Failure,
    FailureOwnership,
    GateRecord,
    TerminalResult,
)
from lumen.operational_log_integration.verdict import (
    Reason,
    Rejection,
)


class TestOperationalLogIntegration2377(unittest.TestCase):
    def test_gate_record_admission_non_contract_inputs(self) -> None:
        rec = GateRecord(
            commit="deadbeef9999",
            environment="staging-prod",
            command="cargo test -p custom",
            output_summary="99 passed",
            evidence_path="custom/path/evidence.json",
        )
        verdict = decide_gate_record(rec)
        self.assertFalse(isinstance(verdict, Rejection))
        self.assertEqual(verdict.record.commit, "deadbeef9999")
        self.assertEqual(verdict.record.environment, "staging-prod")
        self.assertEqual(verdict.record.command, "cargo test -p custom")

    def test_gate_record_refusal_field_paths(self) -> None:
        rec_no_commit = GateRecord(
            commit="",
            environment="env",
            command="cmd",
            output_summary="out",
            evidence_path="ev",
        )
        v = decide_gate_record(rec_no_commit)
        self.assertTrue(isinstance(v, Rejection))
        self.assertEqual(v.reason, Reason.MISSING_REQUIRED_EVIDENCE)
        self.assertEqual(v.field_path, "commit")

    def test_coverage_admission_custom_iterable(self) -> None:
        kinds = ["lifecycle", "security_audit", "retry_or_failure", "success", "extra_kind"]
        v = decide_coverage(kinds)
        self.assertFalse(isinstance(v, Rejection))

    def test_coverage_refusal_missing_kinds(self) -> None:
        v = decide_coverage(["success", "retry_or_failure"])
        self.assertTrue(isinstance(v, Rejection))
        self.assertEqual(v.reason, Reason.REQUIRED_EVENT_KIND_MISSING)
        self.assertEqual(v.missing_kind, "security_audit")

    def test_app_domain_terminalization_preserves_custom_issue_ref(self) -> None:
        fail = Failure(
            ownership=FailureOwnership.APP_DOMAIN_ONLY,
            issue_ref="#8123",
            exact_reproduction="cargo test custom_repro",
        )
        cls_v = classify_failure(fail)
        self.assertFalse(isinstance(cls_v, Rejection))

        term_v = decide_terminal_result(cls_v, "#8123", shared_rerun_complete=True)
        self.assertFalse(isinstance(term_v, Rejection))
        self.assertEqual(term_v.terminal, TerminalResult.TRACKED_SKIP)
        self.assertEqual(term_v.issue_ref, "#8123")

    def test_decide_terminal_result_direct_enum_none(self) -> None:
        v = decide_terminal_result(FailureOwnership.NONE, None, shared_rerun_complete=True)
        self.assertFalse(isinstance(v, Rejection))
        self.assertEqual(v.terminal, TerminalResult.PASSED)

    def test_decide_terminal_result_shared_refused(self) -> None:
        v = decide_terminal_result(FailureOwnership.SHARED_NON_DOMAIN, "#1234", shared_rerun_complete=True)
        self.assertTrue(isinstance(v, Rejection))
        self.assertEqual(v.reason, Reason.SHARED_NON_DOMAIN_FAILURE_REQUIRES_REPAIR)

    def test_decide_terminal_result_mixed_refused(self) -> None:
        v = decide_terminal_result(FailureOwnership.MIXED, "#1234", shared_rerun_complete=True)
        self.assertTrue(isinstance(v, Rejection))
        self.assertEqual(v.reason, Reason.MIXED_FAILURE_REQUIRES_SPLIT)


if __name__ == "__main__":
    unittest.main()
