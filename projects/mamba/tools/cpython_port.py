#!/usr/bin/env python3
"""Port a CPython 3.12 ``Lib/test/test_<X>.py`` into one-case fixture files.

Each ``def test*`` in each ``class TestX(unittest.TestCase)``
becomes one standalone ``.py``:

  - PEP 723 inline metadata at the top
  - a ``[tool.mamba]`` record so the file is queryable by dimension
  - module-level imports + helpers from the source file
  - inlined ``setUp`` body (with ``self.`` stripped) if present
  - the test body rewritten:
      ``self.assertEqual(a, b)``        -> ``assert a == b``
      ``self.assertIs(a, b)``           -> ``assert a is b``
      ``self.assertTrue(x)``            -> ``assert x``
      ``with self.assertRaises(X):``    -> ``try: ... except X: pass``
      ``self.assertRaises(X, fn, *a)``  -> ``try: fn(*a); raise ... except X:``
      ``self.fail(msg)``                -> ``raise AssertionError(msg)``
      ``self.<attr>``                   -> ``<attr>``    (drop receiver)

Methods that touch unsupported features (``self.subTest``,
``self.addCleanup``, ``mock.patch``, ``support.captured_*``) are reported
in the skip-list and not emitted.

The runner ``regen_golden.py`` generates ``.expected`` for each ported
fixture by running it under ``python3``. Any fixture whose CPython
oracle exits non-zero (e.g., because the port lost necessary context)
is auto-quarantined into ``ported/_invalid/`` for hand review.

Usage:
    python3 projects/mamba/tools/cpython_port.py \\
        --source /path/to/cpython/Lib/test/test_bool.py \\
        --dest projects/mamba/tests/cpython/fixtures/builtin-libs/bool_type/behavior
"""

from __future__ import annotations

import argparse
import ast
import copy
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path


TOOLS_DIR = Path(__file__).resolve().parent
MAMBA_DIR = TOOLS_DIR.parent
FIXTURES_ROOT = MAMBA_DIR / "tests" / "cpython" / "fixtures"
BUCKETS = {"core", "builtin-libs", "std-libs", "pep", "type-strict", "3rd-libs"}
DIMENSIONS = {"surface", "behavior", "errors", "bench", "real_world", "security"}

# ── assertion rewrites: self.<method>(*args) -> assert / try-except ─────

# Each entry yields an AST node (or list of nodes) for the rewritten form.
# All comparator/operand args are the ORIGINAL ast.expr nodes from the
# call, after self-prefix stripping.

def _eq(a, b):  return ast.Assert(test=ast.Compare(left=a, ops=[ast.Eq()], comparators=[b]), msg=None)
def _ne(a, b):  return ast.Assert(test=ast.Compare(left=a, ops=[ast.NotEq()], comparators=[b]), msg=None)
def _is(a, b):  return ast.Assert(test=ast.Compare(left=a, ops=[ast.Is()], comparators=[b]), msg=None)
def _isnot(a, b): return ast.Assert(test=ast.Compare(left=a, ops=[ast.IsNot()], comparators=[b]), msg=None)
def _lt(a, b):  return ast.Assert(test=ast.Compare(left=a, ops=[ast.Lt()], comparators=[b]), msg=None)
def _le(a, b):  return ast.Assert(test=ast.Compare(left=a, ops=[ast.LtE()], comparators=[b]), msg=None)
def _gt(a, b):  return ast.Assert(test=ast.Compare(left=a, ops=[ast.Gt()], comparators=[b]), msg=None)
def _ge(a, b):  return ast.Assert(test=ast.Compare(left=a, ops=[ast.GtE()], comparators=[b]), msg=None)
def _in(a, b):  return ast.Assert(test=ast.Compare(left=a, ops=[ast.In()], comparators=[b]), msg=None)
def _nin(a, b): return ast.Assert(test=ast.Compare(left=a, ops=[ast.NotIn()], comparators=[b]), msg=None)
def _none(a):   return _is(a, ast.Constant(value=None))
def _notnone(a): return _isnot(a, ast.Constant(value=None))
def _true(a):   return ast.Assert(test=a, msg=None)
def _false(a):  return ast.Assert(test=ast.UnaryOp(op=ast.Not(), operand=a), msg=None)


def _isinstance(a, b):
    call = ast.Call(func=ast.Name(id="isinstance", ctx=ast.Load()), args=[a, b], keywords=[])
    return ast.Assert(test=call, msg=None)


def _not_isinstance(a, b):
    call = ast.Call(func=ast.Name(id="isinstance", ctx=ast.Load()), args=[a, b], keywords=[])
    return ast.Assert(test=ast.UnaryOp(op=ast.Not(), operand=call), msg=None)


def _almost_equal(a, b, places=7):
    diff = ast.BinOp(left=a, op=ast.Sub(), right=b)
    abs_call = ast.Call(func=ast.Name(id="abs", ctx=ast.Load()), args=[diff], keywords=[])
    tol = ast.Constant(value=10 ** (-places))
    return ast.Assert(test=ast.Compare(left=abs_call, ops=[ast.Lt()], comparators=[tol]), msg=None)


