"""Unit tests for lumen.verification design module (#2337)."""
from __future__ import annotations

import unittest
from typing import Final

from lumen.verification.classification import classify_failure, split_failure
from lumen.verification.verdict import (
    ClassifiedFailure,
    Disposition,
    Failure,
    Reason,
    Rejection,
    SplitFailureVerdict,
    TerminalResult,
    TerminalVerdict,
    decide_terminal_result,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/test-verification-2337"


class TestVerification2337(unittest.TestCase):
    def test_classify_shared_failure(self) -> None:
        failure = Failure(failure_id="custom-h2c-drop", owner="shared", summary="H2C transport dropped")
        res = classify_failure(failure)
        self.assertIsInstance(res, ClassifiedFailure)
        assert isinstance(res, ClassifiedFailure)
        self.assertEqual(res.disposition, Disposition.SHARED_REPAIR_REQUIRED)
        self.assertEqual(res.disposition.value, "shared_repair_required")
        self.assertEqual(res.failure_id, "custom-h2c-drop")

    def test_classify_app_domain_failure(self) -> None:
        failure = Failure(failure_id="custom-otlp-span-miss", owner="app_domain", summary="Span missing")
        res = classify_failure(failure)
        self.assertIsInstance(res, ClassifiedFailure)
        assert isinstance(res, ClassifiedFailure)
        self.assertEqual(res.disposition, Disposition.APP_DOMAIN_TRACKABLE)
        self.assertEqual(res.disposition.value, "app_domain_trackable")
        self.assertEqual(res.failure_id, "custom-otlp-span-miss")

    def test_classify_unknown_owner_refusal(self) -> None:
        failure = Failure(failure_id="external-infra-down", owner="unmanaged_cloud", summary="Cloud failure")
        res = classify_failure(failure)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, Reason.UNKNOWN_FAILURE_OWNER)
        self.assertEqual(res.reason.value, "unknown_failure_owner")
        self.assertEqual(res.field_path, "owner")

    def test_split_failure_multi_item_sequence(self) -> None:
        s1 = Failure(failure_id="shared-audit-log", owner="shared")
        s2 = Failure(failure_id="shared-tls-cert", owner="non_domain")
        a1 = Failure(failure_id="lumen-admission-rule", owner="app_domain")

        res = split_failure((s1, a1, s2))
        self.assertIsInstance(res, SplitFailureVerdict)
        assert isinstance(res, SplitFailureVerdict)
        self.assertEqual(tuple(f.failure_id for f in res.shared_failures), ("shared-audit-log", "shared-tls-cert"))
        self.assertEqual(tuple(f.failure_id for f in res.app_domain_failures), ("lumen-admission-rule",))

    def test_decide_terminal_passed(self) -> None:
        res = decide_terminal_result((), (), rerun_complete=True)
        self.assertIsInstance(res, TerminalVerdict)
        assert isinstance(res, TerminalVerdict)
        self.assertEqual(res.result, TerminalResult.PASSED)
        self.assertEqual(res.result.value, "passed")

    def test_decide_terminal_tracked_skip(self) -> None:
        app_cls = classify_failure(Failure(failure_id="lumen-admission-rule", owner="app_domain"))
        res = decide_terminal_result((app_cls,), ("#9999",), rerun_complete=True)
        self.assertIsInstance(res, TerminalVerdict)
        assert isinstance(res, TerminalVerdict)
        self.assertEqual(res.result, TerminalResult.TRACKED_SKIP)
        self.assertEqual(res.result.value, "tracked_skip")
        self.assertEqual(res.issue_ref, "#9999")

    def test_decide_terminal_app_domain_first_mixed_refused(self) -> None:
        app_cls = classify_failure(Failure(failure_id="lumen-admission-rule", owner="app_domain"))
        shared_cls = classify_failure(Failure(failure_id="shared-audit-log", owner="shared"))

        # App-domain failure is first, shared failure is second
        res = decide_terminal_result((app_cls, shared_cls), ("#9999",), rerun_complete=True)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, Reason.SHARED_REPAIR_REQUIRED)
        self.assertEqual(res.reason.value, "shared_repair_required")
        self.assertEqual(res.field_path, "classifications")

    def test_decide_terminal_incomplete_rerun(self) -> None:
        app_cls = classify_failure(Failure(failure_id="lumen-admission-rule", owner="app_domain"))
        res = decide_terminal_result((app_cls,), ("#9999",), rerun_complete=False)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, Reason.RERUN_INCOMPLETE)
        self.assertEqual(res.reason.value, "rerun_incomplete")
        self.assertEqual(res.field_path, "rerun_complete")

    def test_decide_terminal_multiple_issue_refs(self) -> None:
        app_cls = classify_failure(Failure(failure_id="lumen-admission-rule", owner="app_domain"))
        res = decide_terminal_result((app_cls,), ("#9999", "#8888"), rerun_complete=True)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, Reason.EXACTLY_ONE_ISSUE_REFERENCE)
        self.assertEqual(res.reason.value, "exactly_one_issue_reference")
        self.assertEqual(res.field_path, "issue_refs")

    def test_decide_terminal_missing_issue_ref(self) -> None:
        app_cls = classify_failure(Failure(failure_id="lumen-admission-rule", owner="app_domain"))
        res = decide_terminal_result((app_cls,), (), rerun_complete=True)
        self.assertIsInstance(res, Rejection)
        assert isinstance(res, Rejection)
        self.assertEqual(res.reason, Reason.EXACTLY_ONE_ISSUE_REFERENCE)
        self.assertEqual(res.reason.value, "exactly_one_issue_reference")


if __name__ == "__main__":
    unittest.main()
