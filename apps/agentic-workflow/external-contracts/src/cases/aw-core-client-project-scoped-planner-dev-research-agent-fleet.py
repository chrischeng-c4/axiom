"""Black-box contract for project-scoped planner/dev/research agent roles.

Drives the real `aw new --sync-agents` / `--check-agents` commands against a
fixture `aw.toml` whose registry mirrors the exact shape proven by the Rust
ground-truth test
`cli::init::tests::test_project_role_fleet_uses_registry_scope_and_model_matrix`
(direct `apps/*` projects, the top-level `projects/mamba`, a duplicate-named
non-`apps/*` project row, `projects/sift`, and a nested Mamba library), so the
same scope and model-matrix proof this capability names is exercised through
the compiled CLI end to end rather than only at the internal Rust-function
level.

Three things are proven: (1) only registry rows inside the narrow
`apps/*` + `projects/mamba` scope receive a `<project>-planner` /
`<project>-dev` / `<project>-research` trio -- the duplicate-named,
`projects/sift`, and nested-Mamba-library rows contribute nothing on any
host; (2) each role resolves through the same shared per-tier model table
already proven for the fixed aw-* fleet, differentiated by role rather than
one shared model, with the research role carrying read-only tooling; (3) the
fixed, top-tier `aw-ec-reviewer` fleet agent -- independent EC review -- is
unaffected by and coexists alongside the new project-scoped roles, and the
same read-only-check / narrow-sync producer that governs the fixed fleet
also detects and repairs drift in a project-scoped role file.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import project_fixture, run_aw

CASE_ID = "aw-core-client-project-scoped-planner-dev-research-agent-fleet"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "project-scoped-planner-dev-research-agent-fleet"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-project-scoped-planner-dev-research-agent-fleet"
)
ASSERTIONS = (
    "a registry mirroring direct apps/* projects, the top-level "
    "projects/mamba, a same-named non-apps/* duplicate, projects/sift, and "
    "a nested Mamba library produces a <project>-planner/-dev/-research "
    "trio on all three hosts only for the apps/* and projects/mamba rows "
    "-- the duplicate-named projects/meter row, projects/sift, and the "
    "nested pgkit library contribute zero files anywhere, proving the "
    "scope is genuinely narrow rather than every registered project",
    "each role resolves the same shared per-tier model table already "
    "proven for the fixed aw-* fleet, differentiated by role rather than "
    "one shared model: planner is sonnet/gpt-5.6-terra(xhigh)/Gemini 3.6 "
    "Flash (High) with workspace-write, dev is haiku/gpt-5.6-luna(medium)"
    "/Gemini 3.6 Flash (Medium), and research is opus/gpt-5.6-sol(max)"
    "/Gemini 3.1 Pro (High) with read-only tooling and no write access on "
    "every host",
    "the fixed, top-tier aw-ec-reviewer fleet agent -- independent EC "
    "review -- is still projected byte-sane and unaffected alongside the "
    "new project-scoped roles, and the same read-only --check-agents / "
    "narrow --sync-agents producer that governs the fixed fleet also "
    "detects and repairs drift in a project-scoped role file, proving "
    "both fleets are produced and verified by the one shared mechanism",
)

_REGISTRY_AW_TOML = """
[agentic_workflow.workspace]
mode = "in_place"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "cap"
path = "apps/cap"
[[projects.workspaces]]
paths = ["apps/cap/**"]
target = "rust"

[[projects]]
name = "mamba"
path = "projects/mamba"
[[projects.workspaces]]
paths = ["projects/mamba/**"]
target = "rust"

[[projects]]
name = "meter"
path = "apps/meter"
[[projects.workspaces]]
paths = ["apps/meter/**"]
target = "rust"

[[projects]]
name = "meter"
path = "projects/meter"
[[projects.workspaces]]
paths = ["projects/meter/**"]
target = "rust"

[[projects]]
name = "sift"
path = "projects/sift"
[[projects.workspaces]]
paths = ["projects/sift/**"]
target = "rust"