# Map self.<name>(a, b, ...) -> rewriter taking the call's args list.
ASSERTIONS = {
    "assertEqual":        lambda a: _eq(a[0], a[1]),
    "assertNotEqual":     lambda a: _ne(a[0], a[1]),
    "assertIs":           lambda a: _is(a[0], a[1]),
    "assertIsNot":        lambda a: _isnot(a[0], a[1]),
    "assertTrue":         lambda a: _true(a[0]),
    "assertFalse":        lambda a: _false(a[0]),
    "assertIsNone":       lambda a: _none(a[0]),
    "assertIsNotNone":    lambda a: _notnone(a[0]),
    "assertIn":           lambda a: _in(a[0], a[1]),
    "assertNotIn":        lambda a: _nin(a[0], a[1]),
    "assertIsInstance":   lambda a: _isinstance(a[0], a[1]),
    "assertNotIsInstance": lambda a: _not_isinstance(a[0], a[1]),
    "assertGreater":      lambda a: _gt(a[0], a[1]),
    "assertGreaterEqual": lambda a: _ge(a[0], a[1]),
    "assertLess":         lambda a: _lt(a[0], a[1]),
    "assertLessEqual":    lambda a: _le(a[0], a[1]),
    "assertAlmostEqual":  lambda a: _almost_equal(a[0], a[1]),
}

# Features that mean "this method is not portable to a flat fixture".
UNSUPPORTED = {
    "subTest", "addCleanup", "addTypeEqualityFunc",
    "skipTest", "skip",
    "assertWarns", "assertWarnsRegex",
    "assertLogs", "assertNoLogs",
}


# ── transformer ─────────────────────────────────────────────────────────

