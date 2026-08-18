from __future__ import annotations

import importlib.util
import io
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path


class TestRunnerProtocol(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.here = Path(__file__).resolve().parent
        cls.ext_contracts = cls.here.parents[1]
        cls.src_dir = cls.ext_contracts / "src"
        cls.runner_path = cls.src_dir / "runner.py"

        spec = importlib.util.spec_from_file_location("runner_module", cls.runner_path)
        assert spec is not None and spec.loader is not None
        cls.runner_module = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = cls.runner_module
        spec.loader.exec_module(cls.runner_module)

    def test_a_passing_case_exits_0_and_reports_its_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            orig_env = os.environ.copy()
            orig_argv = sys.argv[:]
            orig_stdout = sys.stdout
            captured = io.StringIO()
            case_id = "static-role-map-authorization-behavior"

            try:
                os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "aw.python-artifact.v1"
                os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"] = "src_digest_test_123"
                os.environ["AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"] = "dep_digest_test_456"
                os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = tmpdir
                sys.argv = ["runner.py", case_id]
                sys.stdout = captured

                with self.assertRaises(SystemExit) as cm:
                    self.runner_module.main()

                self.assertEqual(cm.exception.code, 0)
                envelope = json.loads(captured.getvalue().strip())

                self.assertEqual(
                    envelope["schema_version"], "aw.python-artifact.result.v1"
                )
                self.assertEqual(envelope["status"], "passed")
                self.assertEqual(envelope["source_digest"], "src_digest_test_123")
                self.assertEqual(
                    envelope["dependency_lock_digest"], "dep_digest_test_456"
                )
                self.assertEqual(
                    envelope["evidence"], [f"evidence/{case_id}.json"]
                )

                ev_path = Path(tmpdir) / f"{case_id}.json"
                self.assertTrue(ev_path.is_file())
                self.assertGreater(ev_path.stat().st_size, 0)
                ev_data = json.loads(ev_path.read_text(encoding="utf-8"))
                self.assertEqual(ev_data["case_id"], case_id)
                self.assertTrue(ev_data["passed"])
                self.assertEqual(ev_data["minimum_checks"], 14)
                self.assertEqual(len(ev_data["checks"]), 14)
                self.assertTrue(all(c["passed"] for c in ev_data["checks"]))
            finally:
                os.environ.clear()
                os.environ.update(orig_env)
                sys.argv = orig_argv
                sys.stdout = orig_stdout

    def test_every_case_declares_the_expected_number_of_checks(self) -> None:
        cases = [
            ("static-role-map-authorization-behavior", 14),
            ("static-role-map-authorization-security", 15),
            ("delegated-kubernetes-authorization-behavior", 15),
            ("delegated-kubernetes-authorization-security", 17),
            ("credential-reload-audit-behavior", 15),
            ("credential-reload-audit-security", 16),
        ]
        for case_id, expected_checks in cases:
            with self.subTest(case_id=case_id):
                with tempfile.TemporaryDirectory() as tmpdir:
                    orig_env = os.environ.copy()
                    orig_argv = sys.argv[:]
                    orig_stdout = sys.stdout
                    captured = io.StringIO()

                    try:
                        os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = (
                            "aw.python-artifact.v1"
                        )
                        os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"] = (
                            "src_digest_test_123"
                        )
                        os.environ[
                            "AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"
                        ] = "dep_digest_test_456"
                        os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = tmpdir
                        sys.argv = ["runner.py", case_id]
                        sys.stdout = captured

                        with self.assertRaises(SystemExit) as cm:
                            self.runner_module.main()

                        self.assertEqual(cm.exception.code, 0)
                        envelope = json.loads(captured.getvalue().strip())
                        self.assertEqual(envelope["status"], "passed")

                        ev_path = Path(tmpdir) / f"{case_id}.json"
                        self.assertTrue(ev_path.is_file())
                        ev_data = json.loads(ev_path.read_text(encoding="utf-8"))
                        self.assertEqual(
                            ev_data["minimum_checks"], expected_checks
                        )
                        self.assertEqual(
                            len(ev_data["checks"]), expected_checks
                        )
                        self.assertTrue(
                            all(c["passed"] for c in ev_data["checks"])
                        )
                    finally:
                        os.environ.clear()
                        os.environ.update(orig_env)
                        sys.argv = orig_argv
                        sys.stdout = orig_stdout

    def test_an_unknown_command_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            orig_env = os.environ.copy()
            orig_argv = sys.argv[:]
            orig_stdout = sys.stdout
            captured = io.StringIO()

            try:
                os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "aw.python-artifact.v1"
                os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"] = "src_digest_test_123"
                os.environ["AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"] = "dep_digest_test_456"
                os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = tmpdir
                sys.argv = ["runner.py", "no-such-case"]
                sys.stdout = captured

                with self.assertRaises(SystemExit) as cm:
                    self.runner_module.main()

                self.assertEqual(cm.exception.code, 1)
                envelope = json.loads(captured.getvalue().strip())

                self.assertEqual(
                    envelope["schema_version"], "aw.python-artifact.result.v1"
                )
                self.assertEqual(envelope["status"], "failed")
                self.assertEqual(envelope["source_digest"], "src_digest_test_123")
                self.assertEqual(
                    envelope["dependency_lock_digest"], "dep_digest_test_456"
                )
            finally:
                os.environ.clear()
                os.environ.update(orig_env)
                sys.argv = orig_argv
                sys.stdout = orig_stdout

    def test_a_protocol_mismatch_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            orig_env = os.environ.copy()
            orig_argv = sys.argv[:]
            orig_stdout = sys.stdout
            captured = io.StringIO()

            try:
                os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "aw.python-artifact.v99"
                os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"] = "src_digest_test_123"
                os.environ["AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"] = "dep_digest_test_456"
                os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = tmpdir
                sys.argv = ["runner.py", "static-role-map-authorization-behavior"]
                sys.stdout = captured

                with self.assertRaises(SystemExit) as cm:
                    self.runner_module.main()

                self.assertEqual(cm.exception.code, 1)
                envelope = json.loads(captured.getvalue().strip())

                self.assertEqual(
                    envelope["schema_version"], "aw.python-artifact.result.v1"
                )
                self.assertEqual(envelope["status"], "failed")
            finally:
                os.environ.clear()
                os.environ.update(orig_env)
                sys.argv = orig_argv
                sys.stdout = orig_stdout

    def test_a_missing_evidence_directory_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            orig_env = os.environ.copy()
            orig_argv = sys.argv[:]
            orig_stdout = sys.stdout
            captured = io.StringIO()

            try:
                os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "aw.python-artifact.v1"
                os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"] = "src_digest_test_123"
                os.environ["AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"] = "dep_digest_test_456"
                os.environ.pop("AW_PYTHON_ARTIFACT_EVIDENCE_DIR", None)
                sys.argv = ["runner.py", "static-role-map-authorization-behavior"]
                sys.stdout = captured

                with self.assertRaises(SystemExit) as cm:
                    self.runner_module.main()

                self.assertEqual(cm.exception.code, 1)
                envelope = json.loads(captured.getvalue().strip())

                self.assertEqual(
                    envelope["schema_version"], "aw.python-artifact.result.v1"
                )
                self.assertEqual(envelope["status"], "failed")
            finally:
                os.environ.clear()
                os.environ.update(orig_env)
                sys.argv = orig_argv
                sys.stdout = orig_stdout


if __name__ == "__main__":
    unittest.main()
