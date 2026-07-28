"""Mutation-oriented unit tests for the Guard Python EC oracle."""

from __future__ import annotations

import json
import sys
import subprocess
import tempfile
import unittest
from pathlib import Path


SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
sys.path.insert(0, str(SRC_ROOT))

from guard_contract import (  # noqa: E402
    AdapterExpectation,
    _write_stub,
    assert_dynamic_evidence,
    assert_report_shape,
    assert_scan_consistency,
    parse_json_stdout,
)


def clean_report() -> dict[str, object]:
    return {
        "schema_version": "guard.report/1",
        "tool_version": "0.1.0",
        "verb": "scan",
        "target": "fixture",
        "policy_profile": "guard-baseline-static/1",
        "status": {"state": "clean"},
        "exit_code": 0,
        "summary": {
            "security_findings": 0,
            "evidence_count": 0,
            "evidence_failed": 0,
        },
        "findings": [],
        "completion": {"clean": True, "criteria": [], "missing": []},
        "integrations": {"static_engine": "compass"},
        "agent_prompt": "guard scan is clean",
    }


def clean_dynamic_report() -> tuple[
    dict[str, object],
    dict[str, list[list[str]]],
    dict[str, AdapterExpectation],
]:
    return dynamic_report()


def dynamic_report(
    *,
    report_clean: bool = True,
    adapter_exit: int = 0,
    finding_count: int = 0,
) -> tuple[
    dict[str, object],
    dict[str, list[list[str]]],
    dict[str, AdapterExpectation],
]:
    trace = ["run", "--scenario", "case.toml", "--compact"]
    report = clean_report()
    folded_clean = report_clean and adapter_exit == 0
    findings_preview = [
        {"id": f"rig-finding-{index + 1}"}
        for index in range(finding_count)
    ]
    if not folded_clean:
        report["exit_code"] = 1
        report["status"] = {"state": "findings"}
        report["summary"]["security_findings"] = 1
        report["findings"] = [{"rule": "RIG-EVIDENCE"}]
        report["completion"]["clean"] = False
    report["summary"]["evidence_count"] = 1
    report["summary"]["evidence_failed"] = 0 if folded_clean else 1
    report["evidence"] = [
        {
            "tool": "rig",
            "label": "case.toml",
            "command": ["/tmp/bin/rig", *trace],
            "status": "clean" if folded_clean else "findings",
            "clean": folded_clean,
            "exit_code": adapter_exit,
            "finding_count": finding_count,
            "report": {
                "schema_version": "rig.report/1",
                "clean": report_clean,
                "summary": {"total": finding_count},
                "findings_preview": findings_preview,
            },
        }
    ]
    return (
        report,
        {"rig": [trace]},
        {
            "rig": AdapterExpectation(
                label="case.toml",
                argv=tuple(trace),
                report_clean=report_clean,
                exit_code=adapter_exit,
                finding_count=finding_count,
                findings_preview=tuple(findings_preview),
            )
        },
    )


