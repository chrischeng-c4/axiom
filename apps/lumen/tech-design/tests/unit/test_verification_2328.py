"""Unit tests for lumen.verification design module (#2328)."""
from __future__ import annotations

import unittest
from typing import Final

from lumen.verification.classification import classify_failure, resolve_ownership, split_failure
from lumen.verification.result import decide_terminal_result
from lumen.verification.verdict import (
    AppDomainSlice,
    ClassifiedFailure,
    Failure,
    Ownership,
    Reason,
    Rejection,
    SharedSlice,
    SplitFailureVerdict,
    TerminalDecision,
    TerminalResult,
    VerificationRecord,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/test-verification-2328"


class TestVerification2328(unittest.TestCase):
    def test_ownership_resolution(self) -> None:
        self.assertEqual(resolve_ownership(Ownership.APP_DOMAIN), Ownership.APP_DOMAIN)
        self.assertEqual(resolve_ownership("shared"), Ownership.SHARED)
        self.assertEqual(resolve_ownership("non_domain"), Ownership.NON_DOMAIN)
        self.assertEqual(resolve_ownership("mixed"), Ownership.MIXED)
        self.assertIsNone(resolve_ownership("unsupported_value"))
        self.assertIsNone(resolve_ownership(123))  # type: ignore[arg-type]

    def test_classify_failure_valid(self) -> None:
        f = Failure(
            ownership="app_domain",
            summary="Custom routing failure",
            bounded_issue="#5555",
        )
        res = classify_failure(f)
        self.assertIsInstance(res, ClassifiedFailure)
        assert isinstance(res, ClassifiedFailure)
        self.assertEqual(res.ownership, Ownership.APP_DOMAIN)
        self.assertEqual(res.bounded_issue, "#5555")

    def test_classify_failure_unknown_ownership(self) -> None:
        f = Failure(ownership="invalid_owner", summary="Unknown failure")
        res = classify_failure(f)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, Reason.UNKNOWN_OWNERSHIP)
        self.assertEqual(res.field_path, "ownership")

    def test_split_failure_mixed(self) -> None:
        f = Failure(
            ownership=Ownership.MIXED,
            summary="Combined failure",
            shared_summary="Shared part failed",
            app_domain_summary="App part failed",
            bounded_issue="#7777",
        )
        res = split_failure(f)
        self.assertIsInstance(res, SplitFailureVerdict)
        assert isinstance(res, SplitFailureVerdict)
        self.assertEqual(res.shared_slice.ownership, Ownership.SHARED)
        self.assertEqual(res.shared_slice.disposition, "rerun_required")
        self.assertEqual(res.shared_slice.summary, "Shared part failed")
        self.assertEqual(res.app_domain_slice.ownership, Ownership.APP_DOMAIN)
        self.assertEqual(res.app_domain_slice.issue_ref, "#7777")
        self.assertEqual(res.app_domain_slice.disposition, "tracked_skip(#7777)")

    def test_split_failure_non_mixed_returns_rejection(self) -> None:
        f = Failure(ownership=Ownership.SHARED, summary="Shared failure")
        res = split_failure(f)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.field_path, "ownership")

    def test_decide_terminal_result_passed(self) -> None:
        record = VerificationRecord(
            gate_exit_code=0,
            applicable_work_count=10,
            commit="f00ba11",
            environment="staging-test",
            command="cargo test -p lumen",
            output_summary="10 passed",
            evidence_path="evidence/test.json",
            failure=None,
            terminal_intent="passed",
        )
        res = decide_terminal_result(record)
        self.assertIsInstance(res, TerminalDecision)
        assert isinstance(res, TerminalDecision)
        self.assertEqual(res.terminal, TerminalResult.PASSED)
        self.assertIsNone(res.issue_ref)

    def test_decide_terminal_result_tracked_skip(self) -> None:
        app_failure = Failure(
            ownership=Ownership.APP_DOMAIN,
            summary="App failure",
            bounded_issue="#8888",
        )
        record = VerificationRecord(
            gate_exit_code=0,
            applicable_work_count=5,
            commit="f00ba11",
            environment="staging-test",
            command="cargo test -p lumen",
            output_summary="5 passed, 1 skipped",
            evidence_path="evidence/test.json",
            failure=app_failure,
            terminal_intent="tracked_skip",
        )
        res = decide_terminal_result(record)
        self.assertIsInstance(res, TerminalDecision)
        assert isinstance(res, TerminalDecision)
        self.assertEqual(res.terminal, TerminalResult.TRACKED_SKIP)
        self.assertEqual(res.issue_ref, "#8888")

    def test_decide_terminal_result_negative_work_count(self) -> None:
        record = VerificationRecord(
            gate_exit_code=0,
            applicable_work_count=-1,
            commit="f00ba11",
            environment="staging-test",
            command="cargo test -p lumen",
            output_summary="no work",
            evidence_path="evidence/test.json",
        )
        res = decide_terminal_result(record)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, Reason.NO_APPLICABLE_WORK)
        self.assertEqual(res.field_path, "applicable_work_count")


if __name__ == "__main__":
    unittest.main()