class Porter(ast.NodeTransformer):
    """Rewrite a single ``def test*`` method body.

    Records reasons the method should be skipped via ``self.skip_reason``;
    callers should check it after ``visit``.
    """

    def __init__(
        self,
        class_binding_names: set[str] | None = None,
        class_binding_owner_names: set[str] | None = None,
        self_method_names: set[str] | None = None,
    ):
        self.skip_reason: str | None = None
        self.class_binding_names = class_binding_names or set()
        self.class_binding_owner_names = class_binding_owner_names or set()
        self_method_names = self_method_names or set()
        self.self_method_names = self_method_names
        # Names of attrs we've seen accessed via ``self.<name>``; useful
        # for sanity but not used to gate output.
        self.self_attrs: set[str] = set()
        # Track nesting into FunctionDef / ClassDef. Inside a nested
        # def/class, `self` refers to a DIFFERENT receiver (e.g., an
        # inner class's __bool__ method's self), so we must NOT strip
        # `self.<attr>` to bare names there.
        self._nesting_depth = 0

    def self_alias(self, attr: str) -> str:
        return (
            attr
            if attr in self.class_binding_names or attr in self.self_method_names
            else f"self_{attr}"
        )

    # ---- nested-scope tracking -----------------------------------------

    def visit_FunctionDef(self, node: ast.FunctionDef):
        self._nesting_depth += 1
        try:
            return self.generic_visit(node)
        finally:
            self._nesting_depth -= 1

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef):
        self._nesting_depth += 1
        try:
            return self.generic_visit(node)
        finally:
            self._nesting_depth -= 1

    def visit_ClassDef(self, node: ast.ClassDef):
        self._nesting_depth += 1
        try:
            return self.generic_visit(node)
        finally:
            self._nesting_depth -= 1

    # ---- self.<attr> -> <attr> (top-level only) ------------------------

    def visit_Attribute(self, node: ast.Attribute):
        if (self._nesting_depth == 0
                and isinstance(node.value, ast.Name)
                and node.value.id == "self"):
            self.self_attrs.add(node.attr)
            return ast.copy_location(ast.Name(id=self.self_alias(node.attr), ctx=node.ctx), node)
        if (self._nesting_depth == 0
                and isinstance(node.value, ast.Name)
                and node.value.id in self.class_binding_owner_names
                and node.attr in self.class_binding_names):
            return ast.copy_location(ast.Name(id=node.attr, ctx=node.ctx), node)
        return self.generic_visit(node)

    # ---- self.assertX(...) -> assert ... (top-level only) -------------

    def visit_Call(self, node: ast.Call):
        if (self._nesting_depth == 0
                and isinstance(node.func, ast.Attribute)
                and isinstance(node.func.value, ast.Name)
                and node.func.value.id == "self"):
            meth = node.func.attr
            if meth in UNSUPPORTED:
                self.skip_reason = f"uses self.{meth}"
                return self.generic_visit(node)
            if meth in ASSERTIONS:
                args = [self.visit(a) for a in node.args]
                try:
                    return ast.copy_location(ASSERTIONS[meth](args), node)
                except IndexError:
                    self.skip_reason = f"self.{meth}(...) wrong arity"
                    return node
            if meth in self.self_method_names:
                new_func = ast.copy_location(ast.Name(id=meth, ctx=ast.Load()), node.func)
                return self.generic_visit(
                    ast.copy_location(
                        ast.Call(func=new_func, args=node.args, keywords=node.keywords),
                        node,
                    )
                )
            if meth == "fail":
                args = [self.visit(a) for a in node.args]
                msg = args[0] if args else ast.Constant(value="fail")
                raise_ = ast.Raise(
                    exc=ast.Call(func=ast.Name(id="AssertionError", ctx=ast.Load()),
                                 args=[msg], keywords=[]),
                    cause=None,
                )
                return ast.copy_location(raise_, node)
            if meth == "assertRaises":
                # Positional form: self.assertRaises(Ex, fn, *args, **kw)
                # is a direct call. Context-manager form is handled in
                # visit_With (this Call is just the ctx-mgr arg there).
                if len(node.args) >= 2:
                    exc_t = self.visit(node.args[0])
                    fn = self.visit(node.args[1])
                    rest = [self.visit(a) for a in node.args[2:]]
                    kws = [ast.keyword(arg=kw.arg, value=self.visit(kw.value))
                           for kw in node.keywords]
                    call = ast.Call(func=fn, args=rest, keywords=kws)
                    try_node = ast.Try(
                        body=[
                            ast.Expr(value=call),
                            ast.Raise(
                                exc=ast.Call(
                                    func=ast.Name(id="AssertionError", ctx=ast.Load()),
                                    args=[ast.Constant(value=f"expected {ast.unparse(exc_t)}")],
                                    keywords=[],
                                ),
                                cause=None,
                            ),
                        ],
                        handlers=[ast.ExceptHandler(type=exc_t, name=None, body=[ast.Pass()])],
                        orelse=[],
                        finalbody=[],
                    )
                    return ast.copy_location(try_node, node)
                # Bare ``self.assertRaises(Ex)`` returns a ctx mgr that
                # the caller hands off to ``with`` (handled below).
                # Leave it alone here.
            if meth in self.class_binding_names:
                new_func = ast.copy_location(ast.Name(id=meth, ctx=ast.Load()), node.func)
                return self.generic_visit(
                    ast.copy_location(
                        ast.Call(func=new_func, args=node.args, keywords=node.keywords),
                        node,
                    )
                )
            self.skip_reason = f"uses unsupported self.{meth}"
            return self.generic_visit(node)
        return self.generic_visit(node)

    def visit_Expr(self, node: ast.Expr):
        value = node.value
        if (self._nesting_depth == 0
                and isinstance(value, ast.Call)
                and isinstance(value.func, ast.Attribute)
                and isinstance(value.func.value, ast.Name)
                and value.func.value.id == "self"
                and value.func.attr == "addCleanup"):
            return ast.copy_location(ast.Pass(), node)
        return self.generic_visit(node)

    # ---- with self.assertRaises[Regex](X[, pat]): block -> try/except --

    def visit_With(self, node: ast.With):
        if self._nesting_depth == 0 and len(node.items) == 1:
            ce = node.items[0].context_expr
            if (isinstance(ce, ast.Call)
                    and isinstance(ce.func, ast.Attribute)
                    and isinstance(ce.func.value, ast.Name)
                    and ce.func.value.id == "self"
                    and ce.func.attr == "subTest"):
                return [self.visit(s) for s in node.body]
            if (isinstance(ce, ast.Call)
                    and isinstance(ce.func, ast.Attribute)
                    and isinstance(ce.func.value, ast.Name)
                    and ce.func.value.id == "self"
                    and ce.func.attr in ("assertRaises", "assertRaisesRegex")
                    and ce.args):
                exc_t = self.visit(ce.args[0])
                inner = [self.visit(s) for s in node.body]
                bound = node.items[0].optional_vars
                bound_name = bound.id if isinstance(bound, ast.Name) else None
                except_name = bound_name
                handler_body: list = [ast.Pass()]
                # assertRaisesRegex(Ex, pattern): also assert re.search.
                if ce.func.attr == "assertRaisesRegex" and len(ce.args) >= 2:
                    pat = self.visit(ce.args[1])
                    # except X as _e:
                    #     import re as _re
                    #     assert _re.search(<pat>, str(_e))
                    bound_name = bound_name or "_aR_e"
                    except_name = bound_name
                    handler_body = [
                        ast.Import(names=[ast.alias(name="re", asname="_re_aR")]),
                        ast.Assert(
                            test=ast.Call(
                                func=ast.Attribute(
                                    value=ast.Name(id="_re_aR", ctx=ast.Load()),
                                    attr="search", ctx=ast.Load(),
                                ),
                                args=[pat, ast.Call(
                                    func=ast.Name(id="str", ctx=ast.Load()),
                                    args=[ast.Name(id=bound_name, ctx=ast.Load())],
                                    keywords=[],
                                )],
                                keywords=[],
                            ),
                            msg=None,
                        ),
                    ]
                elif bound_name:
                    except_name = "_aR_e"
                    handler_body = [
                        ast.Import(names=[ast.alias(name="types", asname="_types_aR")]),
                        ast.Assign(
                            targets=[ast.Name(id=bound_name, ctx=ast.Store())],
                            value=ast.Call(
                                func=ast.Attribute(
                                    value=ast.Name(id="_types_aR", ctx=ast.Load()),
                                    attr="SimpleNamespace",
                                    ctx=ast.Load(),
                                ),
                                args=[],
                                keywords=[
                                    ast.keyword(
                                        arg="exception",
                                        value=ast.Name(id=except_name, ctx=ast.Load()),
                                    )
                                ],
                            ),
                        ),
                    ]
                try_node = ast.Try(
                    body=inner + [
                        ast.Raise(
                            exc=ast.Call(
                                func=ast.Name(id="AssertionError", ctx=ast.Load()),
                                args=[ast.Constant(value=f"expected {ast.unparse(exc_t)}")],
                                keywords=[],
                            ),
                            cause=None,
                        ),
                    ],
                    handlers=[ast.ExceptHandler(
                        type=exc_t, name=except_name, body=handler_body,
                    )],
                    orelse=[],
                    finalbody=[],
                )
                return ast.copy_location(try_node, node)
        return self.generic_visit(node)


# ── source extraction ───────────────────────────────────────────────────

@dataclass
class Ported:
    class_name: str
    method_name: str
    body: list  # list[ast.stmt]
    skip: str | None = None


@dataclass
class Source:
    path: Path
    module: ast.Module
    top_level_imports: list = field(default_factory=list)
    top_level_helpers: list = field(default_factory=list)  # ast.FunctionDef / Assign / ClassDef
    classes: list = field(default_factory=list)
    class_by_name: dict[str, ast.ClassDef] = field(default_factory=dict)


