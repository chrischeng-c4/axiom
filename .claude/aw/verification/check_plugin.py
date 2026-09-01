#!/usr/bin/env python3
"""Check seven byte-identical AW skills and their typed-delivery contract."""

from __future__ import annotations

import argparse
import re
from pathlib import Path


SKILLS = (
    "aw-ask-user",
    "aw-e2e-for-wi",
    "aw-grill-me-to-meta",
    "aw-grill-meta-to-wis",
    "aw-impl-for-wi",
    "aw-maint-for-wi",
    "aw-prepare-goal",
)
HEADINGS = ("## Goal", "## How", "## Acceptance", "## Never")
SCRIPTS = (
    "change.py",
    "e2e.py",
    "epic.py",
    "impl.py",
    "maint.py",
    "metadoc.py",
    "milestone.py",
    "type_migration.py",
    "wi_types.py",
    "wis.py",
    "workitem.py",
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
LEGACY_WRITE = re.compile(r"\bepic\.py\s+(?:create|update|close)\b")
AW_INVOCATION = re.compile(r"`aw\s+[a-z]")


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


def aw_directories(root: Path) -> list[str]:
    if not root.is_dir():
        return []
    return sorted(path.name for path in root.iterdir()
                  if path.is_dir() and path.name.startswith("aw-"))


def collect(repo: Path) -> Reporter:
    report = Reporter()
    codex_root = repo / ".agents" / "skills"
    claude_root = repo / ".claude" / "skills"
    scripts_root = repo / ".claude" / "aw" / "scripts"

    report.check("Codex skill root exists", codex_root.is_dir(), str(codex_root))
    report.check("Claude skill root exists", claude_root.is_dir(), str(claude_root))
    report.check("shared AW script root exists", scripts_root.is_dir(), str(scripts_root))
    report.check("Codex AW skill set is exact", aw_directories(codex_root) == list(SKILLS),
                 f"found={aw_directories(codex_root)}")
    report.check("Claude AW skill set is exact", aw_directories(claude_root) == list(SKILLS),
                 f"found={aw_directories(claude_root)}")

    bodies: dict[str, str] = {}
    for skill in SKILLS:
        codex = codex_root / skill / "SKILL.md"
        claude = claude_root / skill / "SKILL.md"
        report.check(f"{skill}: Codex SKILL.md exists", codex.is_file())
        report.check(f"{skill}: Claude SKILL.md exists", claude.is_file())
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
        report.check(f"{skill}: has no retired `aw <verb>` command",
                     not AW_INVOCATION.search(text))
        report.check(f"{skill}: has no direct GitHub write", not GH_WRITE.search(text))
        report.check(f"{skill}: has no legacy issue-epic writer",
                     not LEGACY_WRITE.search(text)
                     and ".claude/aw/scripts/epic.py" not in text
                     and "--label epic:" not in text)
        report.check(f"{skill}: carries no private scripts copy",
                     not (codex.parent / "scripts").exists()
                     and not (claude.parent / "scripts").exists())

        for name in sorted(set(SCRIPT_NAME.findall(text))):
            report.check(f"{skill}: named script {name} exists",
                         (scripts_root / name).is_file())

    for name in SCRIPTS:
        report.check(f"{name} exists", (scripts_root / name).is_file())

    required_phrases = {
        "aw-grill-me-to-meta": (
            "Use before release Milestone planning.",
            "(Milestone #<number>)",
            "Tracking: Not assigned.",
        ),
        "aw-grill-meta-to-wis": (
            "GitHub's native milestone field is the only",
            "<project>@<major>.<minor>.<patch>",
            "## Development Order",
            "milestone.py\" reconcile",
            "milestone.py\" next-version <project> --json",
            "default `minor` bump",
            "--bump patch", "--bump major", "initial version",
            "issue order, and the type of every issue.",
            "type:feat", "type:fix", "type:refactor", "type:perf",
            "type:test", "type:docs", "type:chore",
            "type:spike", "type:report", "legacy `type:change`",
            "global queue order.", "Only its first open row is executable.",
        ),
        "aw-e2e-for-wi": (
            "milestone.py\" next",
            "--json",
            "Never choose a Milestone's issue order yourself.",
            "type:feat", "type:fix", "type:perf", "queue head",
            "flow: behavior", "next_phase: e2e",
            "change.py lifecycle",
        ),
        "aw-impl-for-wi": (
            "milestone.py\" next",
            "--json",
            "Never choose or infer Milestone order.",
            "type:feat", "type:fix", "type:perf", "queue head",
            "flow: behavior", "next_phase: impl",
            "change.py lifecycle", "change.py close",
        ),
        "aw-maint-for-wi": (
            "milestone.py\" next",
            "--json",
            "type:refactor", "type:test", "type:docs", "type:chore",
            "queue head", "flow: maintenance", "next_phase: maint",
            "Maint-Contract:", "Maint-Change-Digest:", "record <iid>",
            "--output-file <path>", "change.py lifecycle", "change.py close",
        ),
        "aw-prepare-goal": (
            "milestone:<number>",
            "milestone.py\" next",
            "Never use a bare number as a Milestone reference.",
            "behavior to e2e then impl, maintenance to maint",
            "Reject `type:change`",
        ),
        "aw-ask-user": (
            "version, milestone order, scope boundary",
            "default minor Milestone bump",
            "major, patch, or exact version override",
            "No file, Git ref, issue, milestone, or release changes.",
        ),
    }
    for skill, phrases in required_phrases.items():
        text = bodies.get(skill, "")
        for phrase in phrases:
            report.check(f"{skill}: carries typed queue contract `{phrase}`",
                         phrase in text)

    phase_commands = {
        "aw-e2e-for-wi": ("e2e.py", ("start", "verify", "test", "commit")),
        "aw-impl-for-wi": ("impl.py", ("start", "red", "verify", "test", "commit")),
    }
    for skill, (script, verbs) in phase_commands.items():
        text = bodies.get(skill, "")
        for verb in verbs:
            command = f'{script}" --project <project> {verb} <iid>'
            report.check(
                f"{skill}: `{verb}` keeps --project before the verb",
                command in text,
            )

    for skill in ("aw-grill-me-to-meta", "aw-grill-meta-to-wis"):
        text = bodies.get(skill, "")
        how = text.partition("## How")[2].partition("## Acceptance")[0]
        first = re.search(r"^(?P<rank>[1-9][0-9]*)\.\s+(?P<step>.+)$", how, re.M)
        report.check(f"{skill}: first step enters Plan mode",
                     bool(first) and first.group("rank") == "1"
                     and first.group("step").startswith("Enter Plan mode"),
                     f"first={first.group(0)!r}" if first else "no numbered step")
        report.check(f"{skill}: Plan mode is fail-closed",
                     re.search(r"Stop if\s+the runtime cannot\s+confirm Plan mode\.", how) is not None)

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
                 and "after.lifecycle.command: change.py close" in maint_source)

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
    print("\n=> GREEN: seven byte-identical typed-delivery AW skill pairs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