[[projects]]
name = "pg"
path = "projects/mamba/mambalibs/pgkit"
[[projects.workspaces]]
paths = ["projects/mamba/mambalibs/pgkit/**"]
target = "rust"
""".lstrip()

_QUALIFYING_PROJECTS = ("cap", "mamba", "meter")
_ROLES = ("planner", "dev", "research")
_HOSTS = (
    (".claude/agents", "md"),
    (".codex/agents", "toml"),
    (".agents/agents", "md"),
)
_EXCLUDED_NAMES = ("sift", "pg")


def _role_paths(project_dir: Path, project: str, role: str) -> list[Path]:
    return [project_dir / host_dir / f"{project}-{role}.{ext}" for host_dir, ext in _HOSTS]


def verify() -> list[str]:
    with project_fixture() as root:
        (root / "aw.toml").write_text(_REGISTRY_AW_TOML, encoding="utf-8")
        project_dir = root

        synced = run_aw(root, "new", "role-fleet-demo", "--path", ".", "--sync-agents")
        assert synced.returncode == 0, synced
        assert "agent fleet synced" in synced.stdout, synced.stdout

        # -- scope: only apps/* + projects/mamba rows get a role trio --------
        for project in _QUALIFYING_PROJECTS:
            for role in _ROLES:
                for path in _role_paths(project_dir, project, role):
                    assert path.exists(), f"expected {path} from a qualifying project row"

        for excluded in _EXCLUDED_NAMES:
            for role in _ROLES:
                for host_dir, ext in _HOSTS:
                    path = project_dir / host_dir / f"{excluded}-{role}.{ext}"
                    assert not path.exists(), (
                        f"{path} must not exist: {excluded} is excluded from project-role scope"
                    )

        # the duplicate-named projects/meter row must not have won: the
        # surviving meter-* files must carry the apps/meter body, not
        # projects/meter's.
        meter_planner_claude = (project_dir / ".claude/agents/meter-planner.md").read_text(
            encoding="utf-8"
        )
        assert "at `apps/meter`" in meter_planner_claude, meter_planner_claude
        assert "at `projects/meter`" not in meter_planner_claude, meter_planner_claude

        # -- shared per-tier model table, differentiated by role -------------
        cap_planner_claude = (project_dir / ".claude/agents/cap-planner.md").read_text(
            encoding="utf-8"
        )
        cap_dev_claude = (project_dir / ".claude/agents/cap-dev.md").read_text(encoding="utf-8")
        cap_research_claude = (project_dir / ".claude/agents/cap-research.md").read_text(
            encoding="utf-8"
        )
        assert "at `apps/cap`" in cap_planner_claude, cap_planner_claude
        assert "model: sonnet" in cap_planner_claude, cap_planner_claude
        assert "effort: xhigh" in cap_planner_claude, cap_planner_claude
        assert "model: haiku" in cap_dev_claude, cap_dev_claude
        assert "effort: medium" in cap_dev_claude, cap_dev_claude
        assert "model: opus" in cap_research_claude, cap_research_claude
        assert "effort: max" in cap_research_claude, cap_research_claude
        assert "tools: Read, Edit" not in cap_research_claude, cap_research_claude

        cap_planner_codex = (project_dir / ".codex/agents/cap-planner.toml").read_text(
            encoding="utf-8"
        )
        cap_dev_codex = (project_dir / ".codex/agents/cap-dev.toml").read_text(encoding="utf-8")
        cap_research_codex = (project_dir / ".codex/agents/cap-research.toml").read_text(
            encoding="utf-8"
        )
        assert 'model = "gpt-5.6-terra"' in cap_planner_codex, cap_planner_codex
        assert 'model_reasoning_effort = "xhigh"' in cap_planner_codex, cap_planner_codex
        assert 'sandbox_mode = "workspace-write"' in cap_planner_codex, cap_planner_codex
        assert 'model = "gpt-5.6-luna"' in cap_dev_codex, cap_dev_codex
        assert 'model_reasoning_effort = "medium"' in cap_dev_codex, cap_dev_codex
        assert 'model = "gpt-5.6-sol"' in cap_research_codex, cap_research_codex
        assert 'model_reasoning_effort = "max"' in cap_research_codex, cap_research_codex
        assert 'sandbox_mode = "read-only"' in cap_research_codex, cap_research_codex

        cap_planner_agy = (project_dir / ".agents/agents/cap-planner.md").read_text(
            encoding="utf-8"
        )
        cap_dev_agy = (project_dir / ".agents/agents/cap-dev.md").read_text(encoding="utf-8")
        cap_research_agy = (project_dir / ".agents/agents/cap-research.md").read_text(
            encoding="utf-8"
        )
        assert "model: Gemini 3.6 Flash (High)" in cap_planner_agy, cap_planner_agy
        assert "enable_write_tools: true" in cap_planner_agy, cap_planner_agy
        assert "model: Gemini 3.6 Flash (Medium)" in cap_dev_agy, cap_dev_agy
        assert "model: Gemini 3.1 Pro (High)" in cap_research_agy, cap_research_agy
        assert "enable_write_tools: false" in cap_research_agy, cap_research_agy

        # -- fixed aw-ec-reviewer fleet agent unaffected; shared check/sync --
        reviewer_claude = (project_dir / ".claude/agents/aw-ec-reviewer.md").read_text(
            encoding="utf-8"
        )
        assert "model: opus" in reviewer_claude, reviewer_claude

        clean_check = run_aw(root, "new", "role-fleet-demo", "--path", ".", "--check-agents")
        assert clean_check.returncode == 0, clean_check
        assert "clean on all three hosts" in clean_check.stdout, clean_check.stdout

        drifted_path = project_dir / ".agents/agents/cap-research.md"
        drifted_path.unlink()
        drift_check = run_aw(
            root,
            "new",
            "role-fleet-demo",
            "--path",
            ".",
            "--check-agents",
            expect_success=False,
        )
        assert drift_check.returncode != 0, drift_check
        combined = drift_check.stdout + drift_check.stderr
        assert "[agy]" in combined, combined
        assert "cap-research.md" in combined, combined
        assert "missing" in combined, combined
        assert not drifted_path.exists(), "a read-only check must never recreate a missing file"

        resynced = run_aw(root, "new", "role-fleet-demo", "--path", ".", "--sync-agents")
        assert resynced.returncode == 0, resynced
        assert drifted_path.exists(), "--sync-agents must recreate the missing role projection"
        assert "Gemini 3.1 Pro (High)" in drifted_path.read_text(encoding="utf-8")

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
