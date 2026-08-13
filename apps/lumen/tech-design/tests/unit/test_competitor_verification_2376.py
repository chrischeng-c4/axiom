"""Unit tests for competitor verification tech design deciders (#2376)."""
from __future__ import annotations

import unittest

from lumen.competitor_verification.admission import (
    decide_evidence_spec,
    decide_failure_disposition,
    decide_terminal_result,
)
from lumen.competitor_verification.spec import (
    EvidenceSpec,
    FailureDispositionRequest,
    FailureOwnership,
    IssueBacking,
    PeerDeclaration,
    TerminalResultRequest,
    WorkloadDeclaration,
)
from lumen.competitor_verification.verdict import (
    AdmittedEvidenceSpec,
    AdmittedTerminalResult,
    MixedFailureDisposition,
    Rejection,
    RejectionReason,
    SingleFailureDisposition,
)


class TestCompetitorVerification2376(unittest.TestCase):
    def test_novel_peer_and_workload_admitted(self) -> None:
        spec = EvidenceSpec(
            semantic_proof_precedes_metrics=True,
            peer=PeerDeclaration(name="elasticsearch-8", declared_appropriate=True),
            workload=WorkloadDeclaration(name="indexing-500000-records", declared_comparable=True),
            required_evidence_fields=(
                "command",
                "work_count",
                "output_summary",
                "evidence_path",
                "duration",
                "resources",
                "environment",
            ),
            metric_vocabulary=(
                "throughput",
                "latency",
                "cpu",
                "memory",
                "lifecycle_overhead",
                "cost",
                "disk_io",
            ),
            intentional_deltas=("custom_routing_semantics",),
            app_domain_delta_route="issue_backed",
        )
        verdict = decide_evidence_spec(spec)
        self.assertIsInstance(verdict, AdmittedEvidenceSpec)
        if isinstance(verdict, AdmittedEvidenceSpec):
            self.assertEqual(verdict.spec.peer.name, "elasticsearch-8")
            self.assertEqual(verdict.spec.workload.name, "indexing-500000-records")

    def test_novel_issue_backing_tracked_skip(self) -> None:
        issue = IssueBacking(
            issue_ref="#8888",
            validated=True,
            bounded=True,
            exact_reproduction="pytest tests/test_lumen_custom.py",
            authoritative_existing_wi_supplied=False,
            authoritative_existing_wi_acceptance_checked=False,
        )
        req = FailureDispositionRequest(
            ownership=FailureOwnership.APP_DOMAIN_ONLY,
            issue=issue,
        )
        disp = decide_failure_disposition(req)
        self.assertIsInstance(disp, SingleFailureDisposition)
        if isinstance(disp, SingleFailureDisposition):
            self.assertEqual(disp.action, "tracked_skip")
            self.assertEqual(disp.issue_ref, "#8888")

    def test_novel_mixed_disposition_split(self) -> None:
        issue = IssueBacking(
            issue_ref="#9999",
            validated=True,
            bounded=True,
            exact_reproduction="cargo test -p lumen --test perf_custom",
            authoritative_existing_wi_supplied=True,
            authoritative_existing_wi_acceptance_checked=True,
        )
        req = FailureDispositionRequest(
            ownership=FailureOwnership.MIXED,
            issue=issue,
        )
        disp = decide_failure_disposition(req)
        self.assertIsInstance(disp, MixedFailureDisposition)
        if isinstance(disp, MixedFailureDisposition):
            self.assertEqual(disp.shared.action, "repair_and_rerun")
            self.assertEqual(disp.app_domain.action, "tracked_skip")
            self.assertEqual(disp.app_domain.issue_ref, "#9999")

    def test_novel_terminal_result_passed(self) -> None:
        req = TerminalResultRequest(
            journey_completed=True,
            ownership=FailureOwnership.NONE,
            issue=None,
        )
        res = decide_terminal_result(req)
        self.assertIsInstance(res, AdmittedTerminalResult)
        if isinstance(res, AdmittedTerminalResult):
            self.assertEqual(res.terminal, "passed")
            self.assertEqual(res.issue_ref, "")

    def test_unbounded_issue_rejected_with_exact_field_path(self) -> None:
        issue = IssueBacking(
            issue_ref="#7777",
            validated=True,
            bounded=False,
            exact_reproduction="cargo test",
        )
        req = FailureDispositionRequest(
            ownership=FailureOwnership.APP_DOMAIN_ONLY,
            issue=issue,
        )
        res = decide_failure_disposition(req)
        self.assertIsInstance(res, Rejection)
        if isinstance(res, Rejection):
            self.assertEqual(res.reason, RejectionReason.BOUNDED_ISSUE_REQUIRED)
            self.assertEqual(res.field_path, "issue.bounded")


if __name__ == "__main__":
    unittest.main()
