"""Authored unit tests for Guard's executable Python tech design."""

from __future__ import annotations

import inspect
import json
import sys
import tempfile
import unittest
from pathlib import Path

from typer.testing import CliRunner


SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
sys.path.insert(0, str(SRC_ROOT))

from cli import app  # noqa: E402
from distribution import DistributionDesign  # noqa: E402
from evidence import (  # noqa: E402
    EvidenceCommand,
    EvidenceDesign,
    EvidenceStatus,
    run_evidence_commands,
)
from policy import PolicyDesign, PolicyProfile  # noqa: E402
from report import GuardReport, ReportDesign, ReportState  # noqa: E402
from scan import ScanOptions, scan_path  # noqa: E402
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
                getattr(module, function_name)
                for function_name in module.__aw_public_behaviors__
            ]
            self.assertTrue(behaviors)
            self.assertTrue(all(inspect.isfunction(function) for function in behaviors))
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
        self.assertEqual(
            PolicyDesign.map_severity(
                PolicyProfile.STRICT,
                policy.DiagnosticSeverity.WARNING,
            ).value,
            "high",
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
        verbs = {
            command.name
            for command in app.registered_commands
            if command.name is not None
        }
        self.assertEqual(verbs, {"scan", "report", "spec", "llm"})
        self.assertIsNotNone(app.registered_callback)

    def test_reference_cli_is_a_usable_security_product(self) -> None:
        runner = CliRunner()
        with tempfile.TemporaryDirectory(prefix="guard-td-") as temp_dir:
            root = Path(temp_dir)
            (root / "unsafe.js").write_text("eval('alert(1)');\n", encoding="utf-8")
            result = runner.invoke(
                app,
                ["scan", str(root), "--compact", "--no-persist"],
            )
        self.assertEqual(result.exit_code, 1, result.output)
        value = json.loads(result.stdout)
        self.assertEqual(value["schema_version"], "guard.report/1")
        self.assertEqual(value["status"]["state"], "findings")
        self.assertEqual(value["findings"][0]["rule"], "JS004")

    def test_reference_scanner_covers_security_profiles_and_waivers(self) -> None:
        with tempfile.TemporaryDirectory(prefix="guard-td-") as temp_dir:
            root = Path(temp_dir)
            (root / "Dockerfile").write_text("FROM ubuntu\n", encoding="utf-8")
            baseline = scan_path(root)
            lint = scan_path(
                root,
                ScanOptions(profile=PolicyProfile.SECURITY_LINT),
            )
            self.assertEqual(baseline.summary.security_findings, 0)
            self.assertEqual(lint.findings[0].rule, "DK002")

            (root / "unsafe.py").write_text(
                "subprocess.run('id', shell=True)\n",
                encoding="utf-8",
            )
            strict = scan_path(root)
            self.assertIn("PY304", {finding.rule for finding in strict.findings})
            (root / ".guard").mkdir()
            (root / ".guard" / "waivers.json").write_text(
                '{"waivers":[{"rule":"PY304","reason":"reviewed fixture"}]}',
                encoding="utf-8",
            )
            waived = scan_path(root)
            self.assertNotIn("PY304", {finding.rule for finding in waived.findings})

    def test_reference_evidence_fold_is_fail_closed(self) -> None:
        command = EvidenceCommand.shell(
            "rig",
            "exploit",
            "printf '%s' '{\"schema_version\":\"rig.report/1\",\"clean\":false,"
            "\"summary\":{\"total\":2},\"findings\":[{\"id\":\"x\"}]}'",
        )
        folded = run_evidence_commands([command])
        self.assertEqual(len(folded), 1)
        self.assertFalse(folded[0].clean)
        self.assertEqual(folded[0].finding_count, 2)
        self.assertEqual(
            folded[0].to_guard_finding(".").rule,
            "RIG-EVIDENCE",
        )

    def test_reference_evidence_rejects_missing_or_inconsistent_verdicts(self) -> None:
        commands = [
            EvidenceCommand.shell("rig", "empty", "true"),
            EvidenceCommand.shell("rig", "object", "printf '%s' '{}'"),
            EvidenceCommand.shell(
                "rig",
                "findings-only",
                "printf '%s' '{\"schema_version\":\"rig.report/1\","
                "\"findings\":[{\"id\":\"x\"}]}'",
            ),
            EvidenceCommand.shell(
                "rig",
                "contradiction",
                "printf '%s' '{\"schema_version\":\"rig.report/1\","
                "\"clean\":true,\"summary\":{\"total\":0},"
                "\"findings\":[{\"id\":\"x\"}]}'",
            ),
            EvidenceCommand.shell(
                "rig",
                "countless-clean",
                "printf '%s' '{\"schema_version\":\"rig.report/1\",\"clean\":true}'",
            ),
        ]
        folded = run_evidence_commands(commands)
        self.assertTrue(all(not item.clean for item in folded))
        self.assertEqual([item.finding_count for item in folded], [0, 0, 1, 1, 0])

    def test_reference_scanner_handles_multiline_shell_and_latest_tag(self) -> None:
        with tempfile.TemporaryDirectory(prefix="guard-td-") as temp_dir:
            root = Path(temp_dir)
            (root / "unsafe.py").write_text(
                "import subprocess\nsubprocess.run(\n    'id',\n    shell=True,\n)\n",
                encoding="utf-8",
            )
            (root / "Dockerfile").write_text("FROM ubuntu:latest\n", encoding="utf-8")
            baseline = scan_path(root)
            lint = scan_path(root, ScanOptions(profile=PolicyProfile.SECURITY_LINT))
        self.assertIn("PY304", {finding.rule for finding in baseline.findings})
        self.assertIn("DK002", {finding.rule for finding in lint.findings})
        self.assertEqual(
            baseline.findings[0].evidence["source"],
            "guard-python-reference",
        )

    def test_reference_report_persists_and_reprojects_exactly(self) -> None:
        report_value = GuardReport.stub("spec", "guard.report/1")
        with tempfile.TemporaryDirectory(prefix="guard-td-") as temp_dir:
            root = Path(temp_dir)
            report_value.persist(root)
            restored = GuardReport.read_last(root)
        self.assertEqual(restored.to_dict(), report_value.to_dict())

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
