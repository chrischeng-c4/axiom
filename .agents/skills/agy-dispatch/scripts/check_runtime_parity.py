#!/usr/bin/env python3
"""Check that the Codex and Claude AGY dispatch runtimes stay equivalent."""

from __future__ import annotations

import argparse
import ast
import hashlib
import re
import sys
import tomllib
from pathlib import Path
from typing import Callable


SHARED_SUPPORT_FILES = (
    "references/inventory-verification.md",
    "references/lifecycle.md",
    "references/one-shot-profile-template.json",
    "references/permissions.md",
    "references/profile-template.json",
    "references/report-verification.md",
    "scripts/run_isolated.py",
    "scripts/teamwork_terminal.py",
    "scripts/test_agy_dispatch.py",
    "scripts/test_run_isolated.py",
)
ENGINE_RELATIVE = "scripts/agy_dispatch.py"
ENGINE_LINK_TARGET = "../../../../scripts/agy_dispatch.py"
ROOT_COMMAND = "python3 scripts/agy_dispatch.py"
GLOBAL_PORTABLE_FILES = (
    "references/inventory-verification.md",
    "references/one-shot-profile-template.json",
    "references/permissions.md",
    "references/profile-template.json",
    "references/report-verification.md",
    "scripts/run_isolated.py",
    "scripts/teamwork_terminal.py",
    "scripts/test_agy_dispatch.py",
    ENGINE_RELATIVE,
)
OLD_DISPATCHER_PATTERN = re.compile(
    r"(?:run_isolated\.py|skills/agy-dispatch/scripts/agy_dispatch\.py)"
)


Check = Callable[[], tuple[bool, str]]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def parse_yaml_frontmatter(path: Path) -> tuple[dict[str, object], bytes]:
    """Return simple YAML scalar/list fields and the header-free body bytes."""
    raw = path.read_bytes()
    if not raw.startswith(b"---\n"):
        raise ValueError(f"{path} does not start with YAML frontmatter")
    boundary = raw.find(b"\n---\n", 4)
    if boundary < 0:
        raise ValueError(f"{path} frontmatter has no closing delimiter")

    header_text = raw[4:boundary].decode("utf-8")
    fields: dict[str, object] = {}
    list_key: str | None = None
    for line_number, line in enumerate(header_text.splitlines(), start=2):
        if not line:
            continue
        if line[0].isspace():
            item = line.strip()
            if list_key is not None and item.startswith("- "):
                values = fields[list_key]
                if not isinstance(values, list):
                    raise ValueError(f"invalid list at frontmatter line {line_number}")
                values.append(item[2:].strip().strip("\"'"))
            continue
        key, separator, value = line.partition(":")
        if not separator:
            raise ValueError(f"invalid frontmatter line {line_number}")
        key = key.strip()
        value = value.strip()
        if value:
            fields[key] = value.strip("\"'")
            list_key = None
        else:
            fields[key] = []
            list_key = key

    body = raw[boundary + len(b"\n---\n") :]
    if body.startswith(b"\n"):
        body = body[1:]
    return fields, body


def parse_claude_agent(path: Path) -> tuple[dict[str, object], bytes]:
    return parse_yaml_frontmatter(path)


def parse_codex_agent(path: Path) -> tuple[dict[str, object], bytes]:
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    body = data.get("developer_instructions")
    if not isinstance(body, str):
        raise ValueError("Codex agent has no developer_instructions string")
    return data, body.encode("utf-8")


