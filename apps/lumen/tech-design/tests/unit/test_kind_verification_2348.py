"""Unit tests for Kind deployment verification tech design components (#2348)."""
from __future__ import annotations

import unittest

from lumen.kind_verification.admission import decide_terminal
from lumen.kind_verification.classification import partition_failures
from lumen.kind_verification.verdict import (
    Admitted,
    Failure,
    Rejection,
    RejectionReason,
    TerminalResult,
    VerificationRecord,
)


class TestKindVerification2348(unittest.TestCase):
    def test_partition_empty(self) -> None:
        partition = partition_failures(())
        self.assertEqual(partition.shared_non_domain, ())
        self.assertEqual(partition.app_domain_only, ())

    def test_partition_custom_failures_and_ordering(self) -> None:
        f_app = Failure(code="custom-app-err", ownership="APP_DOMAIN_ONLY")
        f_shared = Failure(code="custom-shared-err", ownership="SHARED_NON_DOMAIN")

        # Test reverse ordering where APP_DOMAIN_ONLY is first
        partition = partition_failures((f_app, f_shared))
        self.assertEqual(tuple(f.code for f in partition.shared_non_domain), ("custom-shared-err",))
        self.assertEqual(tuple(f.code for f in partition.app_domain_only), ("custom-app-err",))

    def test_decide_terminal_reverse_mixed_order_rejects(self) -> None:
        # Mixed failures with app domain first and shared second
        f_app = Failure(code="app-recovery-failed", ownership="APP_DOMAIN_ONLY")
        f_shared = Failure(code="node-network-down", ownership="SHARED_NON_DOMAIN")
        record = VerificationRecord(
            failures=(f_app, f_shared),
            domain_issue="#9999",
            domain_issue_validated=True,
        )
        verdict = decide_terminal(record)
        self.assertIsInstance(verdict, Rejection)
        if isinstance(verdict, Rejection):
            self.assertEqual(verdict.reason, RejectionReason.SHARED_FAILURE_CANNOT_SKIP)
            self.assertEqual(verdict.reason.value, "shared-failure-cannot-skip")
            self.assertEqual(verdict.field_path, "failures")

    def test_decide_terminal_whitespace_issue_refused(self) -> None:
        f_app = Failure(code="app-crash", ownership="APP_DOMAIN_ONLY")
        record = VerificationRecord(
            failures=(f_app,),
            domain_issue="   ",
            domain_issue_validated=True,
        )
        verdict = decide_terminal(record)
        self.assertIsInstance(verdict, Rejection)
        if isinstance(verdict, Rejection):
            self.assertEqual(verdict.reason, RejectionReason.MISSING_DOMAIN_ISSUE)
            self.assertEqual(verdict.field_path, "domain_issue")

    def test_decide_terminal_passed_with_empty_failures(self) -> None:
        record = VerificationRecord(failures=(), domain_issue="", domain_issue_validated=False)
        verdict = decide_terminal(record)
        self.assertIsInstance(verdict, Admitted)
        if isinstance(verdict, Admitted):
            self.assertEqual(verdict.result, TerminalResult.PASSED)
            self.assertEqual(verdict.result.value, "passed")

    def test_decide_terminal_tracked_skip_valid(self) -> None:
        f_app = Failure(code="app-db-sync", ownership="APP_DOMAIN_ONLY")
        record = VerificationRecord(
            failures=(f_app,),
            domain_issue="#1234",
            domain_issue_validated=True,
        )
        verdict = decide_terminal(record)
        self.assertIsInstance(verdict, Admitted)
        if isinstance(verdict, Admitted):
            self.assertEqual(verdict.result, TerminalResult.TRACKED_SKIP)
            self.assertEqual(verdict.result.value, "tracked_skip")
            self.assertEqual(verdict.issue_ref, "#1234")


if __name__ == "__main__":
    unittest.main()