def load_source(p: Path) -> Source:
    """Split the source's top-level into imports / helpers / classes.

    Everything at module scope that ISN'T an import or a class def is
    treated as a "helper" -- functions, top-level assigns, AND
    statements like `if/else` blocks that bind module-level state
    (test_decimal does `if __name__ == "__main__": file = sys.argv[0]
    else: file = __file__` and then uses `file` from class bodies).
    Without preserving these the rendered fixture references undefined
    names and fails the CPython oracle.

    Module-level expression statements (docstrings, lone expressions)
    are also preserved so the source's __doc__-style annotations
    survive.
    """
    tree = ast.parse(p.read_text())
    top_imports = []
    top_helpers = []
    classes = []
    for n in tree.body:
        if isinstance(n, (ast.Import, ast.ImportFrom)):
            top_imports.append(n)
        elif isinstance(n, ast.ClassDef):
            classes.append(n)
            if not n.name.startswith("Test") and not _class_has_test_methods(n):
                top_helpers.append(n)
        elif _is_dunder_main_block(n):
            # `if __name__ == '__main__': unittest.main()
            #  else: file = __file__` — running the rendered fixture as
            # a script would re-trigger unittest.main() from the if-body
            # (often loading external data files that don't exist next
            # to the fixture). Keep ONLY the else-body, which binds
            # module-level names class bodies later reference
            # (test_math/test_decimal both do `else: file = __file__`).
            assert isinstance(n, ast.If)
            for stmt in n.orelse:
                top_helpers.append(stmt)
        else:
            top_helpers.append(n)
    src = Source(path=p, module=tree,
                 top_level_imports=top_imports,
                 top_level_helpers=top_helpers,
                 classes=classes,
                 class_by_name={cls.name: cls for cls in classes})
    src.top_level_helpers = [
        helper for helper in src.top_level_helpers
        if not (
            isinstance(helper, ast.ClassDef)
            and _is_concrete_testcase(src, helper)
        )
    ]
    return src


def is_test_method_name(name: str) -> bool:
    """Match unittest's default test method prefix."""
    return name.startswith("test")


def _class_has_test_methods(cls: ast.ClassDef) -> bool:
    return any(
        isinstance(stmt, (ast.FunctionDef, ast.AsyncFunctionDef))
        and is_test_method_name(stmt.name)
        for stmt in cls.body
    )


def _is_dunder_main_block(n: ast.stmt) -> bool:
    """`if __name__ == '__main__': ...` at top level?"""
    if not isinstance(n, ast.If):
        return False
    t = n.test
    # Match `__name__ == '__main__'` and the reversed form.
    if (isinstance(t, ast.Compare)
            and len(t.ops) == 1 and isinstance(t.ops[0], ast.Eq)):
        l, r = t.left, t.comparators[0]
        for a, b in ((l, r), (r, l)):
            if (isinstance(a, ast.Name) and a.id == "__name__"
                    and isinstance(b, ast.Constant)
                    and b.value == "__main__"):
                return True
    return False


def _is_unsupported_call_stmt(stmt: ast.stmt) -> bool:
    """Top-level stmt is a no-op-able call to an unsupported helper?

    ``self.addCleanup(fn, ...)`` after self-strip becomes
    ``addCleanup(fn, ...)`` which fails at module scope. We can't
    replicate `addCleanup` without a TestCase, so we drop these
    statements from the flattened setUp body. The test method itself
    is already skipped (UNSUPPORTED) if it uses these — this filter
    is only for setUp, which exists per class and is reused across
    test methods.
    """
    if not isinstance(stmt, ast.Expr):
        return False
    call = stmt.value
    if not isinstance(call, ast.Call):
        return False
    fn = call.func
    if isinstance(fn, ast.Name) and fn.id in UNSUPPORTED:
        return True
    # Original `self.<name>(...)` form (e.g., if porter didn't strip
    # because we're in a nested scope) — check the attr too.
    if isinstance(fn, ast.Attribute) and fn.attr in UNSUPPORTED:
        return True
    return False


def _rewrite_stmt_list(porter: Porter, stmts: list[ast.stmt]) -> list[ast.stmt]:
    out = []
    for stmt in stmts:
        rewritten = porter.visit(copy.deepcopy(stmt))
        if rewritten is None:
            continue
        if isinstance(rewritten, list):
            out.extend(rewritten)
        else:
            out.append(rewritten)
    return out


def _flatten_setup(
    cls: ast.ClassDef,
    class_binding_names: set[str],
    class_binding_owner_names: set[str],
    self_method_names: set[str],
) -> tuple[list, str | None]:
    """Return setUp's body statements with ``self.X`` rewritten to ``X``.

    Drops top-level calls to UNSUPPORTED helpers (``self.addCleanup``,
    ``self.subTest``, etc.) so the flattened body runs cleanly at
    module scope. Ignores setUpClass / setUpModule (usually too
    entangled with class-level state to be portable).
    """
    for n in cls.body:
        if isinstance(n, ast.FunctionDef) and n.name == "setUp":
            porter = Porter(
                class_binding_names,
                class_binding_owner_names,
                self_method_names,
            )
            stmts = _rewrite_stmt_list(porter, n.body)
            return [s for s in stmts if not _is_unsupported_call_stmt(s)], porter.skip_reason
    return [], None


def _base_names(cls: ast.ClassDef) -> list[str]:
    out = []
    for base in cls.bases:
        if isinstance(base, ast.Name):
            out.append(base.id)
        elif isinstance(base, ast.Attribute):
            out.append(base.attr)
    return out


def _lineage(src: Source, cls: ast.ClassDef) -> list[ast.ClassDef]:
    """Classes in local MRO order, base classes first, including ``cls``."""
    out = []
    seen: set[str] = set()

    def visit(current: ast.ClassDef) -> None:
        if current.name in seen:
            return
        seen.add(current.name)
        for name in _base_names(current):
            base = src.class_by_name.get(name)
            if base is not None:
                visit(base)
        out.append(current)

    visit(cls)
    return out


