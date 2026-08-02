"""Black-box contract for pruning the eight retired aw-* skills on install.

Drives two real `aw new` bootstraps against the same fixture project: the
first proves a fresh install never contains any of the eight named
lifecycle-superseded / external-model-helper skills; then, mirroring the
production Rust fixture `cli::init::tests::test_install_skills_prunes_1858_
retired_skills` exactly (plant a retired skill's `SKILL.md` on disk, re-run
the real install producer, assert it is gone), the case plants all eight
retired names directly on disk in both live skill trees -- simulating a
previous install made before they were retired -- and re-runs `aw new
--force` to prove the real producer prunes every one of them from both
`.claude/skills` and `.agents/skills` on every install, while leaving an
unrelated current skill (`aw-change-review`) intact.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import project_fixture, run_aw

CASE_ID = "aw-core-client-stale-skill-retirement"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "stale-skill-retirement"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-stale-skill-retirement"
)
ASSERTIONS = (
    "a fresh `aw new` bootstrap never installs any of the eight named "
    "lifecycle-superseded / external-model-helper aw-* skills "
    "(aw-release-patch, aw-cb-claim, aw-cb-fill, aw-td-create, "
    "aw-capability, aw-codex-review, aw-gemini-explore-codebase, "
    "aw-gemini-explore-specs) in either live skill tree -- they are "
    "genuinely removed from the current template set, not merely hidden",
    "planting all eight retired skill directories directly on disk in both "
    "`.claude/skills` and `.agents/skills` -- simulating a previous install "
    "made before they were retired -- and re-running the real `aw new "
    "--force` install producer prunes every one of the sixteen planted "
    "directories from both trees, matching the production Rust fixture's "
    "plant-then-reinstall recipe through the compiled CLI instead of the "
    "internal Rust function",
    "the same reinstall that prunes all sixteen retired directories leaves "
    "an unrelated current skill (`aw-change-review`) intact and readable in "
    "both trees afterward, proving pruning targets exactly the eight named "
    "retired directories rather than clearing the skill trees wholesale",
)

_RETIRED_SKILLS = (
    "aw-release-patch",
    "aw-cb-claim",
    "aw-cb-fill",
    "aw-td-create",
    "aw-capability",
    "aw-codex-review",
    "aw-gemini-explore-codebase",
    "aw-gemini-explore-specs",
)
_TREES = (".claude", ".agents")
_CURRENT_SKILL = "aw-change-review"


def _plant_legacy_skill(project_dir: Path, tree: str, name: str) -> Path:
    skill_dir = project_dir / tree / "skills" / name
    skill_dir.mkdir(parents=True, exist_ok=True)
    (skill_dir / "SKILL.md").write_text("# legacy\n", encoding="utf-8")
    return skill_dir


def verify() -> list[str]:
    with project_fixture() as root:
        created = run_aw(root, "new", "skill-retirement-demo")
        assert created.returncode == 0, created
        project_dir = root / "skill-retirement-demo"

        # -- phase 1: a fresh install never contains any retired skill ------
        for tree in _TREES:
            for retired in _RETIRED_SKILLS:
                path = project_dir / tree / "skills" / retired
                assert not path.exists(), f"{path} must not exist on a fresh install"

        # -- phase 2: plant all eight retired names in both live trees ------
        planted: list[Path] = []
        for tree in _TREES:
            for retired in _RETIRED_SKILLS:
                planted.append(_plant_legacy_skill(project_dir, tree, retired))
        assert len(planted) == 16, planted
        for path in planted:
            assert path.is_dir(), path
            assert (path / "SKILL.md").exists(), path

        # -- phase 3: reinstall and assert every planted directory is pruned
        reinstalled = run_aw(root, "new", "skill-retirement-demo", "--force")
        assert reinstalled.returncode == 0, reinstalled

        still_present = [path for path in planted if path.exists()]
        assert still_present == [], (
            f"the real install producer must prune every retired skill directory, "
            f"still present: {still_present}"
        )

        # -- phase 4: an unrelated current skill survives the same reinstall
        for tree in _TREES:
            current_skill_md = project_dir / tree / "skills" / _CURRENT_SKILL / "SKILL.md"
            assert current_skill_md.exists(), (
                f"{current_skill_md} must survive pruning of unrelated retired skills"
            )
            assert current_skill_md.read_text(encoding="utf-8").strip() != "", current_skill_md

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
