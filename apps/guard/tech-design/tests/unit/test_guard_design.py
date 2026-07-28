"""Authored unit tests for Guard's executable Python tech design."""

from __future__ import annotations

import inspect
import sys
import unittest
from pathlib import Path


SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
sys.path.insert(0, str(SRC_ROOT))

from guard.design.cli_api import guard_cli_verbs  # noqa: E402
from guard.design.cli_dispatch import adapter_invocation  # noqa: E402
from guard.design.external_evidence import (  # noqa: E402
    EvidenceStatus,
    normalize_external_result,
)
from guard.design.external_security_contracts import required_external_use_cases  # noqa: E402
from guard.design.report_model import (  # noqa: E402
    ReportState,
    reduce_report_state,
)
from guard.design.scan_policy import PolicyProfile, included_rule  # noqa: E402
import guard.public_contracts.dynamic_security_evidence as dynamic_security_evidence  # noqa: E402
import guard.public_contracts.security_ec_profile as security_ec_profile  # noqa: E402
import guard.public_contracts.security_policy_profile as security_policy_profile  # noqa: E402
import guard.public_contracts.static_security_scan as static_security_scan  # noqa: E402


PUBLIC_CONTRACTS = (
    static_security_scan,
    security_policy_profile,
    security_ec_profile,
    dynamic_security_evidence,
)


class GuardDesignTest(unittest.TestCase):
    def test_public_artifacts_are_unique_and_all_behaviors_execute(self) -> None:
        artifact_ids = {module.__aw_artifact_id__ for module in PUBLIC_CONTRACTS}
        self.assertEqual(len(artifact_ids), 4)
        for module in PUBLIC_CONTRACTS:
            self.assertIs(module.__aw_public_contract__, True)
            behaviors = [
                function
                for _, function in inspect.getmembers(module, inspect.isfunction)
                if function.__module__ == module.__name__
            ]
            self.assertTrue(behaviors)
            self.assertTrue(all(function() for function in behaviors))

    def test_report_reducer_is_fail_closed(self) -> None:
        self.assertEqual(reduce_report_state(0, None).state, ReportState.CLEAN)
        self.assertEqual(reduce_report_state(1, None).state, ReportState.FINDINGS)
        decision = reduce_report_state(0, 5)
        self.assertEqual(decision.state, ReportState.TOOL_ERROR)
        self.assertEqual(decision.exit_code, 5)
        self.assertFalse(decision.completion_clean)

    def test_security_lint_adds_only_security_impacting_rules(self) -> None:
        self.assertFalse(included_rule(PolicyProfile.BASELINE_STATIC, "lint", "DK002"))
        self.assertTrue(included_rule(PolicyProfile.SECURITY_LINT, "lint", "DK002"))
        self.assertFalse(included_rule(PolicyProfile.SECURITY_LINT, "lint", "STYLE001"))

    def test_adapter_argv_is_explicit(self) -> None:
        self.assertEqual(
            adapter_invocation("vat", "smoke").leading_arguments,
            ("run", "--json", "smoke"),
        )
        self.assertIn("--scenario", adapter_invocation("rig", "case.toml").leading_arguments)
        self.assertIn("--skip-profile", adapter_invocation("meter", ".").leading_arguments)
        with self.assertRaises(ValueError):
            adapter_invocation("arena", "legacy")

    def test_failed_external_report_cannot_normalize_clean(self) -> None:
        decision = normalize_external_result(True, False, 1)
        self.assertEqual(decision.status, EvidenceStatus.FINDINGS)
        self.assertFalse(decision.clean)

    def test_cli_contract_has_one_json_projection_per_verb(self) -> None:
        verbs = guard_cli_verbs()
        self.assertEqual({verb.name for verb in verbs}, {"scan", "report", "spec", "llm"})
        self.assertTrue(all(verb.emits_json for verb in verbs))

    def test_every_security_tool_artifact_has_all_required_dimensions(self) -> None:
        dimensions_by_capability: dict[str, set[str]] = {}
        for use_case in required_external_use_cases():
            dimensions_by_capability.setdefault(use_case.capability_id, set()).add(
                use_case.dimension
            )
        self.assertEqual(
            dimensions_by_capability,
            {
                "security-ec-profile": {"behavior", "security", "stability"},
                "security-policy-profile": {"behavior", "security", "stability"},
                "static-security-scan": {"behavior", "security", "stability"},
                "dynamic-security-evidence": {"behavior", "security", "stability"},
            },
        )


if __name__ == "__main__":
    unittest.main()