class GuardContractTest(unittest.TestCase):
    def test_accepts_canonical_clean_report(self) -> None:
        assertions = assert_scan_consistency(clean_report())
        self.assertIn("status, exit_code, and completion.clean agree", assertions)

    def test_rejects_schema_drift(self) -> None:
        report = clean_report()
        report["schema_version"] = "guard.report/2"
        with self.assertRaises(AssertionError):
            assert_report_shape(report, verb="scan")

    def test_rejects_summary_list_divergence(self) -> None:
        report = clean_report()
        report["summary"]["security_findings"] = 1
        with self.assertRaises(AssertionError):
            assert_scan_consistency(report)

    def test_rejects_false_clean_completion(self) -> None:
        report = clean_report()
        report["completion"]["clean"] = False
        with self.assertRaises(AssertionError):
            assert_scan_consistency(report)

    def test_rejects_missing_dynamic_tool(self) -> None:
        report = clean_report()
        report["evidence"] = []
        with self.assertRaises(AssertionError):
            assert_dynamic_evidence(
                report,
                {"vat": ["run", "--json", "guard-security-smoke"]},
                expectations={
                    "vat": AdapterExpectation(
                        label="guard-security-smoke",
                        argv=("run", "--json", "guard-security-smoke"),
                        report_clean=True,
                        exit_code=0,
                        finding_count=0,
                        findings_preview=(),
                    )
                },
            )

    def test_accepts_canonical_dynamic_baseline(self) -> None:
        report, traces, expectations = clean_dynamic_report()
        assertions = assert_dynamic_evidence(
            report,
            traces,
            expectations=expectations,
        )
        self.assertIn(
            "folded adapter reports preserve schema, independent outcomes, and findings",
            assertions,
        )

    def test_rejects_dynamic_command_that_differs_from_executed_argv(self) -> None:
        report, traces, expectations = clean_dynamic_report()
        report["evidence"][0]["command"] = ["/tmp/bin/rig", "wrong"]
        with self.assertRaises(AssertionError):
            assert_dynamic_evidence(report, traces, expectations=expectations)

    def test_rejects_corrupted_folded_dynamic_report(self) -> None:
        report, traces, expectations = clean_dynamic_report()
        report["evidence"][0]["report"] = {"wrong": True}
        with self.assertRaises(AssertionError):
            assert_dynamic_evidence(report, traces, expectations=expectations)

    def test_rejects_dynamic_label_that_differs_from_executed_argv(self) -> None:
        report, traces, expectations = clean_dynamic_report()
        report["evidence"][0]["label"] = "different.toml"
        with self.assertRaises(AssertionError):
            assert_dynamic_evidence(report, traces, expectations=expectations)

    def test_rejects_duplicate_dynamic_evidence(self) -> None:
        report, traces, expectations = clean_dynamic_report()
        report["evidence"].append(dict(report["evidence"][0]))
        with self.assertRaises(AssertionError):
            assert_dynamic_evidence(report, traces, expectations=expectations)

    def test_rejects_jointly_corrupted_trace_and_folded_route(self) -> None:
        report, traces, expectations = clean_dynamic_report()
        wrong = ["run", "--scenario", "wrong.toml", "--compact"]
        traces["rig"] = [wrong]
        report["evidence"][0]["command"] = ["/tmp/bin/rig", *wrong]
        report["evidence"][0]["label"] = "wrong.toml"
        with self.assertRaises(AssertionError):
            assert_dynamic_evidence(report, traces, expectations=expectations)

    def test_rejects_duplicate_adapter_invocations(self) -> None:
        report, traces, expectations = clean_dynamic_report()
        traces["rig"].append(list(traces["rig"][0]))
        with self.assertRaises(AssertionError):
            assert_dynamic_evidence(report, traces, expectations=expectations)

    def test_rejects_corrupted_folded_findings_preview(self) -> None:
        report, traces, expectations = dynamic_report(
            report_clean=False,
            adapter_exit=0,
            finding_count=2,
        )
        assert_dynamic_evidence(report, traces, expectations=expectations)
        report["evidence"][0]["report"]["findings_preview"] = [
            {"id": "fabricated-1"},
            {"id": "fabricated-2"},
        ]
        with self.assertRaises(AssertionError):
            assert_dynamic_evidence(report, traces, expectations=expectations)

    def test_rejects_each_corrupted_folded_outcome_field(self) -> None:
        mutations = {
            "schema": lambda item: item["report"].__setitem__(
                "schema_version", "rig.report/2"
            ),
            "clean": lambda item: item["report"].__setitem__("clean", False),
            "summary": lambda item: item["report"].__setitem__(
                "summary", {"total": 1}
            ),
            "exit_code": lambda item: item.__setitem__("exit_code", 1),
            "finding_count": lambda item: item.__setitem__("finding_count", 1),
        }
        for name, mutate in mutations.items():
            with self.subTest(name=name):
                report, traces, expectations = clean_dynamic_report()
                mutate(report["evidence"][0])
                with self.assertRaises(AssertionError):
                    assert_dynamic_evidence(
                        report,
                        traces,
                        expectations=expectations,
                    )

    def test_rejects_diagnostic_noise_before_json_stdout(self) -> None:
        with self.assertRaises(AssertionError):
            parse_json_stdout('diagnostic\n{"schema_version":"guard.report/1"}\n')

    def test_generated_adapter_stub_is_executable_json(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            trace = _write_stub(root, "vat")
            completed = subprocess.run(
                [str(root / "vat"), "run", "--json", "smoke"],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(completed.returncode, 0, completed.stderr)
            payload = parse_json_stdout(completed.stdout)
            self.assertEqual(payload["schema_version"], "vat.report/1")
            self.assertEqual(
                [
                    json.loads(line)
                    for line in trace.read_text(encoding="utf-8").splitlines()
                ],
                [["run", "--json", "smoke"]],
            )


if __name__ == "__main__":
    unittest.main()
