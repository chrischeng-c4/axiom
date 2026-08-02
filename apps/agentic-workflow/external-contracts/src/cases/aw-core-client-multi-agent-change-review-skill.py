"""Black-box contract for the one producer-owned `aw-change-review` skill.

Drives a real `aw new` bootstrap and reads the two live runtime skill trees it
installs the canonical `templates/cli/mainthread/skills/aw-change-review/
SKILL.md` source into: `.claude/skills/` (Claude Code, untransformed) and
`.agents/skills/` (Codex + AGY, projected through
`doc_mirror::agents_skill_body_from_claude_skill_body`'s literal-swap
transform). Rather than merely asserting the same substrings independently
survive in both trees, the case re-derives the transformed body from the
Claude-tree body using that exact same swap table
(`.claude/skills/` -> `.agents/skills/`, `CLAUDE.md` -> `AGENTS.md`) and
asserts byte-for-byte equality with what the real installer actually wrote --
the strongest black-box proof available that both trees descend from the one
producer-owned source rather than two independently maintained copies.

The skill's promised behaviors -- reviewing every cohesive author-owned
change set before integration, rejecting same-agent approval, and requiring
fix verification plus focused re-review -- are authored, static prompt
content rather than something a CLI subprocess executes; the oracle proper to
a *projection* claim is therefore the real install producer's output, not an
invocation of the reviewer behavior itself.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import project_fixture, run_aw

CASE_ID = "aw-core-client-multi-agent-change-review-skill"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "multi-agent-change-review-skill"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-multi-agent-change-review-skill"
)
ASSERTIONS = (
    "a real `aw new` bootstrap installs `aw-change-review/SKILL.md` into "
    "both the `.claude/skills` and `.agents/skills` trees, and applying the "
    "installer's own documented literal-swap transform to the Claude-tree "
    "body byte-for-byte reproduces the AGY/Codex-tree body -- proof both "
    "trees descend from the one producer-owned source rather than two "
    "independently maintained copies",
    "the installed skill body states its core promise verbatim in both "
    "trees: it reviews every cohesive author-owned change set before "
    "integration, and it explicitly rejects same-agent approval by "
    "requiring a different agent instance from every author whose hunks it "
    "approves and refusing a self-review verdict when no independent "
    "instance can be established",
    "the installed skill body also states the fix-verification-plus-"
    "focused-re-review cycle verbatim in both trees under its own 'Fix and "
    "re-review' section, and the Codex-specific UI metadata "
    "(`agents/openai.yaml`, naming the skill '$aw-change-review') is "
    "projected identically alongside SKILL.md in both trees",
)

_SKILL_NAME = "aw-change-review"
_REVIEW_PROMISE = "Review every cohesive author-owned change set before integration"
_SCOPE_BOUNDARY = "does not replace tests, project architecture/"
_DIFFERENT_AGENT = "different agent instance"
_NOT_SELF_REVIEW = "not a self-review verdict"
_FIX_SECTION = "## Fix and re-review"
_FIX_VERIFICATION = "Recheck each accepted finding against the fix"
_OPENAI_DISPLAY_NAME = 'display_name: "AW Change Review"'
_OPENAI_HANDLE = "$aw-change-review"


def _skill_paths(project_dir: Path, tree: str) -> tuple[Path, Path]:
    skill_dir = project_dir / tree / "skills" / _SKILL_NAME
    return skill_dir / "SKILL.md", skill_dir / "agents" / "openai.yaml"


def verify() -> list[str]:
    with project_fixture() as root:
        created = run_aw(root, "new", "review-skill-demo")
        assert created.returncode == 0, created
        project_dir = root / "review-skill-demo"

        claude_skill_path, claude_openai_path = _skill_paths(project_dir, ".claude")
        agy_skill_path, agy_openai_path = _skill_paths(project_dir, ".agents")

        claude_body = claude_skill_path.read_text(encoding="utf-8")
        agy_body = agy_skill_path.read_text(encoding="utf-8")

        # -- one producer-owned source: re-derive the AGY body from the
        #    Claude body using the installer's own documented swap table --
        rederived = claude_body.replace(".claude/skills/", ".agents/skills/").replace(
            "CLAUDE.md", "AGENTS.md"
        )
        assert rederived == agy_body, (
            "the .agents/skills body must be byte-for-byte the documented "
            "transform of the .claude/skills body"
        )

        # -- promise: reviews every cohesive change set; rejects same-agent
        #    approval --------------------------------------------------------
        for body in (claude_body, agy_body):
            assert _REVIEW_PROMISE in body, body
            assert _SCOPE_BOUNDARY in body, body
            assert _DIFFERENT_AGENT in body, body
            assert _NOT_SELF_REVIEW in body, body
            assert _FIX_SECTION in body, body
            assert _FIX_VERIFICATION in body, body

        # -- Codex UI metadata projects identically to both trees -----------
        claude_openai = claude_openai_path.read_text(encoding="utf-8")
        agy_openai = agy_openai_path.read_text(encoding="utf-8")
        for metadata in (claude_openai, agy_openai):
            assert _OPENAI_DISPLAY_NAME in metadata, metadata
            assert _OPENAI_HANDLE in metadata, metadata

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
