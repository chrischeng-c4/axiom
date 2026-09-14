#!/usr/bin/env python3
"""Check the exact byte-identical AW skill set and release-plan contract."""

from __future__ import annotations

import argparse
import ast
import json
import re
from pathlib import Path


LEGACY_SKILLS = (
    "ask-user", "e2e-for", "grill-release", "impl-for", "prepare-goal",
    "review", "test-for",
)
PRODUCT_SKILLS = ("product-ideate", "product-plan", "product-deliver")
SKILLS = LEGACY_SKILLS + PRODUCT_SKILLS
HEADINGS = ("## Goal", "## How", "## Never")
SCRIPTS = (
    "change.py",
    "e2e.py",
    "epic.py",
    "impl.py",
    "leg.py",
    "maint.py",
    "meta.py",
    "metadoc.py",
    "milestone.py",
    "type_migration.py",
    "wi_types.py",
    "wis.py",
    "workitem.py",
    "release_plan.py",
)
MILESTONE_VERBS = (
    "skeleton",
    "validate",
    "show",
    "children",
    "order",
    "next",
    "reconcile",
    "versions",
    "next-version",
    "create",
    "update",
    "close",
)
FRONTMATTER = re.compile(r"\A---\n(?P<body>.*?)\n---\n", re.S)
SCRIPT_NAME = re.compile(r"(?<![A-Za-z0-9_/-])([a-z][a-z0-9_-]*\.py)\b")
GH_WRITE = re.compile(r"\bgh\s+(?:issue|pr)\s+(?:create|edit|close|reopen|delete|comment)\b")
LEGACY_WRITE = re.compile(r"\b(?:epic\.py|aw epic)\s+(?:create|update|close)\b")
# The `aw` name is live again: `apps/aw` exposes the engine as typer groups,
# launched as `uv run --project apps/aw aw <group> ...`. A skill may therefore
# name `aw <group>` freely — what stays refused is a group outside the typer
# registry, which is exactly the shape of a retired `aw <verb>` (wi, ec, td...).
AW_GROUPS = (
    "change", "milestone", "e2e", "impl", "maint",
    "wis", "meta", "metadoc", "release-plan", "version",
)
AW_INVOCATION = re.compile(r"(?:`|\buv run --project apps/aw )aw\s+([a-z0-9-]+)")
LAUNCHER_PREFIX = "uv run --project apps/aw aw "


class Reporter:
    def __init__(self) -> None:
        self.failed: list[str] = []

    def check(self, label: str, ok: bool, detail: str = "") -> None:
        suffix = f" -- {detail}" if detail else ""
        print(f"{'PASS' if ok else 'FAIL'} {label}{suffix}")
        if not ok:
            self.failed.append(label)


def frontmatter(text: str) -> dict[str, str]:
    match = FRONTMATTER.search(text)
    if not match:
        return {}
    fields: dict[str, str] = {}
    for line in match.group("body").splitlines():
        key, found, value = line.partition(":")
        if found:
            fields[key.strip()] = value.strip()
    return fields


def string_constants(source: str) -> dict[str, str]:
    """Return literal module constants only when each name has one write.

    The all-tree write count refuses a later assignment hidden in a branch or
    function.  A caller can therefore treat a missing name as contract drift.
    """
    try:
        tree = ast.parse(source)
    except SyntaxError:
        return {}
    values: dict[str, str] = {}
    for node in tree.body:
        if not isinstance(node, ast.Assign) or len(node.targets) != 1:
            continue
        target = node.targets[0]
        if isinstance(target, ast.Name) and isinstance(node.value, ast.Constant) \
                and isinstance(node.value.value, str):
            values[target.id] = node.value.value
    writes: dict[str, int] = {name: 0 for name in values}
    for node in ast.walk(tree):
        if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)) \
                and node.id in writes:
            writes[node.id] += 1
    return {name: value for name, value in values.items() if writes[name] == 1}


def skill_directories(root: Path) -> list[str]:
    if not root.is_dir():
        return []
    return sorted(path.name for path in root.iterdir()
                  if path.is_dir() and (path / "SKILL.md").is_file())


