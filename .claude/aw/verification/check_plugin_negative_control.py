#!/usr/bin/env python3
"""Prove that check_plugin.py refuses typed skill and script contract drift."""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
CHECKER = HERE / "check_plugin.py"
SKILLS = (
    "aw-ask-user",
    "aw-build",
    "aw-e2e-for",
    "aw-grill-me-to-meta",
    "aw-grill-meta-to-milestone",
    "aw-grill-milestone-to-issue",
    "aw-impl-for",
    "aw-prepare-goal",
    "aw-review",
    "aw-test-for",
)


SCRIPTS_REL = Path("apps/aw/src/aw/scripts")


def fixture(root: Path) -> None:
    for runtime in (".agents", ".claude"):
        for skill in SKILLS:
            target = root / runtime / "skills" / skill
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copytree(REPO / runtime / "skills" / skill, target)
    scripts = root / SCRIPTS_REL
    scripts.mkdir(parents=True)
    for source in (REPO / SCRIPTS_REL).glob("*.py"):
        shutil.copy2(source, scripts / source.name)


def run(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        (sys.executable, str(CHECKER), "--repo", str(root)),
        capture_output=True,
        text=True,
    )


def case(name: str, mutate, expected: str) -> bool:
    with tempfile.TemporaryDirectory(prefix="aw-plugin-control-") as raw:
        root = Path(raw)
        fixture(root)
        baseline = run(root)
        if baseline.returncode != 0:
            print(f"FAIL {name}: baseline was red")
            return False
        mutate(root)
        result = run(root)
        ok = result.returncode != 0 and expected in result.stdout
        print(f"{'PASS' if ok else 'FAIL'} {name}")
        if not ok:
            print(result.stdout)
        return ok


def remove_skill(root: Path) -> None:
    (root / ".agents/skills/aw-ask-user/SKILL.md").unlink()


def drift_pair(root: Path) -> None:
    path = root / ".agents/skills/aw-e2e-for/SKILL.md"
    path.write_text(path.read_text(encoding="utf-8") + "\nDrift.\n", encoding="utf-8")


def restore_issue_epic(root: Path) -> None:
    for runtime in (".agents", ".claude"):
        path = root / runtime / "skills/aw-e2e-for/SKILL.md"
        text = path.read_text(encoding="utf-8")
        text = text.replace("aw milestone next", "aw epic create")
        path.write_text(text, encoding="utf-8")


def remove_next_verb(root: Path) -> None:
    path = root / "apps/aw/src/aw/scripts/milestone.py"
    text = path.read_text(encoding="utf-8")
    text = text.replace('sub.add_parser("next")', 'sub.add_parser("queue")', 1)
    path.write_text(text, encoding="utf-8")


def change_default_milestone_bump(root: Path) -> None:
    path = root / "apps/aw/src/aw/scripts/milestone.py"
    text = path.read_text(encoding="utf-8")
    text = text.replace('DEFAULT_BUMP = "minor"', 'DEFAULT_BUMP = "patch"', 1)
    path.write_text(text, encoding="utf-8")


def remove_plan_first(root: Path) -> None:
    for runtime in (".agents", ".claude"):
        path = root / runtime / "skills/aw-grill-me-to-meta/SKILL.md"
        text = path.read_text(encoding="utf-8")
        text = text.replace("1. Enter Plan mode", "1. Read repository context", 1)
        path.write_text(text, encoding="utf-8")


def make_plan_open(root: Path) -> None:
    for runtime in (".agents", ".claude"):
        path = root / runtime / "skills/aw-grill-meta-to-milestone/SKILL.md"
        text = path.read_text(encoding="utf-8")
        text = text.replace("Stop if\n   the runtime cannot confirm Plan mode.",
                            "Continue when the runtime cannot confirm Plan mode.", 1)
        path.write_text(text, encoding="utf-8")


def erase_behavior_flow(root: Path) -> None:
    path = root / "apps/aw/src/aw/scripts/wi_types.py"
    text = path.read_text(encoding="utf-8")
    text = text.replace('BEHAVIOR_TYPES = ("feat", "fix", "perf")',
                        "BEHAVIOR_TYPES = ()", 1)
    path.write_text(text, encoding="utf-8")


def remove_migration_apply(root: Path) -> None:
    path = root / "apps/aw/src/aw/scripts/type_migration.py"
    text = path.read_text(encoding="utf-8")
    text = text.replace('mode.add_argument("--apply"',
                        'mode.add_argument("--migrate"', 1)
    path.write_text(text, encoding="utf-8")


def weaken_maint_record(root: Path) -> None:
    path = root / "apps/aw/src/aw/scripts/maint.py"
    text = path.read_text(encoding="utf-8")
    text = text.replace('command.add_argument("--output-file", required=True)',
                        'command.add_argument("--result-file", required=True)', 1)
    path.write_text(text, encoding="utf-8")


def bypass_lifecycle_close(root: Path) -> None:
    path = root / "apps/aw/src/aw/scripts/change.py"
    text = path.read_text(encoding="utf-8")
    start = text.index("def cmd_close(args)")
    prefix, close = text[:start], text[start:]
    close = close.replace("required = wi_types.required_legs(wi_type.name)",
                          "required = ()", 1)
    path.write_text(prefix + close, encoding="utf-8")


def main() -> int:
    rows = (
        case("missing Codex mirror", remove_skill,
             "FAIL aw-ask-user: Codex SKILL.md exists"),
        case("pair drift", drift_pair,
             "FAIL aw-e2e-for: mirror bytes match"),
        case("legacy issue-epic writer", restore_issue_epic,
             "FAIL aw-e2e-for: has no legacy issue-epic writer"),
        case("missing Milestone queue-head verb", remove_next_verb,
             "FAIL milestone.py exposes `next`"),
        case("Milestone default bump changes", change_default_milestone_bump,
             "FAIL milestone.py defaults new release Milestones to a minor bump"),
        case("grill skips Plan mode", remove_plan_first,
             "FAIL aw-grill-me-to-meta: first step enters Plan mode"),
        case("grill Plan mode is open", make_plan_open,
             "FAIL aw-grill-meta-to-milestone: Plan mode is fail-closed"),
        case("frozen behavior flow is erased", erase_behavior_flow,
             "FAIL wi_types.py owns the frozen delivery and intake vocabulary"),
        case("legacy migration loses apply", remove_migration_apply,
             "FAIL type_migration.py is the one-time legacy migration surface"),
        case("maintenance record loses output path", weaken_maint_record,
             "FAIL maint.py exposes safe record evidence"),
        case("delivery close bypasses lifecycle", bypass_lifecycle_close,
             "FAIL change.py closes only after its required lifecycle"),
    )
    if all(rows):
        print("\n=> GREEN: every planted defect was refused")
        return 0
    print("\n=> RED: at least one planted defect escaped")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