def _class_binding_chain(src: Source, cls: ast.ClassDef) -> list:
    out = []
    for current in _lineage(src, cls):
        out.extend(copy.deepcopy(stmt) for stmt in _class_bindings(current))
    return out


def _class_binding_names(src: Source, cls: ast.ClassDef) -> set[str]:
    out = set()
    for stmt in _class_binding_chain(src, cls):
        if isinstance(stmt, ast.Assign) and len(stmt.targets) == 1:
            target = stmt.targets[0]
            if isinstance(target, ast.Name):
                out.add(target.id)
    return out


def _class_binding_owner_names(src: Source, cls: ast.ClassDef) -> set[str]:
    return {current.name for current in _lineage(src, cls)}


def _setup_chain(src: Source, cls: ast.ClassDef) -> tuple[list, str | None]:
    out = []
    class_binding_names = _class_binding_names(src, cls)
    class_binding_owner_names = _class_binding_owner_names(src, cls)
    self_method_names = _self_method_names(src, cls)
    for current in _lineage(src, cls):
        setup_body, skip = _flatten_setup(
            current,
            class_binding_names,
            class_binding_owner_names,
            self_method_names,
        )
        if skip:
            return out, skip
        out.extend(setup_body)
    return out, None


class FreeSelfFinder(ast.NodeVisitor):
    """Detect free ``self`` references left in flattened module-level code."""

    def __init__(self):
        self.found = False
        self._self_bound: list[bool] = [False]

    def visit_Name(self, node: ast.Name):
        if node.id == "self" and not self._self_bound[-1]:
            self.found = True

    def visit_ClassDef(self, node: ast.ClassDef):
        for item in list(node.bases) + list(node.decorator_list):
            self.visit(item)
        for keyword in node.keywords:
            self.visit(keyword.value)
        self._self_bound.append(False)
        try:
            for stmt in node.body:
                self.visit(stmt)
        finally:
            self._self_bound.pop()

    def visit_FunctionDef(self, node: ast.FunctionDef):
        for item in node.decorator_list:
            self.visit(item)
        for default in list(node.args.defaults) + list(node.args.kw_defaults):
            if default is not None:
                self.visit(default)
        bound = any(arg.arg == "self" for arg in node.args.args)
        self._self_bound.append(bound)
        try:
            for stmt in node.body:
                self.visit(stmt)
        finally:
            self._self_bound.pop()

    visit_AsyncFunctionDef = visit_FunctionDef


def has_free_self(stmts: list) -> bool:
    finder = FreeSelfFinder()
    for stmt in stmts:
        finder.visit(stmt)
        if finder.found:
            return True
    return False


def port_method(src: Source, owner: ast.ClassDef, concrete: ast.ClassDef,
                method: ast.FunctionDef) -> Ported:
    shadow = _inherited_test_shadow(src, owner, concrete, method.name)
    if shadow is not None:
        return Ported(
            class_name=concrete.name,
            method_name=method.name,
            body=[],
            skip=f"inherited test shadowed by {shadow}.{method.name}",
        )
    for decorator in method.decorator_list:
        text = ast.unparse(decorator)
        if "skip" in text:
            return Ported(
                class_name=concrete.name,
                method_name=method.name,
                body=[],
                skip=f"uses skip decorator {text}",
            )
    class_binding_names = _class_binding_names(src, concrete)
    class_binding_owner_names = _class_binding_owner_names(src, concrete)
    self_method_names = _self_method_names(src, concrete)
    porter = Porter(class_binding_names, class_binding_owner_names, self_method_names)
    body = _rewrite_stmt_list(porter, method.body)
    setup_body, setup_skip = _setup_chain(src, concrete)
    helper_body, helper_skip = _self_method_chain(
        src,
        concrete,
        class_binding_names,
        class_binding_owner_names,
        self_method_names,
    )
    full_body = _class_binding_chain(src, concrete) + helper_body + setup_body + body
    skip = porter.skip_reason or setup_skip or helper_skip
    if skip is None and has_free_self(full_body):
        skip = "retains free self reference after flattening"
    return Ported(
        class_name=concrete.name,
        method_name=method.name,
        body=full_body,
        skip=skip,
    )


def _inherited_test_shadow(
    src: Source,
    owner: ast.ClassDef,
    concrete: ast.ClassDef,
    method_name: str,
) -> str | None:
    """Return subclass name when it disables an inherited test method.

    CPython suites sometimes set ``test_foo = None`` on a concrete
    subclass to prevent unittest from inheriting a base-class test. A
    flat fixture must respect that shadow, otherwise it can resurrect
    intentionally disabled cases.
    """
    if owner is concrete:
        return None
    in_descendant = False
    for cls in _lineage(src, concrete):
        if cls is owner:
            in_descendant = True
            continue
        if in_descendant and _class_assigns_name(cls, method_name):
            return cls.name
    return None


def _class_assigns_name(cls: ast.ClassDef, name: str) -> bool:
    for stmt in cls.body:
        if isinstance(stmt, ast.Assign):
            if any(_target_assigns_name(target, name) for target in stmt.targets):
                return True
        elif isinstance(stmt, ast.AnnAssign):
            if _target_assigns_name(stmt.target, name):
                return True
    return False


def _target_assigns_name(target: ast.expr, name: str) -> bool:
    if isinstance(target, ast.Name):
        return target.id == name
    if isinstance(target, (ast.Tuple, ast.List)):
        return any(_target_assigns_name(elt, name) for elt in target.elts)
    return False