def literal_assignments(path: Path) -> dict[str, object]:
    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    values: dict[str, object] = {}
    for node in tree.body:
        if not isinstance(node, (ast.Assign, ast.AnnAssign)):
            continue
        targets = node.targets if isinstance(node, ast.Assign) else [node.target]
        value_node = node.value
        if value_node is None:
            continue
        try:
            value = ast.literal_eval(value_node)
        except (ValueError, TypeError):
            continue
        for target in targets:
            if isinstance(target, ast.Name):
                values[target.id] = value
    return values


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--global-skill-root",
        type=Path,
        help="also compare the shared files with this installed skill root",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[4]
    canonical_root = repo_root / ".agents/skills/agy-dispatch"
    claude_root = repo_root / ".claude/skills/agy-dispatch"
    root_engine = repo_root / "scripts/agy_dispatch.py"
    codex_agent = repo_root / ".codex/agents/dispatch-operator.toml"
    claude_agent = repo_root / ".claude/agents/dispatch-operator.md"
    legacy_root = repo_root / ".claude/dispatch/legacy/agy-dispatch-v1"

    def check_shared_files() -> tuple[bool, str]:
        failures: list[str] = []
        for relative in SHARED_SUPPORT_FILES:
            canonical_path = canonical_root / relative
            if not canonical_path.is_file():
                failures.append(f"missing canonical {relative}")
                continue
            canonical_digest = sha256(canonical_path)
            candidate = claude_root / relative
            if not candidate.is_file():
                failures.append(f"missing Claude {relative}")
            elif sha256(candidate) != canonical_digest:
                failures.append(f"digest mismatch for Claude {relative}")

        if args.global_skill_root is not None:
            global_root = args.global_skill_root.expanduser().resolve()
            for relative in GLOBAL_PORTABLE_FILES:
                canonical_path = (
                    root_engine if relative == ENGINE_RELATIVE else canonical_root / relative
                )
                candidate = global_root / relative
                if not canonical_path.is_file():
                    failures.append(f"missing canonical source for global {relative}")
                elif not candidate.is_file():
                    failures.append(f"missing global {relative}")
                elif sha256(candidate) != sha256(canonical_path):
                    failures.append(f"digest mismatch for global {relative}")
        if failures:
            return False, "; ".join(failures)
        compared = "Claude support files"
        if args.global_skill_root is not None:
            compared += f" and {len(GLOBAL_PORTABLE_FILES)} global portable file targets"
        return True, f"all {len(SHARED_SUPPORT_FILES)} shared support files match {compared}"

    def check_root_engine() -> tuple[bool, str]:
        source = root_engine.read_text(encoding="utf-8")
        assignments = literal_assignments(root_engine)
        expected = {
            "REQUIRED_MODEL": "gemini-3.7-flash-high",
            "REQUIRED_EFFORT": "high",
            "REQUIRED_WORKTREE_LAYOUT": "in-project",
            "REQUIRED_LAUNCH_CWD": "task-worktree",
        }
        failures = [
            f"{name} is not fixed to {value}"
            for name, value in expected.items()
            if assignments.get(name) != value
        ]
        if root_engine.is_symlink():
            failures.append("root engine is a symlink instead of the full engine")
        banned = {
            "PROJECT_DIR": re.search(r"\bPROJECT_DIR\b", source),
            "repoint_project_root": re.search(r"\brepoint_project_root\b", source),
            "def grant": re.search(r"\bdef\s+grant\s*\(", source),
        }
        failures.extend(f"contains {name}" for name, match in banned.items() if match)
        if failures:
            return False, "; ".join(failures)
        return True, "root engine fixes 3.7 high, in-project, and task-worktree"

    def check_engine_links() -> tuple[bool, str]:
        failures: list[str] = []
        expected_engine = root_engine.resolve(strict=True)
        for label, path in (
            ("Codex", canonical_root / ENGINE_RELATIVE),
            ("Claude", claude_root / ENGINE_RELATIVE),
        ):
            if not path.is_symlink():
                failures.append(f"{label} engine path is not a symlink")
                continue
            target = path.readlink()
            if target.is_absolute() or target.as_posix() != ENGINE_LINK_TARGET:
                failures.append(
                    f"{label} engine link is {target.as_posix()}, not {ENGINE_LINK_TARGET}"
                )
                continue
            if path.resolve(strict=True) != expected_engine:
                failures.append(f"{label} engine link does not resolve to the root engine")
        if failures:
            return False, "; ".join(failures)
        return True, "Codex and Claude use the exact relative link to the root engine"

    def check_operator_models() -> tuple[bool, str]:
        codex, _ = parse_codex_agent(codex_agent)
        claude, _ = parse_claude_agent(claude_agent)
        failures: list[str] = []
        if codex.get("model") != "gpt-5.6-luna":
            failures.append("Codex model is not gpt-5.6-luna")
        if codex.get("model_reasoning_effort") != "medium":
            failures.append("Codex effort is not medium")
        if claude.get("model") != "sonnet":
            failures.append("Claude model is not sonnet")
        if claude.get("effort") != "low":
            failures.append("Claude effort is not low")
        return (False, "; ".join(failures)) if failures else (True, "Luna medium and Sonnet low")

    def check_skill_bindings() -> tuple[bool, str]:
        codex_skill, _ = parse_yaml_frontmatter(canonical_root / "SKILL.md")
        claude_skill, _ = parse_yaml_frontmatter(claude_root / "SKILL.md")
        claude, _ = parse_claude_agent(claude_agent)
        failures: list[str] = []
        if canonical_root.name != "agy-dispatch" or codex_skill.get("name") != "agy-dispatch":
            failures.append("Codex repo skill is not named agy-dispatch")
        if claude_skill.get("name") != "agy:dispatch":
            failures.append("Claude skill is not named agy:dispatch")
        claude_skills = claude.get("skills")
        if not isinstance(claude_skills, list) or "agy:dispatch" not in claude_skills:
            failures.append("Claude operator skills do not include agy:dispatch")
        if failures:
            return False, "; ".join(failures)
        return True, "Codex uses agy-dispatch and Claude operator loads agy:dispatch"

    def check_operator_body_parity() -> tuple[bool, str]:
        _, codex_body = parse_codex_agent(codex_agent)
        _, claude_body = parse_claude_agent(claude_agent)
        if codex_body != claude_body:
            return False, "header-free operator bodies differ"
        return True, f"operator bodies match byte-for-byte ({len(codex_body)} bytes)"

    def check_operator_body_contract() -> tuple[bool, str]:
        _, body_bytes = parse_codex_agent(codex_agent)
        body = body_bytes.decode("utf-8")
        required = {
            "snapshot_mode": re.search(r"snapshot(?:_| )mode", body, re.IGNORECASE) is not None,
            "create": "`create`" in body,
            "reuse": "`reuse`" in body,
            "refresh": "`refresh`" in body,
            "dispatch/create": "`dispatch/create`" in body,
            "resume/reuse": "`resume/reuse`" in body,
            "resume/refresh": "`resume/refresh`" in body,
            "HANDOFF_INCOMPLETE": "HANDOFF_INCOMPLETE" in body,
            "verify/accept prohibition": "Never run `verify`, `accept`" in body,
            "absolute frozen profile token": (
                "Copy its complete\nabsolute string byte-for-byte" in body
            ),
            "adapter probe prohibition": "Never run `--help`, `--version`" in body,
        }
        missing = [name for name, present in required.items() if not present]
        if missing:
            return False, "missing " + ", ".join(missing)
        return True, "snapshot modes, action pairs, refusal, and verify/accept ban are present"

    def check_active_commands() -> tuple[bool, str]:
        _, codex_body = parse_codex_agent(codex_agent)
        _, claude_body = parse_claude_agent(claude_agent)
        documents = (
            ("Codex skill", canonical_root / "SKILL.md", None),
            ("Claude skill", claude_root / "SKILL.md", None),
            ("Codex lifecycle", canonical_root / "references/lifecycle.md", None),
            ("Claude lifecycle", claude_root / "references/lifecycle.md", None),
            ("AGENTS.md", repo_root / "AGENTS.md", None),
            ("CLAUDE.md", repo_root / "CLAUDE.md", None),
            ("Codex operator", None, codex_body.decode("utf-8")),
            ("Claude operator", None, claude_body.decode("utf-8")),
        )
        failures: list[str] = []
        for label, path, supplied_text in documents:
            text = supplied_text if supplied_text is not None else path.read_text(encoding="utf-8")
            if ROOT_COMMAND not in text:
                failures.append(f"{label} does not name {ROOT_COMMAND}")
            old = OLD_DISPATCHER_PATTERN.search(text)
            if old:
                failures.append(f"{label} names retired dispatcher path {old.group(0)}")
        if failures:
            return False, "; ".join(failures)
        return True, "all active controller and operator documents name only the root command"

    def check_legacy_boundary() -> tuple[bool, str]:
        skill_files = sorted(path.relative_to(repo_root).as_posix() for path in legacy_root.rglob("SKILL.md"))
        if skill_files:
            return False, "active skill marker found: " + ", ".join(skill_files)
        return True, "legacy archive has no SKILL.md"

    checks: tuple[tuple[str, Check], ...] = (
        ("shared file digests", check_shared_files),
        ("root engine contract", check_root_engine),
        ("root engine links", check_engine_links),
        ("operator models", check_operator_models),
        ("skill bindings", check_skill_bindings),
        ("operator body parity", check_operator_body_parity),
        ("operator body contract", check_operator_body_contract),
        ("active root commands", check_active_commands),
        ("legacy boundary", check_legacy_boundary),
    )

    failed = False
    for number, (label, check) in enumerate(checks, start=1):
        try:
            passed, detail = check()
        except (OSError, UnicodeError, ValueError, SyntaxError, tomllib.TOMLDecodeError) as error:
            passed, detail = False, str(error)
        state = "OK" if passed else "FAIL"
        print(f"{state} {number}. {label}: {detail}")
        failed = failed or not passed

    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
