#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("run_isolated.py")
SPEC = importlib.util.spec_from_file_location("run_isolated", SCRIPT)
assert SPEC and SPEC.loader
runner = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runner)


class IsolatedRunnerTest(unittest.TestCase):
    def test_dispatcher_resolves_to_repo_root_entrypoint(self) -> None:
        expected = Path(__file__).resolve().parents[4] / "scripts/agy_dispatch.py"
        self.assertEqual(runner.DISPATCHER.resolve(), expected.resolve())

    def test_command_disables_project_discovery(self) -> None:
        built = runner.command(["doctor", "/tmp/profile.json"])
        self.assertEqual(built[:5], ["uv", "run", "--isolated", "--no-project", "--python"])
        self.assertEqual(built[-2:], ["doctor", "/tmp/profile.json"])

    def test_runtime_environment_overrides_repo_venv(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            environment = runner.runtime_environment(
                {
                    "UV_PROJECT_ENVIRONMENT": "/repo/.venv",
                    "AGY_DISPATCH_RUNTIME": raw,
                }
            )
        self.assertEqual(
            environment["UV_PROJECT_ENVIRONMENT"],
            str(Path(raw).resolve()),
        )

    def test_runtime_environment_rejects_non_temp_override(self) -> None:
        with self.assertRaisesRegex(SystemExit, "resolve under the system temp root"):
            runner.runtime_environment(
                {"AGY_DISPATCH_RUNTIME": "/Users/example/repository/.venv"}
            )


if __name__ == "__main__":
    unittest.main()