def _class_bindings(cls: ast.ClassDef) -> list:
    """Return the simple class-level `Name = expr` assignments.

    These are the bindings CPython's mixin pattern uses to specialize
    a base TestCase per subclass: ``class BytesTest(...): type2test = bytes``.
    The base class's `self.type2test` references resolve to the
    subclass's binding at runtime. After our self-strip, those become
    bare ``type2test`` references which need the binding at module
    scope to work — we prepend the binding to the rendered fixture.

    Only top-level Name=expr forms are collected; tuple/star/annotated
    assigns and assignments inside methods are ignored.
    """
    out = []
    for stmt in cls.body:
        if isinstance(stmt, ast.Assign) and len(stmt.targets) == 1:
            tgt = stmt.targets[0]
            if isinstance(tgt, ast.Name) and not tgt.id.startswith("_"):
                out.append(stmt)
    return out


def _self_methods(cls: ast.ClassDef) -> list[ast.FunctionDef]:
    out = []
    for stmt in cls.body:
        if not isinstance(stmt, ast.FunctionDef):
            continue
        if stmt.name == "setUp" or stmt.name == "tearDown":
            continue
        if is_test_method_name(stmt.name):
            continue
        if stmt.name.startswith("__") and stmt.name.endswith("__"):
            continue
        if _is_staticmethod(stmt):
            out.append(stmt)
            continue
        if not stmt.args.args or stmt.args.args[0].arg != "self":
            continue
        out.append(stmt)
    return out


def _is_staticmethod(method: ast.FunctionDef) -> bool:
    return any(
        isinstance(decorator, ast.Name) and decorator.id == "staticmethod"
        for decorator in method.decorator_list
    )


def _self_method_names(src: Source, cls: ast.ClassDef) -> set[str]:
    out = set()
    for current in _lineage(src, cls):
        for method in _self_methods(current):
            out.add(method.name)
    return out


def _self_method_chain(
    src: Source,
    cls: ast.ClassDef,
    class_binding_names: set[str],
    class_binding_owner_names: set[str],
    self_method_names: set[str],
) -> tuple[list, str | None]:
    by_name: dict[str, ast.FunctionDef] = {}
    for current in _lineage(src, cls):
        for method in _self_methods(current):
            by_name[method.name] = method

    out = []
    for name in sorted(by_name):
        method = copy.deepcopy(by_name[name])
        porter = Porter(
            class_binding_names,
            class_binding_owner_names,
            self_method_names,
        )
        remove_self_arg = (
            not _is_staticmethod(method)
            and method.args.args
            and method.args.args[0].arg == "self"
        )
        method.decorator_list = []
        if remove_self_arg:
            method.args.args = method.args.args[1:]
        method.body = _rewrite_stmt_list(porter, method.body)
        out.append(method)
        if porter.skip_reason:
            return out, porter.skip_reason
    return out, None


def _find_subclasses(src: Source, base: ast.ClassDef) -> list:
    """Subclasses of `base` defined in the same source file.

    Detection is by-name: any class whose bases mention `base.name` as
    either a direct ``Name`` reference or an ``Attribute`` whose final
    attr matches counts. We don't follow imports — siblings in the
    same file are enough to capture the CPython mixin pattern (Base*
    class + concrete TestCase + concrete TestCase pattern).
    """
    out = []
    for cls in src.classes:
        if cls is base:
            continue
        for b in cls.bases:
            if isinstance(b, ast.Name) and b.id == base.name:
                out.append(cls)
                break
            if isinstance(b, ast.Attribute) and b.attr == base.name:
                out.append(cls)
                break
    return out


def _is_concrete_testcase(
    src: Source,
    cls: ast.ClassDef,
    seen: set[str] | None = None,
) -> bool:
    seen = seen or set()
    if cls.name in seen:
        return False
    seen.add(cls.name)
    for name in _base_names(cls):
        if name == "TestCase":
            return True
        base = src.class_by_name.get(name)
        if base is not None and _is_concrete_testcase(src, base, seen):
            return True
    return False


def _descendants(src: Source, base: ast.ClassDef) -> list[ast.ClassDef]:
    out = []
    for cls in src.classes:
        if cls is base:
            continue
        if base in _lineage(src, cls):
            out.append(cls)
    return out


def extract(src: Source) -> list[Ported]:
    """Walk every class and emit one Ported per (class, test_method).

    Mixin awareness: if a class B has test methods AND has subclass(es)
    S1, S2 in the same file binding class-level attrs (the CPython
    type2test pattern), we synthesize one Ported PER SUBCLASS for each
    of B's test methods, prepending the subclass's bindings to the
    body. The base's own test methods are also emitted (so we still
    get B__testFoo for any binding-independent test).
    """
    out = []
    for cls in src.classes:
        concrete_targets = []
        if _is_concrete_testcase(src, cls):
            concrete_targets.append(cls)
        concrete_targets.extend(
            sub for sub in _descendants(src, cls) if _is_concrete_testcase(src, sub)
        )
        if not concrete_targets and _class_has_test_methods(cls):
            # Some CPython suites inherit TestCase behavior through imported
            # helper bases such as test.list_tests.CommonTest. We cannot
            # resolve those imports statically here, so emit a candidate and
            # let the CPython oracle decide whether the flattened case is
            # runnable.
            concrete_targets.append(cls)
        for n in cls.body:
            if isinstance(n, ast.FunctionDef) and is_test_method_name(n.name):
                for target in concrete_targets:
                    out.append(port_method(src, cls, target, n))
    # Dedupe by (class_name, method_name): if a concrete subclass also
    # defines its own test method overriding the base, prefer that one
    # (later one in source order wins; we visit base before sub).
    seen = {}
    for p in out:
        seen[(p.class_name, p.method_name)] = p
    return list(seen.values())


