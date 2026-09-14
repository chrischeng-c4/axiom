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
    "ask-user", "e2e-for", "grill-release", "impl-for", "prepare-goal",
    "review", "test-for", "product-ideate", "product-plan", "product-deliver",
    "next-step", "follow-next-step", "approve-next-step",
)


SCRIPTS_REL = Path("apps/aw/src/aw/scripts")


def fixture(root: Path) -> None:
    for runtime in (".agents", ".claude"):
        for skill in SKILLS:
            target = root / runtime / "skills" / skill
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copytree(REPO / runtime / "skills" / skill, target)
    (root / ".claude").mkdir(exist_ok=True)
    shutil.copy2(REPO / ".claude" / "settings.json", root / ".claude" / "settings.json")
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
    (root / ".agents/skills/ask-user/SKILL.md").unlink()


def drift_pair(root: Path) -> None:
    path = root / ".agents/skills/e2e-for/SKILL.md"
    path.write_text(path.read_text(encoding="utf-8") + "\nDrift.\n", encoding="utf-8")


def restore_issue_epic(root: Path) -> None:
    for runtime in (".agents", ".claude"):
        path = root / runtime / "skills/e2e-for/SKILL.md"
        text = path.read_text(encoding="utf-8")
        text = text.replace("controller resolves one eligible behavior queue head",
                            "aw epic create", 1)
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


def remove_legacy_visibility(root: Path) -> None:
    path = root / ".agents/skills/ask-user/agents/openai.yaml"
    path.write_text("policy:\n  allow_implicit_invocation: true\n", encoding="utf-8")


def remove_claude_override(root: Path) -> None:
    path = root / ".claude/settings.json"
    text = path.read_text(encoding="utf-8")
    text = text.replace('    "ask-user": "user-invocable-only"',
                        '    "ask-user": "enabled"', 1)
    path.write_text(text, encoding="utf-8")


def make_legacy_implicit(root: Path) -> None:
    for runtime in (".agents", ".claude"):
        path = root / runtime / "skills/grill-release/SKILL.md"
        text = path.read_text(encoding="utf-8")
        text = text.replace("Never invoke this skill implicitly.",
                            "This skill may run implicitly.", 1)
        path.write_text(text, encoding="utf-8")


def remove_fresh_final_qa(root: Path) -> None:
    for runtime in (".agents", ".claude"):
        path = root / runtime / "skills/product-deliver/SKILL.md"
        text = path.read_text(encoding="utf-8")
        text = text.replace("fresh `<p>-qa` instance", "same QA instance")
        path.write_text(text, encoding="utf-8")


def make_integration_default(root: Path) -> None:
    for runtime in (".agents", ".claude"):
        path = root / runtime / "skills/product-deliver/SKILL.md"
        text = path.read_text(encoding="utf-8")
        text = text.replace("Integration QA only for a cross-project scope",
                            "Integration QA for every scope", 1)
        path.write_text(text, encoding="utf-8")


def remove_shortcut_frontmatter(root: Path) -> None:
    for runtime in (".agents", ".claude"):
        path = root / runtime / "skills/next-step/SKILL.md"
        text = path.read_text(encoding="utf-8")
        path.write_text(text.replace("name: next-step", "name: stale-step", 1), encoding="utf-8")


def drift_shortcut_mirror(root: Path) -> None:
    path = root / ".agents/skills/approve-next-step/SKILL.md"
    path.write_text(path.read_text(encoding="utf-8") + "\nDrift.\n", encoding="utf-8")


def bypass_plan_digest(root: Path) -> None:
    path = root / "apps/aw/src/aw/scripts/release_plan.py"
    text = path.read_text(encoding="utf-8")
    text = text.replace("if sha != args.approved_digest:", "if False:", 1)
    path.write_text(text, encoding="utf-8")


def change_release_plan_schema(root: Path) -> None:
    path = root / "apps/aw/src/aw/scripts/release_plan.py"
    text = path.read_text(encoding="utf-8")
    text = text.replace('SCHEMA = "release-plan-v1"',
                        'SCHEMA = "release-plan-v2"', 1)
    path.write_text(text, encoding="utf-8")


def conditionally_reassign_release_plan_schema(root: Path) -> None:
    path = root / "apps/aw/src/aw/scripts/release_plan.py"
    text = path.read_text(encoding="utf-8")
    text = text.replace(
        'SCHEMA = "release-plan-v1"',
        'SCHEMA = "release-plan-v1"\nif True:\n    SCHEMA = "release-plan-v2"',
        1,
    )
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
             "FAIL ask-user: Codex SKILL.md exists"),
        case("pair drift", drift_pair,
             "FAIL e2e-for: mirror bytes match"),
        case("legacy issue-epic writer", restore_issue_epic,
             "FAIL e2e-for: has no legacy issue-epic writer"),
        case("Codex legacy visibility is enabled", remove_legacy_visibility,
             "FAIL ask-user: Codex disables implicit invocation"),
        case("Claude legacy visibility is enabled", remove_claude_override,
             "FAIL ask-user: Claude is user-invocable only"),
        case("legacy skill is implicitly callable", make_legacy_implicit,
             "FAIL grill-release: says it is explicit-only"),
        case("delivery reuses final QA", remove_fresh_final_qa,
             "FAIL product-deliver: carries routing contract `fresh `<p>-qa` instance`"),
        case("delivery makes Integration QA default", make_integration_default,
             "FAIL product-deliver: carries routing contract "
             "`Integration QA only for a cross-project scope`"),
        case("conversation shortcut frontmatter drifts", remove_shortcut_frontmatter,
             "FAIL next-step: frontmatter name matches"),
        case("conversation shortcut mirror drifts", drift_shortcut_mirror,
             "FAIL approve-next-step: mirror bytes match"),
        case("missing Milestone queue-head verb", remove_next_verb,
             "FAIL milestone.py exposes `next`"),
        case("Milestone default bump changes", change_default_milestone_bump,
             "FAIL milestone.py defaults new release Milestones to a minor bump"),
        case("release plan ignores approved digest", bypass_plan_digest,
             "FAIL release_plan.py keeps validate read-only and apply approval-bound"),
        case("release plan schema changes", change_release_plan_schema,
             "FAIL release_plan.py has frozen schema and verbs"),
        case("release plan schema is reassigned in a branch",
             conditionally_reassign_release_plan_schema,
             "FAIL release_plan.py has frozen schema and verbs"),
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
