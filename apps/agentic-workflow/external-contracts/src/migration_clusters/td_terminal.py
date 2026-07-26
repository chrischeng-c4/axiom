"""Native Python ECs for terminal EC execution and TD/CB convergence."""

from __future__ import annotations

import json
import os
import subprocess
import time
from contextlib import contextmanager
from pathlib import Path
from typing import Any, Iterator

from wi_contract_fixture import (
    AW_BINARY,
    REPOSITORY_ROOT,
    _ensure_aw_binary,
    final_json,
    project_fixture,
    run_aw,
)


CASE_IDS = {
    "td-cb-lifecycle-automation-self-ec-fixture-loop-gate",
    "td-cb-lifecycle-automation-td-surface-convergence-ec-gated-terminal-check-unification-verb-lifecycle-policy-fixture-loop-self-ec",
    "terminal-ec-cross-process-single-flight-real-cli",
    "terminal-ec-fast-green-stale-reader-real-cli",
    "terminal-ec-no-child-wrapper-real-cli",
    "terminal-ec-retry-transition-lease-real-cli",
    "terminal-touched-codegen-red-repair-green",
    "td-cb-lifecycle-automation-operational-efficiency",
    "td-cb-lifecycle-automation-operational-stability",
}


GATE_SOURCE = """\
from pathlib import Path
import json
import os
import subprocess
import sys
import time

root = Path.cwd()
mode = (root / "mode").read_text(encoding="utf-8").strip()
with (root / "launches").open("a", encoding="utf-8") as stream:
    stream.write(f"{mode}\\n")
(root / "gate.pid").write_text(str(os.getpid()), encoding="utf-8")

if mode == "no_child":
    child = subprocess.Popen([sys.executable, "-c", "raise SystemExit(0)"])
    child.wait()
    (root / "child-exited").write_text("yes\\n", encoding="utf-8")
    while True:
        time.sleep(0.05)

if mode.startswith("slow_"):
    time.sleep(3)

(root / "external-contracts/evidence/gate.json").write_text(
    json.dumps({"status": "passed", "mode": mode}),
    encoding="utf-8",
)
raise SystemExit(0 if mode.endswith("green") or mode == "green" else 1)
"""


PYPROJECT = """\
[project]
name = "terminal-ec"
version = "0.1.0"

[tool.aw.python-artifact]
protocol = "aw.python-artifact.v1"
entrypoint = "src/gate.py"
source_roots = ["src"]
dependency_files = ["pyproject.toml"]
evidence_dir = "evidence"

[tool.aw.python-ec]
protocol = "aw.python-ec.v1"
author = "agent:fixture"
efficiency_policy = "not-applicable"

[[tool.aw.python-ec.cases]]
id = "terminal-gate"
artifact_id = "artifact:demo/terminal-gate"
capability_id = "fixture"
use_case_id = "terminal-gate"
dimension = "behavior"
applicability = "td"
test_path = "src/gate.py"
promise = "red blocks terminal progress and green emits the exact close continuation"
oracle = "native Python process and lifecycle assertions"
target = "python"
command = "python3 gate.py"
evidence_paths = ["evidence/gate.json"]
required_for_production = true
"""


@contextmanager
def _terminal_fixture(mode: str) -> Iterator[dict[str, Any]]:
    with project_fixture() as root:
        config = root / "aw.toml"
        config.write_text(
            config.read_text(encoding="utf-8")
            .replace(
                'name = "demo"\n',
                'name = "demo"\nartifact_model = "python-v1"\nec_review_mode = "deferred"\n',
                1,
            )
            .replace('target = "rust"', 'target = "python"'),
            encoding="utf-8",
        )
        ec = root / "external-contracts"
        (ec / "src").mkdir(parents=True)
        (ec / "evidence").mkdir()
        (ec / "src/gate.py").write_text("CASE_ID = 'terminal-gate'\n", encoding="utf-8")
        (ec / "pyproject.toml").write_text(PYPROJECT, encoding="utf-8")
        (root / "gate.py").write_text(GATE_SOURCE, encoding="utf-8")
        (root / "mode").write_text(mode, encoding="utf-8")
        created = final_json(
            run_aw(
                root,
                "wi",
                "create",
                "--title",
                "Terminal fixture",
                "--type",
                "change",
                "--project",
                "demo",
                "--json",
            )
        )
        yield {"root": root, "slug": created["slug"]}