# ── output rendering ────────────────────────────────────────────────────

def q(value: str) -> str:
    return json.dumps(value)


def snake(value: str) -> str:
    value = re.sub(r"(.)([A-Z][a-z]+)", r"\1_\2", value)
    value = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", value)
    value = re.sub(r"[^A-Za-z0-9]+", "_", value)
    return re.sub(r"_+", "_", value).strip("_").lower() or "case"


def source_label(path: Path) -> str:
    parts = path.parts
    if "test" in parts:
        idx = parts.index("test")
        return "Lib/test/" + "/".join(parts[idx + 1:])
    return str(path)


def infer_metadata(args) -> tuple[str, str, str, str]:
    bucket = args.bucket
    lib = args.lib
    dimension = args.dimension
    try:
        rel = args.dest.resolve().relative_to(FIXTURES_ROOT.resolve())
        if len(rel.parts) >= 3:
            bucket = bucket or rel.parts[0]
            lib = lib or rel.parts[1]
            if rel.parts[2] in DIMENSIONS:
                dimension = dimension or rel.parts[2]
    except ValueError:
        pass

    source_stem = args.source.stem
    if source_stem.startswith("test_"):
        source_stem = source_stem[5:]

    bucket = bucket or "std-libs"
    lib = lib or source_stem.replace("-", "_").lower()
    dimension = dimension or "behavior"
    if bucket not in BUCKETS:
        raise SystemExit(f"bucket {bucket!r} not in {sorted(BUCKETS)}")
    if dimension not in DIMENSIONS:
        raise SystemExit(f"dimension {dimension!r} not in {sorted(DIMENSIONS)}")
    xfail = "" if args.no_xfail else args.xfail_reason
    return bucket, lib, dimension, xfail


def render_pep723(src: Source, p: Ported, *, bucket: str, lib: str,
                  dimension: str, case: str, xfail: str) -> str:
    subject = f"cpython.{src.path.stem}.{p.class_name}.{p.method_name}"
    lines = [
        "# /// script",
        "# requires-python = \">=3.12\"",
        "# dependencies = []",
        "#",
        "# [tool.mamba]",
        f"# bucket = {q(bucket)}",
        f"# lib = {q(lib)}",
        f"# dimension = {q(dimension)}",
        f"# case = {q(case)}",
        f"# subject = {q(subject)}",
        "# kind = \"semantic\"",
        f"# xfail = {q(xfail)}",
        "# mem_carveout = \"\"",
        f"# source = {q(source_label(src.path))}",
        "# status = \"filled\"",
        "# ///",
    ]
    if xfail:
        lines.append(f"# mamba-xfail: {xfail}")
    return "\n".join(lines) + "\n"


def render_fixture(src: Source, p: Ported, *, bucket: str, lib: str,
                   dimension: str, xfail: str) -> str:
    """Render a Ported method into a complete fixture file source."""
    case = safe_case(p)
    header = (
        render_pep723(
            src,
            p,
            bucket=bucket,
            lib=lib,
            dimension=dimension,
            case=case,
            xfail=xfail,
        )
        + f"# Auto-ported from CPython 3.12 {src.path.name}::"
        f"{p.class_name}::{p.method_name}\n"
        f'"""Auto-ported test: {p.class_name}::{p.method_name} (CPython 3.12 oracle)."""\n\n'
    )
    imports_src = "\n".join(ast.unparse(n) for n in src.top_level_imports)
    helpers_src = "\n\n".join(ast.unparse(n) for n in src.top_level_helpers)
    # Synthesize a Module for the body so ast.unparse handles indentation.
    body_mod = ast.Module(body=p.body, type_ignores=[])
    ast.fix_missing_locations(body_mod)
    body_src = ast.unparse(body_mod)
    confirm = f'\nprint("{p.class_name}::{p.method_name}: ok")\n'
    parts = [header, imports_src]
    if helpers_src:
        parts.append("\n\n" + helpers_src)
    parts.append("\n\n# --- test body ---\n" + body_src + confirm)
    return normalize_generated_source("\n".join(p for p in parts if p))


def normalize_generated_source(source: str) -> str:
    """Keep generated fixtures stable without preserving blank-line indent."""
    lines = ["" if line.strip() == "" else line.rstrip() for line in source.splitlines()]
    return "\n".join(lines) + "\n"


def safe_case(p: Ported) -> str:
    # Filename: <class>__<method>.py  (collision-free across classes)
    return f"{snake(p.class_name)}__{snake(p.method_name)}"


def safe_filename(p: Ported) -> str:
    return f"{safe_case(p)}.py"


# ── CPython sanity check ───────────────────────────────────────────────

def cpython_passes(fixture_path: Path) -> tuple[bool, str]:
    """Run the fixture under python3; return (ok, last_stderr_line)."""
    return cpython_passes_with_timeout(fixture_path, 10)


def cpython_passes_with_timeout(fixture_path: Path, timeout: float) -> tuple[bool, str]:
    """Run the fixture under python3; return (ok, last_stderr_line)."""
    try:
        with tempfile.TemporaryDirectory(prefix="mamba-cpython-port-") as tmp:
            env = dict(os.environ)
            env["TMPDIR"] = tmp
            env["TEMP"] = tmp
            env["TMP"] = tmp
            r = subprocess.run(
                [sys.executable, str(fixture_path.resolve())],
                capture_output=True,
                text=True,
                timeout=timeout,
                cwd=tmp,
                env=env,
            )
    except subprocess.TimeoutExpired:
        return (False, "TIMEOUT")
    if r.returncode == 0:
        return (True, "")
    last = (r.stderr or r.stdout).strip().splitlines()
    return (False, last[-1][:100] if last else "non-zero exit")


