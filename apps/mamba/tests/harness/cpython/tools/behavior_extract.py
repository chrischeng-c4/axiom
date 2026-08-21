#!/usr/bin/env python3.12
"""Inline-body extractor — flatten a CPython 3.12 Lib/test TestClass.test_method
into a standalone behavior fixture (the high-fidelity form, NOT a stub).

The transformation (AST-based):
  - inline the test file's module-level imports (minus unittest / test.support)
  - inline class-level data assignments as module globals
  - inline setUp's `self.x = ...` as `self_x = ...`
  - inline helper methods (non-test class defs) as module functions (drop self)
  - the target method body, with:
      self.assertEqual(a, b)  -> assert a == b
      self.assertRaises(E, f, *a) -> try: f(*a); raise AssertionError except E: pass
      self.helper(...)        -> helper(...)
      self.attr               -> self_attr
  - trailing print("<Class>::<method>: ok")

Methods that use patterns this flattener can't faithfully reproduce
(`with self.assertRaises(...)`, subTest, mock, addCleanup, skips, resources,
super(), unknown self.X) are reported as SKIP with a reason — never emitted as a
weakened or oracle-red fixture.

    python3.12 behavior_extract.py --module test_calendar --class SundayTestCase --method test_april
    python3.12 behavior_extract.py --module test_calendar           # whole module, all classes
    python3.12 behavior_extract.py --module test_calendar --emit     # actually write fixtures
"""

from __future__ import annotations

import argparse
import ast
import subprocess
import sys
from pathlib import Path

import os

sys.path.insert(0, str(Path(__file__).resolve().parent))
from wall_gen_core import PEP723Header, parse_header  # noqa: E402

MAMBA_DIR = Path(__file__).resolve().parents[4]
CPYTHON_DIR = MAMBA_DIR / "tests" / "cpython"
# The canonical denominator-exclusion file. A parallel batch driver overrides it
# per-worker (MAMBA_BEHAVIOR_GAPS_FILE) to avoid concurrent-append races, then
# merges the shards back into the canonical file.
GAPS_FILE = Path(os.environ.get(
    "MAMBA_BEHAVIOR_GAPS_FILE",
    MAMBA_DIR / "tests" / "harness" / "cpython" / "config" / "behavior_gaps.txt"))

# unittest assert method -> (template builder). Each takes the call's arg nodes
# and returns an ast statement (Assert or a small try/except).
_CMP = {
    "assertEqual": ast.Eq, "assertNotEqual": ast.NotEq,
    "assertIs": ast.Is, "assertIsNot": ast.IsNot,
    "assertIn": ast.In, "assertNotIn": ast.NotIn,
    "assertGreater": ast.Gt, "assertGreaterEqual": ast.GtE,
    "assertLess": ast.Lt, "assertLessEqual": ast.LtE,
}
_UNARY_TRUE = {"assertTrue", "assert_"}
_UNARY_FALSE = {"assertFalse"}
_IS_NONE = {"assertIsNone": True, "assertIsNotNone": False}

# self.X references the flattener refuses to guess at -> skip the whole method.
_UNSUPPORTED_CALLS = {
    "subTest", "skipTest", "addCleanup", "addTypeEqualityFunc", "assertWarns",
    "assertLogs", "assertRaisesRegex", "assertWarnsRegex", "enterContext",
    "run", "defaultTestResult", "fail",
}
_SUPPORTED_SELF = (
    set(_CMP) | _UNARY_TRUE | _UNARY_FALSE | set(_IS_NONE)
    | {"assertRaises", "assertIsInstance", "assertNotIsInstance",
       "assertAlmostEqual", "assertNotAlmostEqual", "assertCountEqual",
       "assertListEqual", "assertDictEqual", "assertTupleEqual",
       "assertSetEqual", "assertSequenceEqual", "assertMultiLineEqual"}
)


class Unsupported(Exception):
    pass


