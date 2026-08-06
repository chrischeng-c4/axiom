"""Black-box contract for the three-host aw-* subagent fleet projection.

Drives the real `aw new`, `aw new --check-agents`, and `aw new --sync-agents`
commands against a freshly bootstrapped fixture project, proving the one
canonical `templates/cli/mainthread/agents/*.md` source projects consistently
to all three agent hosts (`.claude/agents/*.md`, `.codex/agents/*.toml`,
`.agents/agents/*.md`) through the shared per-tier model table, that
`--check-agents` is genuinely read-only, and that `--sync-agents` is a
narrow, targeted re-run of only the fleet producer rather than the full
asset installer.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import project_fixture, run_aw

CASE_ID = "aw-core-client-three-host-agent-fleet-projection"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "three-host-agent-fleet-projection"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-three-host-agent-fleet-projection"
)
ASSERTIONS = (
    "a real `aw new` bootstrap projects every one of the fixed aw-* fleet "
    "agents to all three hosts (`.claude/agents/*.md`, "
    "`.codex/agents/*.toml`, `.agents/agents/*.md`) from the one canonical "
    "template source, and the per-host rendered model differs by the "
    "agent's declared model_tier -- the `top`-tier aw-ec-reviewer and "
    "`cheap`-tier aw-hw-filler resolve to different models on every one "
    "of the three hosts, proving the shared model-tier table actually "
    "drives per-host rendering rather than a single hardcoded model",
    "`aw new --check-agents` on a clean fleet reports success and leaves "
    "the byte content of all 15 projected files (5 agents x 3 hosts) "
    "completely unchanged, and after deliberately deleting one host's "
    "rendered file to introduce drift, a second `--check-agents` run "
    "exits non-zero naming exactly that missing file and host while "
    "still leaving it un-recreated -- the check only ever reports, never "
    "writes",
    "`aw new --sync-agents` -- the narrow producer re-run flag -- both "
    "recreates the deliberately deleted file with correct three-host "
    "content and leaves the full asset installer's other output "
    "(`aw.toml`) byte-for-byte untouched, proving it re-runs only the "
    "fleet producer rather than the full greenfield installer",
)

_PROJECT_NAME = "fleet-demo"
_HOSTS = (
    (".claude/agents", "md"),
    (".codex/agents", "toml"),
    (".agents/agents", "md"),
)
_TOP_TIER_AGENT = "aw-ec-reviewer"
_CHEAP_TIER_AGENT = "aw-hw-filler"
_ALL_AGENTS = ("aw-dev", "aw-td-writer", "aw-ec-writer", "aw-ec-reviewer", "aw-hw-filler")


def _fleet_snapshot(project_dir: Path) -> dict[str, str]:
    snapshot: dict[str, str] = {}
    for host_dir, ext in _HOSTS:
        for agent in _ALL_AGENTS:
            path = project_dir / host_dir / f"{agent}.{ext}"
            snapshot[str(path.relative_to(project_dir))] = path.read_text(encoding="utf-8")
    return snapshot


def verify() -> list[str]:
    with project_fixture() as root:
        project_dir = root / _PROJECT_NAME

        created = run_aw(root, "new", _PROJECT_NAME)
        assert created.returncode == 0, created

        # -- all three hosts, model differs by tier --------------------------
        claude_top = (project_dir / ".claude/agents" / f"{_TOP_TIER_AGENT}.md").read_text(
            encoding="utf-8"
        )
        claude_cheap = (project_dir / ".claude/agents" / f"{_CHEAP_TIER_AGENT}.md").read_text(
            encoding="utf-8"
        )
        assert "model: opus" in claude_top, claude_top
        assert "model: haiku" in claude_cheap, claude_cheap

        codex_top = (project_dir / ".codex/agents" / f"{_TOP_TIER_AGENT}.toml").read_text(
            encoding="utf-8"
        )
        codex_cheap = (project_dir / ".codex/agents" / f"{_CHEAP_TIER_AGENT}.toml").read_text(
            encoding="utf-8"
        )
        assert 'model = "gpt-5.6-sol"' in codex_top, codex_top
        assert 'model = "gpt-5.6-luna"' in codex_cheap, codex_cheap

        agy_top = (project_dir / ".agents/agents" / f"{_TOP_TIER_AGENT}.md").read_text(
            encoding="utf-8"
        )
        agy_cheap = (project_dir / ".agents/agents" / f"{_CHEAP_TIER_AGENT}.md").read_text(
            encoding="utf-8"
        )
        assert "model: Gemini 3.1 Pro (High)" in agy_top, agy_top
        assert "model: Gemini 3.6 Flash (Medium)" in agy_cheap, agy_cheap

        # -- --check-agents is read-only on a clean fleet --------------------
        before = _fleet_snapshot(project_dir)
        clean_check = run_aw(root, "new", _PROJECT_NAME, "--check-agents")
        assert clean_check.returncode == 0, clean_check
        assert "clean on all three hosts" in clean_check.stdout, clean_check.stdout
        after = _fleet_snapshot(project_dir)
        assert before == after, "‑-check-agents must not mutate any projected file"

        # -- introduce drift, --check-agents reports without remediating -----
        drifted_path = project_dir / ".agents/agents" / f"{_CHEAP_TIER_AGENT}.md"
        drifted_path.unlink()
        drift_check = run_aw(root, "new", _PROJECT_NAME, "--check-agents", expect_success=False)
        assert drift_check.returncode != 0, drift_check
        combined = drift_check.stdout + drift_check.stderr
        assert "[agy]" in combined, combined
        assert f"{_CHEAP_TIER_AGENT}.md" in combined, combined
        assert "missing" in combined, combined
        assert "--sync-agents" in combined, combined
        assert not drifted_path.exists(), "a read-only check must never recreate a missing file"

        # -- --sync-agents is the narrow producer re-run --------------------
        aw_toml_path = project_dir / "aw.toml"
        aw_toml_before = aw_toml_path.read_text(encoding="utf-8")

        synced = run_aw(root, "new", _PROJECT_NAME, "--sync-agents")
        assert synced.returncode == 0, synced
        assert "agent fleet synced" in synced.stdout, synced.stdout

        assert drifted_path.exists(), "--sync-agents must recreate the missing projection"
        restored = drifted_path.read_text(encoding="utf-8")
        assert "Gemini 3.6 Flash (Medium)" in restored, restored
        assert aw_toml_path.read_text(encoding="utf-8") == aw_toml_before, (
            "--sync-agents must bypass the full asset installer and leave aw.toml untouched"
        )

        final_check = run_aw(root, "new", _PROJECT_NAME, "--check-agents")
        assert final_check.returncode == 0, final_check
        assert "clean on all three hosts" in final_check.stdout, final_check.stdout

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
