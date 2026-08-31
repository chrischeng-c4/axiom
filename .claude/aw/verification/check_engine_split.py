#!/usr/bin/env python3
"""Keep the generic work-item engine below the one issue-type registry."""

from __future__ import annotations

import ast
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import ENGINE, SCRIPT, WI_TYPES_SCRIPT  # noqa: E402


ACTIVE = ("feat", "fix", "refactor", "perf", "test", "docs", "chore", "spike", "report")
ALL_TYPES = (*ACTIVE, "change", "bug", "enhancement", "epic")
fails: list[str] = []


def check(label: str, ok: bool, detail: str = "") -> None:
    print(f"{'PASS' if ok else 'FAIL'} {label}{(' -- ' + detail) if detail else ''}")
    if not ok:
        fails.append(label)


def docstring_ids(tree: ast.AST) -> set[int]:
    found: set[int] = set()
    for node in ast.walk(tree):
        if not isinstance(node, (ast.Module, ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            continue
        first = node.body[0] if node.body else None
        if (isinstance(first, ast.Expr) and isinstance(first.value, ast.Constant)
                and isinstance(first.value.value, str)):
            found.add(id(first.value))
    return found


def assignment(tree: ast.AST, name: str) -> ast.Assign | ast.AnnAssign | None:
    for node in ast.walk(tree):
        if not isinstance(node, (ast.Assign, ast.AnnAssign)):
            continue
        targets = node.targets if isinstance(node, ast.Assign) else [node.target]
        if any(isinstance(target, ast.Name) and target.id == name for target in targets):
            return node
    return None


def exact_type_literals(path: pathlib.Path) -> list[str]:
    tree = ast.parse(path.read_text(encoding="utf-8"))
    exempt = docstring_ids(tree)
    legacy = assignment(tree, "LEGACY_WORK_ITEM_TYPES")
    if legacy is not None:
        exempt.update(id(node) for node in ast.walk(legacy)
                      if isinstance(node, ast.Constant))
    return [
        node.value for node in ast.walk(tree)
        if isinstance(node, ast.Constant) and isinstance(node.value, str)
        and id(node) not in exempt and node.value in ALL_TYPES
    ]


check("the engine module is beside the script", ENGINE.is_file(), str(ENGINE))
check("the type registry is beside the engine", WI_TYPES_SCRIPT.is_file(), str(WI_TYPES_SCRIPT))
if not ENGINE.is_file() or not WI_TYPES_SCRIPT.is_file():
    print("\n=> RED")
    raise SystemExit(1)

engine_source = ENGINE.read_text(encoding="utf-8")
engine_tree = ast.parse(engine_source)
registry_source = WI_TYPES_SCRIPT.read_text(encoding="utf-8")

check("the engine imports the one type registry", "import wi_types" in engine_source)
check("the active enum is derived from the registry",
      "WORK_ITEM_TYPES = (*wi_types.DELIVERY_TYPES, *wi_types.INTAKE_TYPES)" in engine_source)
check("the engine carries no active or retired type literal in behavior",
      not exact_type_literals(ENGINE), repr(exact_type_literals(ENGINE)))

legacy = assignment(engine_tree, "LEGACY_WORK_ITEM_TYPES")
legacy_values = [node.value for node in ast.walk(legacy)
                 if isinstance(node, ast.Constant) and isinstance(node.value, str)] if legacy else []
check("the engine keeps only epic as a historical staging enum",
      legacy_values == ["epic"], repr(legacy_values))

registry_tree = ast.parse(registry_source)
registry_literals = {
    node.value for node in ast.walk(registry_tree)
    if isinstance(node, ast.Constant) and isinstance(node.value, str)
    and node.value in ALL_TYPES
}
check("positive control: the registry owns every type literal",
      registry_literals == set(ALL_TYPES), repr(sorted(registry_literals)))

facade_source = SCRIPT.read_text(encoding="utf-8")
check("the historical epic facade imports the engine",
      "import workitem" in facade_source and "workitem.dispatch(" in facade_source)

print("\n=> " + ("GREEN" if not fails else f"RED ({len(fails)} failure(s))"))
raise SystemExit(1 if fails else 0)
