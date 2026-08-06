"""Black-box fixture helpers for hand-authored Python EC implementations."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
from collections.abc import Callable, Iterator
from contextlib import contextmanager
from pathlib import Path
from typing import Any


REPOSITORY_ROOT = Path(__file__).resolve().parents[4]
AW_BINARY = REPOSITORY_ROOT / "target" / "debug" / "aw"
EVIDENCE_ROOT = Path(__file__).resolve().parents[1] / "evidence"
_AW_READY = False


def _ensure_aw_binary() -> None:
    global _AW_READY
    if _AW_READY:
        return
    rustup = shutil.which("rustup")
    if rustup is None:
        raise AssertionError("rustup is required to build the AW EC fixture binary")
    completed = subprocess.run(
        [
            rustup,
            "run",
            "stable",
            "cargo",
            "build",
            "-p",
            "agentic-workflow",
            "--bin",
            "aw",
        ],
        cwd=REPOSITORY_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise AssertionError(
            f"failed to build aw:\nstdout={completed.stdout}\nstderr={completed.stderr}"
        )
    _AW_READY = True


@contextmanager
def project_fixture() -> Iterator[Path]:
    with tempfile.TemporaryDirectory(prefix="aw-python-ec-") as raw_root:
        root = Path(raw_root)
        (root / "aw.toml").write_text(
            """
[agentic_workflow.workspace]
mode = "in_place"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "demo"
label = "app:demo"
path = "."
tech_design_path = "tech-design"

[[projects.workspaces]]
name = "demo"
paths = ["**"]
target = "rust"
""".lstrip(),
            encoding="utf-8",
        )
        yield root


def write_python_artifact_unit_test(artifact_root: Path, name: str = "fixture") -> Path:
    """Write the `tests/unit/test_*.py` a canonical Python artifact requires.

    `aw ec check` refuses to read an artifact with no authored unit tests, so
    the fixture needs one before any contract assertion becomes reachable.
    """
    unit_dir = artifact_root / "tests/unit"
    unit_dir.mkdir(parents=True, exist_ok=True)
    test_path = unit_dir / f"test_{name}.py"
    class_name = "".join(part.capitalize() for part in name.split("_"))
    test_path.write_text(
        "import unittest\n"
        "\n"
        "\n"
        f"class {class_name}ManifestTest(unittest.TestCase):\n"
        f"    def test_{name}_declares_a_python_project_manifest(self) -> None:\n"
        "        from pathlib import Path\n"
        "\n"
        "        root = Path(__file__).resolve().parents[2]\n"
        '        manifest = (root / "pyproject.toml").read_text(encoding="utf-8")\n'
        '        self.assertIn("[project]", manifest)\n'
        '        self.assertIn("requires-python", manifest)\n'
        "\n"
        "\n"
        'if __name__ == "__main__":\n'
        "    unittest.main()\n",
        encoding="utf-8",
    )
    return test_path


def git_commit_fixture(root: Path, message: str = "fixture") -> None:
    """Make `root` a committed git repository.

    `aw` resolves native target ownership through `git log`, so a fixture that
    reaches ownership resolution without a repository fails on the environment
    rather than on the contract under test.
    """
    for args in (
        ("init",),
        ("config", "user.email", "fixture@example.com"),
        ("config", "user.name", "Fixture"),
        ("add", "-A"),
        ("commit", "-m", message),
    ):
        completed = subprocess.run(
            ["git", *args],
            cwd=root,
            capture_output=True,
            text=True,
            check=False,
        )
        if completed.returncode != 0:
            raise AssertionError(
                f"git {' '.join(args)} failed in fixture: {completed.stderr}"
            )


def write_python_artifact_lock(
    artifact_root: Path,
    *,
    name: str,
    version: str = "0.1.0",
    requires_python: str = ">=3.11",
) -> Path:
    """Write the `uv.lock` a fixture manifest declares in `dependency_files`.

    A Python artifact whose `dependency_files` names `uv.lock` cannot be read at
    all when the lock is absent, so every assertion behind it is unreachable.
    This mirrors the dependency-free lock `refresh_uv_lock` synthesizes in
    `apps/agentic-workflow/src/services/python_artifact.rs` so the fixture is
    frozen-executable rather than merely present.
    """
    lock = artifact_root / "uv.lock"
    lock.write_text(
        "version = 1\n"
        "revision = 3\n"
        f'requires-python = "{requires_python}"\n'
        "\n"
        "[[package]]\n"
        f'name = "{name}"\n'
        f'version = "{version}"\n'
        'source = { virtual = "." }\n',
        encoding="utf-8",
    )
    return lock


def run_aw(
    root: Path,
    *args: str,
    expect_success: bool | None = True,
    env_overrides: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    _ensure_aw_binary()
    env = os.environ.copy()
    env["AW_FIXTURE_LOCAL_BACKEND"] = "1"
    env["AW_DISABLE_CAP"] = "1"
    if env_overrides:
        env.update(env_overrides)
    completed = subprocess.run(
        [str(AW_BINARY), *args],
        cwd=root,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if expect_success is True and completed.returncode != 0:
        raise AssertionError(
            f"aw {' '.join(args)} failed:\n"
            f"stdout={completed.stdout}\nstderr={completed.stderr}"
        )
    if expect_success is False and completed.returncode == 0:
        raise AssertionError(f"aw {' '.join(args)} unexpectedly succeeded")
    return completed


def final_json(completed: subprocess.CompletedProcess[str]) -> dict[str, Any]:
    raw = completed.stdout
    decoder = json.JSONDecoder()
    values: list[Any] = []
    cursor = 0
    while cursor < len(raw):
        while cursor < len(raw) and raw[cursor].isspace():
            cursor += 1
        if cursor >= len(raw):
            break
        value, cursor = decoder.raw_decode(raw, cursor)
        values.append(value)
    if not values or not isinstance(values[-1], dict):
        raise AssertionError(f"command emitted no JSON:\nstderr={completed.stderr}")
    return values[-1]


def create(root: Path, title: str, work_item_type: str, *extra: str) -> dict[str, Any]:
    return final_json(
        run_aw(
            root,
            "wi",
            "create",
            "--title",
            title,
            "--type",
            work_item_type,
            "--project",
            "demo",
            *extra,
        )
    )


def show(root: Path, slug: str) -> dict[str, Any]:
    payload = final_json(run_aw(root, "wi", "show", slug))
    issue = payload.get("issue")
    return issue if isinstance(issue, dict) else payload


def record_evidence(case_id: str, assertions: list[str]) -> None:
    EVIDENCE_ROOT.mkdir(parents=True, exist_ok=True)
    (EVIDENCE_ROOT / f"{case_id}.json").write_text(
        json.dumps(
            {
                "protocol": "aw.python-ec.evidence.v1",
                "case_id": case_id,
                "status": "passed",
                "assertions": assertions,
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )


def verify_case(case_id: str, verifier: Callable[[], list[str]]) -> None:
    record_evidence(case_id, verifier())