class _Flatten(ast.NodeTransformer):
    """Rewrite self.* within a method/helper body."""

    def __init__(self, helpers: set[str], self_attrs: set[str],
                 class_data: set[str] | None = None):
        self.helpers = helpers
        self.self_attrs = self_attrs          # setUp-assigned -> self_X
        self.class_data = class_data or set()  # class-level data -> X (module global)

    def _is_self(self, node) -> bool:
        return isinstance(node, ast.Name) and node.id == "self"

    def visit_Expr(self, node):
        # self.assertX(...) statement -> assert / try-except. Helper and other
        # self.* calls fall through to visit_Call.
        call = node.value
        if (isinstance(call, ast.Call) and isinstance(call.func, ast.Attribute)
                and self._is_self(call.func.value)):
            name = call.func.attr
            if name in _UNSUPPORTED_CALLS:
                raise Unsupported(f"self.{name}(...)")
            if name in _SUPPORTED_SELF:
                return ast.copy_location(self._assert_stmt(name, call), node)
        self.generic_visit(node)
        return node

    def _assert_stmt(self, name, call):
        a = [self.visit(arg) for arg in call.args]
        if name in _CMP and len(a) >= 2:
            return ast.Assert(test=ast.Compare(a[0], [_CMP[name]()], [a[1]]), msg=None)
        if name in _UNARY_TRUE and len(a) >= 1:
            return ast.Assert(test=a[0], msg=None)
        if name in _UNARY_FALSE and len(a) >= 1:
            return ast.Assert(test=ast.UnaryOp(ast.Not(), a[0]), msg=None)
        if name in _IS_NONE and len(a) >= 1:
            op = ast.Is() if _IS_NONE[name] else ast.IsNot()
            return ast.Assert(test=ast.Compare(a[0], [op], [ast.Constant(None)]), msg=None)
        if name in {"assertIsInstance", "assertNotIsInstance"} and len(a) >= 2:
            t = ast.Call(ast.Name("isinstance", ast.Load()), [a[0], a[1]], [])
            if name == "assertNotIsInstance":
                t = ast.UnaryOp(ast.Not(), t)
            return ast.Assert(test=t, msg=None)
        if name in {"assertAlmostEqual", "assertNotAlmostEqual"} and len(a) >= 2:
            diff = ast.Call(ast.Name("abs", ast.Load()),
                            [ast.BinOp(a[0], ast.Sub(), a[1])], [])
            cmp = ast.Compare(diff, [ast.Lt()], [ast.Constant(1e-7)])
            if name == "assertNotAlmostEqual":
                cmp = ast.UnaryOp(ast.Not(), cmp)
            return ast.Assert(test=cmp, msg=None)
        if name in {"assertEqual", "assertListEqual", "assertDictEqual",
                    "assertTupleEqual", "assertSetEqual", "assertSequenceEqual",
                    "assertMultiLineEqual", "assertCountEqual"} and len(a) >= 2:
            return ast.Assert(test=ast.Compare(a[0], [ast.Eq()], [a[1]]), msg=None)
        if name == "assertRaises" and len(a) >= 2:
            exc, fn, rest = a[0], a[1], a[2:]
            body = [ast.Expr(ast.Call(fn, rest, [])),
                    ast.Raise(ast.Call(ast.Name("AssertionError", ast.Load()),
                                       [ast.Constant(f"{name}: no raise")], []), None)]
            handler = ast.ExceptHandler(type=exc, name=None, body=[ast.Pass()])
            return ast.Try(body=body, handlers=[handler], orelse=[], finalbody=[])
        raise Unsupported(f"assert form self.{name}/{len(a)}")

    def visit_Call(self, node):
        # self.helper(...) -> helper(...)
        if (isinstance(node.func, ast.Attribute) and self._is_self(node.func.value)):
            name = node.func.attr
            if name in _UNSUPPORTED_CALLS:
                raise Unsupported(f"self.{name}(...)")
            if name in self.helpers:
                node.func = ast.copy_location(ast.Name(name, ast.Load()), node.func)
                node.args = [self.visit(a) for a in node.args]
                node.keywords = [self.visit(k) for k in node.keywords]
                return node
            if name in _SUPPORTED_SELF:  # assert used as expression (rare) — unsupported here
                raise Unsupported(f"self.{name} as expr")
        self.generic_visit(node)
        return node

    def visit_Attribute(self, node):
        # self.X -> X (class data, inlined as a module global) or self_X (setUp
        # attr). Unknown self.X is something we can't faithfully inline -> skip.
        if self._is_self(node.value):
            if node.attr in self.class_data:
                return ast.copy_location(ast.Name(node.attr, node.ctx), node)
            if node.attr in self.self_attrs:
                return ast.copy_location(ast.Name(f"self_{node.attr}", node.ctx), node)
            if node.attr in self.helpers:
                return ast.copy_location(ast.Name(node.attr, node.ctx), node)
            raise Unsupported(f"self.{node.attr}")
        self.generic_visit(node)
        return node


