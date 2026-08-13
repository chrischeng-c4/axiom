#!/usr/bin/env python3
"""Refuse an engine that has learned which work-item type it is serving.

`workitem.py` was split out of `epic.py` so a second type -- change, spike,
report -- is a thin facade rather than a copied file. Nothing enforced that
split: the module keeps working perfectly with `if wi_type.name == "epic"`
written into it, and each such line is one more thing the next type has to
either satisfy or fork around. By the time that is noticed, the fork has
already happened.

So the assertion is not "the engine is generic" as prose but as a measurement:
no identifier and no string literal in the engine's *code* names a work-item
type. Docstrings and comments are excluded deliberately -- explaining what the
epic facade does with a label is documentation, while embedding `type:epic` in
a branch is behavior, and a gate that cannot tell them apart forces the module
to be undocumented in order to stay green.

The extractor carries its control: run over `epic.py`, which is type-bound by
design, it must find plenty. An extractor that reports "no leaks" on the file
whose whole job is naming the epic is measuring nothing.
"""
import ast
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import ENGINE, SCRIPT  # noqa: E402

# The closed type enum, plus the label prefixes a type is spelled with. `epic`
# is the only one present today; the rest are here so the gate is already
# watching the axis this split exists to open.
TYPE_NAMES = ("epic", "change", "spike", "report")

fails = []


def check(label, ok, detail=""):
    print(f"{'PASS' if ok else 'FAIL'} {label}{(' -- ' + detail) if detail else ''}")
    if not ok:
        fails.append(label)


def docstring_ids(tree):
    """The `id()` of every Constant node that is a docstring, not a value."""
    found = set()
    for node in ast.walk(tree):
        if not isinstance(node, (ast.Module, ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            continue
        first = node.body[0] if node.body else None
        if (isinstance(first, ast.Expr) and isinstance(first.value, ast.Constant)
                and isinstance(first.value.value, str)):
            found.add(id(first.value))
    return found


def enum_ids(tree):
    """The `id()` of every Constant inside the `WORK_ITEM_TYPES` assignment.

    The closed enum is the one place the engine is *supposed* to name every
    type: it is the axis itself, consumed as argparse `choices`, and moving it
    to a facade would give each type its own idea of what the other types are.
    Enumerating the set is type-independent; branching on a member is not, and
    only the enum's own literals are exempt -- an `== "epic"` two lines below
    it is still a leak.
    """
    found = set()
    for node in ast.walk(tree):
        if not isinstance(node, (ast.Assign, ast.AnnAssign)):
            continue
        targets = node.targets if isinstance(node, ast.Assign) else [node.target]
        if not any(isinstance(t, ast.Name) and t.id == "WORK_ITEM_TYPES" for t in targets):
            continue
        for inner in ast.walk(node):
            if isinstance(inner, ast.Constant) and isinstance(inner.value, str):
                found.add(id(inner))
    return found


def leaks(path):
    """Every code-level mention of a work-item type name in `path`.

    Returns (literal_leaks, name_leaks). Comments never reach the AST at all,
    so they need no exclusion; docstrings do, and are excluded by identity
    rather than by re-matching their text. So is the closed enum, on the same
    identity basis -- see `enum_ids`.
    """
    tree = ast.parse(path.read_text(encoding="utf-8"))
    exempt = docstring_ids(tree) | enum_ids(tree)

    literals = []
    for node in ast.walk(tree):
        if not (isinstance(node, ast.Constant) and isinstance(node.value, str)):
            continue
        if id(node) in exempt:
            continue
        if any(t in node.value.lower() for t in TYPE_NAMES):
            literals.append(node.value)

    names = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Name):
            names.add(node.id)
        elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            names.add(node.name)
        elif isinstance(node, ast.Attribute):
            names.add(node.attr)
        elif isinstance(node, ast.arg):
            names.add(node.arg)
    named = sorted(n for n in names if any(t in n.lower() for t in TYPE_NAMES))
    return literals, named


check("the engine module is beside the script", ENGINE.is_file(), str(ENGINE))
if not ENGINE.is_file():
    print("\n=> RED (1 failure(s))")
    sys.exit(1)

engine_tree = ast.parse(ENGINE.read_text(encoding="utf-8"))
exempted = enum_ids(engine_tree)

engine_literals, engine_names = leaks(ENGINE)
check("the engine's code carries no work-item type in a string literal",
      not engine_literals, f"leaked={engine_literals}")
check("the engine's code carries no work-item type in an identifier",
      not engine_names, f"leaked={engine_names}")

# The enum exemption has to stay one assignment wide. If it ever covers more
# than the four members of the closed enum, the assertion above is exempting
# whatever was added to it rather than measuring the engine.
check("the enum exemption covers exactly the closed enum",
      len(exempted) == len(TYPE_NAMES), f"exempted={len(exempted)} literals")

# Control: the same extractor, run over the file that is type-bound on purpose.
# Without this, a broken parse or a wrong exclusion reports a clean engine and
# the gate becomes a green light with nothing behind it.
facade_literals, facade_names = leaks(SCRIPT)
check("positive control: the extractor finds type names in the epic facade",
      bool(facade_literals) and bool(facade_names),
      f"literals={len(facade_literals)} identifiers={len(facade_names)}")

# ...and it must be finding them in *code*, not merely tripping over prose.
check("positive control: the facade's leaks include a real label literal",
      "type:epic" in facade_literals,
      f"looked for 'type:epic' among {len(facade_literals)} literal(s)")

# The split is only worth enforcing if the engine is actually what the facade
# runs on. A facade that re-implements the engine would satisfy every
# assertion above while sharing nothing.
facade_src = SCRIPT.read_text(encoding="utf-8")
check("the facade imports the engine rather than re-implementing it",
      "import workitem" in facade_src and "workitem.dispatch(" in facade_src)

print("\n=> " + ("GREEN" if not fails else f"RED ({len(fails)} failure(s))"))
sys.exit(1 if fails else 0)
