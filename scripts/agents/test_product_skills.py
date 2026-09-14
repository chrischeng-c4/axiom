#!/usr/bin/env python3
"""Static checks for the simple product skills and explicit legacy skills."""

from __future__ import annotations

import argparse
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CLAUDE_SKILLS = ROOT / ".claude" / "skills"
CODEX_SKILLS = ROOT / ".agents" / "skills"
LEGACY = (
    "prepare-goal",
    "grill-release",
    "e2e-for",
    "impl-for",
    "test-for",
    "review",
    "ask-user",
)
PRODUCT = ("product-ideate", "product-plan", "product-deliver")
CONVERSATION_SHORTCUTS = ("next-step", "follow-next-step", "approve-next-step")


def text(root: Path, name: str) -> str:
    return (root / name / "SKILL.md").read_text(encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--claude-only", action="store_true")
    args = parser.parse_args()
    failures: list[str] = []
    for name in LEGACY:
        body = text(CLAUDE_SKILLS, name)
        mirror = text(CODEX_SKILLS, name)
        policy = CODEX_SKILLS / name / "agents" / "openai.yaml"
        if f"name: {name}" not in body:
            failures.append(f"{name}: wrong frontmatter name")
        if "explicitly" not in body or "Never invoke this skill implicitly." not in body:
            failures.append(f"{name}: missing explicit-only boundary")
        if body != mirror:
            failures.append(f"{name}: Claude/Codex SKILL.md drift")
        if not policy.is_file() or policy.read_text(encoding="utf-8") != "policy:\n  allow_implicit_invocation: false\n":
            failures.append(f"{name}: wrong Codex invocation policy")
        if (CLAUDE_SKILLS / f"aw-{name}").exists() or (CODEX_SKILLS / f"aw-{name}").exists():
            failures.append(f"{name}: old aw-prefixed alias remains")
    for name in PRODUCT:
        body = text(CLAUDE_SKILLS, name)
        if body != text(CODEX_SKILLS, name):
            failures.append(f"{name}: Claude/Codex SKILL.md drift")
        if f"name: {name}" not in body:
            failures.append(f"{name}: wrong frontmatter name")
    shortcut_contracts = {
        "next-step": ("current conversation first", "read-only check", "at most two options",
                      "needed authority", "Never execute the recommended action"),
        "follow-next-step": ("latest unfinished, clear main recommendation",
                             "Plan mode", "blocked-writer policy", "Explain the difference"),
        "approve-next-step": ("latest pending, well-scoped request",
                              "explicit user invocation", "safety- or correctness-relevant current state",
                              "exact approved action", "same consent", "blanket future authority"),
    }
    for name, phrases in shortcut_contracts.items():
        claude = CLAUDE_SKILLS / name / "SKILL.md"
        codex = CODEX_SKILLS / name / "SKILL.md"
        if not claude.is_file():
            failures.append(f"{name}: Claude SKILL.md missing")
            continue
        body = claude.read_text(encoding="utf-8")
        if f"name: {name}" not in body:
            failures.append(f"{name}: wrong frontmatter name")
        if (CLAUDE_SKILLS / name / "agents" / "openai.yaml").exists():
            failures.append(f"{name}: Claude disables default discovery")
        for phrase in phrases:
            if phrase not in body:
                failures.append(f"{name}: missing {phrase!r} boundary")
        if not args.claude_only:
            if not codex.is_file():
                failures.append(f"{name}: Codex twin missing")
            elif body != codex.read_text(encoding="utf-8"):
                failures.append(f"{name}: Claude/Codex SKILL.md drift")
            elif (CODEX_SKILLS / name / "agents" / "openai.yaml").exists():
                failures.append(f"{name}: Codex disables default discovery")
    delivery = text(CLAUDE_SKILLS, "product-deliver")
    for mode in ("e2e-only", "impl-only", "test-only", "read-only-review"):
        if mode not in delivery:
            failures.append(f"product-deliver: missing {mode} mode")
    for required in ("fresh `<p>-qa`", "Integration QA only for a cross-project", "permission readiness check", "controller owns Git"):
        if required not in delivery:
            failures.append(f"product-deliver: missing {required!r} boundary")
    settings = (ROOT / ".claude" / "settings.json").read_text(encoding="utf-8")
    for name in LEGACY:
        if f'"{name}": "user-invocable-only"' not in settings:
            failures.append(f"settings: {name} is not explicit-only")
    if failures:
        print("FAIL")
        print("\n".join(failures))
        return 1
    print("PASS: product, legacy, and conversation shortcut boundaries")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