def _module_imports(tree: ast.Module) -> list[ast.stmt]:
    out = []
    for n in tree.body:
        if isinstance(n, (ast.Import, ast.ImportFrom)):
            mod = getattr(n, "module", "") or ""
            names = ",".join(a.name for a in n.names)
            if "unittest" in mod or "unittest" in names:
                continue
            if mod.startswith("test.") or names.startswith("test.") or mod == "test":
                continue
            out.append(n)
    return out


def _find_class(tree: ast.Module, cls: str):
    for n in tree.body:
        if isinstance(n, ast.ClassDef) and n.name == cls:
            return n
    return None


def extract(test_src: str, cls_name: str, method: str) -> str:
    """Return a standalone fixture body or raise Unsupported."""
    tree = ast.parse(test_src)
    cls = _find_class(tree, cls_name)
    if cls is None:
        raise Unsupported(f"class {cls_name} not found")
    class_map = {c.name: c for c in tree.body if isinstance(c, ast.ClassDef)}

    # Resolve the inheritance chain inside this file (most-derived first). Bases
    # must be plain names/attributes and either reach a *TestCase or be defined
    # in this module; anything else (mixin from another module) -> skip.
    chain: list[ast.ClassDef] = []
    seen: set[str] = set()
    reaches_testcase = False
    work = [cls]
    while work:
        c = work.pop(0)
        if c.name in seen:
            continue
        seen.add(c.name)
        chain.append(c)
        for b in c.bases:
            if not isinstance(b, (ast.Name, ast.Attribute)):
                raise Unsupported("exotic base expression")
            bn = b.attr if isinstance(b, ast.Attribute) else b.id
            if bn in class_map:
                work.append(class_map[bn])      # same-file ancestor — follow it
            elif "TestCase" in bn:
                reaches_testcase = True          # unittest.TestCase (terminal)
            elif bn == "object":
                pass
            else:
                raise Unsupported(f"base `{bn}` defined outside this module")
    if not reaches_testcase:
        raise Unsupported("does not reach TestCase")

    # collect members across the chain, base-first so a derived class overrides.
    class_data: list[ast.stmt] = []
    class_data_names: set[str] = set()
    helpers: dict[str, ast.FunctionDef] = {}
    setup = None
    target = None
    for c in reversed(chain):
        for m in c.body:
            if isinstance(m, ast.Assign):
                class_data.append(m)
                for t in m.targets:
                    if isinstance(t, ast.Name):
                        class_data_names.add(t.id)
            elif isinstance(m, (ast.FunctionDef, ast.AsyncFunctionDef)):
                if m.name == method:
                    target = m
                elif m.name == "setUp":
                    setup = m
                elif m.name in ("tearDown", "setUpClass", "tearDownClass", "asyncSetUp"):
                    pass  # dropped
                elif not m.name.startswith("test"):
                    helpers[m.name] = m  # type: ignore[assignment]
    if target is None:
        raise Unsupported(f"method {method} not found")
    if target.decorator_list:
        raise Unsupported("decorated method (skip/expectedFailure/etc)")
    if target.args.args and len(target.args.args) > 1:
        raise Unsupported("method takes args beyond self")

    # self attrs assignable: class-data names + setUp self.x targets
    self_attrs: set[str] = set()
    setup_stmts: list[ast.stmt] = []
    if setup is not None:
        if setup.decorator_list:
            raise Unsupported("decorated setUp")
        for s in setup.body:
            for t in ast.walk(s):
                if (isinstance(t, ast.Attribute) and isinstance(t.value, ast.Name)
                        and t.value.id == "self" and isinstance(t.ctx, ast.Store)):
                    self_attrs.add(t.attr)
        setup_stmts = setup.body

    flat = _Flatten(set(helpers), self_attrs, class_data_names)

    # transform helpers (drop self param, rewrite body)
    helper_defs: list[ast.stmt] = []
    for h in helpers.values():
        if h.decorator_list:
            continue  # property/staticmethod helper — skip emitting; refs may fail later
        newargs = ast.arguments(
            posonlyargs=[], args=[a for a in h.args.args if a.arg != "self"],
            vararg=h.args.vararg, kwonlyargs=h.args.kwonlyargs,
            kw_defaults=h.args.kw_defaults, kwarg=h.args.kwarg, defaults=h.args.defaults)
        body = [flat.visit(ast.fix_missing_locations(s)) for s in h.body]
        helper_defs.append(ast.FunctionDef(name=h.name, args=newargs, body=body,
                                            decorator_list=[], returns=None))

    # transform setUp body (self.x=... -> self_x=...) and method body
    def rewrite_block(stmts):
        out = []
        for s in stmts:
            s2 = flat.visit(ast.fix_missing_locations(s))
            out.append(s2)
        return out

    setup_block = _rewrite_setup(setup_stmts, self_attrs, class_data_names, set(helpers)) if setup_stmts else []
    body_block = rewrite_block(target.body)

    # assemble module
    mod = ast.Module(body=[], type_ignores=[])
    mod.body.extend(_module_imports(tree))
    mod.body.extend(class_data)
    mod.body.extend(helper_defs)
    mod.body.extend(setup_block)
    mod.body.extend(body_block)
    ast.fix_missing_locations(mod)
    src = ast.unparse(mod)
    src += f'\n\nprint("{cls_name}::{method}: ok")\n'
    return src


