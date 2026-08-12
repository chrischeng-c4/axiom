"""Unit tests for lumen.container_kustomize design components (#2343)."""

from __future__ import annotations

import unittest
from dataclasses import FrozenInstanceError

from lumen.container_kustomize.classification import (
    decide_failure_outcome,
    decide_mixed_failure,
)
from lumen.container_kustomize.result import (
    TerminalResult,
    decide_terminal_result,
)
from lumen.container_kustomize.spec import (
    Action,
    BoundedIssue,
    FailureOwner,
    TerminalState,
    extract_issue_number,
)
from lumen.container_kustomize.verdict import (
    FailureOutcome,
    MixedFailureOutcome,
    Reason,
    Rejection,
)


class TestContainerKustomizeDesign(unittest.TestCase):
    def test_bounded_issue_helper_and_properties(self) -> None:
        issue_valid = BoundedIssue(number=9999)
        self.assertTrue(issue_valid.is_bounded)
        self.assertEqual(extract_issue_number(issue_valid), 9999)

        issue_invalid = BoundedIssue(number=0)
        self.assertFalse(issue_invalid.is_bounded)
        self.assertEqual(extract_issue_number(issue_invalid), 0)

        issue_negative = BoundedIssue(number=-42)
        self.assertFalse(issue_negative.is_bounded)
        self.assertEqual(extract_issue_number(issue_negative), -42)

        self.assertEqual(extract_issue_number(8888), 8888)
        self.assertEqual(extract_issue_number(None), 0)

    def test_decide_failure_outcome_custom_values(self) -> None:
        # Shared failure requiring shared repair
        res_shared = decide_failure_outcome(FailureOwner.SHARED, BoundedIssue(number=7777))
        self.assertIsInstance(res_shared, FailureOutcome)
        self.assertEqual(res_shared.action, Action.SHARED_REPAIR_REQUIRED)

        # Non-domain failure requiring shared repair
        res_nondomain = decide_failure_outcome(FailureOwner.NON_DOMAIN, BoundedIssue(number=6666))
        self.assertIsInstance(res_nondomain, FailureOutcome)
        self.assertEqual(res_nondomain.action, Action.SHARED_REPAIR_REQUIRED)

        # App domain failure with valid bounded issue -> tracked skip
        res_domain = decide_failure_outcome(FailureOwner.APP_DOMAIN, BoundedIssue(number=5555))
        self.assertIsInstance(res_domain, FailureOutcome)
        self.assertEqual(res_domain.action, Action.TRACKED_SKIP)
        self.assertEqual(res_domain.issue_number, 5555)

        # App domain failure with integer issue -> tracked skip
        res_domain_int = decide_failure_outcome(FailureOwner.APP_DOMAIN, 4444)
        self.assertIsInstance(res_domain_int, FailureOutcome)
        self.assertEqual(res_domain_int.action, Action.TRACKED_SKIP)
        self.assertEqual(res_domain_int.issue_number, 4444)

        # App domain failure with unbounded issue -> Rejection
        res_unbounded = decide_failure_outcome(FailureOwner.APP_DOMAIN, BoundedIssue(number=0))
        self.assertIsInstance(res_unbounded, Rejection)
        self.assertEqual(res_unbounded.reason, Reason.BOUNDED_ISSUE_REQUIRED)
        self.assertEqual(res_unbounded.field_path, "bounded_issue.number")

        res_none_issue = decide_failure_outcome(FailureOwner.APP_DOMAIN, None)
        self.assertIsInstance(res_none_issue, Rejection)
        self.assertEqual(res_none_issue.reason, Reason.BOUNDED_ISSUE_REQUIRED)

    def test_decide_mixed_failure_custom_values(self) -> None:
        mixed_valid = decide_mixed_failure(
            FailureOwner.SHARED, FailureOwner.APP_DOMAIN, BoundedIssue(number=3333)
        )
        self.assertIsInstance(mixed_valid, MixedFailureOutcome)
        self.assertEqual(mixed_valid.shared.action, Action.REPAIR_AND_RERUN)
        self.assertEqual(mixed_valid.domain.action, Action.TRACKED_SKIP)
        self.assertEqual(mixed_valid.domain.issue_number, 3333)

        mixed_invalid = decide_mixed_failure(
            FailureOwner.NON_DOMAIN, FailureOwner.APP_DOMAIN, BoundedIssue(number=-1)
        )
        self.assertIsInstance(mixed_invalid, Rejection)
        self.assertEqual(mixed_invalid.reason, Reason.BOUNDED_ISSUE_REQUIRED)
        self.assertEqual(mixed_invalid.field_path, "bounded_issue.number")

    def test_decide_terminal_result_custom_values(self) -> None:
        # Successful rerun passes regardless of owner
        t_pass_shared = decide_terminal_result(FailureOwner.SHARED, BoundedIssue(number=2222), True)
        self.assertEqual(t_pass_shared.state, TerminalState.PASSED)

        t_pass_nondomain = decide_terminal_result(FailureOwner.NON_DOMAIN, BoundedIssue(number=1111), True)
        self.assertEqual(t_pass_nondomain.state, TerminalState.PASSED)

        t_pass_domain = decide_terminal_result(FailureOwner.APP_DOMAIN, BoundedIssue(number=1234), True)
        self.assertEqual(t_pass_domain.state, TerminalState.PASSED)

        # Failed rerun for shared/non-domain stays open
        t_open_shared = decide_terminal_result(FailureOwner.SHARED, BoundedIssue(number=2222), False)
        self.assertEqual(t_open_shared.state, TerminalState.OPEN)
        self.assertEqual(t_open_shared.reason, Reason.SHARED_RERUN_REQUIRED)
        self.assertEqual(t_open_shared.field_path, "shared_rerun_succeeded")

        t_open_nondomain = decide_terminal_result(FailureOwner.NON_DOMAIN, BoundedIssue(number=1111), False)
        self.assertEqual(t_open_nondomain.state, TerminalState.OPEN)
        self.assertEqual(t_open_nondomain.reason, Reason.SHARED_RERUN_REQUIRED)
        self.assertEqual(t_open_nondomain.field_path, "shared_rerun_succeeded")

        # Failed rerun for app-domain with bounded issue -> tracked skip
        t_skip_domain = decide_terminal_result(FailureOwner.APP_DOMAIN, BoundedIssue(number=9876), False)
        self.assertEqual(t_skip_domain.state, TerminalState.TRACKED_SKIP)
        self.assertEqual(t_skip_domain.issue_number, 9876)

        # Failed rerun for app-domain with unbounded issue -> open with bounded issue required
        t_open_domain = decide_terminal_result(FailureOwner.APP_DOMAIN, BoundedIssue(number=0), False)
        self.assertEqual(t_open_domain.state, TerminalState.OPEN)
        self.assertEqual(t_open_domain.reason, Reason.BOUNDED_ISSUE_REQUIRED)
        self.assertEqual(t_open_domain.field_path, "bounded_issue.number")

    def test_immutability(self) -> None:
        issue = BoundedIssue(number=100)
        with self.assertRaises(FrozenInstanceError):
            issue.number = 200  # type: ignore[misc]

        outcome = FailureOutcome(action=Action.SHARED_REPAIR_REQUIRED)
        with self.assertRaises(FrozenInstanceError):
            outcome.action = Action.TRACKED_SKIP  # type: ignore[misc]

        rejection = Rejection(reason=Reason.BOUNDED_ISSUE_REQUIRED, field_path="bounded_issue.number")
        with self.assertRaises(FrozenInstanceError):
            rejection.field_path = "other"  # type: ignore[misc]

        result = TerminalResult(state=TerminalState.PASSED)
        with self.assertRaises(FrozenInstanceError):
            result.state = TerminalState.OPEN  # type: ignore[misc]


if __name__ == "__main__":
    unittest.main()