def collect(repo: Path) -> Reporter:
    report = Reporter()
    codex_root = repo / ".agents" / "skills"
    claude_root = repo / ".claude" / "skills"
    scripts_root = repo / "apps" / "aw" / "src" / "aw" / "scripts"

    report.check("Codex skill root exists", codex_root.is_dir(), str(codex_root))
    report.check("Claude skill root exists", claude_root.is_dir(), str(claude_root))
    report.check("shared AW script root exists", scripts_root.is_dir(), str(scripts_root))
    codex_skills = skill_directories(codex_root)
    claude_skills = skill_directories(claude_root)
    report.check("Codex required product and legacy skills exist",
                 set(SKILLS).issubset(codex_skills), f"found={codex_skills}")
    report.check("Claude required product and legacy skills exist",
                 set(SKILLS).issubset(claude_skills), f"found={claude_skills}")
    report.check("Codex has no retired aw-prefixed skill directory",
                 not any(skill.startswith("aw-") for skill in codex_skills))
    report.check("Claude has no retired aw-prefixed skill directory",
                 not any(skill.startswith("aw-") for skill in claude_skills))

    bodies: dict[str, str] = {}
    for skill in SKILLS:
        codex = codex_root / skill / "SKILL.md"
        claude = claude_root / skill / "SKILL.md"
        report.check(f"{skill}: Codex SKILL.md exists", codex.is_file())
        report.check(f"{skill}: Claude SKILL.md exists", claude.is_file())
        if claude.is_file():
            bodies[skill] = claude.read_text(encoding="utf-8")
        if not codex.is_file() or not claude.is_file():
            continue
        codex_bytes = codex.read_bytes()
        claude_bytes = claude.read_bytes()
        report.check(f"{skill}: mirror bytes match", codex_bytes == claude_bytes)
        text = codex_bytes.decode("utf-8")
        bodies[skill] = text

        fields = frontmatter(text)
        report.check(f"{skill}: frontmatter is present", bool(fields))
        report.check(f"{skill}: frontmatter name matches",
                     fields.get("name") == skill,
                     f"found={fields.get('name')!r}")
        report.check(f"{skill}: uses portable frontmatter",
                     set(fields) == {"name", "description"},
                     f"keys={sorted(fields)}")
        for heading in HEADINGS:
            report.check(f"{skill}: carries {heading}",
                         re.search(rf"^{re.escape(heading)}\s*$", text, re.M) is not None)

        report.check(f"{skill}: has no retired plugin path",
                     "${CLAUDE_PLUGIN_ROOT}" not in text)
        unknown = sorted({group for group in AW_INVOCATION.findall(text)
                          if group not in AW_GROUPS})
        report.check(f"{skill}: has no retired `aw <verb>` command",
                     not unknown, f"unknown groups={unknown}" if unknown else "")
        launchers = [line.strip() for line in text.splitlines()
                     if "uv run" in line]
        off_form = [line for line in launchers
                    if LAUNCHER_PREFIX not in line]
        report.check(f"{skill}: every `uv run` line uses the apps/aw launcher",
                     not off_form, f"off-form={off_form}" if off_form else "")
        report.check(f"{skill}: has no direct GitHub write", not GH_WRITE.search(text))
        # A bare `epic.py` mention is already refused below: the SCRIPT_NAME
        # sweep requires every named script to exist under the scripts root,
        # and `epic.py` is not there.
        report.check(f"{skill}: has no legacy issue-epic writer",
                     not LEGACY_WRITE.search(text)
                     and "--label epic:" not in text)
        report.check(f"{skill}: carries no private scripts copy",
                     not (codex.parent / "scripts").exists()
                     and not (claude.parent / "scripts").exists())

        for name in sorted(set(SCRIPT_NAME.findall(text))):
            report.check(f"{skill}: named script {name} exists",
                         (scripts_root / name).is_file())

    settings_path = repo / ".claude" / "settings.json"
    try:
        settings = json.loads(settings_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        settings = {}
    overrides = settings.get("skillOverrides", {})
    report.check("Claude settings parse", isinstance(overrides, dict))
    for skill in LEGACY_SKILLS:
        codex_metadata = codex_root / skill / "agents" / "openai.yaml"
        report.check(f"{skill}: Codex disables implicit invocation",
                     codex_metadata.read_text(encoding="utf-8")
                     == "policy:\n  allow_implicit_invocation: false\n"
                     if codex_metadata.is_file() else False)
        report.check(f"{skill}: Claude is user-invocable only",
                     overrides.get(skill) == "user-invocable-only")
        report.check(f"{skill}: says it is explicit-only",
                     "Never invoke this skill implicitly." in bodies.get(skill, ""))
    for skill in PRODUCT_SKILLS:
        report.check(f"{skill}: Claude does not override default visibility",
                     skill not in overrides)

    for name in SCRIPTS:
        report.check(f"{name} exists", (scripts_root / name).is_file())

    required_phrases = {
        "ask-user": ("explicitly requested scope", "changes no project state"),
        "prepare-goal": ("legacy e2e then impl legs", "all AW state authority"),
        "e2e-for": ("controller resolves one eligible behavior queue head",
                    "QA has no Git, tracker, lifecycle, or close authority"),
        "impl-for": ("controller resolves the current queue head",
                     "Dev cannot weaken or replace QA's e2e test"),
        "test-for": ("fresh QA instance", "complete, unfiltered gate"),
        "review": ("read-only project review", "controller reads the summary"),
        "grill-release": ("validate a plan read-only",
                           "exact plan digest the human explicitly approved"),
        "product-ideate": ("owning PM", "return a short option table"),
        "product-plan": ("PM owns product docs", "TL drafts tasks and does not write source",
                         "controller owns Git and tracker writes"),
        "product-deliver": ("e2e-only", "impl-only", "test-only", "read-only-review",
                            "fresh `<p>-qa` instance", "Integration QA only for a cross-project scope",
                            "controller owns Git, tracker changes, scope changes, and final acceptance"),
    }
    for skill, phrases in required_phrases.items():
        text = bodies.get(skill, "")
        for phrase in phrases:
            report.check(f"{skill}: carries routing contract `{phrase}`",
                         phrase in text)

    retired = tuple("aw-grill-" + suffix for suffix in (
        "me-to-meta", "meta-to-milestone", "milestone-to-issue",
    ))
    for skill, text in bodies.items():
        report.check(f"{skill}: has no retired grill name", not any(name in text for name in retired))

    release_plan_path = scripts_root / "release_plan.py"
    release_plan_source = release_plan_path.read_text(encoding="utf-8") if release_plan_path.is_file() else ""
    release_plan_constants = string_constants(release_plan_source)
    report.check("release_plan.py has frozen schema and verbs",
                 release_plan_constants.get("SCHEMA") == "release-plan-v1"
                 and release_plan_constants.get("RECEIPT_SCHEMA")
                 == "release-plan-receipt-v1"
                 and '"validate"' in release_plan_source and '"apply"' in release_plan_source
                 and '"resume"' in release_plan_source)
    report.check("release_plan.py keeps validate read-only and apply approval-bound",
                 'stdin_ok=True' in release_plan_source and 'stdin_ok=False' in release_plan_source
                 and 'if sha != args.approved_digest:' in release_plan_source
                 and 'approved digest does not match canonical plan' in release_plan_source)
    report.check("release_plan.py binds baselines and resumable write identities",
                 '"tracker_baseline"' in release_plan_source
                 and "def _tracker_snapshot" in release_plan_source
                 and "def _write_receipt" in release_plan_source
                 and "def _recover_pending_issue" in release_plan_source
                 and "def _verify_complete" in release_plan_source
                 and "_after_write(\"milestone\")" in release_plan_source
                 and "_after_write(\"meta_commit\")" in release_plan_source
                 and "_after_write(\"finalize\")" in release_plan_source)

    milestone_path = scripts_root / "milestone.py"
    milestone_source = milestone_path.read_text(encoding="utf-8") if milestone_path.is_file() else ""
    declared_verbs = set(re.findall(r'add_parser\("([a-z-]+)"', milestone_source))
    declared_verbs.update(re.findall(r'\("([a-z-]+)",\s*cmd_[a-z_]+\)',
                                     milestone_source))
    for verb in MILESTONE_VERBS:
        report.check(f"milestone.py exposes `{verb}`",
                     verb in declared_verbs)
    report.check("milestone.py refuses bare numeric references",
                 "bare `{ref}` is ambiguous" in milestone_source)
    report.check("milestone.py enforces SemVer core without a base-64 ceiling",
                 "CORE_SEMVER_RULE" in milestone_source
                 and "minor > 63" not in milestone_source
                 and "patch > 63" not in milestone_source)
    report.check("milestone.py defaults new release Milestones to a minor bump",
                 'DEFAULT_BUMP = "minor"' in milestone_source
                 and "def next_release_identity" in milestone_source
                 and 'list_milestones(args.repo, "all")' in milestone_source)
    report.check("milestone.py owns an explicit Development Order",
                 'SECTIONS = ("Goal", "Development Order", "Acceptance")' in milestone_source)
    report.check("milestone.py exposes one typed queue head",
                 "def cmd_next" in milestone_source
                 and "queue head" in milestone_source
                 and "next_phase" in milestone_source
                 and "wi_types.flow_for" in milestone_source)

    epic_path = scripts_root / "epic.py"
    epic_source = epic_path.read_text(encoding="utf-8") if epic_path.is_file() else ""
    report.check("legacy epic facade refuses all issue-epic writes",
                 all(verb in epic_source for verb in ('"create"', '"update"', '"close"'))
                 and "issue-based epics are retired" in epic_source)

    change_path = scripts_root / "change.py"
    change_source = change_path.read_text(encoding="utf-8") if change_path.is_file() else ""
    report.check("change.py uses the native Milestone surface",
                 "resolve_milestone" in change_source
                 and '"--milestone"' in change_source
                 and '"--remove-milestone"' in change_source)
    report.check("change.py closes only after its required lifecycle",
                 "required = wi_types.required_legs(wi_type.name)" in change_source
                 and "lifecycle_errors" in change_source
                 and "cmd_lifecycle" in change_source
                 and 'sub.add_parser("close"' in change_source)

    types_path = scripts_root / "wi_types.py"
    types_source = types_path.read_text(encoding="utf-8") if types_path.is_file() else ""
    report.check("wi_types.py owns the frozen delivery and intake vocabulary",
                 'DELIVERY_TYPES = (\n    "feat", "fix", "refactor", "perf", "test", "docs", "chore",' in types_source
                 and 'BEHAVIOR_TYPES = ("feat", "fix", "perf")' in types_source
                 and 'MAINTENANCE_TYPES = ("refactor", "test", "docs", "chore")' in types_source
                 and 'INTAKE_TYPES = ("spike", "report")' in types_source
                 and '"maintenance": ("maint",)' in types_source)

    migration_path = scripts_root / "type_migration.py"
    migration_source = migration_path.read_text(encoding="utf-8") if migration_path.is_file() else ""
    report.check("type_migration.py is the one-time legacy migration surface",
                 "MIGRATABLE_LEGACY_TYPES" in migration_source
                 and 'mode.add_argument("--apply"' in migration_source
                 and 'mode.add_argument("--resume"' in migration_source
                 and "preflight(args.repo, rows)" in migration_source)

    maint_path = scripts_root / "maint.py"
    maint_source = maint_path.read_text(encoding="utf-8") if maint_path.is_file() else ""
    report.check("maint.py exposes safe record evidence",
                 '"record", parents=[wi]' in maint_source
                 and 'command.add_argument("--when", required=True, choices=("before", "after"))' in maint_source
                 and 'command.add_argument("--output-file", required=True)' in maint_source
                 and "Maint-Contract:" in maint_source
                 and "Maint-Change-Digest:" in maint_source
                 and "after.lifecycle.command: {AW_CLI} change close" in maint_source)

    metadoc_path = scripts_root / "metadoc.py"
    metadoc_source = metadoc_path.read_text(encoding="utf-8") if metadoc_path.is_file() else ""
    report.check("metadoc.py recognises Milestone bindings",
                 "Milestone[ \\t]+#" in metadoc_source)

    wis_path = scripts_root / "wis.py"
    wis_source = wis_path.read_text(encoding="utf-8") if wis_path.is_file() else ""
    report.check("wis.py measures release Milestones",
                 "import milestone" in wis_source and "release milestones" in wis_source)
    return report


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path,
                        default=Path(__file__).resolve().parents[3])
    args = parser.parse_args(argv)
    report = collect(args.repo.resolve())
    if report.failed:
        print(f"\n=> RED: {len(report.failed)} failure(s)")
        return 1
    print(f"\n=> GREEN: {len(SKILLS)} byte-identical product and explicit-only legacy skill pairs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