def _rewrite_setup(setup_stmts, self_attrs, class_data, helpers):
    """self.x = ... -> self_x = ... ; other self.* handled by _Flatten."""
    flat = _Flatten(helpers, self_attrs, class_data)
    out = []
    for s in setup_stmts:
        # rename Store targets self.x -> self_x first (placeholder, so the Load
        # side still routes through _Flatten's class-data/attr logic)
        for t in ast.walk(s):
            if (isinstance(t, ast.Attribute) and isinstance(t.value, ast.Name)
                    and t.value.id == "self" and isinstance(t.ctx, ast.Store)):
                t.value = ast.Name("__SELF_RENAME__", ast.Store())  # placeholder
        s2 = flat.visit(ast.fix_missing_locations(s))
        out.append(s2)
    # turn placeholder Attribute(__SELF_RENAME__, x) into Name(self_x)
    class _Fix(ast.NodeTransformer):
        def visit_Attribute(self, node):
            if isinstance(node.value, ast.Name) and node.value.id == "__SELF_RENAME__":
                return ast.copy_location(ast.Name(f"self_{node.attr}", node.ctx), node)
            self.generic_visit(node)
            return node
    return [_Fix().visit(s) for s in out]


def lib_test_dir():
    out = subprocess.run(["python3.12", "-c", "import test,os;print(os.path.dirname(test.__file__))"],
                         capture_output=True, text=True, timeout=30).stdout.strip()
    return Path(out)


def resolve_module_files(testdir: Path, module: str) -> list[Path]:
    """All Lib/test files whose stem == `module`, top-level and in subpackages.

    `behavior_wall_gen.candidates` keys candidates on the file stem, so a stem
    that occurs in several subpackages (e.g. test_abc in both test/ and
    test/test_importlib/) is one `--module` value covering every such file."""
    return sorted(testdir.rglob(f"{module}.py"))


def dotted_pkg_for(testdir: Path, f: Path) -> str:
    """The importable package a Lib/test file lives in.

    `test/test_X.py`              -> `test`
    `test/test_email/test_X.py`   -> `test.test_email`
    The `test` dir's own parent is on sys.path under the oracle, so the dotted
    name is `test` + the subpackage chain between testdir and the file."""
    rel = f.relative_to(testdir).parts[:-1]  # drop the file leaf
    return ".".join(("test", *rel))


def case_name(cls: str, method: str) -> str:
    import hashlib
    import re
    inner = cls.split(".")[-1]  # nested class: keep the innermost name
    snake = re.sub(r"(?<!^)(?=[A-Z])", "_", inner).lower()
    stem = f"{snake}__{method}".lower()
    # The case key MUST equal the filename stem (fixture_lint), and the fixture
    # filesystem is case-insensitive — so two methods differing only in case
    # (e.g. test_d vs test_D in CPython's getargs format-code tests) would alias
    # to one file. Lowercase the whole stem and, when the original carried any
    # uppercase, append a short hash of the original {cls}.{method} to keep
    # distinct cases distinct and the record == filename in lock-step.
    if any(c.isupper() for c in f"{inner}.{method}"):
        h = hashlib.sha1(f"{cls}.{method}".encode()).hexdigest()[:6]
        stem = f"{stem}_uc{h}"
    return stem


