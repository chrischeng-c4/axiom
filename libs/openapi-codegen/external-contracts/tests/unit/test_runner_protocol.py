"""Protocol and fail-closed tests for openapi-codegen external contract runner."""

from __future__ import annotations

import io
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any

EC_ROOT = Path(__file__).resolve().parent.parent.parent
SRC_DIR = EC_ROOT / "src"

sys.path.insert(0, str(SRC_DIR))
import runner  # type: ignore

DECLARED_CASES: dict[str, int] = {
    "tolerant-openapi-document-subset-behavior": 16,
    "tolerant-openapi-document-subset-security": 14,
    "deterministic-identifier-naming-behavior": 14,
    "deterministic-identifier-naming-security": 14,
    "language-neutral-operation-ir-behavior": 18,
    "language-neutral-operation-ir-security": 16,
    "per-language-type-mapping-behavior": 18,
    "per-language-type-mapping-security": 16,
    "versioned-target-profiles-behavior": 16,
    "versioned-target-profiles-security": 16,
    "contained-output-materialization-behavior": 14,
    "contained-output-materialization-security": 16,
}


class TestRunnerProtocol(unittest.TestCase):
    """Test artifact runner protocol, env contract, 12 positive commands, and fail-closed negative branches."""

    def setUp(self) -> None:
        self._orig_env = dict(os.environ)
        self._orig_argv = list(sys.argv)
        self._orig_path = list(sys.path)
        self._orig_modules = dict(sys.modules)
        self._orig_stdout = sys.stdout
        self._orig_stderr = sys.stderr
        self._orig_src_dir = runner.SRC_DIR

    def tearDown(self) -> None:
        os.environ.clear()
        os.environ.update(self._orig_env)
        sys.argv = list(self._orig_argv)
        sys.path = list(self._orig_path)
        sys.stdout = self._orig_stdout
        sys.stderr = self._orig_stderr
        runner.SRC_DIR = self._orig_src_dir
        for mod in list(sys.modules.keys()):
            if mod not in self._orig_modules:
                del sys.modules[mod]

    def _set_valid_env(self, evidence_dir: str) -> None:
        os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "aw.python-artifact.v1"
        os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"] = "src_digest_test_123"
        os.environ["AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"] = "lock_digest_test_456"
        os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = evidence_dir

    def _run_main_with_captured_output(self, argv: list[str]) -> tuple[int, dict[str, Any]]:
        stdout_buf = io.StringIO()
        stderr_buf = io.StringIO()
        saved_stdout = sys.stdout
        saved_stderr = sys.stderr
        try:
            sys.stdout = stdout_buf
            sys.stderr = stderr_buf
            code = runner.main(argv)
        finally:
            sys.stdout = saved_stdout
            sys.stderr = saved_stderr

        output_str = stdout_buf.getvalue()
        try:
            envelope = json.loads(output_str)
        except Exception:
            envelope = {}
        return code, envelope

    def test_production_positive_commands_matrix(self) -> None:
        """Test each of the 12 declared commands against production src/."""
        with tempfile.TemporaryDirectory() as tmp_ev_dir:
            self._set_valid_env(tmp_ev_dir)
            for case_id in DECLARED_CASES:
                code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
                # In Round A, case files do not exist yet, so this fails in Round A and passes in Round B.
                self.assertEqual(
                    code,
                    0,
                    f"Runner failed for declared command {case_id}. Round B materialization required.",
                )

    def test_synthetic_positive_run_artifact_contract(self) -> None:
        """Verify complete artifact envelope and evidence format on a valid synthetic case."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            min_checks = DECLARED_CASES[case_id]
            mod_file = Path(tmp_src) / f"{case_id}.py"

            matrix_items = [f"('check_{i}', ('value_{i}',))" for i in range(min_checks)]
            matrix_str = f"SUBSET_MATRIX = [{', '.join(matrix_items)}]"

            appends_lines = []
            for idx in range(min_checks):
                appends_lines.append(
                    f"    obs_{idx} = parse_type_field('value_{idx}')\n"
                    f"    checks.append({{'name': 'check_{idx}', 'observed': obs_{idx}, 'expected': ('value_{idx}',), 'passed': obs_{idx} == ('value_{idx}',), 'summary': 'ok'}})"
                )
            appends_str = "\n".join(appends_lines)

            code_lines = [
                "from __future__ import annotations",
                "from openapi_codegen.application.document import parse_type_field",
                "MINIMUM_CHECKS = 16",
                matrix_str,
                "def verify_tolerant_openapi_document_subset_behavior() -> dict[str, object]:",
                "    checks = []",
                appends_str,
                f"    return {{'case_id': '{case_id}', 'minimum_checks': 16, 'passed': True, 'checks': checks}}",
                "",
            ]
            mod_file.write_text("\n".join(code_lines), encoding="utf-8")

            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])

            self.assertEqual(code, 0)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "passed")
            self.assertEqual(envelope.get("source_digest"), "src_digest_test_123")
            self.assertEqual(envelope.get("dependency_lock_digest"), "lock_digest_test_456")

            ev_paths = envelope.get("evidence", [])
            self.assertEqual(len(ev_paths), 1)
            self.assertEqual(ev_paths[0], f"evidence/{case_id}.json")

            expected_ev_file = Path(tmp_ev) / f"{case_id}.json"
            self.assertTrue(expected_ev_file.is_file())

            ev_content = json.loads(expected_ev_file.read_text(encoding="utf-8"))
            self.assertEqual(ev_content.get("case_id"), case_id)
            self.assertEqual(ev_content.get("minimum_checks"), min_checks)
            self.assertEqual(ev_content.get("passed"), True)
            self.assertEqual(len(ev_content.get("checks", [])), min_checks)
            self.assertEqual(ev_content.get("source_digest"), "src_digest_test_123")
            self.assertEqual(ev_content.get("lock_digest"), "lock_digest_test_456")

    def test_missing_protocol(self) -> None:
        """Runner fails closed with failed envelope if AW_PYTHON_ARTIFACT_PROTOCOL is missing."""
        with tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            os.environ.pop("AW_PYTHON_ARTIFACT_PROTOCOL")
            code, envelope = self._run_main_with_captured_output(["runner.py", "tolerant-openapi-document-subset-behavior"])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("source_digest"), "src_digest_test_123")
            self.assertEqual(envelope.get("dependency_lock_digest"), "lock_digest_test_456")
            self.assertEqual(envelope.get("evidence"), [])

    def test_wrong_protocol(self) -> None:
        """Runner fails closed with failed envelope if AW_PYTHON_ARTIFACT_PROTOCOL is wrong."""
        with tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "wrong.protocol.v0"
            code, envelope = self._run_main_with_captured_output(["runner.py", "tolerant-openapi-document-subset-behavior"])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("source_digest"), "src_digest_test_123")
            self.assertEqual(envelope.get("dependency_lock_digest"), "lock_digest_test_456")
            self.assertEqual(envelope.get("evidence"), [])

    def test_missing_source_digest(self) -> None:
        """Runner fails closed with failed envelope if AW_PYTHON_ARTIFACT_SOURCE_DIGEST is missing."""
        with tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            os.environ.pop("AW_PYTHON_ARTIFACT_SOURCE_DIGEST")
            code, envelope = self._run_main_with_captured_output(["runner.py", "tolerant-openapi-document-subset-behavior"])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("source_digest"), "")
            self.assertEqual(envelope.get("dependency_lock_digest"), "lock_digest_test_456")
            self.assertEqual(envelope.get("evidence"), [])

    def test_missing_lock_digest(self) -> None:
        """Runner fails closed with failed envelope if AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST is missing."""
        with tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            os.environ.pop("AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST")
            code, envelope = self._run_main_with_captured_output(["runner.py", "tolerant-openapi-document-subset-behavior"])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("source_digest"), "src_digest_test_123")
            self.assertEqual(envelope.get("dependency_lock_digest"), "")
            self.assertEqual(envelope.get("evidence"), [])

    def test_missing_evidence_dir(self) -> None:
        """Runner fails closed with failed envelope if AW_PYTHON_ARTIFACT_EVIDENCE_DIR is missing."""
        with tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            os.environ.pop("AW_PYTHON_ARTIFACT_EVIDENCE_DIR")
            code, envelope = self._run_main_with_captured_output(["runner.py", "tolerant-openapi-document-subset-behavior"])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("source_digest"), "src_digest_test_123")
            self.assertEqual(envelope.get("dependency_lock_digest"), "lock_digest_test_456")
            self.assertEqual(envelope.get("evidence"), [])

    def test_missing_command_arg(self) -> None:
        """Runner fails closed with exit code 2 when command arg is missing."""
        with tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            code, envelope = self._run_main_with_captured_output(["runner.py"])
            self.assertEqual(code, 2)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_extra_command_args(self) -> None:
        """Runner fails closed with exit code 2 when extra args are supplied."""
        with tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            code, envelope = self._run_main_with_captured_output(["runner.py", "tolerant-openapi-document-subset-behavior", "extra_arg"])
            self.assertEqual(code, 2)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_undeclared_existing_file_rejected(self) -> None:
        """Runner rejects dispatch for an undeclared command even if a matching module exists."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "undeclared-module-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            mod_file.write_text(
                "def verify_undeclared_module_behavior(): return {'case_id': 'undeclared-module-behavior', 'minimum_checks': 0, 'passed': True, 'checks': []}\n",
                encoding="utf-8",
            )
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 2)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_declared_missing_file_fails_closed(self) -> None:
        """Runner fails closed when declared case module file does not exist."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", "tolerant-openapi-document-subset-behavior"])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_spec_import_error_fails_closed(self) -> None:
        """Runner fails closed when importing case module raises an error."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            mod_file.write_text("import non_existent_pkg_xyz\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_missing_entrypoint_fails_closed(self) -> None:
        """Runner fails closed when entrypoint function is missing from module."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            mod_file.write_text("other_fn = 123\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_non_callable_entrypoint_fails_closed(self) -> None:
        """Runner fails closed when entrypoint is non-callable."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            mod_file.write_text("verify_tolerant_openapi_document_subset_behavior = 123\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_verifier_exception_fails_closed(self) -> None:
        """Runner fails closed when entrypoint function raises an exception."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            mod_file.write_text("def verify_tolerant_openapi_document_subset_behavior(): raise ValueError('Crash')\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_non_dict_result_fails_closed(self) -> None:
        """Runner fails closed when verifier returns a non-dict result."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            mod_file.write_text("def verify_tolerant_openapi_document_subset_behavior(): return 'not a dict'\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_missing_case_identity_fails_closed(self) -> None:
        """Runner fails closed when result dict missing case_id key."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            mod_file.write_text("def verify_tolerant_openapi_document_subset_behavior(): return {'minimum_checks': 16, 'passed': True, 'checks': []}\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_wrong_case_identity_fails_closed(self) -> None:
        """Runner fails closed when result dict has wrong case_id."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            mod_file.write_text("def verify_tolerant_openapi_document_subset_behavior(): return {'case_id': 'wrong-id', 'minimum_checks': 16, 'passed': True, 'checks': []}\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_boolean_minimum_checks_fails_closed(self) -> None:
        """Runner fails closed when minimum_checks is boolean True."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            mod_file.write_text("def verify_tolerant_openapi_document_subset_behavior(): return {'case_id': 'tolerant-openapi-document-subset-behavior', 'minimum_checks': True, 'passed': True, 'checks': []}\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_wrong_minimum_checks_fails_closed(self) -> None:
        """Runner fails closed when minimum_checks is wrong integer."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            mod_file.write_text("def verify_tolerant_openapi_document_subset_behavior(): return {'case_id': 'tolerant-openapi-document-subset-behavior', 'minimum_checks': 10, 'passed': True, 'checks': []}\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_non_list_checks_fails_closed(self) -> None:
        """Runner fails closed when checks field is not a list."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            mod_file.write_text("def verify_tolerant_openapi_document_subset_behavior(): return {'case_id': 'tolerant-openapi-document-subset-behavior', 'minimum_checks': 16, 'passed': True, 'checks': 'not a list'}\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_wrong_length_checks_fails_closed(self) -> None:
        """Runner fails closed when checks list length does not match required floor."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            mod_file.write_text("def verify_tolerant_openapi_document_subset_behavior(): return {'case_id': 'tolerant-openapi-document-subset-behavior', 'minimum_checks': 16, 'passed': True, 'checks': [{'name': 'c0', 'observed': 1, 'expected': 1, 'passed': True}]}\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_empty_check_name_fails_closed(self) -> None:
        """Runner fails closed when a check item has an empty name string."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            checks_repr = repr([{"name": "", "observed": 1, "expected": 1, "passed": True}] * 16)
            mod_file.write_text(f"def verify_tolerant_openapi_document_subset_behavior(): return {{'case_id': 'tolerant-openapi-document-subset-behavior', 'minimum_checks': 16, 'passed': True, 'checks': {checks_repr}}}\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_duplicate_check_names_fails_closed(self) -> None:
        """Runner fails closed when check names are duplicated."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            checks_repr = repr([{"name": "dup_name", "observed": 1, "expected": 1, "passed": True}] * 16)
            mod_file.write_text(f"def verify_tolerant_openapi_document_subset_behavior(): return {{'case_id': 'tolerant-openapi-document-subset-behavior', 'minimum_checks': 16, 'passed': True, 'checks': {checks_repr}}}\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_false_check_status_fails_closed(self) -> None:
        """Runner fails closed when check status is False, missing observed/expected, or observed != expected."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"

            # Scenario A: Check passed field is False
            checks = [{"name": f"c_{i}", "observed": i, "expected": i, "passed": True} for i in range(15)]
            checks.append({"name": "c_fail", "observed": 15, "expected": 15, "passed": False})
            mod_file.write_text(f"def verify_tolerant_openapi_document_subset_behavior(): return {{'case_id': 'tolerant-openapi-document-subset-behavior', 'minimum_checks': 16, 'passed': True, 'checks': {checks!r}}}\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

            # Scenario B: Check missing 'observed' field
            checks_no_obs = [{"name": f"c_{i}", "observed": i, "expected": i, "passed": True} for i in range(15)]
            checks_no_obs.append({"name": "c_fail", "expected": 15, "passed": True})
            mod_file.write_text(f"def verify_tolerant_openapi_document_subset_behavior(): return {{'case_id': 'tolerant-openapi-document-subset-behavior', 'minimum_checks': 16, 'passed': True, 'checks': {checks_no_obs!r}}}\n", encoding="utf-8")
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

            # Scenario C: Check missing 'expected' field
            checks_no_exp = [{"name": f"c_{i}", "observed": i, "expected": i, "passed": True} for i in range(15)]
            checks_no_exp.append({"name": "c_fail", "observed": 15, "passed": True})
            mod_file.write_text(f"def verify_tolerant_openapi_document_subset_behavior(): return {{'case_id': 'tolerant-openapi-document-subset-behavior', 'minimum_checks': 16, 'passed': True, 'checks': {checks_no_exp!r}}}\n", encoding="utf-8")
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

            # Scenario D: Check passed=True but observed != expected mismatch
            checks_mismatch = [{"name": f"c_{i}", "observed": i, "expected": i, "passed": True} for i in range(15)]
            checks_mismatch.append({"name": "c_fail", "observed": 15, "expected": 999, "passed": True})
            mod_file.write_text(f"def verify_tolerant_openapi_document_subset_behavior(): return {{'case_id': 'tolerant-openapi-document-subset-behavior', 'minimum_checks': 16, 'passed': True, 'checks': {checks_mismatch!r}}}\n", encoding="utf-8")
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

            # Scenario E: Check passed=True but scalar cross-type equality (1 vs 1.0)
            checks_type_mismatch = [{"name": f"c_{i}", "observed": i, "expected": i, "passed": True} for i in range(15)]
            checks_type_mismatch.append({"name": "c_fail", "observed": 1, "expected": 1.0, "passed": True})
            mod_file.write_text(f"def verify_tolerant_openapi_document_subset_behavior(): return {{'case_id': 'tolerant-openapi-document-subset-behavior', 'minimum_checks': 16, 'passed': True, 'checks': {checks_type_mismatch!r}}}\n", encoding="utf-8")
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

            # Scenario F: Check passed=True but dict key cross-type equality ({1: "x"} vs {True: "x"})
            checks_dict_key_mismatch = [{"name": f"c_{i}", "observed": i, "expected": i, "passed": True} for i in range(15)]
            checks_dict_key_mismatch.append({"name": "c_fail", "observed": {1: "x"}, "expected": {True: "x"}, "passed": True})
            mod_file.write_text(f"def verify_tolerant_openapi_document_subset_behavior(): return {{'case_id': 'tolerant-openapi-document-subset-behavior', 'minimum_checks': 16, 'passed': True, 'checks': {checks_dict_key_mismatch!r}}}\n", encoding="utf-8")
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

            # Scenario G: Direct strict_equals set and frozenset assertions
            self.assertFalse(runner.strict_equals({1}, {True}))
            self.assertFalse(runner.strict_equals(frozenset({1}), frozenset({True})))
            self.assertFalse(runner.strict_equals({1}, frozenset({1})))
            self.assertTrue(runner.strict_equals({1, "x"}, {"x", 1}))
            self.assertTrue(runner.strict_equals(frozenset({1, "x"}), frozenset({"x", 1})))

            # Scenario H: validate_result rejects claimed-passing check with {1} vs {True} before evidence writing
            bad_set_result = {
                "case_id": case_id,
                "minimum_checks": 16,
                "passed": True,
                "checks": [{"name": f"c_{i}", "observed": i, "expected": i, "passed": True} for i in range(15)]
                + [{"name": "c_fail", "observed": {1}, "expected": {True}, "passed": True}],
            }
            valid, msg, _ = runner.validate_result(case_id, bad_set_result, 16, "src_digest_test_123", "lock_digest_test_456")
            self.assertFalse(valid)
            self.assertIn("does not strictly equal", msg)

    def test_non_boolean_check_status_fails_closed(self) -> None:
        """Runner fails closed when a check passed status is a non-boolean string."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            checks = [{"name": f"c_{i}", "observed": 1, "expected": 1, "passed": "true"} for i in range(16)]
            mod_file.write_text(f"def verify_tolerant_openapi_document_subset_behavior(): return {{'case_id': 'tolerant-openapi-document-subset-behavior', 'minimum_checks': 16, 'passed': True, 'checks': {checks!r}}}\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_false_top_level_status_fails_closed(self) -> None:
        """Runner fails closed when top-level passed status is False."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            checks = [{"name": f"c_{i}", "observed": 1, "expected": 1, "passed": True} for i in range(16)]
            mod_file.write_text(f"def verify_tolerant_openapi_document_subset_behavior(): return {{'case_id': 'tolerant-openapi-document-subset-behavior', 'minimum_checks': 16, 'passed': False, 'checks': {checks!r}}}\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])

    def test_non_boolean_top_level_status_fails_closed(self) -> None:
        """Runner fails closed when top-level passed status is non-boolean integer."""
        with tempfile.TemporaryDirectory() as tmp_src, tempfile.TemporaryDirectory() as tmp_ev:
            self._set_valid_env(tmp_ev)
            case_id = "tolerant-openapi-document-subset-behavior"
            mod_file = Path(tmp_src) / f"{case_id}.py"
            checks = [{"name": f"c_{i}", "observed": 1, "expected": 1, "passed": True} for i in range(16)]
            mod_file.write_text(f"def verify_tolerant_openapi_document_subset_behavior(): return {{'case_id': 'tolerant-openapi-document-subset-behavior', 'minimum_checks': 16, 'passed': 1, 'checks': {checks!r}}}\n", encoding="utf-8")
            runner.SRC_DIR = Path(tmp_src)
            code, envelope = self._run_main_with_captured_output(["runner.py", case_id])
            self.assertEqual(code, 1)
            self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
            self.assertEqual(envelope.get("status"), "failed")
            self.assertEqual(envelope.get("evidence"), [])


if __name__ == "__main__":
    unittest.main()