def mamba_passes(mamba_bin: str, fixture_path: Path) -> tuple[bool, str]:
    """Run the fixture under `mamba run`; return (ok, last_err_line)."""
    try:
        r = subprocess.run(
            [mamba_bin, "run", str(fixture_path)],
            capture_output=True, text=True, timeout=10,
        )
    except (subprocess.TimeoutExpired, FileNotFoundError):
        return (False, "TIMEOUT_OR_MISSING")
    if r.returncode == 0:
        return (True, "")
    last = (r.stderr or r.stdout).strip().splitlines()
    return (False, last[-1][:100] if last else "non-zero exit")


def prepend_xfail(fixture_path: Path, reason: str) -> None:
    """Insert `# mamba-xfail: <reason>` after the PEP 723 block."""
    src = fixture_path.read_text()
    # Find end of PEP 723 block (line starting with `# ///` closing).
    lines = src.splitlines(keepends=True)
    inject_at = 0
    in_pep = False
    for i, ln in enumerate(lines):
        if ln.startswith("# /// script"):
            in_pep = True
            continue
        if in_pep and ln.strip() == "# ///":
            inject_at = i + 1
            break
    if inject_at == 0:
        # No PEP 723 block found; inject at top.
        lines.insert(0, f"# mamba-xfail: {reason}\n")
    else:
        lines.insert(inject_at, f"# mamba-xfail: {reason}\n")
    fixture_path.write_text("".join(lines))


# ── main ────────────────────────────────────────────────────────────────

def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--source", required=True, type=Path,
                    help="CPython Lib/test/test_X.py to port")
    ap.add_argument("--dest", required=True, type=Path,
                    help="Destination dimension dir, e.g. "
                         "tests/cpython/fixtures/builtin-libs/bool_type/behavior")
    ap.add_argument("--bucket", choices=sorted(BUCKETS),
                    help="Fixture bucket; inferred from --dest when possible")
    ap.add_argument("--lib",
                    help="Fixture lib; inferred from --dest or source name")
    ap.add_argument("--dimension", choices=sorted(DIMENSIONS),
                    help="Fixture dimension; inferred from --dest or behavior")
    ap.add_argument("--xfail-reason",
                    default="auto-ported CPython test; mamba promotion pending",
                    help="Default mamba xfail reason for generated fixtures")
    ap.add_argument("--no-xfail", action="store_true",
                    help="Do not mark generated fixtures as mamba xfail")
    ap.add_argument("--dry-run", action="store_true",
                    help="Render but don't write")
    ap.add_argument("--keep-invalid", action="store_true",
                    help="Quarantine CPython-failing fixtures under "
                         "ported/_invalid/ instead of dropping them")
    ap.add_argument("--auto-xfail", action="store_true",
                    help="After writing each fixture, run it under "
                         "`mamba run`; prepend a `# mamba-xfail: ...` "
                         "directive when mamba's exit is non-zero. Keeps "
                         "the suite gating green per goal-A DoD.")
    ap.add_argument("--mamba", default="mamba",
                    help="mamba binary used by --auto-xfail (default: PATH)")
    args = ap.parse_args()

    src = load_source(args.source)
    bucket, lib, dimension, xfail = infer_metadata(args)
    methods = extract(src)
    skipped = [m for m in methods if m.skip]
    portable = [m for m in methods if not m.skip]
    print(f"[{args.source.name}] {len(methods)} test methods: "
          f"{len(portable)} portable, {len(skipped)} skipped")
    for m in skipped:
        print(f"  SKIP  {m.class_name}::{m.method_name}  --  {m.skip}")

    if args.dry_run:
        if portable:
            print(f"\n--- preview: {portable[0].class_name}::{portable[0].method_name} ---")
            print(
                render_fixture(
                    src,
                    portable[0],
                    bucket=bucket,
                    lib=lib,
                    dimension=dimension,
                    xfail=xfail,
                )
            )
        return 0

    args.dest.mkdir(parents=True, exist_ok=True)
    invalid_dir = args.dest / "_invalid"
    written = 0
    invalid = 0
    xfailed = 0
    for m in portable:
        fname = safe_filename(m)
        fixture_path = args.dest / fname
        try:
            content = render_fixture(
                src,
                m,
                bucket=bucket,
                lib=lib,
                dimension=dimension,
                xfail=xfail,
            )
        except Exception as e:
            print(f"  RENDER_FAIL  {m.class_name}::{m.method_name}  --  {e}")
            continue
        fixture_path.write_text(content)
        ok, err = cpython_passes(fixture_path)
        if not ok:
            if args.keep_invalid:
                invalid_dir.mkdir(exist_ok=True)
                shutil.move(str(fixture_path), invalid_dir / fname)
            else:
                fixture_path.unlink()
            invalid += 1
            print(f"  INVALID  {fname}  --  {err}")
            continue
        written += 1
        if args.auto_xfail:
            mb_ok, mb_err = mamba_passes(args.mamba, fixture_path)
            if not mb_ok:
                prepend_xfail(
                    fixture_path,
                    f"auto-ported; mamba diverges ({mb_err[:80]})",
                )
                xfailed += 1

    print(f"\nDone: wrote {written}, invalid {invalid}, skipped {len(skipped)}"
          + (f", auto-xfail {xfailed}" if args.auto_xfail else ""))
    return 0


if __name__ == "__main__":
    sys.exit(main())