def _load_gaps() -> set[str]:
    if not GAPS_FILE.exists():
        return set()
    return {ln.strip() for ln in GAPS_FILE.read_text(encoding="utf-8").splitlines()
            if ln.strip() and not ln.startswith("#")}


_GAPS_HEADER = (
    "# Behavior wall (2) denominator exclusions — CPython-fail / resource / platform-skip.\n"
    "#\n"
    "# One key per line: mod.Class.method (Class may be dotted for nested classes).\n"
    "# behavior_extract.py appends a key here when the live CPython 3.12 oracle itself\n"
    "# does not pass the case (so it is NOT mamba's to pass); wall_status.py subtracts\n"
    "# these from the (2) Behavior denominator — the same honest exclusion (1) Type\n"
    "# applies to unwrongable signatures. Sorted, dedup, machine-maintained.\n"
)


def _record_gaps(new_keys: list[str]) -> int:
    """Append CPython-fail / resource keys to behavior_gaps.txt (dedup, sorted).

    These are the honest denominator exclusions: cases the live CPython 3.12
    oracle itself does not pass (CPython-fail, resource-gated, platform-skip),
    so they are not mamba's to pass and must leave the (2) Behavior denominator
    — exactly like (1) Type excludes unwrongable signatures."""
    existing = _load_gaps()
    added = sorted(k for k in new_keys if k and k not in existing)
    if not added:
        return 0
    GAPS_FILE.parent.mkdir(parents=True, exist_ok=True)
    merged = sorted(existing | set(added))
    GAPS_FILE.write_text(_GAPS_HEADER + "\n".join(merged) + "\n", encoding="utf-8")
    return len(added)


def _walk_classes(body, prefix=None):
    """Yield (dotted_class, test_method) for class-bound test* methods, recursing
    into nested classes and If-version blocks — same shape as the wall denominator
    in behavior_wall_gen so the gap set lines up exactly."""
    for n in body:
        if isinstance(n, ast.ClassDef):
            sub = n.name if prefix is None else f"{prefix}.{n.name}"
            yield from _walk_classes(n.body, sub)
        elif isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)):
            if prefix is not None and n.name.startswith("test"):
                yield prefix, n.name
        elif isinstance(n, ast.If):
            yield from _walk_classes(n.body, prefix)
            yield from _walk_classes(n.orelse, prefix)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--module", required=True)
    ap.add_argument("--class", dest="cls")
    ap.add_argument("--method")
    ap.add_argument("--emit", action="store_true")
    args = ap.parse_args()

    testdir = lib_test_dir()
    files = resolve_module_files(testdir, args.module)
    if not files:
        print(f"no Lib/test file with stem {args.module}")
        return 1

    lib = args.module[5:] if args.module.startswith("test_") else args.module
    bucket = lib_bucket(lib)
    # gap-only: skip Class.method already covered by a behavior fixture (loaded
    # once) and never within the same run write the same key twice.
    covered = _load_covered() if args.emit else set()
    emitted_keys: set[str] = set()
    inline = runner = red = skip = dup = 0
    gap_keys: list[str] = []

    for f in files:
        src = f.read_text(encoding="utf-8", errors="replace")
        try:
            tree = ast.parse(src)
        except SyntaxError:
            continue
        dotted_pkg = dotted_pkg_for(testdir, f)
        source_rel = str(f.relative_to(testdir))  # e.g. test_email/test__header_value_parser.py

        targets = []
        for cls, method in _walk_classes(tree.body):
            if args.cls and cls.split(".")[-1] != args.cls and cls != args.cls:
                continue
            if args.method and method != args.method:
                continue
            targets.append((cls, method))

        for cls, method in targets:
            key = f"{cls.split('.')[-1]}.{method}"
            if args.emit and (key in covered or key in emitted_keys):
                dup += 1
                continue  # an existing fixture (or this run) already covers it
            try:
                body = extract(src, cls, method)
                fallback = False
            except Unsupported as e:
                if not args.emit:
                    skip += 1
                    print(f"  SKIP {cls}.{method}: {e}")
                    continue
                body = _runner_body(dotted_pkg, args.module, cls, method)
                fallback = True
            if not args.emit:
                inline += 1
                if len(targets) == 1 and len(files) == 1:
                    print(body)
                continue
            fixture = _fixture_text(bucket, lib, args.module, cls, method, body, source_rel)
            if not _oracle_ok(fixture):
                # An inline flatten that the oracle rejects is an extractor
                # fidelity gap, not necessarily a CPython-fail — fall back to the
                # runner form (the authoritative CPython verdict) before deciding.
                if not fallback:
                    body = _runner_body(dotted_pkg, args.module, cls, method)
                    fixture = _fixture_text(bucket, lib, args.module, cls, method, body, source_rel)
                    fallback = True
                if not _oracle_ok(fixture):
                    # The live CPython 3.12 oracle itself does not pass this case
                    # (CPython-fail / resource / platform skip) -> denominator gap.
                    red += 1
                    gap_keys.append(f"{args.module}.{cls}.{method}")
                    continue
            path = CPYTHON_DIR / "behavior" / bucket / lib / f"{case_name(cls, method)}.py"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(fixture, encoding="utf-8")
            emitted_keys.add(key)
            if fallback:
                runner += 1
            else:
                inline += 1

    if args.emit:
        added = _record_gaps(gap_keys)
        print(f"inline={inline} runner={runner} oracle_red={red} "
              f"(gaps_added={added}) dup={dup}")
    else:
        print(f"extractable={inline} skip={skip}")
    return 0