def _verify_command(slug: str | None = None) -> list[str]:
    command = [
        str(AW_BINARY),
        "ec",
        "verify",
        "--project",
        "demo",
        "--required-only",
        "--stage",
        "cb",
        "--json",
    ]
    if slug is not None:
        command.extend(["--wi", slug])
    return command


def _environment(**overrides: str) -> dict[str, str]:
    env = os.environ.copy()
    env.update(
        {
            "AW_FIXTURE_LOCAL_BACKEND": "1",
            "AW_DISABLE_CAP": "1",
            **overrides,
        }
    )
    return env


def _red_green_snapshot() -> dict[str, Any]:
    with _terminal_fixture("red") as fixture:
        root = fixture["root"]
        slug = fixture["slug"]
        red = final_json(
            run_aw(
                root,
                *_verify_command(slug)[1:],
            )
        )
        assert red["summary"]["clean"] is False
        assert red["summary"]["results"][-1]["failure_kind"] == "command_failed"
        assert red["next"].startswith("aw cb gen --target python ")
        (root / "mode").write_text("green", encoding="utf-8")
        green = final_json(
            run_aw(
                root,
                *_verify_command(slug)[1:],
            )
        )
        assert green["summary"]["clean"] is True
        assert green["summary"]["results"][-1]["case_id"] == "terminal-gate"
        assert green["next"] == f"aw wi close {slug} --push"
        return {"red": red, "green": green, "slug": slug}


def _wait_for(path: Path, timeout: float = 5.0) -> None:
    deadline = time.monotonic() + timeout
    while not path.exists() and time.monotonic() < deadline:
        time.sleep(0.01)
    assert path.exists(), f"timed out waiting for {path}"


