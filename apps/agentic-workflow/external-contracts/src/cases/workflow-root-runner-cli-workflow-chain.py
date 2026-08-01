"""Black-box contract for the EC-first CLI workflow chain (#3298)."""

from __future__ import annotations

import shlex
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw


CASE_ID = "workflow-root-runner-cli-workflow-chain"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "cli-workflow-chain"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case workflow-root-runner-cli-workflow-chain"
)
ASSERTIONS = (
    "the first next.command a fresh, validated Change root emits from a live aw goal wi dispatch is a real aw ec check invocation, captured from the runtime rather than hardcoded",
    "literally re-executing that captured command against the same real binary chains to a real aw ec draft next.command, which itself chains to a real aw ec check next.command -- three consecutive real hops, none rejected by clap",
    "a bare command independently known to omit a clap-required argument is a negative control: it fails with the exact clap 'required arguments were not provided' usage rejection this check exists to catch, proving the positive hops above are genuinely chain-valid and not merely unchecked",
)

_CLAP_REJECTED_PREFIX = "error: the following required arguments were not provided:"

_UNIT_TEST_SOURCE = (
    "import unittest\n"
    "\n"
    "\n"
    "class FixtureManifestTest(unittest.TestCase):\n"
    "    def test_fixture_declares_a_python_project_manifest(self) -> None:\n"
    "        from pathlib import Path\n"
    "\n"
    "        root = Path(__file__).resolve().parents[2]\n"
    '        manifest = (root / "pyproject.toml").read_text(encoding="utf-8")\n'
    '        self.assertIn("[project]", manifest)\n'
    '        self.assertIn("requires-python", manifest)\n'
    "\n"
    "\n"
    'if __name__ == "__main__":\n'
    "    unittest.main()\n"
)


def _change_body(in_scope: str) -> str:
    return (
        "## Problem\n\nDemonstrate the EC-first CLI workflow chain end to end.\n\n"
        "## Capability Alignment\n\n"
        "Capability: Workflow root runner\n"
        "Capability Gap: none, this fixture only drives the existing chain\n"
        "Progress Evidence: the public goal wi envelope is the evidence\n\n"
        "## Requirements\n\n- R1: trace the emitted command chain.\n\n"
        f"## Scope\n\n### In Scope\n- {in_scope}\n\n"
        "### Out of Scope\n- Rework unrelated lifecycle stages.\n\n"
        "## Acceptance Criteria\n\n- AC1: every hop is chain-valid.\n\n"
        "## Reference Context\n\n### Related Specs\n"
        "| Spec | Relevance |\n|------|-----------|\n"
        "| complete-platform.md | describes the environment |\n\n"
        "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
        "|---------|--------|---------------|\n"
        "| chain-trace | update | complete-platform.md |\n"
    )


def _is_clap_rejected(stderr: str) -> bool:
    return stderr.strip().startswith(_CLAP_REJECTED_PREFIX)


def _run_captured(root: Path, command: str) -> subprocess.CompletedProcess[str]:
    assert command.startswith("aw "), command
    argv = shlex.split(command)[1:]
    return run_aw(root, *argv, expect_success=None)


def verify() -> list[str]:
    with project_fixture() as root:
        created = create(
            root,
            "Trace the EC-first chain",
            "change",
            "--body",
            _change_body("trace the chain"),
        )
        slug = created["slug"]

        validated = final_json(run_aw(root, "wi", "validate", slug))
        assert validated["passed"] is True, validated
        assert validated["new_state"] == "open", validated

        dispatch = final_json(run_aw(root, "goal", "wi", slug))
        assert dispatch["current"] == {"kind": "change", "id": slug}, dispatch
        first_command = dispatch["next"]["command"]
        assert first_command == f"aw ec check --project demo --wi {slug}", dispatch

        # Hop 1: literally re-execute the captured next.command. A fresh
        # project has no EC inventory yet, so the real chain must ask for a
        # scaffold rather than reject the command as clap-invalid.
        hop1 = _run_captured(root, first_command)
        assert not _is_clap_rejected(hop1.stderr), hop1.stderr
        assert hop1.returncode == 0, (hop1.stdout, hop1.stderr)
        hop1_json = final_json(hop1)
        assert hop1_json["action"] == "python_ec_scaffold_required", hop1_json
        second_command = hop1_json["next"]["command"]
        assert second_command == (
            f"aw ec draft {slug} --project demo --json --wi {slug}"
        ), hop1_json

        # Hop 2: the draft command is itself a live emit site. Executing it
        # for real must author the scaffold and chain to another real check.
        hop2 = _run_captured(root, second_command)
        assert not _is_clap_rejected(hop2.stderr), hop2.stderr
        assert hop2.returncode == 0, (hop2.stdout, hop2.stderr)
        hop2_json = final_json(hop2)
        assert hop2_json["action"] == "python_ec_scaffold_created", hop2_json
        assert set(hop2_json["artifacts"]) == {
            "external-contracts/pyproject.toml",
            "external-contracts/uv.lock",
            "external-contracts/src/runner.py",
            f"external-contracts/src/{slug}.py",
        }, hop2_json
        third_command = hop2_json["next"]["command"]
        assert third_command == (
            f"aw ec check --project demo --json --wi {slug}"
        ), hop2_json

        # `aw ec check` refuses to read a Python artifact with no authored
        # unit tests, so the fixture needs one before the third hop is
        # reachable -- this mirrors write_python_artifact_unit_test.
        unit_dir = root / "external-contracts" / "tests" / "unit"
        unit_dir.mkdir(parents=True, exist_ok=True)
        (unit_dir / "test_fixture.py").write_text(_UNIT_TEST_SOURCE, encoding="utf-8")

        # Hop 3: the check command chains back to itself and now surfaces a
        # real, structured finding derived from the artifact hop 2 created --
        # proof this is genuine causal chaining, not a stubbed reply.
        hop3 = _run_captured(root, third_command)
        assert not _is_clap_rejected(hop3.stderr), hop3.stderr
        assert hop3.returncode == 1, (hop3.stdout, hop3.stderr)
        hop3_json = final_json(hop3)
        assert hop3_json["configured"] is True, hop3_json
        assert any(
            f"{slug}-behavior" in finding for finding in hop3_json["findings"]
        ), hop3_json

        # Negative control: a bare command with a known-missing clap-required
        # flag must fail with the exact usage-rejection signature the chain
        # check is built to catch, proving the three hops above are actually
        # discriminating rather than accepting anything handed to them.
        negative_control = run_aw(
            root,
            "wi",
            "spike",
            "resolve",
            "does-not-exist",
            expect_success=False,
        )
        assert negative_control.returncode == 2, negative_control
        assert _is_clap_rejected(negative_control.stderr), negative_control.stderr

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