def _load_covered() -> set[str]:
    """Class.method keys already covered by an existing behavior fixture."""
    covered: set[str] = set()
    for py in (CPYTHON_DIR / "behavior").rglob("*.py"):
        s = parse_header(py).get("subject", "")
        segs = s.split(".")
        if len(segs) >= 2:
            covered.add(f"{segs[-2]}.{segs[-1]}")
    return covered


def _runner_body(dotted_pkg: str, module: str, cls: str, method: str) -> str:
    """Fallback for non-flattenable methods: run the CPython test method itself
    via unittest and assert CPython passes it. mamba is later checked against the
    same case. Self-contained except for the CPython `test` package the oracle
    provides — this verifies the *case*, which is what (2) Behavior is about.

    `dotted_pkg` is the importable package the module lives in: `test` for a
    top-level `test/test_X.py`, or `test.test_email` for the subpackage form
    `test/test_email/test_X.py`."""
    return (
        "import unittest, io\n"
        f"from {dotted_pkg} import {module}\n"
        f'_suite = unittest.defaultTestLoader.loadTestsFromName("{cls}.{method}", {module})\n'
        "_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)\n"
        f'assert _result.wasSuccessful(), "CPython {cls}.{method} did not pass"\n'
        f'print("{cls}::{method}: ok")\n'
    )


def lib_bucket(lib: str) -> str:
    for bucket in ("std-libs", "builtin-libs", "core", "pep", "3rd-libs"):
        if (CPYTHON_DIR / "behavior" / bucket / lib).is_dir():
            return bucket
    return "std-libs"


def _fixture_text(bucket, lib, module, cls, method, body, source_rel=None) -> str:
    case = case_name(cls, method)
    subject = f"cpython.{module}.{cls}.{method}"
    src_path = source_rel or f"{module}.py"
    header = PEP723Header(
        bucket=bucket, lib=lib, dimension="behavior", case=case, subject=subject,
        kind="semantic", xfail="auto-extracted CPython test; mamba promotion pending",
        mem_carveout="", source=f"Lib/test/{src_path}", status="filled",
    ).render()
    return header + body


def _oracle_ok(fixture_text: str) -> bool:
    import tempfile
    with tempfile.NamedTemporaryFile("w", suffix=".py", delete=False) as fh:
        fh.write(fixture_text)
        tmp = fh.name
    try:
        r = subprocess.run(["python3.12", tmp], capture_output=True, text=True, timeout=30)
        return r.returncode == 0
    except subprocess.SubprocessError:
        return False
    finally:
        Path(tmp).unlink(missing_ok=True)


if __name__ == "__main__":
    raise SystemExit(main())
