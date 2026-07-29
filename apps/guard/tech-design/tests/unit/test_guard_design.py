"""Authored unit tests for Guard's executable Python tech design."""

from __future__ import annotations

import inspect
import sys
import unittest
from pathlib import Path


SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
sys.path.insert(0, str(SRC_ROOT))

from cli import CliDesign  # noqa: E402
from distribution import DistributionDesign  # noqa: E402
from evidence import EvidenceDesign, EvidenceStatus  # noqa: E402
from policy import PolicyDesign, PolicyProfile  # noqa: E402
from report import ReportDesign, ReportState  # noqa: E402
import evidence  # noqa: E402
import policy  # noqa: E402
import report  # noqa: E402
import scan  # noqa: E402


PUBLIC_CONTRACTS = (
    scan,
    policy,
    report,
    evidence,
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
        self.assertEqual(ReportDesign.reduce_state(0, None).state, ReportState.CLEAN)
        self.assertEqual(ReportDesign.reduce_state(1, None).state, ReportState.FINDINGS)
        decision = ReportDesign.reduce_state(0, 5)
        self.assertEqual(decision.state, ReportState.TOOL_ERROR)
        self.assertEqual(decision.exit_code, 5)
        self.assertFalse(decision.completion_clean)

    def test_security_lint_adds_only_security_impacting_rules(self) -> None:
        self.assertFalse(
            PolicyDesign.included_rule(PolicyProfile.BASELINE_STATIC, "lint", "DK002")
        )
        self.assertTrue(
            PolicyDesign.included_rule(PolicyProfile.SECURITY_LINT, "lint", "DK002")
        )
        self.assertFalse(
            PolicyDesign.included_rule(PolicyProfile.SECURITY_LINT, "lint", "STYLE001")
        )

    def test_adapter_argv_is_explicit(self) -> None:
        self.assertEqual(
            EvidenceDesign.adapter_invocation("vat", "smoke").leading_arguments,
            ("run", "--json", "smoke"),
        )
        self.assertIn(
            "--scenario",
            EvidenceDesign.adapter_invocation("rig", "case.toml").leading_arguments,
        )
        self.assertIn(
            "--skip-profile",
            EvidenceDesign.adapter_invocation("meter", ".").leading_arguments,
        )
        with self.assertRaises(ValueError):
            EvidenceDesign.adapter_invocation("arena", "legacy")

    def test_failed_external_report_cannot_normalize_clean(self) -> None:
        decision = EvidenceDesign.normalize_result(True, False, 1)
        self.assertEqual(decision.status, EvidenceStatus.FINDINGS)
        self.assertFalse(decision.clean)

    def test_cli_contract_has_one_json_projection_per_verb(self) -> None:
        verbs = CliDesign.verbs()
        self.assertEqual({verb.name for verb in verbs}, {"scan", "report", "spec", "llm"})
        self.assertTrue(all(verb.emits_json for verb in verbs))

    def test_every_security_tool_artifact_has_all_required_dimensions(self) -> None:
        dimensions_by_capability: dict[str, set[str]] = {}
        for use_case in DistributionDesign.required_external_use_cases():
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

    def test_distribution_is_one_standalone_package(self) -> None:
        layout = DistributionDesign.project_layout()
        self.assertEqual(layout.cargo_package, "guard")
        self.assertEqual(layout.binary_target, "guard")
        for profile in DistributionDesign.build_profiles():
            self.assertNotIn("guard-cli", profile.cargo_arguments)
            self.assertIn("guard", profile.cargo_arguments)

    def test_ec_td_and_codebase_share_one_shallow_domain_map(self) -> None:
        slices = DistributionDesign.domain_artifact_slices()
        self.assertEqual(
            tuple(slice_.domain for slice_ in slices),
            ("scan", "policy", "report", "evidence", "distribution"),
        )
        self.assertTrue(
            all(
                Path(slice_.external_contract_root).parts
                == ("src", slice_.domain)
                for slice_ in slices
            )
        )
        self.assertTrue(
            all(
                Path(module).parent == Path("src")
                for slice_ in slices
                for module in slice_.tech_design_modules
            )
        )
        self.assertEqual(
            next(
                slice_.codebase_artifacts
                for slice_ in slices
                if slice_.domain == "policy"
            ),
            ("src/policy.rs",),
        )
        distribution = next(
            slice_ for slice_ in slices if slice_.domain == "distribution"
        )
        self.assertEqual(
            distribution.tech_design_modules,
            ("src/distribution.py", "src/cli.py"),
        )
        self.assertIn("src/main.rs", distribution.codebase_artifacts)
        self.assertIn("src/cli.rs", distribution.codebase_artifacts)


if __name__ == "__main__":
    unittest.main()
