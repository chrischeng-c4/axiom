"""Black-box contract for the aw-review skill's dual-tree projection (#3310)."""

from __future__ import annotations

import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import run_aw

CASE_ID = "existing-project-standardization-aw-review-skill-and-doc-projection"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "aw-review-skill-and-doc-projection"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case existing-project-standardization-aw-review-skill-and-doc-projection"
)
ASSERTIONS = (
    "aw new <name>, run against an empty target directory, writes an "
    "aw-review/SKILL.md file into both the .claude/skills/ and .agents/skills/ "
    "trees of the freshly bootstrapped project -- the same skill installer "
    "that projects every other aw-* skill -- proving aw-review is a first-class "
    "member of the greenfield skill set rather than a Claude-only or "
    "hand-maintained addition",
    "the installed aw-review/SKILL.md body, in both trees, states the literal "
    "invocation 'aw review --project' and names 'aw health' as the sibling "
    "readiness surface, proving the projected skill actually documents the real "
    "aw review CLI contract and the aw health/aw review ownership boundary "
    "rather than shipping a stale or generic placeholder",
)


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-ec-review-skill-") as raw_root:
        root = Path(raw_root)

        run_aw(root, "new", "demo")

        claude_skill = root / "demo" / ".claude" / "skills" / "aw-review" / "SKILL.md"
        agents_skill = root / "demo" / ".agents" / "skills" / "aw-review" / "SKILL.md"
        assert claude_skill.exists(), f"missing {claude_skill}"
        assert agents_skill.exists(), f"missing {agents_skill}"

        for skill_path in (claude_skill, agents_skill):
            content = skill_path.read_text(encoding="utf-8")
            assert "aw review --project" in content, (skill_path, content)
            assert "aw health" in content, (skill_path, content)

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
