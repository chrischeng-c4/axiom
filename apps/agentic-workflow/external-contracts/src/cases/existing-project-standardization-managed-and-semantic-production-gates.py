"""Black-box contract for managed/semantic production-gate prioritization (#3309)."""

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import run_aw

CASE_ID = "existing-project-standardization-managed-and-semantic-production-gates"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "managed-and-semantic-production-gates"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case existing-project-standardization-managed-and-semantic-production-gates"
)
ASSERTIONS = (
    "given two Python source files where one has no semantic TD Changes-section "
    "coverage at all and the other is TD-covered but still carries a source "
    "HANDWRITE marker (itself a competing, lower-priority generator-primitive "
    "gap candidate because every covered-and-handwritten Python file is "
    "generator-promotable), aw health --project <p> blockers's global blocker "
    "list names the uncovered file specifically -- the literal "
    "'next semantic gap: src/z.py semantic_td_missing' -- proving the "
    "missing-TD gap is selected and surfaced ahead of the coexisting "
    "generator-primitive gap rather than either being silently dropped or the "
    "two being reported ambiguously",
    "the same blockers section reports the managed/semantic coverage fraction "
    "directly in prose -- the literal 'semantic TD coverage incomplete: 1/2' "
    "-- matching the exact covered-vs-total source-file count, proving the "
    "managed and semantic production gates compute real per-file coverage "
    "rather than a placeholder readiness flag",
)

_HANDWRITE_TEMPLATE = (
    '# <HANDWRITE gap="g" tracker="t" reason="r">\n'
    "def {name}():\n"
    "    return {value}\n"
    "# </HANDWRITE>\n"
)


def _git(root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run(
        ["git", *args], cwd=root, capture_output=True, text=True, check=False
    )
    if completed.returncode != 0:
        raise AssertionError(f"git {' '.join(args)} failed: {completed.stderr}")
    return completed


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-ec-semantic-gap-") as raw_root:
        root = Path(raw_root)
        (root / "aw.toml").write_text(
            "[agentic_workflow.workspace]\n"
            'mode = "in_place"\n\n'
            "[agentic_workflow.issue_platform]\n"
            'type = "local"\n\n'
            "[[projects]]\n"
            'name = "demo"\n'
            'label = "app:demo"\n'
            'path = "."\n'
            'tech_design_path = "tech-design"\n\n'
            "[[projects.workspaces]]\n"
            'name = "demo"\n'
            'paths = ["**"]\n'
            'target = "rust"\n',
            encoding="utf-8",
        )
        (root / "src").mkdir(parents=True)
        (root / "tech-design/features").mkdir(parents=True)
        # `a.py` is semantic-TD-covered (below) but still HANDWRITE-marked, so
        # it is also a generator-primitive-promotion candidate.
        (root / "src/a.py").write_text(
            _HANDWRITE_TEMPLATE.format(name="covered", value=1), encoding="utf-8"
        )
        # `z.py` has no TD Changes-section coverage at all.
        (root / "src/z.py").write_text(
            _HANDWRITE_TEMPLATE.format(name="uncovered", value=2), encoding="utf-8"
        )
        (root / "tech-design/features/a.md").write_text(
            "---\nid: a\ntype: semantic\n---\n\n"
            "## Changes\n```yaml\nchanges:\n  - path: src/a.py\n    action: modify\n```\n",
            encoding="utf-8",
        )
        _git(root, "init")
        _git(root, "config", "user.email", "fixture@example.com")
        _git(root, "config", "user.name", "Fixture")
        _git(root, "add", "-A")
        _git(root, "commit", "-m", "fixture")

        blockers = run_aw(root, "health", "--project", "demo", "blockers", expect_success=False)
        combined = blockers.stdout + blockers.stderr
        assert "next semantic gap: src/z.py semantic_td_missing" in combined, combined
        assert "semantic TD coverage incomplete: 1/2" in combined, combined

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
