import json
import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).parents[2]
CLI = ROOT / "src/task_cli/interface/cli.py"


def run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = {**os.environ, "PYTHONPATH": str(ROOT / "src")}
    return subprocess.run([sys.executable, str(CLI), *args], text=True, capture_output=True, env=env)


def test_behavior_create_emits_deterministic_public_json() -> None:
    result = run_cli("create", "ship release", "--actor", "alice")
    assert result.returncode == 0
    assert json.loads(result.stdout) == {"actor": "alice", "title": "ship release"}


def test_security_path_like_title_is_rejected_at_public_boundary() -> None:
    result = run_cli("create", "../secrets")
    assert result.returncode == 2
    assert json.loads(result.stdout)["error"] == "title contains a forbidden path token"