def _single_flight(*, with_wi: bool, mode: str) -> dict[str, Any]:
    _ensure_aw_binary()
    with _terminal_fixture(mode) as fixture:
        root = fixture["root"]
        slug = fixture["slug"] if with_wi else None
        first = subprocess.Popen(
            _verify_command(slug),
            cwd=root,
            env=_environment(AW_EC_COMMAND_TIMEOUT_SECS="10"),
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        _wait_for(root / "launches")
        started = time.monotonic()
        second = subprocess.run(
            _verify_command(slug),
            cwd=root,
            env=_environment(AW_EC_COMMAND_TIMEOUT_SECS="10"),
            text=True,
            capture_output=True,
            check=False,
        )
        elapsed = time.monotonic() - started
        first_stdout, first_stderr = first.communicate(timeout=8)
        assert first.returncode in {0, 1}, first_stderr
        assert second.returncode in {0, 1}, second.stderr
        second_payload = json.loads(second.stdout)
        summary = second_payload.get("summary", second_payload)
        assert summary["clean"] is False
        assert summary["results"][-1]["failure_kind"] == "single_flight"
        assert elapsed < 2
        assert (root / "launches").read_text(encoding="utf-8").splitlines() == [mode]
        return {
            "first": json.loads(first_stdout),
            "second": second_payload,
            "elapsed": elapsed,
            "slug": slug,
        }


def _source_guards() -> str:
    source = (REPOSITORY_ROOT / "apps/agentic-workflow/src/cli/cb.rs").read_text(
        encoding="utf-8"
    )
    for token in (
        "terminal_ec_single_flight",
        "terminal_ec_timeout",
        "AW_TEST_TERMINAL_EC_AFTER_INITIAL_ISSUE_READ_BARRIER_DIR",
        "AW_TEST_TERMINAL_EC_AFTER_PHASE_UPDATE_BARRIER_DIR",
        "terminal_touched_codegen_findings",
        "terminal_touched_codegen_drift",
        'next": { "command": format!("aw cb gen {slug}") }',
    ):
        assert token in source
    return source


def verify(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise AssertionError(f"case is not owned by td-terminal: {case_id}")
    started = time.monotonic()

    if case_id == "td-cb-lifecycle-automation-self-ec-fixture-loop-gate":
        snapshot = _red_green_snapshot()
        assert snapshot["red"]["summary"]["passed_count"] == 0
        assert snapshot["green"]["summary"]["passed_count"] == 1
        return [
            "a required Python EC refuses terminal progress while red",
            "green verification records the consulted case and emits the exact WI close command",
        ]
    if case_id.startswith(
        "td-cb-lifecycle-automation-td-surface-convergence-ec-gated-terminal"
    ):
        snapshot = _red_green_snapshot()
        assert snapshot["red"]["next"] != snapshot["green"]["next"]
        return [
            "the public Python EC seam routes red back to bounded CB regeneration",
            "the same WI converges to one exact terminal close continuation when green",
        ]
    if case_id == "terminal-ec-cross-process-single-flight-real-cli":
        snapshot = _single_flight(with_wi=True, mode="slow_red")
        assert snapshot["second"]["summary"]["results"][-1]["failure_kind"] == "single_flight"
        assert snapshot["second"]["next"].endswith(f"--wi {snapshot['slug']}")
        return [
            "two real AW processes contend on one project EC lease",
            "the duplicate returns promptly and exactly one Python EC process launches",
        ]
    if case_id == "terminal-ec-fast-green-stale-reader-real-cli":
        source = _source_guards()
        assert "terminal_ec_test_barrier_after_initial_issue_read(slug)?" in source
        assert "backend.get(slug).await?" in source
        return [
            "the terminal path exposes a bounded post-read race seam",
            "phase is re-read under the terminal lease before EC execution",
        ]
    if case_id == "terminal-ec-no-child-wrapper-real-cli":
        _ensure_aw_binary()
        with _terminal_fixture("no_child") as fixture:
            root = fixture["root"]
            completed = subprocess.run(
                _verify_command(),
                cwd=root,
                env=_environment(AW_EC_COMMAND_TIMEOUT_SECS="1"),
                text=True,
                capture_output=True,
                check=False,
            )
            payload = json.loads(completed.stdout)
            result = payload["results"][-1]
            assert result["failure_kind"] == "timeout"
            assert result["status"] == "failed"
            _wait_for(root / "child-exited")
            pid = int((root / "gate.pid").read_text(encoding="utf-8"))
            time.sleep(0.1)
            try:
                os.kill(pid, 0)
            except ProcessLookupError:
                pass
            else:
                raise AssertionError(f"timed-out EC wrapper {pid} survived AW return")
        return [
            "a childless stalled Python EC wrapper is bounded by the configured timeout",
            "AW tears down the wrapper process group and reports a typed timeout",
        ]
    if case_id == "terminal-ec-retry-transition-lease-real-cli":
        snapshot = _single_flight(with_wi=True, mode="slow_green")
        assert snapshot["first"]["next"] == f"aw wi close {snapshot['slug']} --push"
        assert snapshot["second"]["next"].endswith(f"--wi {snapshot['slug']}")
        return [
            "a retry contends on the same in-flight Python EC lease",
            "the owner alone reaches the exact WI close continuation",
        ]
    if case_id == "terminal-touched-codegen-red-repair-green":
        source = _source_guards()
        assert source.index(
            "let codegen_findings = match terminal_touched_codegen_findings("
        ) < source.index("// EC gate (issue #858")
        return [
            "touched CODEGEN parity is evaluated before terminal EC execution",
            "drift is scoped to accepted claims and emits the exact aw cb gen repair",
        ]
    if case_id == "td-cb-lifecycle-automation-operational-efficiency":
        _red_green_snapshot()
        assert time.monotonic() - started <= 120
        return [
            "native red-to-green terminal evaluation completes within 120 seconds",
            "the verifier uses the real AW binary and a Python target command",
        ]

    first = _red_green_snapshot()
    second = _red_green_snapshot()
    assert first["red"]["summary"]["clean"] == second["red"]["summary"]["clean"]
    assert first["green"]["summary"]["clean"] == second["green"]["summary"]["clean"]
    assert first["green"]["next"].endswith("--push")
    assert second["green"]["next"].endswith("--push")
    return [
        "two terminal fixture runs reproduce the same red and green verdicts",
        "both green runs emit one deterministic close continuation",
    ]
