"""Static independence tests for openapi-codegen external contract cases."""

from __future__ import annotations

import ast
import importlib.util
import sys
import tempfile
import textwrap
import unittest
from unittest import mock
from pathlib import Path
from typing import Any

EC_ROOT = Path(__file__).resolve().parent.parent.parent
SRC_DIR = EC_ROOT / "src"

DECLARED_CASES: dict[str, int] = {
    "tolerant-openapi-document-subset-behavior": 16,
    "tolerant-openapi-document-subset-security": 14,
    "deterministic-identifier-naming-behavior": 14,
    "deterministic-identifier-naming-security": 14,
    "language-neutral-operation-ir-behavior": 18,
    "language-neutral-operation-ir-security": 16,
    "per-language-type-mapping-behavior": 18,
    "per-language-type-mapping-security": 16,
    "versioned-target-profiles-behavior": 16,
    "versioned-target-profiles-security": 16,
    "contained-output-materialization-behavior": 14,
    "contained-output-materialization-security": 16,
}

ALLOWED_IMPORT_PREFIXES = ("__future__", "openapi_codegen")
DISALLOWED_IDENTIFIERS = {
    "open",
    "__import__",
    "exec",
    "eval",
    "globals",
    "locals",
    "vars",
    "importlib",
    "getattr",
    "setattr",
    "delattr",
    "__getattr__",
    "__getattribute__",
    "__setattr__",
    "__delattr__",
    "subprocess",
    "Path",
    "read_text",
    "read_bytes",
    "os",
    "system",
    "popen",
    "socket",
    "connect",
    "requests",
    "urlopen",
    "urlretrieve",
    "input",
    "compile",
    "breakpoint",
    "help",
    "dir",
    "__builtins__",
    "builtins",
    "__loader__",
    "__spec__",
    "__file__",
    "__cached__",
    "__package__",
    "__dict__",
    "__class__",
    "__globals__",
    "__subclasses__",
    "__mro__",
    "__bases__",
    "__code__",
    "__closure__",
    "__traceback__",
    "tb_frame",
    "tb_next",
    "f_back",
    "f_builtins",
    "f_globals",
    "f_locals",
    "f_code",
    "gi_frame",
    "ag_frame",
    "cr_frame",
    "m_frame",
    "currentframe",
    "getframeinfo",
    "_getframe",
    "__reduce__",
    "__reduce_ex__",
    "__getstate__",
    "__setstate__",
}


def is_literal_node(node: ast.AST | None) -> bool:
    """Recursively verify if an AST node is a literal expression."""
    if node is None:
        return False
    if isinstance(node, (ast.Constant, ast.Str, ast.Num, ast.Bytes, ast.NameConstant)):
        return True
    if isinstance(node, (ast.List, ast.Tuple, ast.Set)):
        return all(is_literal_node(elt) for elt in node.elts)
    if isinstance(node, ast.Dict):
        if any(k is None for k in node.keys):
            return False  # Reject dict unpack/spread {**x}
        return all(is_literal_node(k) for k in node.keys) and all(is_literal_node(v) for v in node.values)
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, (ast.UAdd, ast.USub)):
        return is_literal_node(node.operand)
    return False


def eval_literal_node(node: ast.AST | None) -> Any:
    """Safely evaluate a proven literal AST node to its Python value."""
    if node is None:
        return None
    if isinstance(node, (ast.Constant, ast.Str, ast.Num, ast.Bytes, ast.NameConstant)):
        return getattr(node, "value", getattr(node, "n", getattr(node, "s", None)))
    if isinstance(node, ast.Tuple):
        return tuple(eval_literal_node(elt) for elt in node.elts)
    if isinstance(node, ast.List):
        return [eval_literal_node(elt) for elt in node.elts]
    if isinstance(node, ast.Set):
        return {eval_literal_node(elt) for elt in node.elts}
    if isinstance(node, ast.Dict):
        return {eval_literal_node(k): eval_literal_node(v) for k, v in zip(node.keys, node.values)}  # type: ignore
    if isinstance(node, ast.UnaryOp):
        val = eval_literal_node(node.operand)
        if isinstance(node.op, ast.USub):
            return -val
        if isinstance(node.op, ast.UAdd):
            return +val
    return None


def get_ast_dump(node: ast.AST | None) -> str:
    """Return attribute-free AST dump for exact structural comparison."""
    if node is None:
        return ""
    return ast.dump(node, include_attributes=False)


def are_asts_equal(node1: ast.AST | None, node2: ast.AST | None) -> bool:
    """Check if two AST nodes are structurally identical."""
    return get_ast_dump(node1) == get_ast_dump(node2)


def is_dict_str_object_annotation(node: ast.AST | None) -> bool:
    """Check if AST return annotation is dict[str, object]."""
    if node is None or not isinstance(node, ast.Subscript):
        return False
    if not (isinstance(node.value, ast.Name) and node.value.id == "dict"):
        return False
    slice_node = node.slice
    if isinstance(slice_node, ast.Index):
        slice_node = slice_node.value  # type: ignore
    if not isinstance(slice_node, ast.Tuple) or len(slice_node.elts) != 2:
        return False
    k, v = slice_node.elts[0], slice_node.elts[1]
    return (
        isinstance(k, ast.Name)
        and k.id == "str"
        and isinstance(v, ast.Name)
        and v.id == "object"
    )


def extract_annotation_nodes(tree: ast.AST) -> set[ast.AST]:
    """Collect all AST nodes that belong to type annotations (returns, args, etc)."""
    annotation_nodes: set[ast.AST] = set()

    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            if node.returns:
                for sub in ast.walk(node.returns):
                    annotation_nodes.add(sub)
            all_args = (
                node.args.args
                + getattr(node.args, "posonlyargs", [])
                + getattr(node.args, "kwonlyargs", [])
            )
            if node.args.vararg:
                all_args.append(node.args.vararg)
            if node.args.kwarg:
                all_args.append(node.args.kwarg)

            for arg in all_args:
                if arg.annotation:
                    for sub in ast.walk(arg.annotation):
                        annotation_nodes.add(sub)

        elif isinstance(node, ast.AnnAssign) and node.annotation:
            for sub in ast.walk(node.annotation):
                annotation_nodes.add(sub)

    return annotation_nodes


def get_binding_events(tree: ast.AST) -> list[tuple[str, ast.AST, str]]:
    """Non-recursive-per-node binding event extractor.

    Returns list of (bound_name, binding_node, binding_kind).
    """
    events: list[tuple[str, ast.AST, str]] = []

    for node in ast.walk(tree):
        if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)):
            events.append((node.id, node, "name_store_del"))
        elif isinstance(node, ast.arg):
            events.append((node.arg, node, "arg"))
        elif isinstance(node, ast.alias):
            bound_name = node.asname or node.name.split(".")[0]
            events.append((bound_name, node, "alias"))
        elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            events.append((node.name, node, "def_or_class"))
        elif isinstance(node, ast.ExceptHandler) and node.name:
            events.append((node.name, node, "except_handler"))
        elif isinstance(node, (ast.Global, ast.Nonlocal)):
            for name in node.names:
                events.append((name, node, "global_nonlocal"))
        elif hasattr(ast, "MatchAs") and isinstance(node, ast.MatchAs) and node.name:
            events.append((node.name, node, "match_as"))
        elif hasattr(ast, "MatchStar") and isinstance(node, ast.MatchStar) and node.name:
            events.append((node.name, node, "match_star"))
        elif hasattr(ast, "MatchMapping") and isinstance(node, ast.MatchMapping) and getattr(node, "rest", None):
            events.append((node.rest, node, "match_mapping_rest"))  # type: ignore
        elif hasattr(ast, "TypeAlias") and isinstance(node, ast.TypeAlias):
            if isinstance(node.name, ast.Name):
                events.append((node.name.id, node, "type_alias"))
        elif hasattr(ast, "TypeParam") and isinstance(node, ast.TypeParam):
            name_val = getattr(node, "name", None)
            if isinstance(name_val, str):
                events.append((name_val, node, "type_param"))

    return events


def extract_provenance_names(tree: ast.AST, entrypoint_fn: ast.FunctionDef) -> tuple[dict[str, ast.alias], set[str]]:
    """Build canonical design import identity and transitive provenance set of design-derived local variable names."""
    canonical_design_aliases: dict[str, ast.alias] = {}

    # 1. Collect module-level top-level canonical design import aliases from direct tree.body
    for stmt in getattr(tree, "body", []):
        if isinstance(stmt, ast.Import):
            for alias in stmt.names:
                if alias.name.startswith("openapi_codegen"):
                    bound_name = alias.asname or alias.name.split(".")[0]
                    if bound_name in canonical_design_aliases:
                        raise ValueError(f"Duplicate canonical design import alias: {bound_name}")
                    canonical_design_aliases[bound_name] = alias
        elif isinstance(stmt, ast.ImportFrom):
            if stmt.module and stmt.module.startswith("openapi_codegen"):
                for alias in stmt.names:
                    bound_name = alias.asname or alias.name
                    if bound_name in canonical_design_aliases:
                        raise ValueError(f"Duplicate canonical design import alias: {bound_name}")
                    canonical_design_aliases[bound_name] = alias

    def get_root_name(node: ast.AST | None) -> str | None:
        while isinstance(node, ast.Attribute):
            node = node.value
        if isinstance(node, ast.Name):
            return node.id
        return None

    def is_direct_design_call(call_node: ast.AST | None) -> bool:
        if call_node is None or not isinstance(call_node, ast.Call):
            return False
        root_id = get_root_name(call_node.func)
        return root_id is not None and root_id in canonical_design_aliases

    # 2. Strictly narrow observed-provenance grammar: direct design calls or extraction from direct design call results.
    # Witness validation is deliberately structural and fail-closed: a recorder
    # is trusted only when it is a blank, zero-argument, argument-preserving log.
    design_derived_vars: set[str] = set()
    witness_instances: set[str] = set()
    witness_classes: set[str] = set()

    def _meaningful_body(fn: ast.FunctionDef) -> list[ast.stmt]:
        body = list(fn.body)
        if body and isinstance(body[0], ast.Expr) and isinstance(body[0].value, ast.Constant) and isinstance(body[0].value.value, str):
            body = body[1:]
        return body

    for stmt in getattr(tree, "body", []):
        if not isinstance(stmt, ast.ClassDef):
            continue
        # Do not infer effective recorder behavior through inheritance or
        # class construction machinery; those boundaries are fail-closed.
        if stmt.bases or stmt.decorator_list or stmt.keywords:
            continue
        # Class-scope log state is not an isolated recorder: reject any class
        # declaration that reads, writes, rebinds, or mutates ``write_log``.
        class_scope_log = any(
            isinstance(node, ast.Name) and node.id == "write_log"
            for child in stmt.body
            if not isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef))
            for node in ast.walk(child)
        ) or any(
            isinstance(node, ast.Attribute) and node.attr == "write_log"
            for child in stmt.body
            if not isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef))
            for node in ast.walk(child)
        )
        if class_scope_log:
            continue
        methods = [x for x in stmt.body if isinstance(x, (ast.FunctionDef, ast.AsyncFunctionDef))]
        inits = [x for x in methods if x.name == "__init__"]
        writes = [x for x in methods if x.name in {"write_text", "write"}]
        if len(inits) != 1 or len(writes) != 1:
            continue
        init, write = inits[0], writes[0]
        if init.decorator_list or write.decorator_list:
            continue
        init_args = list(init.args.posonlyargs) + list(init.args.args)
        init_ok = (
            len(init_args) == 1
            and init_args[0].arg == "self"
            and not init.args.vararg and not init.args.kwarg
            and not init.args.defaults and not init.args.kw_defaults
        )
        init_body = _meaningful_body(init)
        init_ok = init_ok and len(init_body) == 1
        if init_ok:
            init_stmt = init_body[0]
            init_target: ast.AST | None = None
            init_value: ast.AST | None = None
            if isinstance(init_stmt, ast.Assign) and len(init_stmt.targets) == 1:
                init_target, init_value = init_stmt.targets[0], init_stmt.value
            elif isinstance(init_stmt, ast.AnnAssign):
                init_target, init_value = init_stmt.target, init_stmt.value
            init_ok = (
                isinstance(init_target, ast.Attribute)
                and isinstance(init_target.value, ast.Name)
                and init_target.value.id == "self"
                and init_target.attr == "write_log"
                and isinstance(init_value, ast.List)
                and not init_value.elts
            )
        write_args = list(write.args.posonlyargs) + list(write.args.args)
        write_body = _meaningful_body(write)
        write_ok = (
            len(write_args) == 3
            and [arg.arg for arg in write_args] == ["self", "path", "contents"]
            and not write.args.vararg and not write.args.kwarg
            and not write.args.defaults and not write.args.kw_defaults
            and len(write_body) == 1
        )
        if write_ok:
            stmt0 = write_body[0]
            write_ok = (
                isinstance(stmt0, ast.AugAssign)
                and isinstance(stmt0.op, ast.Add)
                and isinstance(stmt0.target, ast.Attribute)
                and isinstance(stmt0.target.value, ast.Name)
                and stmt0.target.value.id == "self"
                and stmt0.target.attr == "write_log"
                and isinstance(stmt0.value, ast.List)
                and len(stmt0.value.elts) == 1
                and isinstance(stmt0.value.elts[0], ast.Tuple)
                and len(stmt0.value.elts[0].elts) == 2
                and all(isinstance(elt, ast.Name) and elt.id in {"path", "contents"} for elt in stmt0.value.elts[0].elts)
                and [elt.id for elt in stmt0.value.elts[0].elts] == ["path", "contents"]
            )
        # No other method may read, write, rebind, or mutate self.write_log.
        harmless = {"__init__", write.name, "read_text"}
        class_log_ok = True
        for method in methods:
            if method.name in harmless and method in {init, write}:
                continue
            if method.name == "read_text":
                if any(isinstance(n, ast.Attribute) and n.attr == "write_log" for n in ast.walk(method)):
                    class_log_ok = False
            else:
                if any(isinstance(n, ast.Attribute) and n.attr == "write_log" for n in ast.walk(method)):
                    class_log_ok = False
        if init_ok and write_ok and class_log_ok:
            witness_classes.add(stmt.name)

    def is_expr_direct_call_derived(expr: ast.AST | None) -> bool:
        if expr is None:
            return False
        if is_direct_design_call(expr):
            return True
        if isinstance(expr, ast.Attribute):
            return is_expr_direct_call_derived(expr.value) or (isinstance(expr.value, ast.Name) and expr.value.id in design_derived_vars)
        if isinstance(expr, ast.Subscript):
            return is_expr_direct_call_derived(expr.value) or (isinstance(expr.value, ast.Name) and expr.value.id in design_derived_vars)
        if isinstance(expr, ast.Name):
            return expr.id in design_derived_vars or expr.id in canonical_design_aliases
        if isinstance(expr, ast.Call) and isinstance(expr.func, ast.Name) and expr.func.id == "tuple" and len(expr.args) == 1:
            arg = expr.args[0]
            return (
                "tuple" not in shadowed_builtin_names
                and isinstance(arg, ast.Attribute)
                and isinstance(arg.value, ast.Name)
                and (
                    (arg.value.id in witness_instances and arg.value.id not in invalid_witnesses)
                    or any(root not in invalid_witnesses and arg.value.id in recorder_aliases_for(root) for root in witness_candidates)
                )
                and arg.attr == "write_log"
            ) or (
                        isinstance(arg, ast.Name)
                        and any(root not in invalid_witnesses and arg.id in aliases_for(root) for root in witness_candidates)
                    ) or is_expr_direct_call_derived(arg)
        if isinstance(expr, ast.Call) and isinstance(expr.func, ast.Name) and expr.func.id == "sorted" and len(expr.args) == 1:
            return is_expr_direct_call_derived(expr.args[0])
        if isinstance(expr, ast.Call) and isinstance(expr.func, ast.Attribute) and expr.func.attr in {"has_inputs"}:
            return is_expr_direct_call_derived(expr.func.value)
        if isinstance(expr, (ast.Tuple, ast.List)):
            # R13 permits pure composites assembled solely from already
            # design-derived leaves; calls/operators/conditionals remain
            # untrusted because they are not accepted by this recursion.
            return bool(expr.elts) and all(is_expr_direct_call_derived(elt) for elt in expr.elts)
        if isinstance(expr, ast.IfExp):
            # Narrow optional extraction: same derived root guarded by
            # ``root is not None`` with a literal None fallback.
            if not (isinstance(expr.test, ast.Compare) and len(expr.test.ops) == 1
                    and isinstance(expr.test.ops[0], ast.IsNot)
                    and len(expr.test.comparators) == 1
                    and isinstance(expr.test.comparators[0], ast.Constant)
                    and expr.test.comparators[0].value is None
                    and isinstance(expr.orelse, ast.Constant)
                    and expr.orelse.value is None):
                return False
            root = expr.test.left
            def chain(node: ast.AST) -> tuple[object, ...]:
                parts: list[object] = []
                while isinstance(node, (ast.Attribute, ast.Subscript)):
                    if isinstance(node, ast.Attribute):
                        parts.append(("attr", node.attr))
                    else:
                        parts.append(("sub", ast.dump(node.slice, include_attributes=False)))
                    node = node.value
                if not isinstance(node, ast.Name):
                    return ()
                parts.append(("name", node.id))
                return tuple(reversed(parts))
            guard_chain = chain(root)
            body_chain = chain(expr.body)
            return (is_expr_direct_call_derived(root) and is_expr_direct_call_derived(expr.body)
                    and bool(guard_chain) and len(body_chain) >= len(guard_chain)
                    and body_chain[:len(guard_chain)] == guard_chain)
        return False

    # Count verifier binding events using the central binding event model.
    class _VerifierBindings(ast.NodeVisitor):
        def __init__(self) -> None:
            self.events: list[tuple[str, ast.AST, str]] = []
        def visit_Name(self, node: ast.Name) -> None:
            if isinstance(node.ctx, (ast.Store, ast.Del)):
                self.events.append((node.id, node, "name_store_del"))
        def visit_arg(self, node: ast.arg) -> None:
            self.events.append((node.arg, node, "arg"))
        def _definition(self, node: ast.FunctionDef | ast.AsyncFunctionDef) -> None:
            self.events.append((node.name, node, "def_or_class"))
            for expr in [*node.decorator_list, *node.args.defaults, *[x for x in node.args.kw_defaults if x is not None]]:
                self.visit(expr)
        def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
            self._definition(node)
            if node is entrypoint_fn:
                for statement in node.body:
                    self.visit(statement)
        def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> None:
            self._definition(node)
            if node is entrypoint_fn:
                for statement in node.body:
                    self.visit(statement)
        def visit_ClassDef(self, node: ast.ClassDef) -> None:
            self.events.append((node.name, node, "def_or_class"))
            for expr in [*node.decorator_list, *node.bases, *(kw.value for kw in node.keywords)]:
                self.visit(expr)
        def visit_Lambda(self, node: ast.Lambda) -> None:
            for expr in [*node.args.defaults, *[x for x in node.args.kw_defaults if x is not None]]:
                self.visit(expr)
        def _comp(self, node: ast.AST) -> None:
            for generator in getattr(node, "generators", []):
                self.visit(generator.iter)
                for condition in generator.ifs:
                    self.visit(condition)
            self.visit(node.key if isinstance(node, ast.DictComp) else node.elt) if isinstance(node, (ast.DictComp, ast.ListComp, ast.SetComp, ast.GeneratorExp)) else None
            if isinstance(node, ast.DictComp):
                self.visit(node.value)
        visit_ListComp = _comp
        visit_SetComp = _comp
        visit_DictComp = _comp
        visit_GeneratorExp = _comp

    _binding_collector = _VerifierBindings()
    _binding_collector.visit(entrypoint_fn)
    verifier_events = _binding_collector.events
    safe_builtin_names = {"tuple", "list", "sorted", "len", "isinstance", "issubclass", "type"}

    class _LexicalBindings(ast.NodeVisitor):
        def __init__(self) -> None:
            self.names: set[str] = set()
        def visit_Name(self, node: ast.Name) -> None:
            if isinstance(node.ctx, (ast.Store, ast.Del)):
                self.names.add(node.id)
        def visit_arg(self, node: ast.arg) -> None:
            self.names.add(node.arg)
        def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
            self.names.add(node.name)
            for decorator in node.decorator_list:
                self.visit(decorator)
            for default in [*node.args.defaults, *node.args.kw_defaults]:
                if default is not None:
                    self.visit(default)
        def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> None:
            self.visit_FunctionDef(node)  # type: ignore[arg-type]
        def visit_ClassDef(self, node: ast.ClassDef) -> None:
            self.names.add(node.name)
            for decorator in node.decorator_list:
                self.visit(decorator)
            for base in node.bases:
                self.visit(base)
            for keyword in node.keywords:
                self.visit(keyword.value)
        def visit_Lambda(self, node: ast.Lambda) -> None:
            for default in [*node.args.defaults, *node.args.kw_defaults]:
                if default is not None:
                    self.visit(default)
        def visit_ExceptHandler(self, node: ast.ExceptHandler) -> None:
            if node.name:
                self.names.add(node.name)
            for child in node.body:
                self.visit(child)
        def _visit_comp(self, node: ast.AST) -> None:
            generators = getattr(node, "generators", [])
            for generator in generators:
                self.visit(generator.iter)
                for condition in generator.ifs:
                    self.visit(condition)
            if isinstance(node, ast.DictComp):
                self.visit(node.key)
                self.visit(node.value)
            else:
                self.visit(node.elt)
        def visit_ListComp(self, node: ast.ListComp) -> None:
            self._visit_comp(node)
        visit_SetComp = visit_ListComp
        visit_DictComp = visit_ListComp
        visit_GeneratorExp = visit_ListComp

    module_bindings: set[str] = set()
    for statement in tree.body:
        if isinstance(statement, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            module_bindings.add(statement.name)
        else:
            visitor = _LexicalBindings()
            visitor.visit(statement)
            module_bindings.update(visitor.names)
    verifier_bindings = _LexicalBindings()
    for statement in entrypoint_fn.body:
        verifier_bindings.visit(statement)
    shadowed_builtin_names = (module_bindings | verifier_bindings.names) & safe_builtin_names
    var_event_counts: dict[str, int] = {}
    for bound_name, _, _ in verifier_events:
        var_event_counts[bound_name] = var_event_counts.get(bound_name, 0) + 1

    # Sequentially process the verifier.  A witness is interaction-derived only
    # after the exact instance appears in a canonical design call; later writes
    # or rebinding invalidate it for the remainder of the function.
    witness_candidates: set[str] = set()
    invalid_witnesses: set[str] = set()
    log_aliases: dict[str, set[str]] = {}
    log_holder_paths: dict[str, dict[str, set[tuple[object, ...]]]] = {}
    recorder_holder_paths: dict[str, dict[str, set[tuple[object, ...]]]] = {}
    literal_lengths: dict[str, int] = {}
    derived_holder_paths: dict[str, dict[str, set[tuple[object, ...]]]] = {}
    invalid_derived: set[str] = set()
    derived_aliases: dict[str, set[str]] = {}
    recorder_aliases: dict[str, set[str]] = {}
    mutators = {
        "list": {
            "append", "clear", "extend", "insert", "pop", "remove",
            "reverse", "sort", "__delitem__", "__iadd__", "__imul__",
            "__setitem__",
        },
        "dict": {
            "clear", "pop", "popitem", "setdefault", "update",
            "__delitem__", "__ior__", "__setitem__",
        },
        "set": {
            "add", "clear", "difference_update", "discard",
            "intersection_update", "pop", "remove",
            "symmetric_difference_update", "update", "__iand__",
            "__ior__", "__isub__", "__ixor__",
        },
    }
    bound_mutators = set().union(*mutators.values())
    read_only_unbound = {"list": {"count", "index"}, "dict": {"get", "items", "keys", "values"}, "set": set()}
    callable_aliases: dict[str, tuple[str, str | None]] = {}
    helper_mutated_params: dict[str, set[int]] = {}

    def recorder_aliases_for(name: str) -> set[str]:
        found: set[str] = set()
        pending = [name]
        while pending:
            root = pending.pop()
            for alias in recorder_aliases.get(root, set()):
                if alias not in found:
                    found.add(alias)
                    pending.append(alias)
        return found

    def derived_aliases_for(name: str) -> set[str]:
        found: set[str] = set()
        pending = [name]
        while pending:
            root = pending.pop()
            for alias in derived_aliases.get(root, set()):
                if alias not in found:
                    found.add(alias)
                    pending.append(alias)
        return found

    def aliases_for(name: str) -> set[str]:
        found: set[str] = set()
        pending = [name]
        while pending:
            root = pending.pop()
            for alias in log_aliases.get(root, set()):
                if alias not in found:
                    found.add(alias)
                    pending.append(alias)
        return found

    def literal_log_paths(node: ast.AST, root: str, path: tuple[object, ...] = ()) -> set[tuple[object, ...]]:
        paths: set[tuple[object, ...]] = set()
        if isinstance(node, ast.Attribute) and node.attr == "write_log" and isinstance(node.value, ast.Name) and (node.value.id == root or node.value.id in recorder_aliases_for(root)):
            paths.add(path)
        elif isinstance(node, ast.Name) and node.id in aliases_for(root):
            paths.add(path)
        elif isinstance(node, (ast.Tuple, ast.List)):
            for index, item in enumerate(node.elts):
                paths.update(literal_log_paths(item, root, path + (index,)))
        elif isinstance(node, ast.Dict):
            for key, value in zip(node.keys, node.values):
                if isinstance(key, ast.Constant) and isinstance(key.value, (str, int)):
                    paths.update(literal_log_paths(value, root, path + (key.value,)))
        return paths

    def literal_recorder_paths(node: ast.AST, root: str, path: tuple[object, ...] = ()) -> set[tuple[object, ...]]:
        if isinstance(node, ast.Name) and (node.id == root or node.id in recorder_aliases_for(root)):
            return {path}
        if isinstance(node, (ast.Tuple, ast.List)):
            return {p for i, item in enumerate(node.elts) for p in literal_recorder_paths(item, root, path + (i,))}
        if isinstance(node, ast.Dict):
            paths: set[tuple[object, ...]] = set()
            for key, value in zip(node.keys, node.values):
                if isinstance(key, ast.Constant) and isinstance(key.value, (str, int)):
                    paths.update(literal_recorder_paths(value, root, path + (key.value,)))
            return paths
        return set()

    def literal_derived_paths(node: ast.AST, root: str, path: tuple[object, ...] = ()) -> set[tuple[object, ...]]:
        if isinstance(node, ast.Name) and (node.id == root or node.id in derived_aliases_for(root)):
            return {path}
        if isinstance(node, (ast.Tuple, ast.List)):
            return {p for i, item in enumerate(node.elts) for p in literal_derived_paths(item, root, path + (i,))}
        if isinstance(node, ast.Dict):
            paths: set[tuple[object, ...]] = set()
            for key, value in zip(node.keys, node.values):
                if isinstance(key, ast.Constant) and isinstance(key.value, (str, int)):
                    paths.update(literal_derived_paths(value, root, path + (key.value,)))
            return paths
        return set()

    def subscript_path(node: ast.AST) -> tuple[ast.Name, tuple[object, ...]] | None:
        parts: list[object] = []
        current = node
        while isinstance(current, ast.Subscript):
            selector = current.slice
            if isinstance(selector, ast.Constant) and isinstance(selector.value, (str, int)):
                parts.append(selector.value)
            else:
                return None
            current = current.value
        if not isinstance(current, ast.Name):
            return None
        return current, tuple(reversed(parts))

    def destructure_paths(target: ast.AST, path: tuple[object, ...] = ()) -> list[tuple[ast.Name, tuple[object, ...]]]:
        if isinstance(target, ast.Starred):
            return destructure_paths(target.value, path)
        if isinstance(target, (ast.Tuple, ast.List)):
            return [item for index, child in enumerate(target.elts) for item in destructure_paths(child, path + (index,))]
        return [(target, path)] if isinstance(target, ast.Name) else []

    def bind_derived_target(target: ast.AST, value: ast.AST) -> list[str]:
        bound: list[str] = []
        if isinstance(target, ast.Starred):
            return bind_derived_target(target.value, value)
        if isinstance(target, ast.Name) and isinstance(value, ast.Name) and value.id in design_derived_vars:
            bound.append(value.id)
            derived_aliases.setdefault(value.id, set()).add(target.id)
        elif isinstance(target, ast.Name) and isinstance(value, (ast.Tuple, ast.List)):
            for item in value.elts:
                for root in design_derived_vars:
                    if literal_derived_paths(item, root):
                        bound.append(root)
                        derived_aliases.setdefault(root, set()).add(target.id)
        elif isinstance(target, (ast.Tuple, ast.List)) and isinstance(value, (ast.Tuple, ast.List)):
            for target_item, value_item in zip(target.elts, value.elts):
                bound.extend(bind_derived_target(target_item, value_item))
        return bound

    def starred_names(target: ast.AST) -> list[str]:
        if isinstance(target, ast.Starred) and isinstance(target.value, ast.Name):
            return [target.value.id]
        if isinstance(target, (ast.Tuple, ast.List)):
            return [name for item in target.elts for name in starred_names(item)]
        return []

    class _ExecutedNodes(ast.NodeVisitor):
        def __init__(self) -> None:
            self.nodes: list[ast.AST] = []
            self.class_depth = 0
        def generic_visit(self, node: ast.AST) -> None:
            self.nodes.append(node)
            super().generic_visit(node)
        def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
            self.nodes.append(node)
            for expr in [*node.decorator_list, *node.args.defaults, *[x for x in node.args.kw_defaults if x is not None]]:
                self.visit(expr)
        def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> None:
            self.visit_FunctionDef(node)  # type: ignore[arg-type]
        def visit_ClassDef(self, node: ast.ClassDef) -> None:
            self.nodes.append(node)
            for expr in [*node.decorator_list, *node.bases, *(kw.value for kw in node.keywords)]:
                self.visit(expr)
            self.class_depth += 1
            for statement in node.body:
                self.visit(statement)
            self.class_depth -= 1
        def visit_Name(self, node: ast.Name) -> None:
            if self.class_depth and isinstance(node.ctx, (ast.Store, ast.Del)):
                return
            self.nodes.append(node)
        def visit_Lambda(self, node: ast.Lambda) -> None:
            self.nodes.append(node)
            for expr in [*node.args.defaults, *[x for x in node.args.kw_defaults if x is not None]]:
                self.visit(expr)
        def _comp(self, node: ast.AST) -> None:
            self.nodes.append(node)
            for generator in getattr(node, "generators", []):
                self.visit(generator.iter)
                for condition in generator.ifs:
                    self.visit(condition)
            if isinstance(node, ast.DictComp):
                self.visit(node.key)
                self.visit(node.value)
            else:
                self.visit(node.elt)
        visit_ListComp = _comp
        visit_SetComp = _comp
        visit_DictComp = _comp
        visit_GeneratorExp = _comp

    class _HelperExecutedNodes(ast.NodeVisitor):
        """Collect only effects executed while a helper is called.

        Nested callable bodies are definition-time objects, not calls.  Their
        decorators/defaults (and class bases/keywords) still execute while
        the outer helper is defined, so those expressions remain visible.
        """

        def __init__(self, root: ast.FunctionDef) -> None:
            self.root = root
            self.nodes: list[ast.AST] = []

        def generic_visit(self, node: ast.AST) -> None:
            self.nodes.append(node)
            super().generic_visit(node)

        def _definition_exprs(self, node: ast.FunctionDef | ast.AsyncFunctionDef) -> None:
            self.nodes.append(node)
            for expr in [
                *node.decorator_list,
                *node.args.defaults,
                *[x for x in node.args.kw_defaults if x is not None],
            ]:
                self.visit(expr)

        def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
            if node is self.root:
                self.nodes.append(node)
                for statement in node.body:
                    self.visit(statement)
            else:
                self._definition_exprs(node)

        def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> None:
            if node is self.root:
                self.nodes.append(node)
                for statement in node.body:
                    self.visit(statement)
            else:
                self._definition_exprs(node)

        def visit_Lambda(self, node: ast.Lambda) -> None:
            self.nodes.append(node)
            for expr in [*node.args.defaults, *[x for x in node.args.kw_defaults if x is not None]]:
                self.visit(expr)

        def visit_ClassDef(self, node: ast.ClassDef) -> None:
            self.nodes.append(node)
            for expr in [*node.decorator_list, *node.bases, *(kw.value for kw in node.keywords)]:
                self.visit(expr)
            # A class body executes at class definition time.  Nested callable
            # bodies encountered inside it are still handled by this visitor.
            for statement in node.body:
                self.visit(statement)

    def mutation_nodes(statement: ast.stmt) -> list[ast.AST]:
        collector = _ExecutedNodes()
        collector.visit(statement)
        return collector.nodes

    def _expr_has_derived_provenance(expr: ast.AST, root: str) -> bool:
        """Whether an expression carries the selected derived root."""
        if isinstance(expr, ast.Name):
            return expr.id == root or expr.id in derived_aliases_for(root)
        if isinstance(expr, ast.Subscript):
            parsed = subscript_path(expr)
            if parsed is not None:
                holder, path = parsed
                return any(
                    candidate[: len(path)] == path
                    for candidate in derived_holder_paths.get(holder.id, {}).get(root, set())
                )
        if isinstance(expr, (ast.Tuple, ast.List, ast.Set)):
            return any(_expr_has_derived_provenance(item, root) for item in expr.elts)
        if isinstance(expr, ast.Dict):
            return any(_expr_has_derived_provenance(item, root) for item in expr.values)
        return False

    def _expr_has_recorder_provenance(expr: ast.AST, root: str) -> bool:
        if isinstance(expr, ast.Name):
            return expr.id == root or expr.id in recorder_aliases_for(root) or expr.id in aliases_for(root)
        if isinstance(expr, ast.Attribute) and expr.attr == "write_log":
            return isinstance(expr.value, ast.Name) and (
                expr.value.id == root or expr.value.id in recorder_aliases_for(root)
            )
        if isinstance(expr, ast.Subscript):
            parsed = subscript_path(expr)
            if parsed is not None:
                holder, path = parsed
                return any(
                    candidate[: len(path)] == path
                    for candidate in recorder_holder_paths.get(holder.id, {}).get(root, set())
                )
        if isinstance(expr, (ast.Tuple, ast.List, ast.Set)):
            return any(_expr_has_recorder_provenance(item, root) for item in expr.elts)
        if isinstance(expr, ast.Dict):
            return any(_expr_has_recorder_provenance(item, root) for item in expr.values)
        return False

    def _record_exact_selector_insertions(statement: ast.stmt) -> None:
        """Record provenance introduced by exact bound/unbound map writes."""
        for node in mutation_nodes(statement):
            if not isinstance(node, ast.Call) or not isinstance(node.func, ast.Attribute):
                continue
            method = node.func.attr
            if method not in {"__setitem__", "setdefault"} or len(node.args) < 2:
                continue
            if isinstance(node.func.value, ast.Name) and node.func.value.id == "dict":
                if len(node.args) < 3:
                    continue
                holder = node.args[0] if isinstance(node.args[0], ast.Name) else None
                selector = node.args[1]
                value = node.args[2]
            else:
                holder = node.func.value if isinstance(node.func.value, ast.Name) else None
                selector = node.args[0]
                value = node.args[1]
            if holder is None or not isinstance(selector, ast.Constant) or not isinstance(selector.value, (str, int)):
                continue
            for root in tuple(design_derived_vars):
                if _expr_has_derived_provenance(value, root):
                    derived_holder_paths.setdefault(holder.id, {}).setdefault(root, set()).add((selector.value,))
            for root in tuple(witness_candidates):
                if _expr_has_recorder_provenance(value, root):
                    # This map entry may contain either write_log or the
                    # recorder instance; both are intentionally tracked.
                    recorder_holder_paths.setdefault(holder.id, {}).setdefault(root, set()).add((selector.value,))

    def _is_witness_mutation(stmt: ast.stmt, name: str) -> bool:
        aliases = aliases_for(name)
        instance_aliases = recorder_aliases_for(name)
        pending = list(instance_aliases)
        while pending:
            root = pending.pop()
            for alias in recorder_aliases.get(root, set()):
                if alias not in instance_aliases:
                    instance_aliases.add(alias)
                    pending.append(alias)
        def log_expr(node: ast.AST) -> bool:
            if isinstance(node, ast.Attribute) and node.attr == "write_log" and isinstance(node.value, ast.Name) and node.value.id == name:
                return True
            if isinstance(node, ast.Name) and node.id in aliases:
                return True
            if isinstance(node, ast.Subscript):
                parsed = subscript_path(node)
                if parsed is not None and parsed[0].id in aliases:
                    holder, path = parsed
                    holder_paths = log_holder_paths.get(holder.id, {}).get(name, set())
                    return any(candidate[:len(path)] == path for candidate in holder_paths)
                return log_expr(node.value)
            return any(log_expr(child) for child in ast.iter_child_nodes(node))
        for node in mutation_nodes(stmt):
            if isinstance(node, ast.Attribute) and isinstance(node.ctx, (ast.Store, ast.Del)):
                if isinstance(node.value, ast.Name) and node.value.id in ({name} | instance_aliases) and node.attr == "write_log":
                    return True
            if isinstance(node, ast.Subscript) and isinstance(node.ctx, (ast.Store, ast.Del)):
                if isinstance(node.value, ast.Attribute) and isinstance(node.value.value, ast.Name) and node.value.value.id in ({name} | instance_aliases) and node.value.attr == "write_log":
                    return True
                parsed = subscript_path(node)
                if parsed is not None:
                    holder, path = parsed
                    if holder.id in aliases:
                        holder_paths = log_holder_paths.get(holder.id, {}).get(name, set())
                        if any(candidate[:len(path)] == path for candidate in holder_paths) or path == (0,):
                            return True
                else:
                    root = node.value
                    while isinstance(root, (ast.Attribute, ast.Subscript)):
                        root = root.value
                    if isinstance(root, ast.Name) and (root.id == name or root.id in aliases):
                        return True
            if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute):
                receiver = node.func.value
                if (isinstance(receiver, ast.Name) and node.func.attr in {"pop", "setdefault", "__setitem__", "__delitem__"}
                        and node.args and receiver.id in log_holder_paths):
                    tracked = log_holder_paths.get(receiver.id, {}).get(name, set())
                    selector = node.args[0]
                    if tracked and isinstance(selector, ast.Constant) and not any(path and path[0] == selector.value for path in tracked):
                        # Exact insertion creates a new tracked holder path;
                        # it does not mutate the existing recorder value.
                        continue
                    if tracked and node.func.attr in {"__setitem__", "setdefault"} and len(node.args) >= 2 and log_expr(node.args[1]):
                        continue
                    if tracked and not isinstance(selector, ast.Constant) and node.func.attr in {"pop", "__delitem__", "__setitem__", "setdefault"}:
                        return True
                    if tracked and isinstance(selector, ast.Constant) and node.func.attr in {"pop", "__delitem__"} and any(path and path[0] == selector.value for path in tracked):
                        return True
                if isinstance(receiver, ast.Name) and receiver.id == "dict" and node.func.attr in {"__setitem__", "pop", "setdefault", "__delitem__"} and len(node.args) >= 2:
                    holder_arg, selector_arg = node.args[0], node.args[1]
                    if isinstance(holder_arg, ast.Name):
                        tracked = log_holder_paths.get(holder_arg.id, {}).get(name, set())
                        if tracked and isinstance(selector_arg, ast.Constant) and not any(path and path[0] == selector_arg.value for path in tracked):
                            continue
                        if tracked and node.func.attr in {"__setitem__", "setdefault"} and len(node.args) >= 3 and log_expr(node.args[2]):
                            continue
                        if tracked and not isinstance(selector_arg, ast.Constant) and node.func.attr in {"pop", "__delitem__", "__setitem__", "setdefault"}:
                            return True
                        if tracked and isinstance(selector_arg, ast.Constant) and node.func.attr in {"pop", "__delitem__"} and any(path and path[0] == selector_arg.value for path in tracked):
                            return True
                if isinstance(receiver, ast.Subscript):
                    parsed = subscript_path(receiver)
                    if parsed is not None:
                        holder, path = parsed
                        if holder.id in aliases or holder.id in log_holder_paths or holder.id in recorder_holder_paths:
                            holder_paths = log_holder_paths.get(holder.id, {}).get(name, set())
                            recorder_paths = recorder_holder_paths.get(holder.id, {}).get(name, set())
                            if not holder_paths and not recorder_paths and path == (0,) and holder.id in log_holder_paths:
                                return True
                            if not any(candidate[:len(path)] == path for candidate in holder_paths | recorder_paths):
                                continue
                            if node.func.attr in bound_mutators:
                                return True
                if isinstance(receiver, ast.Attribute) and isinstance(receiver.value, ast.Name) and receiver.value.id in ({name} | instance_aliases) and receiver.attr == "write_log":
                    return True
                if isinstance(receiver, ast.Attribute) and receiver.attr == "write_log" and isinstance(receiver.value, ast.Subscript):
                    parsed = subscript_path(receiver.value)
                    if parsed is not None:
                        holder, path = parsed
                        paths = recorder_holder_paths.get(holder.id, {}).get(name, set())
                        if any(candidate[:len(path)] == path for candidate in paths):
                            return True
                if isinstance(receiver, ast.Name) and receiver.id in aliases:
                    return True
            if isinstance(node, ast.Call) and any(log_expr(arg) for arg in [*node.args, *(kw.value for kw in node.keywords)]):
                if not (isinstance(node.func, ast.Name) and node.func.id == "tuple" and "tuple" not in shadowed_builtin_names):
                    return True
            if isinstance(node, ast.Call) and (log_expr(node.func) or any(log_expr(arg) for arg in [*node.args, *(kw.value for kw in node.keywords)])) and not (isinstance(node.func, ast.Name) and node.func.id == "tuple" and "tuple" not in shadowed_builtin_names):
                return True
            if isinstance(node, ast.Attribute) and isinstance(node.value, ast.Subscript) and isinstance(node.value.value, ast.Name) and node.value.value.id in aliases:
                parsed = subscript_path(node.value)
                if parsed is None:
                    return True
                holder, path = parsed
                holder_paths = log_holder_paths.get(holder.id, {}).get(name, set())
                if any(candidate[:len(path)] == path for candidate in holder_paths):
                    return True
            if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)) and node.id == name:
                # The creation assignment itself is handled before this check;
                # any later binding is a rebinding and therefore invalid.
                return True
            if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)) and node.id in aliases:
                return True
        return False

    def _is_derived_mutation(stmt: ast.stmt, name: str, shadowed_names: set[str] | None = None) -> bool:
        aliases = derived_aliases_for(name)
        family = ({name} | aliases) - (shadowed_names or set())
        if not family:
            return False
        def family_expr(node: ast.AST) -> bool:
            root = node
            while isinstance(root, (ast.Attribute, ast.Subscript)):
                root = root.value
            return isinstance(root, ast.Name) and root.id in family
        def tracked_expr(node: ast.AST) -> bool:
            if family_expr(node):
                return True
            if isinstance(node, ast.Starred):
                return tracked_expr(node.value)
            if isinstance(node, (ast.Tuple, ast.List, ast.Set)):
                return any(tracked_expr(item) for item in node.elts)
            if isinstance(node, ast.Dict):
                return any(tracked_expr(item) for item in node.values)
            if isinstance(node, ast.Subscript) and isinstance(node.slice, ast.Constant) and isinstance(node.slice.value, (int, str)):
                base = node.value
                selected: ast.AST | None = None
                if isinstance(base, (ast.Tuple, ast.List)) and isinstance(node.slice.value, int) and 0 <= node.slice.value < len(base.elts):
                    selected = base.elts[node.slice.value]
                elif isinstance(base, ast.Dict):
                    for key, value in zip(base.keys, base.values):
                        if isinstance(key, ast.Constant) and key.value == node.slice.value:
                            selected = value
                            break
                if selected is not None:
                    return tracked_expr(selected)
                if isinstance(base, ast.Subscript):
                    return tracked_expr(base)
                parsed = subscript_path(node)
                if parsed is not None:
                    holder, path = parsed
                    return any(candidate[:len(path)] == path for candidate in derived_holder_paths.get(holder.id, {}).get(name, set()))
            return False
        for node in mutation_nodes(stmt):
            if isinstance(node, ast.Name) and node.id in family and isinstance(node.ctx, (ast.Store, ast.Del)):
                return True
            if isinstance(node, (ast.Attribute, ast.Subscript)) and isinstance(node.ctx, (ast.Store, ast.Del)):
                root = node.value
                while isinstance(root, (ast.Attribute, ast.Subscript)):
                    root = root.value
                if isinstance(root, ast.Name) and root.id in family:
                    return True
            if isinstance(node, ast.Call):
                if isinstance(node.func, ast.Attribute):
                    if (isinstance(node.func.value, ast.Name) and node.func.attr in {"pop", "setdefault", "__setitem__", "__delitem__"}
                            and node.args and node.func.value.id in derived_holder_paths):
                        tracked = derived_holder_paths.get(node.func.value.id, {}).get(name, set())
                        selector = node.args[0] if node.args else None
                        if tracked and isinstance(selector, ast.Constant) and not any(path and path[0] == selector.value for path in tracked):
                            # See the recorder analogue above: path insertion
                            # is recorded before this mutation check.
                            continue
                        if tracked and node.func.attr in {"__setitem__", "setdefault"} and len(node.args) >= 2 and tracked_expr(node.args[1]):
                            continue
                        if tracked and not isinstance(selector, ast.Constant) and node.func.attr in {"pop", "__delitem__", "__setitem__", "setdefault"}:
                            return True
                        if tracked and isinstance(selector, ast.Constant) and node.func.attr in {"pop", "__delitem__"} and any(path and path[0] == selector.value for path in tracked):
                            return True
                    if (isinstance(node.func.value, ast.Name) and node.func.value.id == "dict"
                            and node.func.attr in {"__setitem__", "pop", "setdefault", "__delitem__"}
                            and len(node.args) >= 2 and isinstance(node.args[0], ast.Name)):
                        holder_name = node.args[0].id
                        selector_node = node.args[1]
                        if not isinstance(selector_node, ast.Constant):
                            if derived_holder_paths.get(holder_name, {}).get(name):
                                return True
                            continue
                        selector = selector_node.value
                        tracked = derived_holder_paths.get(holder_name, {}).get(name, set())
                        if tracked and not any(path and path[0] == selector for path in tracked):
                            continue
                        if tracked and node.func.attr in {"__setitem__", "setdefault"} and len(node.args) >= 3 and tracked_expr(node.args[2]):
                            continue
                        if tracked and node.func.attr in {"pop", "__delitem__"} and any(path and path[0] == selector for path in tracked):
                            return True
                    if isinstance(node.func.value, ast.Subscript):
                        selected = subscript_path(node.func.value)
                        if selected is not None:
                            holder, path = selected
                            if not any(candidate[:len(path)] == path for candidate in derived_holder_paths.get(holder.id, {}).get(name, set())):
                                continue
                    if (isinstance(node.func.value, ast.Name) and node.func.value.id in mutators
                            and node.func.attr in mutators[node.func.value.id] and node.args
                            and isinstance(node.args[0], ast.Subscript)):
                        selected = subscript_path(node.args[0])
                        if selected is not None:
                            holder, path = selected
                            if not any(candidate[:len(path)] == path for candidate in derived_holder_paths.get(holder.id, {}).get(name, set())):
                                continue
                    if node.func.attr in {"has_inputs"}:
                        continue
                    if node.func.attr in bound_mutators and family_expr(node.func.value):
                        return True
                    if (
                        isinstance(node.func.value, ast.Name)
                        and node.func.value.id in mutators
                        and node.func.attr in mutators[node.func.value.id]
                        and node.args
                        and family_expr(node.args[0])
                    ):
                        return True
                elif isinstance(node.func, ast.Name):
                    alias = callable_aliases.get(node.func.id)
                    if alias is not None and alias[0] == "bound" and family_expr(ast.Name(id=alias[1] or "")):
                        return True
                    if alias is not None and alias[0] == "unbound" and node.args and family_expr(node.args[0]):
                        return True
                    positions = helper_mutated_params.get(node.func.id, set())
                    for index in positions:
                        if index < len(node.args) and family_expr(node.args[index]):
                            return True
                    helper_def = next((fn for fn in helper_defs if fn.name == node.func.id), None)
                    if helper_def is not None:
                        params = [*helper_def.args.posonlyargs, *helper_def.args.args]
                        for index in positions:
                            if index >= len(params):
                                continue
                            param_name = params[index].arg
                            if any(kw.arg == param_name and family_expr(kw.value) for kw in node.keywords):
                                return True
                    builtin_safe = node.func.id in safe_builtin_names and node.func.id not in shadowed_builtin_names
                    if not builtin_safe and not positions and node.func.id not in helper_by_name and not is_direct_design_call(node) and any(tracked_expr(arg) for arg in [*node.args, *(kw.value for kw in node.keywords)]):
                        return True
                tracked_args = [*node.args, *(kw.value for kw in node.keywords)]
                if any(tracked_expr(arg) for arg in tracked_args) and not isinstance(node.func, ast.Name):
                    if isinstance(node.func, ast.Attribute) and node.func.attr in {"append", "extend"} and isinstance(node.func.value, ast.Name) and node.func.value.id not in family:
                        continue
                    if (
                        isinstance(node.func, ast.Attribute)
                        and isinstance(node.func.value, ast.Name)
                        and node.func.value.id in read_only_unbound
                        and node.func.attr in read_only_unbound[node.func.value.id]
                    ):
                        continue
                    if is_direct_design_call(node):
                        continue
                    return True
        return False

    # Record local helper parameters that perform a recognized mutation. Calls
    # through these helpers must not launder a TD-derived value past the gate.
    helper_defs = [
        stmt for stmt in getattr(tree, "body", [])
        if isinstance(stmt, ast.FunctionDef) and stmt is not entrypoint_fn
    ] + [stmt for stmt in entrypoint_fn.body if isinstance(stmt, ast.FunctionDef)]
    for stmt in helper_defs:
        if not isinstance(stmt, ast.FunctionDef):
            continue
        params = [*stmt.args.posonlyargs, *stmt.args.args]
        mutated: set[int] = set()
        helper_nodes = _HelperExecutedNodes(stmt)
        helper_nodes.visit(stmt)
        param_aliases: dict[str, int] = {param.arg: index for index, param in enumerate(params)}
        local_callables: dict[str, tuple[str, str]] = {}
        for binding in helper_nodes.nodes:
            if isinstance(binding, ast.Assign) and len(binding.targets) == 1 and isinstance(binding.targets[0], ast.Name) and isinstance(binding.value, ast.Name):
                if binding.value.id in param_aliases:
                    param_aliases[binding.targets[0].id] = param_aliases[binding.value.id]
                if binding.value.id in local_callables:
                    local_callables[binding.targets[0].id] = local_callables[binding.value.id]
            if isinstance(binding, ast.Assign) and len(binding.targets) == 1 and isinstance(binding.targets[0], ast.Name) and isinstance(binding.value, ast.Attribute) and isinstance(binding.value.value, ast.Name):
                if binding.value.value.id in mutators and binding.value.attr in mutators[binding.value.value.id]:
                    local_callables[binding.targets[0].id] = ("unbound", binding.value.attr)
        for node in helper_nodes.nodes:
            if isinstance(node, ast.AugAssign) and isinstance(node.target, ast.Name):
                if node.target.id in param_aliases:
                    mutated.add(param_aliases[node.target.id])
            if isinstance(node, (ast.Assign, ast.AnnAssign, ast.Delete)):
                targets = list(getattr(node, "targets", [])) if isinstance(node, ast.Assign) else [getattr(node, "target", None)] if isinstance(node, ast.AnnAssign) else list(node.targets)
                for target in targets:
                    root = target
                    while isinstance(root, (ast.Attribute, ast.Subscript)):
                        root = root.value
                    if isinstance(root, ast.Name):
                        if root.id in param_aliases:
                            mutated.add(param_aliases[root.id])
            if isinstance(node, ast.Call):
                if isinstance(node.func, ast.Attribute):
                    receiver = node.func.value
                    if isinstance(receiver, ast.Name) and node.func.attr in bound_mutators:
                        if receiver.id in param_aliases:
                            mutated.add(param_aliases[receiver.id])
                    if isinstance(receiver, ast.Name) and receiver.id in mutators and node.func.attr in mutators[receiver.id]:
                        if node.args and isinstance(node.args[0], ast.Name) and node.args[0].id in param_aliases:
                            mutated.add(param_aliases[node.args[0].id])
                elif isinstance(node.func, ast.Name) and node.func.id in local_callables:
                    kind, _method = local_callables[node.func.id]
                    if kind == "unbound" and node.args and isinstance(node.args[0], ast.Name) and node.args[0].id in param_aliases:
                        mutated.add(param_aliases[node.args[0].id])
        if mutated:
            helper_mutated_params[stmt.name] = mutated
    # Close helper mutation effects over helper-to-helper delegation. This is
    # intentionally structural: only a parameter forwarded directly to a
    # known mutating helper inherits its effect.
    helper_by_name = {stmt.name: stmt for stmt in helper_defs}
    changed = True
    while changed:
        changed = False
        for stmt in helper_defs:
            params = [*stmt.args.posonlyargs, *stmt.args.args]
            current = helper_mutated_params.setdefault(stmt.name, set())
            local_aliases: dict[str, tuple[str, int | str]] = {}
            helper_nodes = _HelperExecutedNodes(stmt)
            helper_nodes.visit(stmt)
            for binding in helper_nodes.nodes:
                if not isinstance(binding, ast.Assign):
                    continue
                targets = [binding.targets[0]] if binding.targets else []
                if len(targets) != 1 or not isinstance(targets[0], ast.Name):
                    continue
                target_name = targets[0].id
                value = binding.value
                if isinstance(value, ast.Attribute) and isinstance(value.value, ast.Name):
                    receiver = value.value.id
                    if value.attr in bound_mutators and any(receiver == param.arg for param in params):
                        for index, param in enumerate(params):
                            if receiver == param.arg:
                                local_aliases[target_name] = ("bound", index)
                    elif receiver in mutators and value.attr in mutators[receiver]:
                        local_aliases[target_name] = ("unbound", 0)
                elif isinstance(value, ast.Name) and value.id in helper_mutated_params:
                    local_aliases[target_name] = ("helper", value.id)
            before_effects = set(current)
            for node in helper_nodes.nodes:
                if not isinstance(node, ast.Call) or not isinstance(node.func, ast.Name):
                    continue
                local_alias = local_aliases.get(node.func.id)
                if local_alias is not None:
                    if local_alias[0] == "bound":
                        current.add(int(local_alias[1]))
                    elif local_alias[0] == "unbound" and node.args and isinstance(node.args[0], ast.Name):
                        for index, param in enumerate(params):
                            if node.args[0].id == param.arg:
                                current.add(index)
                    elif local_alias[0] == "helper":
                        callee_effects = helper_mutated_params.get(str(local_alias[1]), set())
                        for pos in callee_effects:
                            if pos < len(node.args) and isinstance(node.args[pos], ast.Name):
                                for index, param in enumerate(params):
                                    if node.args[pos].id == param.arg:
                                        current.add(index)
                callee_positions = helper_mutated_params.get(node.func.id, set())
                callee_params = helper_by_name.get(node.func.id)
                if callee_params is None:
                    continue
                callee_args = [*callee_params.args.posonlyargs, *callee_params.args.args]
                for pos in callee_positions:
                    if pos >= len(callee_args):
                        continue
                    callee_name = callee_args[pos].arg
                    if pos < len(node.args) and isinstance(node.args[pos], ast.Name):
                        for caller_pos, param in enumerate(params):
                            if node.args[pos].id == param.arg and caller_pos not in current:
                                current.add(caller_pos)
                                changed = True
                    for kw in node.keywords:
                        if kw.arg == callee_name:
                            for caller_pos, param in enumerate(params):
                                if isinstance(kw.value, ast.Name) and kw.value.id == param.arg and caller_pos not in current:
                                    current.add(caller_pos)
                                    changed = True
            if current != before_effects:
                changed = True

    def flow_statements(statements: list[ast.stmt]) -> list[ast.stmt]:
        result: list[ast.stmt] = []
        for statement in statements:
            result.append(statement)
            class _FlowNamedExpr(ast.NodeVisitor):
                def __init__(self) -> None:
                    self.nodes: list[ast.NamedExpr] = []
                def visit_NamedExpr(self, node: ast.NamedExpr) -> None:
                    self.nodes.append(node)
                    self.visit(node.value)
                def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
                    for expr in [*node.decorator_list, *node.args.defaults, *[x for x in node.args.kw_defaults if x is not None]]:
                        self.visit(expr)
                visit_AsyncFunctionDef = visit_FunctionDef
                def visit_Lambda(self, node: ast.Lambda) -> None:
                    for expr in [*node.args.defaults, *[x for x in node.args.kw_defaults if x is not None]]:
                        self.visit(expr)
                def visit_ClassDef(self, node: ast.ClassDef) -> None:
                    for expr in [*node.decorator_list, *node.bases, *(kw.value for kw in node.keywords)]:
                        self.visit(expr)
                def _comp(self, node: ast.AST) -> None:
                    for generator in getattr(node, "generators", []):
                        self.visit(generator.iter)
                        for condition in generator.ifs:
                            self.visit(condition)
                    if isinstance(node, ast.DictComp):
                        self.visit(node.key); self.visit(node.value)
                    else:
                        self.visit(node.elt)
                visit_ListComp = _comp
                visit_SetComp = _comp
                visit_DictComp = _comp
                visit_GeneratorExp = _comp
            _named_exprs = _FlowNamedExpr()
            _named_exprs.visit(statement)
            for node in _named_exprs.nodes:
                if isinstance(node, ast.NamedExpr) and isinstance(node.target, ast.Name):
                    result.append(ast.Assign(targets=[node.target], value=node.value))
            if isinstance(statement, (ast.For, ast.AsyncFor)):
                if isinstance(statement.iter, (ast.Tuple, ast.List)):
                    result.extend(ast.Assign(targets=[statement.target], value=item) for item in statement.iter.elts)
                else:
                    result.append(ast.Assign(targets=[statement.target], value=statement.iter))
            if isinstance(statement, (ast.If, ast.For, ast.AsyncFor, ast.While, ast.With, ast.AsyncWith)):
                for block in [statement.body, statement.orelse] if isinstance(statement, (ast.If, ast.For, ast.AsyncFor, ast.While)) else [statement.body]:
                    result.extend(flow_statements(block))
            elif isinstance(statement, ast.Try):
                result.extend(flow_statements(statement.body))
                result.extend(flow_statements(statement.orelse))
                result.extend(flow_statements(statement.finalbody))
                for handler in statement.handlers:
                    result.extend(flow_statements(handler.body))
        return result

    def exact_destructure_pairs(target: ast.AST, value: ast.AST) -> list[tuple[ast.Name, ast.AST, int]]:
        if not isinstance(target, (ast.Tuple, ast.List)) or not isinstance(value, (ast.Tuple, ast.List)):
            return [(target, value, 0)] if isinstance(target, ast.Name) else []
        star_index = next((i for i, item in enumerate(target.elts) if isinstance(item, ast.Starred)), None)
        if star_index is None:
            pairs: list[tuple[ast.Name, ast.AST, int]] = []
            for left, right in zip(target.elts, value.elts):
                pairs.extend((name, source, index) for index, (name, source, _) in enumerate(exact_destructure_pairs(left, right)))
            return pairs
        suffix_count = len(target.elts) - star_index - 1
        pairs = []
        for index, item in enumerate(target.elts[:star_index]):
            pairs.extend((name, source, index) for name, source, _ in exact_destructure_pairs(item, value.elts[index]))
        star_target = target.elts[star_index]
        if isinstance(star_target, ast.Starred) and isinstance(star_target.value, ast.Name):
            for source_index in range(star_index, len(value.elts) - suffix_count):
                pairs.append((star_target.value, value.elts[source_index], source_index - star_index))
        for offset, item in enumerate(target.elts[star_index + 1:]):
            pairs.extend((name, source, len(value.elts) - suffix_count + offset) for name, source, _ in exact_destructure_pairs(item, value.elts[len(value.elts) - suffix_count + offset]))
        return pairs

    def bind_star_name_paths(target: ast.AST, source_name: str, paths_by_root: dict[str, set[tuple[object, ...]]], holder_paths: dict[str, dict[str, set[tuple[object, ...]]]], aliases_by_root: dict[str, set[str]], lengths: dict[str, int]) -> None:
        if not isinstance(target, (ast.Tuple, ast.List)) or source_name not in lengths:
            return
        star_index = next((i for i, item in enumerate(target.elts) if isinstance(item, ast.Starred)), None)
        if star_index is None:
            return
        suffix_count = len(target.elts) - star_index - 1
        length = lengths[source_name]
        star_target = target.elts[star_index]
        for root, paths in paths_by_root.items():
            for path in paths:
                if not path or not isinstance(path[0], int):
                    continue
                index = path[0]
                if isinstance(star_target, ast.Starred) and isinstance(star_target.value, ast.Name) and star_index <= index < length - suffix_count:
                    holder_paths.setdefault(star_target.value.id, {}).setdefault(root, set()).add((index - star_index, *path[1:]))
                    aliases_by_root.setdefault(root, set()).add(star_target.value.id)
                elif index >= length - suffix_count:
                    target_item = target.elts[star_index + 1 + index - (length - suffix_count)]
                    if isinstance(target_item, ast.Name) and len(path) == 1:
                        aliases_by_root.setdefault(root, set()).add(target_item.id)

    def _class_effect_roots(node: ast.ClassDef) -> set[str]:
        """Return derived roots mutated by this executed class body.

        Class locals are isolated from the enclosing verifier.  Temporary
        provenance aliases are used only while scanning the class body.
        ``nonlocal`` assignments deliberately remain enclosing-scope effects.
        """
        added: dict[str, set[str]] = {}
        invalid: set[str] = set()
        local_shadows: set[str] = set()
        nonlocals = {
            name
            for statement in node.body
            if isinstance(statement, ast.Nonlocal)
            for name in statement.names
        }

        def add_alias(alias: str, root: str) -> None:
            if alias == root:
                return
            derived_aliases.setdefault(root, set()).add(alias)
            added.setdefault(root, set()).add(alias)

        def has_outer_provenance(expr: ast.AST, root: str) -> bool:
            if isinstance(expr, ast.Name) and expr.id in local_shadows:
                return expr.id in derived_aliases_for(root)
            return _expr_has_derived_provenance(expr, root)

        try:
            for statement in node.body:
                if isinstance(statement, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    # Only definition-time expressions execute here.
                    for expression in [
                        *statement.decorator_list,
                        *statement.args.defaults,
                        *[x for x in statement.args.kw_defaults if x is not None],
                    ]:
                        synthetic = ast.Expr(value=expression)
                        for root in tuple(design_derived_vars):
                            if _is_derived_mutation(synthetic, root, local_shadows):
                                invalid.add(root)
                    continue
                if isinstance(statement, ast.ClassDef):
                    for expression in [*statement.decorator_list, *statement.bases, *(kw.value for kw in statement.keywords)]:
                        synthetic = ast.Expr(value=expression)
                        for root in tuple(design_derived_vars):
                            if _is_derived_mutation(synthetic, root, local_shadows):
                                invalid.add(root)
                    invalid.update(_class_effect_roots(statement))
                    continue
                if isinstance(statement, (ast.Assign, ast.AnnAssign, ast.NamedExpr)):
                    targets = getattr(statement, "targets", [getattr(statement, "target", None)])
                    value = getattr(statement, "value", None)
                    if value is not None:
                        for target in targets:
                            if not isinstance(target, ast.Name):
                                continue
                            if target.id in nonlocals:
                                for root in tuple(design_derived_vars):
                                    if target.id == root or _expr_has_derived_provenance(value, root):
                                        invalid.add(root)
                            else:
                                captured = False
                                for root in tuple(design_derived_vars):
                                    if has_outer_provenance(value, root):
                                        add_alias(target.id, root)
                                        captured = True
                                if captured:
                                    local_shadows.discard(target.id)
                                else:
                                    local_shadows.add(target.id)
                        # Assignment expressions nested inside an ordinary
                        # class-local binding execute in the same sequential
                        # class namespace.  Process them before checking the
                        # containing assignment so ``shadow = (result :=
                        # [])`` shadows the enclosing result as well.
                        for named in [n for n in ast.walk(value) if isinstance(n, ast.NamedExpr)]:
                            target = named.target
                            if not isinstance(target, ast.Name):
                                continue
                            if target.id in nonlocals:
                                for root in tuple(design_derived_vars):
                                    if target.id == root or has_outer_provenance(named.value, root):
                                        invalid.add(root)
                            else:
                                captured = False
                                for root in tuple(design_derived_vars):
                                    if has_outer_provenance(named.value, root):
                                        add_alias(target.id, root)
                                        captured = True
                                if captured:
                                    local_shadows.discard(target.id)
                                else:
                                    local_shadows.add(target.id)
                        for root in tuple(design_derived_vars):
                            if _is_derived_mutation(statement, root, local_shadows):
                                # A plain local binding (for example
                                # ``result = []``) shadows the enclosing root;
                                # mutations through aliases captured earlier
                                # remain visible through the scoped family.
                                if not has_outer_provenance(value, root):
                                    invalid.add(root)
                        continue
                for named in [n for n in ast.walk(statement) if isinstance(n, ast.NamedExpr)]:
                    target = named.target
                    if isinstance(target, ast.Name):
                        if target.id in nonlocals:
                            for root in tuple(design_derived_vars):
                                if target.id == root or _expr_has_derived_provenance(named.value, root):
                                    invalid.add(root)
                        else:
                            captured = False
                            for root in tuple(design_derived_vars):
                                if has_outer_provenance(named.value, root):
                                    add_alias(target.id, root)
                                    captured = True
                            if captured:
                                local_shadows.discard(target.id)
                            else:
                                local_shadows.add(target.id)
                for root in tuple(design_derived_vars):
                    if _is_derived_mutation(statement, root, local_shadows):
                        invalid.add(root)
        finally:
            for root, aliases in added.items():
                current = derived_aliases.get(root, set())
                current.difference_update(aliases)
        return invalid

    def _comprehension_effect_roots(statement: ast.stmt) -> set[str]:
        """Scan executed comprehension expressions with isolated targets."""
        invalid: set[str] = set()
        for node in ast.walk(statement):
            if not isinstance(node, (ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)):
                continue
            added: dict[str, set[str]] = {}
            local_shadows: set[str] = set()
            try:
                for generator in node.generators:
                    iterable_roots: set[str] = set()
                    for root in tuple(design_derived_vars):
                        if _expr_has_derived_provenance(generator.iter, root):
                            iterable_roots.add(root)
                    targets = [generator.target] if isinstance(generator.target, ast.Name) else [x for x, _ in destructure_paths(generator.target)]
                    for target in targets:
                        if isinstance(target, ast.Name):
                            for root in iterable_roots:
                                derived_aliases.setdefault(root, set()).add(target.id)
                                added.setdefault(root, set()).add(target.id)
                            if not iterable_roots:
                                local_shadows.add(target.id)
                for root in tuple(design_derived_vars):
                    if _is_derived_mutation(statement, root, local_shadows):
                        invalid.add(root)
            finally:
                for root, aliases in added.items():
                    derived_aliases.get(root, set()).difference_update(aliases)
        return invalid

    for stmt in flow_statements(entrypoint_fn.body):
        _record_exact_selector_insertions(stmt)
        if isinstance(stmt, ast.ClassDef):
            for root in _class_effect_roots(stmt):
                invalid_derived.add(root)
                design_derived_vars.discard(root)
        for root in _comprehension_effect_roots(stmt):
            invalid_derived.add(root)
            design_derived_vars.discard(root)
        if isinstance(stmt, (ast.Assign, ast.AnnAssign, ast.NamedExpr)):
            targets = getattr(stmt, "targets", [getattr(stmt, "target", None)])
            val_node = getattr(stmt, "value", None)
            if val_node is None:
                continue

            comprehension_shadows: set[str] = set()
            for comp in ast.walk(stmt):
                if not isinstance(comp, (ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)):
                    continue
                for generator in comp.generators:
                    iterable_roots = {
                        root for root in tuple(design_derived_vars)
                        if _expr_has_derived_provenance(generator.iter, root)
                    }
                    targets = [generator.target] if isinstance(generator.target, ast.Name) else [x for x, _ in destructure_paths(generator.target)]
                    for target in targets:
                        if isinstance(target, ast.Name) and target.id in design_derived_vars and target.id not in iterable_roots:
                            comprehension_shadows.add(target.id)

            for name in tuple(design_derived_vars):
                if _is_derived_mutation(stmt, name, comprehension_shadows) and not (isinstance(val_node, ast.Call) and any(isinstance(t, ast.Name) and t.id == name for t in targets)):
                    invalid_derived.add(name)
                    design_derived_vars.discard(name)

            for tgt in targets:
                if (
                    isinstance(tgt, ast.Name)
                    and isinstance(val_node, ast.Attribute)
                    and isinstance(val_node.value, ast.Name)
                    and val_node.value.id in mutators
                    and val_node.attr in mutators[val_node.value.id]
                ):
                    callable_aliases[tgt.id] = ("unbound", val_node.attr)
                elif (
                    isinstance(tgt, ast.Name)
                    and isinstance(val_node, ast.Attribute)
                    and isinstance(val_node.value, (ast.Name, ast.Attribute, ast.Subscript))
                    and isinstance(val_node.value, ast.Name)
                    and (val_node.value.id in design_derived_vars or any(val_node.value.id in derived_aliases_for(root) for root in design_derived_vars))
                    and val_node.attr in bound_mutators
                ):
                    callable_aliases[tgt.id] = ("bound", val_node.value.id)
                elif isinstance(tgt, ast.Name) and isinstance(val_node, ast.Name) and val_node.id in callable_aliases:
                    callable_aliases[tgt.id] = callable_aliases[val_node.id]
                elif isinstance(tgt, ast.Name) and isinstance(val_node, ast.Name) and val_node.id in helper_mutated_params:
                    helper_mutated_params[tgt.id] = set(helper_mutated_params[val_node.id])
                if isinstance(tgt, ast.Name) and isinstance(val_node, ast.Name) and val_node.id in design_derived_vars:
                    derived_aliases.setdefault(val_node.id, set()).add(tgt.id)
                    if val_node.id in derived_holder_paths:
                        for root, paths in derived_holder_paths[val_node.id].items():
                            derived_holder_paths.setdefault(tgt.id, {}).setdefault(root, set()).update(paths)
                if isinstance(tgt, ast.Name) and isinstance(val_node, ast.Name):
                    if val_node.id in literal_lengths:
                        literal_lengths[tgt.id] = literal_lengths[val_node.id]
                    for root in tuple(design_derived_vars):
                        if val_node.id in derived_aliases_for(root):
                            derived_aliases.setdefault(root, set()).add(tgt.id)
                            for holder_root, paths in derived_holder_paths.get(val_node.id, {}).items():
                                derived_holder_paths.setdefault(tgt.id, {}).setdefault(holder_root, set()).update(paths)
                    for root, paths in log_holder_paths.get(val_node.id, {}).items():
                        log_holder_paths.setdefault(tgt.id, {}).setdefault(root, set()).update(paths)
                if isinstance(tgt, (ast.Tuple, ast.List)) and isinstance(val_node, (ast.Tuple, ast.List)):
                    bind_derived_target(tgt, val_node)
                    for target_item, source_item, source_index in exact_destructure_pairs(tgt, val_node):
                        for root in tuple(design_derived_vars):
                            if literal_derived_paths(source_item, root):
                                derived_holder_paths.setdefault(target_item.id, {}).setdefault(root, set()).add((source_index,))
                            if isinstance(source_item, ast.Name) and source_item.id in ({root} | derived_aliases_for(root)):
                                derived_aliases.setdefault(root, set()).add(target_item.id)
                if isinstance(tgt, ast.Name) and isinstance(val_node, (ast.Tuple, ast.List, ast.Dict)):
                    def holder_contains(node: ast.AST, root_name: str) -> bool:
                        if isinstance(node, ast.Name):
                            return node.id == root_name or node.id in derived_aliases_for(root_name)
                        if isinstance(node, (ast.Attribute, ast.Subscript)):
                            root_node = node
                            while isinstance(root_node, (ast.Attribute, ast.Subscript)):
                                root_node = root_node.value
                            return isinstance(root_node, ast.Name) and (root_node.id == root_name or root_node.id in derived_aliases_for(root_name))
                        if isinstance(node, (ast.Tuple, ast.List)):
                            return any(holder_contains(elt, root_name) for elt in node.elts)
                        if isinstance(node, ast.Dict):
                            return any(holder_contains(v, root_name) for v in node.values)
                        return False
                    for root in tuple(design_derived_vars):
                        if holder_contains(val_node, root):
                            derived_aliases.setdefault(root, set()).add(tgt.id)
                        paths = literal_derived_paths(val_node, root)
                        if paths:
                            derived_holder_paths.setdefault(tgt.id, {}).setdefault(root, set()).update(paths)
                if isinstance(tgt, ast.Name) and isinstance(val_node, ast.Call) and isinstance(val_node.func, ast.Name) and val_node.func.id in {"tuple", "list"}:
                    for root in tuple(design_derived_vars):
                        if any(isinstance(node, ast.Name) and node.id in ({root} | derived_aliases_for(root)) for node in ast.walk(val_node)):
                            derived_holder_paths.setdefault(tgt.id, {}).setdefault(root, set()).add((-1,))
                if isinstance(tgt, ast.Name) and isinstance(val_node, ast.Subscript):
                    parsed = subscript_path(val_node)
                    if parsed is not None:
                        holder, path = parsed
                        for root in tuple(design_derived_vars):
                            paths = derived_holder_paths.get(holder.id, {}).get(root, set())
                            if any(candidate[:len(path)] == path for candidate in paths):
                                derived_aliases.setdefault(root, set()).add(tgt.id)
                if isinstance(tgt, (ast.Tuple, ast.List)) and isinstance(val_node, ast.Name):
                    bind_star_name_paths(tgt, val_node.id, derived_holder_paths.get(val_node.id, {}), derived_holder_paths, derived_aliases, literal_lengths)
                    if not (starred_names(tgt) and val_node.id in literal_lengths):
                        for item, path in destructure_paths(tgt):
                            for root in tuple(design_derived_vars):
                                paths = derived_holder_paths.get(val_node.id, {}).get(root, set())
                                if any(candidate[:len(path)] == path for candidate in paths):
                                    derived_aliases.setdefault(root, set()).add(item.id)
                if isinstance(tgt, ast.Name) and isinstance(val_node, ast.Name) and (val_node.id in witness_candidates or val_node.id in witness_instances or any(val_node.id in recorder_aliases_for(root) for root in witness_candidates)):
                    root = next((root for root in witness_candidates if val_node.id == root or val_node.id in recorder_aliases_for(root)), val_node.id)
                    recorder_aliases.setdefault(root, set()).add(tgt.id)

            # A once-bound blank witness starts as a candidate.
            for tgt in targets:
                if isinstance(tgt, ast.Name) and isinstance(val_node, ast.Call) and isinstance(val_node.func, ast.Name) and val_node.func.id in witness_classes:
                    if var_event_counts.get(tgt.id, 0) == 1 and not val_node.args and not val_node.keywords:
                        witness_candidates.add(tgt.id)

            # Mark candidates only when the assignment value itself is the
            # evaluated canonical design call.  Nested calls in short-circuit,
            # lambda, conditional, comprehensions, or other delayed forms do
            # not constitute an observed interaction.
            if isinstance(val_node, ast.Call) and is_direct_design_call(val_node):
                args = list(val_node.args) + [kw.value for kw in val_node.keywords]
                for arg in args:
                    if isinstance(arg, ast.Name) and arg.id in witness_candidates and arg.id not in invalid_witnesses:
                        witness_instances.add(arg.id)

            if any(starred_names(tgt) for tgt in targets if isinstance(tgt, (ast.Tuple, ast.List))):
                for root in tuple(witness_candidates):
                    tracked_in_star = any(
                        isinstance(tgt, (ast.Tuple, ast.List))
                        and any(
                            isinstance(item, ast.Starred)
                            and any(literal_log_paths(source, root) for source in val_node.elts[index:max(index, len(val_node.elts) - (len(tgt.elts) - index - 1))])
                            for index, item in enumerate(tgt.elts)
                        )
                        for tgt in targets
                    ) if isinstance(val_node, (ast.Tuple, ast.List)) else any(
                        isinstance(tgt, (ast.Tuple, ast.List)) and isinstance(val_node, ast.Name)
                        and any(
                            isinstance(item, ast.Starred)
                            and any(path and index <= path[0] < literal_lengths.get(val_node.id, 0) - (len(tgt.elts) - index - 1) for path in log_holder_paths.get(val_node.id, {}).get(root, set()))
                            for index, item in enumerate(tgt.elts)
                        )
                        for tgt in targets
                    )
                    unknown_star = isinstance(val_node, ast.Name) and val_node.id not in literal_lengths and (tracked_in_star or any(path and path[0] == -1 for path in log_holder_paths.get(val_node.id, {}).get(root, set())) or any(path and path[0] == -1 for path in recorder_holder_paths.get(val_node.id, {}).get(root, set())))
                    if unknown_star:
                        invalid_witnesses.add(root)
                        witness_instances.discard(root)

            # Any external mutation of a candidate/validated witness invalidates
            # it.  Do this after detecting the design call, but exempt the
            # creation assignment itself.
            for name in tuple(witness_candidates):
                if not (isinstance(val_node, ast.Call) and any(isinstance(t, ast.Name) and t.id == name for t in targets)) and _is_witness_mutation(stmt, name):
                    invalid_witnesses.add(name)
                    witness_instances.discard(name)

            for tgt in targets:
                if isinstance(tgt, ast.Name) and isinstance(val_node, (ast.Tuple, ast.List)):
                    literal_lengths[tgt.id] = len(val_node.elts)
                if isinstance(tgt, ast.Name) and isinstance(val_node, ast.Attribute) and isinstance(val_node.value, ast.Name) and val_node.attr == "write_log":
                    for root in tuple(witness_candidates):
                        if val_node.value.id in ({root} | recorder_aliases_for(root)):
                            log_aliases.setdefault(root, set()).add(tgt.id)
                if isinstance(tgt, ast.Name) and isinstance(val_node, ast.Name):
                    for root in tuple(witness_candidates):
                        if val_node.id in aliases_for(root):
                            log_aliases.setdefault(root, set()).add(tgt.id)
                            if val_node.id in log_holder_paths:
                                for holder_root, paths in log_holder_paths[val_node.id].items():
                                    log_holder_paths.setdefault(tgt.id, {}).setdefault(holder_root, set()).update(paths)
                            if val_node.id in recorder_holder_paths:
                                for holder_root, paths in recorder_holder_paths[val_node.id].items():
                                    recorder_holder_paths.setdefault(tgt.id, {}).setdefault(holder_root, set()).update(paths)
                if isinstance(tgt, ast.Name) and isinstance(val_node, ast.Subscript):
                    parsed = subscript_path(val_node)
                    if parsed is not None:
                        holder, path = parsed
                        for root in tuple(witness_candidates):
                            holder_paths = log_holder_paths.get(holder.id, {}).get(root, set())
                            if any(candidate[:len(path)] == path for candidate in holder_paths):
                                log_aliases.setdefault(root, set()).add(tgt.id)
                            recorder_paths = recorder_holder_paths.get(holder.id, {}).get(root, set())
                            if any(candidate[:len(path)] == path for candidate in recorder_paths):
                                recorder_aliases.setdefault(root, set()).add(tgt.id)
                if isinstance(tgt, (ast.Tuple, ast.List)) and isinstance(val_node, ast.Subscript):
                    parsed = subscript_path(val_node)
                    if parsed is not None:
                        holder, path = parsed
                        for root in tuple(witness_candidates):
                            holder_paths = log_holder_paths.get(holder.id, {}).get(root, set())
                            if any(candidate[:len(path)] == path for candidate in holder_paths):
                                for item in tgt.elts:
                                    if isinstance(item, ast.Name):
                                        log_aliases.setdefault(root, set()).add(item.id)
                if isinstance(tgt, (ast.Tuple, ast.List)) and isinstance(val_node, ast.Name):
                    bind_star_name_paths(tgt, val_node.id, log_holder_paths.get(val_node.id, {}), log_holder_paths, log_aliases, literal_lengths)
                    if not (starred_names(tgt) and val_node.id in literal_lengths):
                        for item, path in destructure_paths(tgt):
                            for root in tuple(witness_candidates):
                                paths = log_holder_paths.get(val_node.id, {}).get(root, set())
                                if any(candidate[:len(path)] == path for candidate in paths):
                                    log_aliases.setdefault(root, set()).add(item.id)
                if isinstance(tgt, ast.Name) and isinstance(val_node, (ast.Tuple, ast.List, ast.Dict)):
                    for root in tuple(witness_candidates):
                        paths = literal_log_paths(val_node, root)
                        if paths:
                            log_holder_paths.setdefault(tgt.id, {}).setdefault(root, set()).update(paths)
                        recorder_paths = literal_recorder_paths(val_node, root)
                        if recorder_paths:
                            recorder_holder_paths.setdefault(tgt.id, {}).setdefault(root, set()).update(recorder_paths)
                if isinstance(tgt, ast.Name) and isinstance(val_node, ast.Call) and isinstance(val_node.func, ast.Name) and val_node.func.id in {"tuple", "list"}:
                    for root in tuple(witness_candidates):
                        if any(isinstance(node, ast.Name) and (node.id in aliases_for(root) or node.id in log_holder_paths or node.id in recorder_holder_paths) for node in ast.walk(val_node)) or any(isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name) and node.value.id in ({root} | recorder_aliases_for(root)) and node.attr == "write_log" for node in ast.walk(val_node)):
                            recorder_holder_paths.setdefault(tgt.id, {}).setdefault(root, set()).add((-1,))
                    leaves = [n for n in ast.walk(val_node) if isinstance(n, ast.Name)]
                    for root in tuple(witness_candidates):
                        if any(n.id in aliases_for(root) for n in leaves) or any(isinstance(n, ast.Attribute) and isinstance(n.value, ast.Name) and n.value.id == root and n.attr == "write_log" for n in ast.walk(val_node)):
                            log_aliases.setdefault(root, set()).add(tgt.id)
                if isinstance(tgt, (ast.Tuple, ast.List)) and isinstance(val_node, (ast.Tuple, ast.List)):
                    for target_item, value_item in zip(tgt.elts, val_node.elts):
                        if not isinstance(target_item, ast.Name):
                            continue
                        for root in tuple(witness_candidates):
                            if (
                                (isinstance(value_item, ast.Attribute) and isinstance(value_item.value, ast.Name)
                                 and value_item.value.id == root and value_item.attr == "write_log")
                                or (isinstance(value_item, ast.Name) and value_item.id in aliases_for(root))
                            ):
                                log_aliases.setdefault(root, set()).add(target_item.id)
                        if isinstance(target_item, ast.Name) and isinstance(value_item, ast.Attribute):
                            if isinstance(value_item.value, ast.Name) and value_item.value.id in mutators and value_item.attr in mutators[value_item.value.id]:
                                callable_aliases[target_item.id] = ("unbound", value_item.attr)
                            elif isinstance(value_item.value, ast.Name) and value_item.value.id in design_derived_vars and value_item.attr in bound_mutators:
                                callable_aliases[target_item.id] = ("bound", value_item.value.id)

            def destructured_pairs(target: ast.AST, value: ast.AST) -> list[tuple[ast.Name, ast.AST]]:
                if isinstance(target, ast.Starred):
                    return destructured_pairs(target.value, value)
                if isinstance(target, (ast.Tuple, ast.List)) and isinstance(value, (ast.Tuple, ast.List)):
                    pairs: list[tuple[ast.Name, ast.AST]] = []
                    for left, right in zip(target.elts, value.elts):
                        pairs.extend(destructured_pairs(left, right))
                    return pairs
                return [(target, value)] if isinstance(target, ast.Name) else []

            for target in targets:
                if isinstance(target, (ast.Tuple, ast.List)) and isinstance(val_node, (ast.Tuple, ast.List)):
                    for item, source, source_index in exact_destructure_pairs(target, val_node):
                        for root in tuple(witness_candidates):
                            if literal_log_paths(source, root):
                                log_holder_paths.setdefault(item.id, {}).setdefault(root, set()).add((source_index,))
                            if isinstance(source, ast.Attribute) and source.attr == "write_log" and isinstance(source.value, ast.Name) and (source.value.id == root or source.value.id in recorder_aliases_for(root)):
                                log_aliases.setdefault(root, set()).add(item.id)
                    for item, source in destructured_pairs(target, val_node):
                        for root in tuple(design_derived_vars):
                            if isinstance(source, ast.Name) and source.id in ({root} | derived_aliases_for(root)):
                                derived_aliases.setdefault(root, set()).add(item.id)
                        for root in tuple(witness_candidates):
                            if isinstance(source, ast.Attribute) and source.attr == "write_log" and isinstance(source.value, ast.Name) and (source.value.id == root or source.value.id in recorder_aliases_for(root)):
                                log_aliases.setdefault(root, set()).add(item.id)

            if is_expr_direct_call_derived(val_node):
                for tgt in targets:
                    if isinstance(tgt, ast.Name):
                        # Required to have EXACTLY ONE binding event in the verifier subtree
                        if var_event_counts.get(tgt.id, 0) == 1:
                            design_derived_vars.add(tgt.id)
            if any(starred_names(tgt) for tgt in targets if isinstance(tgt, (ast.Tuple, ast.List))):
                for root in tuple(design_derived_vars):
                    tracked_in_star = any(
                        isinstance(tgt, (ast.Tuple, ast.List))
                        and any(
                            isinstance(item, ast.Starred)
                            and any(literal_derived_paths(source, root) for source in val_node.elts[index:max(index, len(val_node.elts) - (len(tgt.elts) - index - 1))])
                            for index, item in enumerate(tgt.elts)
                        )
                        for tgt in targets
                    ) if isinstance(val_node, (ast.Tuple, ast.List)) else any(
                        isinstance(tgt, (ast.Tuple, ast.List)) and isinstance(val_node, ast.Name)
                        and any(
                            isinstance(item, ast.Starred)
                            and (
                                val_node.id == root
                                or val_node.id in derived_aliases_for(root)
                                or any(path and index <= path[0] < literal_lengths.get(val_node.id, 0) - (len(tgt.elts) - index - 1) for path in derived_holder_paths.get(val_node.id, {}).get(root, set()))
                            )
                            for index, item in enumerate(tgt.elts)
                        )
                        for tgt in targets
                    )
                    unknown_star = isinstance(val_node, ast.Name) and (
                        val_node.id not in literal_lengths
                        or any(path and path[0] == -1 for path in derived_holder_paths.get(val_node.id, {}).get(root, set()))
                    ) and (tracked_in_star or any(path and path[0] == -1 for path in derived_holder_paths.get(val_node.id, {}).get(root, set())))
                    if unknown_star:
                        invalid_derived.add(root)
                        design_derived_vars.discard(root)
        elif not isinstance(stmt, ast.ClassDef):
            for name in tuple(witness_candidates):
                if _is_witness_mutation(stmt, name):
                    invalid_witnesses.add(name)
                    witness_instances.discard(name)
            comprehension_shadows: set[str] = set()
            for comp in ast.walk(stmt):
                if not isinstance(comp, (ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)):
                    continue
                for generator in comp.generators:
                    iterable_roots = {root for root in tuple(design_derived_vars) if _expr_has_derived_provenance(generator.iter, root)}
                    targets = [generator.target] if isinstance(generator.target, ast.Name) else [x for x, _ in destructure_paths(generator.target)]
                    for target in targets:
                        if isinstance(target, ast.Name) and target.id in design_derived_vars and target.id not in iterable_roots:
                            comprehension_shadows.add(target.id)
            for name in tuple(design_derived_vars):
                if _is_derived_mutation(stmt, name, comprehension_shadows):
                    invalid_derived.add(name)
                    design_derived_vars.discard(name)

    return canonical_design_aliases, design_derived_vars


class TestCaseIndependence(unittest.TestCase):
    """Statically verify structural independence properties of all EC case modules."""

    def test_synthetic_provenance_forms_and_fail_closed_boundaries(self) -> None:
        """Exercise the composite/optional/recording grammar without case-name whitelists."""
        snippets: dict[str, tuple[str, bool]] = {
            "pure_composite": ("""
                from openapi_codegen.design import observe
                def verify_case() -> dict[str, object]:
                    result = observe()
                    value = (result.left, result.right)
                    return {"value": value}
            """, True),
            "empty_tuple": ("""
                from openapi_codegen.design import observe
                def verify_case() -> dict[str, object]:
                    result = observe()
                    value = ()
                    return {"value": value}
            """, False),
            "empty_list": ("""
                from openapi_codegen.design import observe
                def verify_case() -> dict[str, object]:
                    result = observe()
                    value = []
                    return {"value": value}
            """, False),
            "same_root_optional": ("""
                from openapi_codegen.design import observe
                def verify_case() -> dict[str, object]:
                    result = observe()
                    value = result.value if result is not None else None
                    return {"value": value}
            """, True),
            "blank_passed_witness": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, True),
            "unpassed_preseeded": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = [("/fake", "seed")]
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    value = tuple(witness.write_log)
                    return {"value": value}
            """, False),
            "passed_preseeded": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = [("/fake", "seed")]
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "constant_inserting": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [("/fake", "seed")]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "filtered_recorder": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        if path == "/expected":
                            self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "observation_before_call": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    value = tuple(witness.write_log)
                    result = run(witness)
                    return {"result": result, "value": value}
            """, False),
            "argument_constructor": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self, seed: str) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder("seed")
                    result = run(witness)
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "rebound": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    witness = Recorder()
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "externally_mutated": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    witness.write_log += [("/fake", "seed")]
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "cross_root_optional": ("""
                from openapi_codegen.design import observe
                def verify_case() -> dict[str, object]:
                    first = observe()
                    second = observe()
                    value = second.value if first is not None else None
                    return {"value": value}
            """, False),
            "sibling_chain_optional": ("""
                from openapi_codegen.design import observe
                def verify_case() -> dict[str, object]:
                    result = observe()
                    value = result.body.schema if result.other is not None else None
                    return {"value": value}
            """, False),
            "unreachable_call": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    value = False and run(witness)
                    return {"value": value}
            """, False),
            "lambda_contained_call": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    value = (lambda: run(witness))
                    return {"value": value}
            """, False),
            "duplicate_writer": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                    def write(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "duplicate_same_writer": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "class_level_log": ("""
                from openapi_codegen.design import run
                class Recorder:
                    write_log = []
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "inherited_descriptor_log": ("""
                from openapi_codegen.design import run
                class Base:
                    write_log = property(lambda self: [("fake", "seed")])
                class Recorder(Base):
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "decorated_recorder": ("""
                from openapi_codegen.design import run
                def deco(cls):
                    return cls
                @deco
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "decorated_initializer": ("""
                from openapi_codegen.design import run
                def deco(fn):
                    return fn
                class Recorder:
                    @deco
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "decorated_writer": ("""
                from openapi_codegen.design import run
                def deco(fn):
                    return fn
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    @deco
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "log_alias_mutated": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    alias = witness.write_log
                    alias.clear()
                    alias += [("/fake", "seed")]
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "recorder_instance_alias_mutated": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    alias = witness
                    result = run(witness)
                    alias.write_log.clear()
                    alias.write_log.extend([("/fake", "seed")])
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "recorder_instance_alias_transitive_mutated": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    alias1 = witness
                    alias2 = alias1
                    result = run(witness)
                    alias2.write_log[:] = [("/fake", "expected")]
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "recorder_holder_extracted_alias_mutated": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    holder = {"log": witness.write_log}
                    alias = holder["log"]
                    alias2 = alias
                    alias.clear()
                    alias2 += [("/fake", "expected")]
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "recorder_alias_immutable_snapshot_control": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    alias = witness.write_log
                    value = tuple(alias)
                    return {"result": result, "value": value}
            """, True),
            "builtin_alias_mutated": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    alias = witness.write_log
                    result = run(witness)
                    list.clear(alias)
                    list.extend(alias, [("/fake", "seed")])
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "holder_alias_mutated": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    holder = (witness.write_log,)
                    result = run(witness)
                    holder[0].clear()
                    holder[0].extend([("/fake", "seed")])
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "direct_attribute_log_mutated": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    list.clear(witness.write_log)
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "dict_holder_log_mutated": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    holder = {"log": witness.write_log}
                    result = run(witness)
                    holder["log"].clear()
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "derived_rebound": ("""
                from openapi_codegen.design import inline_params
                from openapi_codegen.application.document import Item, Parameter
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    result[:] = [Parameter("expected", "query")]
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_alias_mutated": ("""
                from openapi_codegen.design import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params(())
                    alias = result
                    alias.clear()
                    value = result
                    return {"value": value}
            """, False),
            "derived_alias_control": ("""
                from openapi_codegen.design import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params(())
                    alias = result
                    value = alias
                    return {"value": value}
            """, True),
            "derived_append_payload_control": ("""
                from openapi_codegen.design import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params(())
                    checks = []
                    checks.append({"observed": result})
                    value = result
                    return {"value": value}
            """, True),
            "derived_holder_per_root_control": ("""
                from openapi_codegen.design import first, second
                def verify_case() -> dict[str, object]:
                    first_value = first()
                    second_value = second()
                    holder = (first_value,)
                    holder[0].__iadd__(["x"])
                    value = second_value
                    return {"value": value}
            """, True),
            "derived_unbound_iadd_mutation": ("""
                from openapi_codegen.design import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params(())
                    list.__iadd__(result, [])
                    value = result
                    return {"value": value}
            """, False),
            "derived_unbound_imul_mutation": ("""
                from openapi_codegen.design import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params(())
                    list.__imul__(result, 1)
                    value = result
                    return {"value": value}
            """, False),
            "derived_unbound_dict_setdefault_mutation": ("""
                from openapi_codegen.design import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params(())
                    dict.setdefault(result, "key", "value")
                    value = result
                    return {"value": value}
            """, False),
            "derived_unbound_dict_popitem_mutation": ("""
                from openapi_codegen.design import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params(())
                    dict.popitem(result)
                    value = result
                    return {"value": value}
            """, False),
            "derived_unbound_set_difference_update_mutation": ("""
                from openapi_codegen.design import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params(())
                    set.difference_update(result, set())
                    value = result
                    return {"value": value}
            """, False),
            "derived_unbound_set_ior_mutation": ("""
                from openapi_codegen.design import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params(())
                    set.__ior__(result, set())
                    value = result
                    return {"value": value}
            """, False),
            "derived_constructor_copy_control": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    snapshot = list(result)
                    _ = tuple(snapshot)
                    value = result[0].name
                    return {"value": value}
            """, True),
            "derived_unbound_read_control": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    _ = list.count(result, result[0])
                    value = result[0].name
                    return {"value": value}
            """, True),
            "derived_nonalias_holder_control": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    holder = (len(result),)
                    holder_list = list(holder)
                    holder_list[0] = 0
                    value = result[0].name
                    return {"value": value}
            """, True),
            "derived_literal_holder_method_mutated": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    holder = (result,)
                    holder[0].clear()
                    holder[0].extend([Parameter("expected", "query")])
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_literal_holder_slice_mutated": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    holder = (result,)
                    holder[0][:] = [Parameter("expected", "query")]
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_alias_unbound_list_mutated": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    alias = result
                    list.clear(alias)
                    list.extend(alias, [Parameter("expected", "query")])
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_callable_alias_mutated": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    clear = list.clear
                    iadd = list.__iadd__
                    clear(result)
                    iadd(result, [Parameter("expected", "query")])
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_local_helper_mutated": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def replace(items: list[Parameter]) -> None:
                    items.clear()
                    items += [Parameter("expected", "query")]
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    replace(result)
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_helper_internal_bound_alias": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def replace(items: list[Parameter]) -> None:
                    clear = items.clear
                    iadd = items.__iadd__
                    clear()
                    iadd([Parameter("expected", "query")])
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    replace(result)
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_helper_internal_unbound_alias": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def replace(items: list[Parameter]) -> None:
                    clear = list.clear
                    iadd = list.__iadd__
                    clear(items)
                    iadd(items, [Parameter("expected", "query")])
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    replace(result)
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_helper_internal_helper_alias": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def replace_inner(items: list[Parameter]) -> None:
                    items.clear()
                    items += [Parameter("expected", "query")]
                def replace(items: list[Parameter]) -> None:
                    alias = replace_inner
                    alias(items)
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    replace(result)
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_callable_holder_expression": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    mutators = {"clear": list.clear, "iadd": list.__iadd__}
                    mutators["clear"](result)
                    mutators["iadd"](result, [Parameter("expected", "query")])
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_staticmethod_mutator": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                class Replacer:
                    @staticmethod
                    def replace(items: list[Parameter]) -> None:
                        items.clear()
                        items += [Parameter("expected", "query")]
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    Replacer.replace(result)
                    value = result[0].name
                    return {"value": value}
            """, False),
            "recorder_mixed_holder_unrelated_mutation_control": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    holder = {"log": witness.write_log, "scratch": []}
                    scratch = holder["scratch"]
                    scratch += [("/fake", "expected")]
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, True),
            "recorder_holder_unrelated_root_control": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    first = Recorder()
                    second = Recorder()
                    first_result = run(first)
                    second_result = run(second)
                    holder = {"first": first.write_log, "second": second.write_log}
                    first_alias = holder["first"]
                    first_alias.clear()
                    first_alias += [("/fake", "expected")]
                    value = tuple(second.write_log)
                    return {"first_result": first_result, "second_result": second_result, "value": value}
            """, True),
            "derived_destructured_alias": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    alias, = (result,)
                    alias.clear()
                    alias += [Parameter("expected", "query")]
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_holder_extracted_alias": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    holder = {"data": result}
                    alias = holder["data"]
                    alias.clear()
                    alias += [Parameter("expected", "query")]
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_nested_block_alias": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    if True:
                        alias = result
                        alias.clear()
                        alias += [Parameter("expected", "query")]
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_bound_alias_from_alias": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    alias = result
                    clear = alias.clear
                    extend = alias.extend
                    clear()
                    extend([Parameter("expected", "query")])
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_holder_rebound": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    holder = {"data": result}
                    holder_alias = holder
                    alias = holder_alias["data"]
                    alias.clear()
                    alias += [Parameter("expected", "query")]
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_holder_name_destructured": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    holder = (result,)
                    alias, = holder
                    alias.clear()
                    alias += [Parameter("expected", "query")]
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_for_target": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    for alias in (result,):
                        alias.clear()
                        alias += [Parameter("expected", "query")]
                    value = result[0].name
                    return {"value": value}
            """, False),
            "derived_walrus_alias": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    if (alias := result) is not None:
                        alias.clear()
                        alias += [Parameter("expected", "query")]
                    value = result[0].name
                    return {"value": value}
            """, False),
            "shadowed_constructor_name": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                class Replacer:
                    def __call__(self, items: object) -> None:
                        items.clear()
                        items += [Parameter("expected", "query")]
                list = Replacer()
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    list(result)
                    value = result[0].name
                    return {"value": value}
            """, False),
            "shadowed_type_name": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                class Replacer:
                    def __call__(self, items: object) -> None:
                        items.clear()
                        items += [Parameter("expected", "query")]
                type = Replacer()
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    type(result)
                    value = result[0].name
                    return {"value": value}
            """, False),
            "unknown_starred_argument": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                class Replacer:
                    def __call__(self, items: object) -> None:
                        items.clear()
                        items += [Parameter("expected", "query")]
                mutate = Replacer()
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    mutate(*[result])
                    value = result[0].name
                    return {"value": value}
            """, False),
            "helper_data_alias": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def replace(items: list[Parameter]) -> None:
                    alias = items
                    alias.clear()
                    alias += [Parameter("expected", "query")]
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    replace(result)
                    value = result[0].name
                    return {"value": value}
            """, False),
            "helper_transitive_callable_alias": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                def replace(items: list[Parameter]) -> None:
                    clear1 = list.clear
                    clear2 = clear1
                    extend1 = list.extend
                    extend2 = extend1
                    clear2(items)
                    extend2(items, [Parameter("expected", "query")])
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    replace(result)
                    value = result[0].name
                    return {"value": value}
            """, False),
            "unknown_callable_object": ("""
                from openapi_codegen.application.document import Item, Parameter
                from openapi_codegen.application.operations import inline_params
                class Replacer:
                    def __call__(self, items: list[Parameter]) -> None:
                        items.clear()
                        items += [Parameter("expected", "query")]
                mutate = Replacer()
                def verify_case() -> dict[str, object]:
                    result = inline_params((Item(Parameter("actual", "query")),))
                    mutate(result)
                    value = result[0].name
                    return {"value": value}
            """, False),
            "recorder_holder_via_instance_alias": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    recorder_alias = witness
                    holder = {"log": recorder_alias.write_log}
                    alias = holder["log"]
                    alias.clear()
                    alias += [("/fake", "expected")]
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "recorder_nested_destructured_alias": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    ((alias,),) = ((witness.write_log,),)
                    alias.clear()
                    alias += [("/fake", "expected")]
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "recorder_direct_unrelated_selector": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    holder = {"log": witness.write_log, "scratch": []}
                    holder["scratch"] += [("unrelated", "data")]
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, True),
            "recorder_direct_instance_alias": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    recorder_alias = witness
                    alias = recorder_alias.write_log
                    alias.clear()
                    alias += [("/fake", "expected")]
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "recorder_holder_rebound": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    holder = {"log": witness.write_log}
                    holder_alias = holder
                    alias = holder_alias["log"]
                    alias.clear()
                    alias += [("/fake", "expected")]
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "recorder_holder_name_destructured": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    holder = ((witness.write_log,),)
                    ((alias,),) = holder
                    alias.clear()
                    alias += [("/fake", "expected")]
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "recorder_instance_alias_snapshot": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    recorder_alias = witness
                    value_alias = tuple(recorder_alias.write_log)
                    value = value_alias
                    return {"result": result, "value": value}
            """, True),
            "recorder_unrelated_selector_method": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    holder = {"log": witness.write_log, "scratch": []}
                    holder["scratch"].__iadd__([("unrelated", "data")])
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, True),
            "metaclass_recorder": ("""
                from openapi_codegen.design import run
                class Meta(type):
                    pass
                class Recorder(metaclass=Meta):
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "log_mutating_reader": ("""
                from openapi_codegen.design import run
                class Recorder:
                    def __init__(self) -> None:
                        self.write_log = []
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                    def read_text(self, path: str) -> str:
                        self.write_log.clear()
                        return "{}"
                def verify_case() -> dict[str, object]:
                    witness = Recorder()
                    result = run(witness)
                    value = tuple(witness.write_log)
                    return {"result": result, "value": value}
            """, False),
            "non_none_fallback": ("""
                from openapi_codegen.design import observe
                def verify_case() -> dict[str, object]:
                    result = observe()
                    value = result.value if result is not None else "seed"
                    return {"value": value}
            """, False),
        }
        for label, (source, expected) in snippets.items():
            tree = ast.parse(textwrap.dedent(source))
            verifier = next(node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == "verify_case")
            _, names = extract_provenance_names(tree, verifier)
            self.assertEqual("value" in names, expected, label)

    def test_inventory_and_case_files(self) -> None:
        """Verify exact 12 case files exist with no orphan python files in src/."""
        self.assertEqual(len(DECLARED_CASES), 12)
        self.assertEqual(sum(DECLARED_CASES.values()), 188)

        expected_files = {f"{case_id}.py" for case_id in DECLARED_CASES} | {"runner.py"}
        actual_files = {p.name for p in SRC_DIR.glob("*.py")}

        missing = expected_files - actual_files
        self.assertFalse(
            missing,
            f"Missing required case files: {sorted(missing)}. Round B materialization required.",
        )

        orphans = actual_files - expected_files
        self.assertFalse(
            orphans,
            f"Orphan Python files found in src/: {sorted(orphans)}",
        )

    def _run_r24_full_gate_regressions(self) -> None:
        cases = {
            "derived_holder_rebound": False,
            "derived_holder_name_destructured": False,
            "derived_for_target": False,
            "derived_walrus_alias": False,
            "shadowed_constructor_name": False,
            "unknown_starred_argument": False,
            "recorder_direct_instance_alias": False,
            "recorder_holder_rebound": False,
            "recorder_holder_name_destructured": False,
            "recorder_instance_alias_snapshot": True,
            "recorder_unrelated_selector_method": True,
            "module_class_shadows_list": False,
            "shadowed_tuple_mutates_log": False,
            "mutated_log_alias_observed": False,
            "nested_literal_star_expansion": False,
            "destructured_for_target": False,
            "recorder_alias_holder_rebound": False,
            "unrelated_nested_builtin_binding": True,
            "unrelated_selector_unbound_method": True,
            "except_handler_shadows_list": False,
            "comprehension_binding_does_not_shadow_list": True,
            "nested_chained_literal_star_expansion": False,
            "two_slot_destructured_for_target": False,
            "recorder_instance_inside_holder": False,
            "derived_unrelated_unbound_selector": True,
            "unrelated_unbound_dict_selector": True,
            "nested_default_walrus_shadows_list": False,
            "starred_assignment_destructuring": False,
            "recorder_starred_destructuring": False,
            "derived_unrelated_bound_selector": True,
            "derived_unrelated_dict_setitem": True,
            "recorder_unrelated_dict_pop": True,
            "arbitrary_depth_recorder_holder_control": False,
            "derived_star_holder_alias_tracked": False,
            "recorder_star_holder_alias_tracked": False,
            "lambda_default_walrus_shadows_list": False,
            "async_default_comprehension_walrus": False,
            "class_decorator_comprehension_walrus": False,
            "async_default_dict_comprehension_walrus": False,
            "dict_comprehension_induction_positive": True,
            "derived_prefix_star_tracked": False,
            "derived_middle_star_tracked": False,
            "derived_suffix_star_tracked": False,
            "recorder_middle_star_tracked": False,
            "derived_star_disjoint_prefix": True,
            "derived_star_disjoint_suffix": True,
            "recorder_star_disjoint_prefix": True,
            "derived_bound_pop_disjoint": True,
            "derived_bound_setdefault_disjoint": True,
            "derived_bound_setitem_disjoint": True,
            "derived_bound_delitem_disjoint": True,
            "recorder_bound_pop_disjoint": True,
            "recorder_bound_setdefault_disjoint": True,
            "recorder_bound_setitem_disjoint": True,
            "recorder_bound_delitem_disjoint": True,
            "derived_bound_selector_matching": False,
            "derived_bound_selector_unknown": False,
            "recorder_bound_selector_matching": False,
            "recorder_bound_selector_unknown": False,
            "derived_known_suffix_mutated": False,
            "recorder_known_suffix_mutated": False,
            "derived_known_suffix_alias_mutated": False,
            "recorder_known_suffix_alias_mutated": False,
            "derived_unknown_length_tracked_star": False,
            "recorder_unknown_length_tracked_star": False,
            "derived_tracked_star_read_only": True,
            "recorder_tracked_star_read_only": True,
            "derived_disjoint_setitem_launders": False,
            "recorder_disjoint_setitem_launders": False,
            "derived_disjoint_unbound_setitem_launders": False,
            "recorder_disjoint_unbound_setitem_launders": False,
            "derived_nested_body_binding_isolated": True,
            "derived_comprehension_target_isolated": True,
            "derived_nested_body_mutation_isolated": True,
            "recorder_nested_body_mutation_isolated": True,
            "derived_nested_class_binding_isolated": True,
            "derived_class_body_walrus_isolated": True,
            "derived_lambda_body_binding_isolated": True,
            "derived_nested_function_body_walrus_isolated": True,
            "class_local_alias_mutates": False,
            "class_local_shadow_then_mutates": True,
            "class_nonlocal_rebinds": False,
            "comprehension_target_alias_mutates": False,
            "comprehension_same_name_shadow_then_mutates": True,
            "nested_uncalled_body_in_called_helper": True,
            "bound_setitem_read_only_capture": True,
            "unbound_setitem_read_only_capture": True,
            "recorder_instance_setitem_launders": False,
            "derived_direct_unknown_star_launders": False,
            "known_star_disjoint_suffix": True,
            "setdefault_tracked_value_launders": False,
        }
        required_r30 = {"lambda_default_walrus_shadows_list", "async_default_comprehension_walrus", "class_decorator_comprehension_walrus", "async_default_dict_comprehension_walrus", "dict_comprehension_induction_positive", "derived_prefix_star_tracked", "derived_middle_star_tracked", "derived_suffix_star_tracked", "recorder_middle_star_tracked", "derived_star_disjoint_prefix", "derived_star_disjoint_suffix", "recorder_star_disjoint_prefix", "derived_bound_pop_disjoint", "derived_bound_setdefault_disjoint", "derived_bound_setitem_disjoint", "derived_bound_delitem_disjoint", "recorder_bound_pop_disjoint", "recorder_bound_setdefault_disjoint", "recorder_bound_setitem_disjoint", "recorder_bound_delitem_disjoint", "derived_bound_selector_matching", "derived_bound_selector_unknown", "recorder_bound_selector_matching", "recorder_bound_selector_unknown", "derived_star_holder_alias_tracked", "recorder_star_holder_alias_tracked"}
        self.assertTrue(required_r30 <= set(cases))
        required_r31 = {"derived_known_suffix_mutated", "recorder_known_suffix_mutated", "derived_known_suffix_alias_mutated", "recorder_known_suffix_alias_mutated", "derived_unknown_length_tracked_star", "recorder_unknown_length_tracked_star", "derived_tracked_star_read_only", "recorder_tracked_star_read_only", "derived_disjoint_setitem_launders", "recorder_disjoint_setitem_launders", "derived_disjoint_unbound_setitem_launders", "recorder_disjoint_unbound_setitem_launders", "derived_nested_body_binding_isolated", "derived_comprehension_target_isolated", "derived_nested_body_mutation_isolated", "recorder_nested_body_mutation_isolated"}
        self.assertTrue(required_r31 <= set(cases))
        required_r32 = {"derived_nested_class_binding_isolated", "derived_class_body_walrus_isolated", "derived_lambda_body_binding_isolated", "derived_nested_function_body_walrus_isolated"}
        self.assertTrue(required_r32 <= set(cases))
        required_r33 = {"class_local_alias_mutates", "class_nonlocal_rebinds", "comprehension_target_alias_mutates", "nested_uncalled_body_in_called_helper", "bound_setitem_read_only_capture", "unbound_setitem_read_only_capture", "recorder_instance_setitem_launders", "derived_direct_unknown_star_launders", "known_star_disjoint_suffix", "setdefault_tracked_value_launders"}
        self.assertTrue(required_r33 <= set(cases))
        required_r34 = {"class_local_shadow_then_mutates", "comprehension_same_name_shadow_then_mutates"}
        self.assertTrue(required_r34 <= set(cases))

        def normalize(raw: str) -> str:
            return textwrap.dedent(raw).lstrip() + "\n"

        def indent_operation(operation: str) -> str:
            lines = operation.splitlines()
            return textwrap.indent("\n".join(line[4:] if line.startswith("    ") else line for line in lines), "    ")

        executed_labels: set[str] = set()
        def source_for(label: str) -> str:
            derived = label.startswith("derived_") or label in {"shadowed_constructor_name", "unknown_starred_argument", "module_class_shadows_list", "nested_literal_star_expansion", "destructured_for_target", "unrelated_nested_builtin_binding", "except_handler_shadows_list", "comprehension_binding_does_not_shadow_list", "nested_chained_literal_star_expansion", "two_slot_destructured_for_target", "derived_unrelated_unbound_selector", "nested_default_walrus_shadows_list", "lambda_default_walrus_shadows_list", "async_default_comprehension_walrus", "class_decorator_comprehension_walrus", "async_default_dict_comprehension_walrus", "dict_comprehension_induction_positive", "starred_assignment_destructuring", "derived_unrelated_bound_selector", "derived_unrelated_dict_setitem", "derived_star_holder_alias_tracked", "derived_known_suffix_mutated", "derived_known_suffix_alias_mutated", "derived_tracked_star_read_only", "derived_unknown_length_tracked_star", "derived_disjoint_setitem_launders", "derived_disjoint_unbound_setitem_launders", "derived_nested_body_binding_isolated", "derived_comprehension_target_isolated", "derived_nested_body_mutation_isolated", "nested_uncalled_body_in_called_helper", "bound_setitem_read_only_capture", "unbound_setitem_read_only_capture", "derived_direct_unknown_star_launders", "known_star_disjoint_suffix", "setdefault_tracked_value_launders", "class_local_alias_mutates", "class_local_shadow_then_mutates", "class_nonlocal_rebinds", "comprehension_target_alias_mutates", "comprehension_same_name_shadow_then_mutates"}
            if derived:
                operation = {
                    "derived_holder_rebound": 'holder = {"data": result}\n    holder_alias = holder\n    alias = holder_alias["data"]\n    alias.clear()\n    alias += [Parameter("expected", "query")]',
                    "derived_holder_name_destructured": "holder = (result,)\n    alias, = holder\n    alias.clear()\n    alias += [Parameter(\"expected\", \"query\")]",
                    "derived_for_target": "for alias in (result,):\n        alias.clear()\n        alias += [Parameter(\"expected\", \"query\")]",
                    "derived_walrus_alias": "if (alias := result) is not None:\n        alias.clear()\n        alias += [Parameter(\"expected\", \"query\")]",
                    "shadowed_constructor_name": "list = Replacer()\n    list(result)",
                    "unknown_starred_argument": "mutate = Replacer()\n    mutate(*[result])",
                    "module_class_shadows_list": "list(result)\n    alias = result\n    alias.clear()\n    alias += [Parameter(\"expected\", \"query\")]",
                    "nested_literal_star_expansion": "mutate = Replacer()\n    mutate(*((result,),)[0])",
                    "destructured_for_target": "for (alias,) in ((result,),):\n        alias.clear()\n        alias += [Parameter(\"expected\", \"query\")]",
                    "unrelated_nested_builtin_binding": "copied = list(result)",
                    "except_handler_shadows_list": "try:\n        raise Replacer()\n    except Replacer as list:\n        list(result)",
                    "comprehension_binding_does_not_shadow_list": "copied = list(result)\n    unrelated = [list for list in (0,)]",
                    "nested_chained_literal_star_expansion": "mutate = Replacer()\n    mutate(*(((result,),),)[0][0])",
                    "two_slot_destructured_for_target": "for ignored, alias in ((None, result),):\n        alias.clear()\n        alias += [Parameter(\"expected\", \"query\")]",
                    "derived_unrelated_unbound_selector": 'holder = {"data": result, "scratch": []}\n    list.__iadd__(holder["scratch"], [Parameter("unrelated", "query")])',
                    "nested_default_walrus_shadows_list": "def helper(unused: object = (list := Replacer())) -> None:\n        return None\n    list(result)",
                    "starred_assignment_destructuring": "holder = (result,)\n    (*aliases,) = holder\n    aliases[0].clear()\n    aliases[0] += [Parameter(\"expected\", \"query\")]",
                    "derived_unrelated_bound_selector": 'holder = {"data": result, "scratch": []}\n    holder["scratch"].__iadd__([Parameter("unrelated", "query")])',
                    "derived_unrelated_dict_setitem": 'holder = {"data": result, "scratch": []}\n    dict.__setitem__(holder, "scratch", [Parameter("unrelated", "query")])',
                    "derived_star_holder_alias_tracked": 'holder = ([], result, None)\n    holder_alias = holder\n    prefix, *aliases, suffix = holder_alias\n    aliases[0].clear()\n    aliases[0] += [Parameter("expected", "query")]',
                    "lambda_default_walrus_shadows_list": "helper = lambda unused=(list := Replacer()): None\n    list(result)",
                    "async_default_comprehension_walrus": "async def helper(unused: object = [(list := Replacer()) for _ in (0,)]) -> None:\n        return None\n    list(result)",
                    "class_decorator_comprehension_walrus": "@[(list := Replacer()) for _ in (0,)][0]\n    class Helper:\n        pass\n    list(result)",
                    "async_default_dict_comprehension_walrus": "async def helper(unused: object = {key: (list := Replacer()) for key in (0,)}) -> None:\n        return None\n    list(result)",
                    "dict_comprehension_induction_positive": "copied = list(result)\n    unrelated = {list: list for list in (0,)}",
                    "derived_prefix_star_tracked": 'holder = ([], result, None)\n    prefix, *aliases, suffix = holder\n    aliases[0].clear()\n    aliases[0] += [Parameter("expected", "query")]',
                    "derived_middle_star_tracked": 'holder = (None, [], result, None)\n    prefix, *aliases, suffix = holder\n    aliases[1].clear()\n    aliases[1] += [Parameter("expected", "query")]',
                    "derived_suffix_star_tracked": 'holder = (None, [], result)\n    prefix, *aliases = holder\n    aliases[1].clear()\n    aliases[1] += [Parameter("expected", "query")]',
                    "derived_star_disjoint_prefix": 'holder = (result, [], None)\n    prefix, *aliases, suffix = holder\n    aliases[0].clear()',
                    "derived_star_disjoint_suffix": 'holder = (None, [], result)\n    prefix, *aliases, suffix = holder\n    aliases[0].clear()',
                    "derived_bound_pop_disjoint": 'holder = {"data": result, "scratch": []}\n    holder.pop("scratch")',
                    "derived_bound_setdefault_disjoint": 'holder = {"data": result, "scratch": []}\n    holder.setdefault("other", [])',
                    "derived_bound_setitem_disjoint": 'holder = {"data": result, "scratch": []}\n    holder.__setitem__("scratch", [Parameter("unrelated", "query")])',
                    "derived_bound_delitem_disjoint": 'holder = {"data": result, "scratch": []}\n    holder.__delitem__("scratch")',
                    "derived_bound_selector_matching": 'holder = {"data": result, "scratch": []}\n    holder.pop("data")',
                    "derived_bound_selector_unknown": 'holder = {"data": result, "scratch": []}\n    key = "scratch"\n    holder.pop(key)',
                    "derived_known_suffix_mutated": 'holder = (None, [], None, result)\n    prefix, *aliases, suffix = holder\n    suffix.clear()\n    suffix += [Parameter("expected", "query")]',
                    "derived_known_suffix_alias_mutated": 'holder = (None, [], None, result)\n    holder_alias = holder\n    prefix, *aliases, suffix = holder_alias\n    suffix.clear()\n    suffix += [Parameter("expected", "query")]',
                    "derived_tracked_star_read_only": 'holder = (None, result, None)\n    prefix, *aliases, suffix = holder\n    copied = list(aliases[0])',
                    "derived_unknown_length_tracked_star": 'holder = tuple((result,))\n    (*aliases,) = holder\n    aliases[0].clear()\n    aliases[0] += [Parameter("expected", "query")]',
                    "derived_disjoint_setitem_launders": 'holder = {"data": result, "scratch": []}\n    holder.__setitem__("scratch", result)\n    holder["scratch"].clear()\n    holder["scratch"].__iadd__([Parameter("expected", "query")])',
                    "derived_disjoint_unbound_setitem_launders": 'holder = {"data": result, "scratch": []}\n    dict.__setitem__(holder, "scratch", result)\n    holder["scratch"].clear()\n    holder["scratch"].__iadd__([Parameter("expected", "query")])',
                    "derived_nested_body_binding_isolated": 'def unused() -> None:\n        result = []\n    copied = list(result)',
                    "derived_comprehension_target_isolated": 'unrelated = [result for result in (0,)]\n    copied = list(result)',
                    "derived_nested_body_mutation_isolated": 'def unused() -> None:\n        result.clear()\n    copied = list(result)',
                    "derived_nested_class_binding_isolated": 'class Helper:\n        result = []\n    copied = list(result)',
                    "derived_class_body_walrus_isolated": 'class Helper:\n        (result := [])\n    copied = list(result)',
                    "derived_lambda_body_binding_isolated": 'helper = lambda: (result := [])\n    copied = list(result)',
                    "derived_nested_function_body_walrus_isolated": 'def unused() -> None:\n        (result := [])\n    copied = list(result)',
                    "class_local_alias_mutates": 'class Trigger:\n        alias = result\n        alias.clear()\n        alias += [Parameter("expected", "query")]',
                    "class_local_shadow_then_mutates": 'class Trigger:\n        result = []\n        result.clear()\n        result += [Parameter("expected", "query")]\ncopied = list(result)',
                    "class_nonlocal_rebinds": 'class Trigger:\n        nonlocal result\n        result = [Parameter("expected", "query")]',
                    "comprehension_target_alias_mutates": 'effects = [(alias.clear(), alias.__iadd__([Parameter("expected", "query")])) for alias in (result,)]',
                    "comprehension_same_name_shadow_then_mutates": 'effects = [result.clear() for result in ([],)]\ncopied = list(result)',
                    "nested_uncalled_body_in_called_helper": 'def called(items: list[Parameter]) -> None:\n        def unused() -> None:\n            items.clear()\n        return None\n    called(result)\n    copied = list(result)',
                    "bound_setitem_read_only_capture": 'holder = {"data": result, "scratch": []}\n    holder.__setitem__("scratch", result)\n    copied = list(result)',
                    "unbound_setitem_read_only_capture": 'holder = {"data": result, "scratch": []}\n    dict.__setitem__(holder, "scratch", result)\n    copied = list(result)',
                    "derived_direct_unknown_star_launders": '(*aliases,) = result\n    aliases[0].schema.clear()\n    aliases[0].schema.__iadd__(["expected"])',
                    "known_star_disjoint_suffix": 'holder = (None, [], result, [])\n    prefix, *aliases, suffix = holder\n    suffix.clear()\n    copied = list(result)',
                    "setdefault_tracked_value_launders": 'holder = {"data": result}\n    holder.setdefault("scratch", result)\n    holder["scratch"].clear()\n    holder["scratch"].__iadd__([Parameter("expected", "query")])',
                }[label]
                value_line = "value = result[0].name"
                expected_value = "actual" if label in {"comprehension_binding_does_not_shadow_list", "derived_unrelated_unbound_selector", "unrelated_nested_builtin_binding", "derived_unrelated_bound_selector", "derived_unrelated_dict_setitem", "dict_comprehension_induction_positive", "derived_star_disjoint_prefix", "derived_star_disjoint_suffix", "derived_bound_pop_disjoint", "derived_bound_setdefault_disjoint", "derived_bound_setitem_disjoint", "derived_bound_delitem_disjoint", "derived_bound_selector_matching", "derived_bound_selector_unknown", "derived_tracked_star_read_only", "derived_nested_body_binding_isolated", "derived_comprehension_target_isolated", "derived_nested_body_mutation_isolated", "derived_nested_class_binding_isolated", "derived_class_body_walrus_isolated", "derived_lambda_body_binding_isolated", "derived_nested_function_body_walrus_isolated", "nested_uncalled_body_in_called_helper", "bound_setitem_read_only_capture", "unbound_setitem_read_only_capture", "known_star_disjoint_suffix", "class_local_shadow_then_mutates", "comprehension_same_name_shadow_then_mutates"} else "expected"
                prefix = ""
                if label in {"shadowed_constructor_name", "unknown_starred_argument", "nested_literal_star_expansion", "nested_chained_literal_star_expansion"}:
                    prefix = "\nclass Replacer:\n    def __call__(self, items: list[Parameter]) -> None:\n        items.clear()\n        items += [Parameter(\"expected\", \"query\")]\n"
                elif label == "module_class_shadows_list":
                    prefix = "\nclass list:\n    def __init__(self, items: list[Parameter]) -> None:\n        items.clear()\n        items += [Parameter(\"expected\", \"query\")]\n"
                elif label == "destructured_for_target":
                    prefix = ""
                elif label == "unrelated_nested_builtin_binding":
                    prefix = "\nclass Helper:\n    def note(self) -> None:\n        list = 0\n"
                elif label in {"except_handler_shadows_list"}:
                    prefix = "\nclass Replacer(Exception):\n    def __call__(self, items: list[Parameter]) -> None:\n        items.clear()\n        items += [Parameter(\"expected\", \"query\")]\n"
                elif label in {"nested_default_walrus_shadows_list", "lambda_default_walrus_shadows_list", "async_default_comprehension_walrus", "async_default_dict_comprehension_walrus"}:
                    prefix = "\nclass Replacer:\n    def __call__(self, items: list[Parameter]) -> None:\n        items.clear()\n        items += [Parameter(\"expected\", \"query\")]\n"
                elif label == "class_decorator_comprehension_walrus":
                    prefix = "\nclass Replacer:\n    def __call__(self, item: object) -> object:\n        if isinstance(item, type):\n            return item\n        item.clear()\n        item += [Parameter(\"expected\", \"query\")]\n        return item\n"
                rendered = normalize(f'''\
                    from __future__ import annotations
                    from openapi_codegen.application.document import Item, Parameter
                    from openapi_codegen.application.operations import inline_params
                    R24_REGRESSION_MATRIX = [("launder", "{expected_value}")]
                    MINIMUM_CHECKS = 1
                    __PREFIX__
                    def verify_r24_regression() -> dict[str, object]:
                        checks = []
                        result = inline_params((Item(Parameter("actual", "query")),))
                        __OPERATION__
                        {value_line}
                        checks.append({{"name": "launder", "observed": value, "expected": "{expected_value}", "passed": value == "{expected_value}"}})
                        return {{"checks": checks}}
                ''')
                rendered = rendered.replace("    __PREFIX__", prefix.lstrip("\n")).replace("__PREFIX__", prefix.lstrip("\n"))
                if label == "derived_direct_unknown_star_launders":
                    rendered = rendered.replace(
                        'result = inline_params((Item(Parameter("actual", "query")),))',
                        'result = inline_params((Item(Parameter("actual", "query", schema=["actual"])),))',
                        1,
                    ).replace("value = result[0].name", "value = result[0].schema[0]", 1)
                return rendered.replace("    __OPERATION__", indent_operation(operation)).replace("__OPERATION__", indent_operation(operation))
            operation = {
                "recorder_direct_instance_alias": "recorder_alias = witness\n    alias = recorder_alias.write_log\n    alias.clear()\n    alias += [(\"/fake\", \"expected\")]",
                "recorder_holder_rebound": 'holder = {"log": witness.write_log}\n    holder_alias = holder\n    alias = holder_alias["log"]\n    alias.clear()\n    alias += [("/fake", "expected")]',
                "recorder_holder_name_destructured": "holder = ((witness.write_log,),)\n    ((alias,),) = holder\n    alias.clear()\n    alias += [(\"/fake\", \"expected\")]",
                "recorder_instance_alias_snapshot": "recorder_alias = witness\n    value_alias = tuple(recorder_alias.write_log)\n    value = value_alias",
                "recorder_unrelated_selector_method": 'holder = {"log": witness.write_log, "scratch": []}\n    holder["scratch"].__iadd__([("unrelated", "data")])',
                "shadowed_tuple_mutates_log": "alias = witness.write_log\n    value = tuple(alias)",
                "mutated_log_alias_observed": "alias = witness.write_log\n    alias.clear()\n    alias += [(\"/fake\", \"expected\")]\n    value = tuple(alias)",
                    "recorder_alias_holder_rebound": 'recorder_alias = witness\n    holder = {"log": recorder_alias.write_log}\n    holder_alias = holder\n    alias = holder_alias["log"]\n    alias.clear()\n    alias += [("/fake", "expected")]',
                "unrelated_selector_unbound_method": 'holder = {"log": witness.write_log, "scratch": []}\n    list.__iadd__(holder["scratch"], [("unrelated", "data")])',
                "unrelated_nested_builtin_binding": "copied = list(result)",
                "recorder_instance_inside_holder": 'recorder_alias = witness\n    holder = {"recorder": recorder_alias}\n    extracted = holder["recorder"]\n    alias = extracted.write_log\n    alias.clear()\n    alias += [("/fake", "expected")]',
                "unrelated_unbound_dict_selector": 'holder = {"log": witness.write_log, "scratch": []}\n    dict.__setitem__(holder, "scratch", [("unrelated", "data")])',
                "recorder_starred_destructuring": "holder = (witness.write_log,)\n    (*aliases,) = holder\n    aliases[0].clear()\n    aliases[0] += [(\"/fake\", \"expected\")]",
                "recorder_unrelated_dict_pop": 'holder = {"log": witness.write_log, "scratch": []}\n    dict.pop(holder, "scratch")',
                "arbitrary_depth_recorder_holder_control": 'holder = {"outer": (((witness,),),)}\n    extracted = holder["outer"][0][0][0]\n    alias = extracted.write_log\n    alias.clear()\n    alias += [("/fake", "expected")]',
                "recorder_star_holder_alias_tracked": 'holder = (None, [], witness.write_log, None)\n    holder_alias = holder\n    prefix, *aliases, suffix = holder_alias\n    aliases[1].clear()\n    aliases[1] += [("/fake", "expected")]',
                "recorder_middle_star_tracked": 'holder = (None, [], witness.write_log, None)\n    prefix, *aliases, suffix = holder\n    aliases[1].clear()\n    aliases[1] += [("/fake", "expected")]',
                "recorder_star_disjoint_prefix": 'holder = (witness.write_log, [], None)\n    prefix, *aliases, suffix = holder\n    aliases[0].clear()',
                "recorder_bound_pop_disjoint": 'holder = {"log": witness.write_log, "scratch": []}\n    holder.pop("scratch")',
                "recorder_bound_setdefault_disjoint": 'holder = {"log": witness.write_log, "scratch": []}\n    holder.setdefault("other", [])',
                "recorder_bound_setitem_disjoint": 'holder = {"log": witness.write_log, "scratch": []}\n    holder.__setitem__("scratch", [("unrelated", "data")])',
                "recorder_bound_delitem_disjoint": 'holder = {"log": witness.write_log, "scratch": []}\n    holder.__delitem__("scratch")',
                "recorder_bound_selector_matching": 'holder = {"log": witness.write_log, "scratch": []}\n    holder.pop("log")',
                "recorder_bound_selector_unknown": 'holder = {"log": witness.write_log, "scratch": []}\n    key = "scratch"\n    holder.pop(key)',
                "recorder_known_suffix_mutated": 'holder = (None, [], None, witness.write_log)\n    prefix, *aliases, suffix = holder\n    suffix.clear()\n    suffix += [("/fake", "expected")]',
                "recorder_known_suffix_alias_mutated": 'holder = (None, [], None, witness.write_log)\n    holder_alias = holder\n    prefix, *aliases, suffix = holder_alias\n    suffix.clear()\n    suffix += [("/fake", "expected")]',
                "recorder_tracked_star_read_only": 'holder = (None, witness.write_log, None)\n    prefix, *aliases, suffix = holder\n    copied = tuple(aliases[0])',
                "recorder_unknown_length_tracked_star": 'source = (witness.write_log,)\n    holder = tuple(source)\n    (*aliases,) = holder\n    aliases[0].clear()\n    aliases[0] += [("/fake", "expected")]',
                "recorder_disjoint_setitem_launders": 'holder = {"log": witness.write_log, "scratch": []}\n    holder.__setitem__("scratch", witness.write_log)\n    holder["scratch"].clear()\n    holder["scratch"].__iadd__([("/fake", "expected")])',
                "recorder_disjoint_unbound_setitem_launders": 'holder = {"log": witness.write_log, "scratch": []}\n    dict.__setitem__(holder, "scratch", witness.write_log)\n    holder["scratch"].clear()\n    holder["scratch"].__iadd__([("/fake", "expected")])',
                "recorder_nested_body_mutation_isolated": 'def unused() -> None:\n        witness.write_log.clear()\n    copied = tuple(witness.write_log)',
                "recorder_instance_setitem_launders": 'holder = {"log": witness.write_log, "scratch": None}\n    holder.__setitem__("scratch", witness)\n    holder["scratch"].write_log.clear()\n    holder["scratch"].write_log.__iadd__([("/fake", "expected")])',
            }[label]
            value_line = "" if label == "recorder_instance_alias_snapshot" else "value = tuple(witness.write_log)"
            expected_value = '(("/out/actual", "actual"),)' if label in {"recorder_instance_alias_snapshot", "recorder_unrelated_selector_method", "unrelated_selector_unbound_method", "unrelated_unbound_dict_selector", "recorder_unrelated_dict_pop", "recorder_star_disjoint_prefix", "recorder_bound_pop_disjoint", "recorder_bound_setdefault_disjoint", "recorder_bound_setitem_disjoint", "recorder_bound_delitem_disjoint", "recorder_bound_selector_matching", "recorder_bound_selector_unknown", "recorder_tracked_star_read_only", "recorder_nested_body_mutation_isolated"} else '(("/fake", "expected"),)'
            recorder_prefix = ""
            if label == "shadowed_tuple_mutates_log":
                recorder_prefix = "class Replacer:\n    def __call__(self, items: list[tuple[str, str]]) -> tuple[tuple[str, str], ...]:\n        items.clear()\n        items += [(\"/fake\", \"expected\")]\n        return ((\"/fake\", \"expected\"),)\n\n"
                recorder_prefix += "tuple = Replacer()\n"
            rendered = normalize(f'''\
                from __future__ import annotations
                from openapi_codegen.domain.lang import Lang
                from openapi_codegen.infrastructure.options import GenOptions, GeneratedFile, HttpClient, legacy
                from openapi_codegen.infrastructure.runner import run
                R24_REGRESSION_MATRIX = [("observe", {expected_value})]
                MINIMUM_CHECKS = 1
                __RECORDER_PREFIX__class Recorder:
                    def __init__(self) -> None:
                        self.write_log: list[tuple[str, str]] = []
                    def read_text(self, path: str) -> str | None:
                        return "{{}}"
                    def write_text(self, path: str, contents: str) -> None:
                        self.write_log += [(path, contents)]
                def verify_r24_regression() -> dict[str, object]:
                    checks = []
                    opts = GenOptions(Lang.TS, None, "/spec", "/out", "C", HttpClient.FETCH, True, True, True)
                    generated = legacy((GeneratedFile("actual", "actual"),))
                    witness = Recorder()
                    result = run(opts, witness, lambda spec, options: generated)
                    __OPERATION__
                    {value_line}
                    checks.append({{"name": "observe", "observed": value, "expected": {expected_value}, "passed": value == {expected_value}}})
                    return {{"checks": checks}}
            ''')
            rendered = rendered.replace("__RECORDER_PREFIX__", recorder_prefix)
            return rendered.replace("    __OPERATION__", indent_operation(operation)).replace("__OPERATION__", indent_operation(operation))

        global SRC_DIR, DECLARED_CASES
        original_src_dir, original_declared = SRC_DIR, DECLARED_CASES
        active_present = hasattr(self, "_r24_harness_active")
        original_active = getattr(self, "_r24_harness_active", None)
        self._r24_harness_active = True
        try:
            for label, expected_pass in cases.items():
                self.assertNotIn(label, executed_labels)
                executed_labels.add(label)
                with tempfile.TemporaryDirectory() as temp_dir:
                    SRC_DIR = Path(temp_dir)
                    DECLARED_CASES = {"r24-regression": 1}
                    fixture_source = source_for(label)
                    self.assertNotIn("__PREFIX__", fixture_source)
                    self.assertNotIn("__OPERATION__", fixture_source)
                    self.assertNotIn("__RECORDER_PREFIX__", fixture_source)
                    parsed_fixture = ast.parse(fixture_source)
                    if label in {"derived_prefix_star_tracked", "derived_middle_star_tracked", "derived_suffix_star_tracked", "recorder_middle_star_tracked", "derived_star_disjoint_prefix", "derived_star_disjoint_suffix", "recorder_star_disjoint_prefix", "derived_star_holder_alias_tracked", "recorder_star_holder_alias_tracked", "starred_assignment_destructuring"}:
                        self.assertIn("*aliases", fixture_source, label)
                    bound_snippets = {
                        "derived_bound_pop_disjoint": '.pop("scratch")', "derived_bound_setdefault_disjoint": '.setdefault("other", [])', "derived_bound_setitem_disjoint": '.__setitem__("scratch"', "derived_bound_delitem_disjoint": '.__delitem__("scratch")',
                        "derived_bound_selector_matching": '.pop("data")', "derived_bound_selector_unknown": '.pop(key)',
                        "recorder_bound_pop_disjoint": '.pop("scratch")', "recorder_bound_setdefault_disjoint": '.setdefault("other", [])', "recorder_bound_setitem_disjoint": '.__setitem__("scratch"', "recorder_bound_delitem_disjoint": '.__delitem__("scratch")',
                        "recorder_bound_selector_matching": '.pop("log")', "recorder_bound_selector_unknown": '.pop(key)',
                    }
                    if label in bound_snippets:
                        self.assertIn(bound_snippets[label], fixture_source, label)
                    if label.endswith("_selector_unknown"):
                        self.assertIn("key = \"scratch\"", fixture_source, label)
                    if label.endswith("_selector_matching"):
                        self.assertNotIn("key =", fixture_source, label)
                    if label == "lambda_default_walrus_shadows_list":
                        self.assertTrue(any(isinstance(node, ast.Lambda) and node.args.defaults for node in ast.walk(parsed_fixture)), label)
                    if "dict_comprehension" in label:
                        self.assertTrue(any(isinstance(node, ast.DictComp) for node in ast.walk(parsed_fixture)), label)
                    if label == "class_decorator_comprehension_walrus":
                        self.assertTrue(any(isinstance(node, ast.ClassDef) and node.decorator_list for node in ast.walk(parsed_fixture)), label)
                    fixture_path = SRC_DIR / "r24-regression.py"
                    fixture_path.write_text(fixture_source, encoding="utf-8")
                    (SRC_DIR / "runner.py").write_text("from __future__ import annotations\n", encoding="utf-8")
                    sys.path.insert(0, str(EC_ROOT.parent / "tech-design" / "src"))
                    try:
                        module_name = "r26_regression_" + label
                        previous_module = sys.modules.get(module_name)
                        spec = importlib.util.spec_from_file_location(module_name, fixture_path)
                        self.assertIsNotNone(spec)
                        self.assertIsNotNone(spec.loader)
                        module = importlib.util.module_from_spec(spec)
                        sys.modules[module_name] = module
                        spec.loader.exec_module(module)  # type: ignore[union-attr]
                        runtime = module.verify_r24_regression()
                        self.assertTrue(runtime["checks"][0]["passed"], label)
                    finally:
                        if previous_module is None:
                            sys.modules.pop(module_name, None)
                        else:
                            sys.modules[module_name] = previous_module
                        sys.path.remove(str(EC_ROOT.parent / "tech-design" / "src"))
                    if expected_pass:
                        self.test_case_modules_static_properties()
                    else:
                        caught: AssertionError | None = None
                        try:
                            self.test_case_modules_static_properties()
                        except AssertionError as err:
                            caught = err
                            pass
                        else:
                            self.fail(f"R24 negative unexpectedly passed full gate: {label}")
                        self.assertIn("no direct-call design provenance", str(caught), label)
        finally:
            SRC_DIR, DECLARED_CASES = original_src_dir, original_declared
            if active_present:
                self._r24_harness_active = original_active
            else:
                del self._r24_harness_active
        self.assertEqual(executed_labels, set(cases))

    def test_case_modules_static_properties(self) -> None:
        """Statically inspect each case module for independence rules."""
        for case_id, min_checks in DECLARED_CASES.items():
            case_file = SRC_DIR / f"{case_id}.py"
            self.assertTrue(case_file.is_file(), f"Missing case file: {case_file}")

            content = case_file.read_text(encoding="utf-8")
            tree = ast.parse(content, filename=str(case_file))

            parent_map: dict[ast.AST, ast.AST] = {}
            for parent in ast.walk(tree):
                for child in ast.iter_child_nodes(parent):
                    parent_map[child] = parent

            annotation_nodes = extract_annotation_nodes(tree)

            # 1. First import statement must be `from __future__ import annotations`
            future_found = False
            for stmt in tree.body:
                if isinstance(stmt, ast.Expr) and isinstance(stmt.value, (ast.Str, ast.Constant)):
                    continue  # Skip docstrings
                if isinstance(stmt, (ast.Import, ast.ImportFrom)):
                    if (
                        isinstance(stmt, ast.ImportFrom)
                        and stmt.module == "__future__"
                        and any(alias.name == "annotations" for alias in stmt.names)
                    ):
                        future_found = True
                        break
                    else:
                        self.fail(
                            f"First import in {case_id} is not 'from __future__ import annotations'"
                        )

            self.assertTrue(future_found, f"{case_id} missing 'from __future__ import annotations'")

            # 2. Imports restriction: only __future__ and openapi_codegen
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    for alias in node.names:
                        top = alias.name.split(".")[0]
                        self.assertIn(
                            top,
                            ALLOWED_IMPORT_PREFIXES,
                            f"Disallowed import '{alias.name}' in {case_id}",
                        )
                elif isinstance(node, ast.ImportFrom):
                    if node.module:
                        top = node.module.split(".")[0]
                        self.assertIn(
                            top,
                            ALLOWED_IMPORT_PREFIXES,
                            f"Disallowed import from '{node.module}' in {case_id}",
                        )

            # Extract design import aliases
            top_level_verifiers: list[ast.FunctionDef] = []
            for stmt in tree.body:
                if isinstance(stmt, (ast.FunctionDef, ast.AsyncFunctionDef)) and stmt.name.startswith("verify_"):
                    top_level_verifiers.append(stmt)  # type: ignore

            self.assertEqual(
                len(top_level_verifiers),
                1,
                f"Expected exactly 1 top-level verify_ entrypoint in {case_id}, found {len(top_level_verifiers)}",
            )
            entrypoint_node = top_level_verifiers[0]

            try:
                canonical_design_aliases, design_derived_vars = extract_provenance_names(tree, entrypoint_node)
            except ValueError as err:
                self.fail(f"Canonical design import error in {case_id}: {err}")

            # 3. Central Exhaustive Immutability Check of imported design aliases using get_binding_events
            all_module_events = get_binding_events(tree)
            design_events_by_name: dict[str, list[tuple[ast.AST, str]]] = {
                name: [] for name in canonical_design_aliases
            }
            for bound_name, node, kind in all_module_events:
                if bound_name in design_events_by_name:
                    design_events_by_name[bound_name].append((node, kind))

            for bound_name, canonical_alias_node in canonical_design_aliases.items():
                ev_list = design_events_by_name[bound_name]
                self.assertEqual(
                    len(ev_list),
                    1,
                    f"Design alias '{bound_name}' in {case_id} has {len(ev_list)} binding events, expected exactly 1 (canonical alias)",
                )
                self.assertIs(
                    ev_list[0][0],
                    canonical_alias_node,
                    f"Design alias '{bound_name}' in {case_id} binding event is not the canonical module-level alias",
                )

            # Reject Attribute Store/Del rooted at any design package/module alias (prevent monkeypatching)
            for node in ast.walk(tree):
                if isinstance(node, ast.Attribute) and isinstance(node.ctx, (ast.Store, ast.Del)):
                    curr: ast.AST | None = node
                    while isinstance(curr, ast.Attribute):
                        curr = curr.value
                    if isinstance(curr, ast.Name) and curr.id in canonical_design_aliases:
                        self.fail(
                            f"Attribute mutation/deletion rooted at design import alias '{curr.id}' in {case_id}"
                        )

            # 4. Full function/method type annotations & exact TOP-LEVEL verify entrypoint
            underscored_name = case_id.replace("-", "_")
            expected_entrypoint = f"verify_{underscored_name}"
            self.assertEqual(
                entrypoint_node.name,
                expected_entrypoint,
                f"Entrypoint name mismatch: expected {expected_entrypoint}, got {entrypoint_node.name}",
            )

            # All functions in module must have type annotations
            for node in ast.walk(tree):
                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    self.assertIsNotNone(
                        node.returns,
                        f"Function {node.name} in {case_id} missing return type annotation",
                    )
                    all_args = (
                        node.args.args
                        + getattr(node.args, "posonlyargs", [])
                        + getattr(node.args, "kwonlyargs", [])
                    )
                    if node.args.vararg:
                        all_args.append(node.args.vararg)
                    if node.args.kwarg:
                        all_args.append(node.args.kwarg)

                    for arg in all_args:
                        if arg.arg in ("self", "cls"):
                            continue
                        self.assertIsNotNone(
                            arg.annotation,
                            f"Argument '{arg.arg}' in function {node.name} of {case_id} missing type annotation",
                        )

            # Exact zero-arity check on top-level verifier
            self.assertEqual(
                len(entrypoint_node.args.args),
                0,
                f"Entrypoint {expected_entrypoint} in {case_id} must have 0 positional args",
            )
            self.assertEqual(
                len(getattr(entrypoint_node.args, "posonlyargs", [])),
                0,
                f"Entrypoint {expected_entrypoint} in {case_id} must have 0 posonly args",
            )
            self.assertEqual(
                len(getattr(entrypoint_node.args, "kwonlyargs", [])),
                0,
                f"Entrypoint {expected_entrypoint} in {case_id} must have 0 kwonly args",
            )
            self.assertIsNone(
                entrypoint_node.args.vararg,
                f"Entrypoint {expected_entrypoint} in {case_id} must not accept *args",
            )
            self.assertIsNone(
                entrypoint_node.args.kwarg,
                f"Entrypoint {expected_entrypoint} in {case_id} must not accept **kwargs",
            )

            # Exact AST return annotation dict[str, object]
            self.assertTrue(
                is_dict_str_object_annotation(entrypoint_node.returns),
                f"Entrypoint {expected_entrypoint} in {case_id} return annotation must be dict[str, object]",
            )

            # 5. Top-level MINIMUM_CHECKS and *_MATRIX definitions
            min_checks_assigns: list[ast.Assign] = []
            matrix_nodes: list[ast.Assign] = []

            for stmt in tree.body:
                if isinstance(stmt, ast.Assign):
                    for target in stmt.targets:
                        if isinstance(target, ast.Name):
                            if target.id == "MINIMUM_CHECKS":
                                min_checks_assigns.append(stmt)
                            elif target.id.endswith("_MATRIX"):
                                matrix_nodes.append(stmt)

            self.assertEqual(
                len(min_checks_assigns),
                1,
                f"Expected exactly 1 top-level MINIMUM_CHECKS assignment in {case_id}, found {len(min_checks_assigns)}",
            )
            min_checks_expr = min_checks_assigns[0].value
            self.assertTrue(
                isinstance(min_checks_expr, (ast.Constant, ast.Num)) and type(getattr(min_checks_expr, "value", getattr(min_checks_expr, "n", None))) is int,
                f"MINIMUM_CHECKS in {case_id} must be a literal integer",
            )
            min_checks_val = getattr(min_checks_expr, "value", getattr(min_checks_expr, "n", None))
            self.assertEqual(
                min_checks_val,
                min_checks,
                f"MINIMUM_CHECKS in {case_id} must equal {min_checks}, got {min_checks_val}",
            )

            self.assertEqual(
                len(matrix_nodes),
                1,
                f"Expected exactly one top-level *_MATRIX definition in {case_id}, found {len(matrix_nodes)}",
            )

            matrix_expr = matrix_nodes[0].value
            self.assertIsInstance(
                matrix_expr,
                (ast.List, ast.Tuple),
                f"*_MATRIX in {case_id} must be a literal list or tuple",
            )
            matrix_elts = matrix_expr.elts  # type: ignore
            self.assertEqual(
                len(matrix_elts),
                min_checks,
                f"*_MATRIX length in {case_id} must equal floor {min_checks}, got {len(matrix_elts)}",
            )

            # Check distinct literal non-empty string row names & literal expected answers
            matrix_row_map: dict[str, ast.AST] = {}
            for row_idx, row_node in enumerate(matrix_elts):
                self.assertIsInstance(
                    row_node,
                    (ast.Tuple, ast.List),
                    f"Matrix row {row_idx} in {case_id} must be a tuple/list (name, expected)",
                )
                row_items = row_node.elts  # type: ignore
                self.assertEqual(
                    len(row_items),
                    2,
                    f"Matrix row {row_idx} in {case_id} must have length 2 (name, expected)",
                )

                name_node, expected_node = row_items[0], row_items[1]
                self.assertTrue(
                    is_literal_node(name_node),
                    f"Matrix row {row_idx} name in {case_id} must be a literal string",
                )
                row_name = (
                    name_node.value
                    if isinstance(name_node, ast.Constant)
                    else getattr(name_node, "s", None)
                )
                self.assertTrue(
                    type(row_name) is str and len(row_name.strip()) > 0,
                    f"Matrix row {row_idx} name in {case_id} must be a non-empty string",
                )
                self.assertNotIn(
                    row_name,
                    matrix_row_map,
                    f"Duplicate matrix row name {row_name!r} in {case_id}",
                )
                self.assertTrue(
                    is_literal_node(expected_node),
                    f"Matrix row {row_idx} ({row_name}) expected value in {case_id} must be a recursive literal expression",
                )
                matrix_row_map[row_name] = expected_node

            # 6. Exhaustive Un-aliasable 'checks' Name Occurrences Classification
            all_checks_name_nodes: list[ast.Name] = []
            for subnode in ast.walk(tree):
                if isinstance(subnode, ast.Name) and subnode.id == "checks":
                    all_checks_name_nodes.append(subnode)

            self.assertEqual(
                len(all_checks_name_nodes),
                min_checks + 2,
                f"Module {case_id} contains invalid or extra occurrences of Name 'checks'. Total allowed is {min_checks + 2}, got {len(all_checks_name_nodes)}",
            )

            verifier_checks_assigns: list[ast.Assign] = []
            for stmt in entrypoint_node.body:
                if isinstance(stmt, ast.Assign):
                    for target in stmt.targets:
                        if isinstance(target, ast.Name) and target.id == "checks":
                            verifier_checks_assigns.append(stmt)
                elif isinstance(stmt, (ast.Global, ast.Nonlocal)):
                    if "checks" in stmt.names:
                        self.fail(f"Global/nonlocal 'checks' statement in verifier body of {case_id}")

            self.assertEqual(
                len(verifier_checks_assigns),
                1,
                f"Verifier function in {case_id} must contain exactly 1 direct 'checks = []' initialization statement",
            )
            init_val = verifier_checks_assigns[0].value
            self.assertTrue(
                isinstance(init_val, ast.List) and len(init_val.elts) == 0,
                f"Verifier 'checks' initialization in {case_id} must be an empty list literal []",
            )
            store_checks_node = verifier_checks_assigns[0].targets[0]

            verifier_returns: list[ast.Return] = [
                stmt for stmt in entrypoint_node.body if isinstance(stmt, ast.Return)
            ]
            self.assertEqual(
                len(verifier_returns),
                1,
                f"Verifier function in {case_id} must contain exactly 1 top-level return statement",
            )
            return_val = verifier_returns[0].value
            self.assertIsInstance(
                return_val,
                ast.Dict,
                f"Verifier in {case_id} must return a dict literal",
            )

            return_checks_node: ast.Name | None = None
            for k_node, v_node in zip(return_val.keys, return_val.values):  # type: ignore
                k_val = k_node.value if isinstance(k_node, ast.Constant) else getattr(k_node, "s", None)
                if k_val == "checks" and isinstance(v_node, ast.Name) and v_node.id == "checks":
                    return_checks_node = v_node

            self.assertIsNotNone(
                return_checks_node,
                f"Verifier return dict in {case_id} must map literal key 'checks' to local variable 'checks'",
            )

            # 7. Exact append counting & parent/enclosing function ownership & exact shape
            total_appends: list[ast.Call] = []
            for node in ast.walk(tree):
                if isinstance(node, ast.Call):
                    if isinstance(node.func, ast.Attribute):
                        if node.func.attr == "append":
                            total_appends.append(node)
                        elif node.func.attr == "extend":
                            self.fail(f"Disallowed '.extend()' call found in {case_id}")

            self.assertEqual(
                len(total_appends),
                min_checks,
                f"Module {case_id} must contain exactly {min_checks} calls to .append(), found {len(total_appends)}",
            )

            append_receiver_nodes: set[ast.Name] = set()
            for app_call in total_appends:
                self.assertIsInstance(app_call.func, ast.Attribute)
                recv = app_call.func.value  # type: ignore
                self.assertIsInstance(
                    recv,
                    ast.Name,
                    f"Append receiver in {case_id} must be direct Name 'checks'",
                )
                self.assertEqual(recv.id, "checks")
                append_receiver_nodes.add(recv)

            allowed_name_nodes = {store_checks_node, return_checks_node} | append_receiver_nodes
            for chk_node in all_checks_name_nodes:
                self.assertIn(
                    chk_node,
                    allowed_name_nodes,
                    f"Illegal occurrence of Name 'checks' found in {case_id}",
                )

            # 8. Exact matrix / observation / expectation binding checks
            append_row_names: set[str] = set()
            for app_idx, app_call in enumerate(total_appends):
                curr: ast.AST | None = app_call
                enclosing_fn: ast.AST | None = None
                while curr in parent_map:
                    curr = parent_map[curr]
                    if isinstance(curr, (ast.FunctionDef, ast.AsyncFunctionDef)):
                        enclosing_fn = curr
                        break
                    if isinstance(
                        curr,
                        (ast.For, ast.While, ast.ListComp, ast.DictComp, ast.SetComp, ast.GeneratorExp),
                    ):
                        self.fail(
                            f"Append call at index {app_idx} in {case_id} is beneath a loop or comprehension node"
                        )

                self.assertIs(
                    enclosing_fn,
                    entrypoint_node,
                    f"Append call at index {app_idx} in {case_id} is not directly owned by top-level verifier",
                )

                self.assertEqual(
                    len(app_call.args),
                    1,
                    f"checks.append() call at {app_idx} in {case_id} must accept exactly 1 positional argument",
                )
                self.assertEqual(
                    len(app_call.keywords),
                    0,
                    f"checks.append() call at {app_idx} in {case_id} must accept 0 keyword arguments",
                )

                arg_dict = app_call.args[0]
                self.assertIsInstance(
                    arg_dict,
                    ast.Dict,
                    f"checks.append() arg at {app_idx} in {case_id} must be a dict literal",
                )
                self.assertFalse(
                    any(k is None for k in arg_dict.keys),
                    f"Dict unpack in checks.append() at {app_idx} in {case_id} is disallowed",
                )

                dict_keys = {}
                for k_node, v_node in zip(arg_dict.keys, arg_dict.values):  # type: ignore
                    if isinstance(k_node, ast.Constant):
                        dict_keys[k_node.value] = v_node
                    elif isinstance(k_node, ast.Str):
                        dict_keys[k_node.s] = v_node  # type: ignore

                self.assertIn("name", dict_keys, f"Check dict at {app_idx} in {case_id} missing 'name' key")
                self.assertIn("observed", dict_keys, f"Check dict at {app_idx} in {case_id} missing 'observed' key")
                self.assertIn("expected", dict_keys, f"Check dict at {app_idx} in {case_id} missing 'expected' key")
                self.assertIn("passed", dict_keys, f"Check dict at {app_idx} in {case_id} missing 'passed' key")

                chk_name_node = dict_keys["name"]
                self.assertTrue(
                    is_literal_node(chk_name_node),
                    f"Check dict 'name' at {app_idx} in {case_id} must be a literal string",
                )
                chk_name_val = (
                    chk_name_node.value
                    if isinstance(chk_name_node, ast.Constant)
                    else getattr(chk_name_node, "s", None)
                )
                self.assertTrue(
                    type(chk_name_val) is str and len(chk_name_val.strip()) > 0,
                    f"Check dict 'name' at {app_idx} in {case_id} must be a non-empty string",
                )
                self.assertIn(
                    chk_name_val,
                    matrix_row_map,
                    f"Check name {chk_name_val!r} at {app_idx} in {case_id} does not match any matrix row",
                )
                append_row_names.add(chk_name_val)

                chk_expected_node = dict_keys["expected"]
                self.assertTrue(
                    is_literal_node(chk_expected_node),
                    f"Check dict 'expected' at {app_idx} in {case_id} must be a literal expression",
                )
                matrix_expected_node = matrix_row_map[chk_name_val]
                self.assertTrue(
                    are_asts_equal(chk_expected_node, matrix_expected_node),
                    f"Check dict 'expected' at {app_idx} ({chk_name_val}) in {case_id} does not match matrix row expected AST",
                )

                # Observed AST binding: MUST be a plain local ast.Name with direct-call design provenance
                chk_observed_node = dict_keys["observed"]
                self.assertIsInstance(
                    chk_observed_node,
                    ast.Name,
                    f"Check dict 'observed' at {app_idx} ({chk_name_val}) in {case_id} must be a plain local ast.Name",
                )
                obs_name = chk_observed_node.id  # type: ignore
                self.assertIn(
                    obs_name,
                    design_derived_vars,
                    f"Observed name {obs_name!r} in {case_id} at row {chk_name_val!r} has no direct-call design provenance",
                )

                # Passed AST binding: must be ast.Compare between observed Name and expected literal
                passed_node = dict_keys["passed"]
                self.assertIsInstance(
                    passed_node,
                    ast.Compare,
                    f"Check 'passed' at {app_idx} ({chk_name_val}) in {case_id} must be a comparison expression (ast.Compare)",
                )
                self.assertEqual(
                    len(passed_node.comparators),
                    1,
                    f"Check 'passed' at {app_idx} ({chk_name_val}) in {case_id} must have exactly 1 comparator",
                )
                self.assertIsInstance(
                    passed_node.ops[0],
                    ast.Eq,
                    f"Check 'passed' at {app_idx} ({chk_name_val}) in {case_id} must use exact == operator",
                )

                left_node, right_node = passed_node.left, passed_node.comparators[0]
                cond1 = (are_asts_equal(left_node, chk_observed_node) and are_asts_equal(right_node, chk_expected_node))
                cond2 = (are_asts_equal(left_node, chk_expected_node) and are_asts_equal(right_node, chk_observed_node))

                self.assertTrue(
                    cond1 or cond2,
                    f"Check 'passed' at {app_idx} ({chk_name_val}) in {case_id} must compare observed Name {obs_name!r} and expected literal",
                )

            self.assertEqual(
                append_row_names,
                set(matrix_row_map.keys()),
                f"Appended check names in {case_id} must match matrix row names one-to-one",
            )

            # 9. Refuse ambient escape calls, attributes, frame roots, and EXECUTABLE subscript selectors (skipping type annotations)
            for node in ast.walk(tree):
                if isinstance(node, ast.Name):
                    self.assertNotIn(
                        node.id,
                        DISALLOWED_IDENTIFIERS,
                        f"Disallowed ambient identifier Name '{node.id}' in {case_id}",
                    )
                elif isinstance(node, ast.Attribute):
                    self.assertNotIn(
                        node.attr,
                        DISALLOWED_IDENTIFIERS,
                        f"Disallowed ambient identifier Attribute '{node.attr}' in {case_id}",
                    )
                elif isinstance(node, ast.Subscript):
                    if node in annotation_nodes:
                        continue  # Ignore type annotations like dict[str, object]

                    slice_node = node.slice
                    if isinstance(slice_node, ast.Index):
                        slice_node = slice_node.value  # type: ignore

                    if not is_literal_node(slice_node):
                        self.fail(
                            f"Non-literal executable subscript selector slice in {case_id}: AST {ast.dump(slice_node)}"
                        )

                    slice_val = eval_literal_node(slice_node)
                    if isinstance(slice_val, str) and slice_val in DISALLOWED_IDENTIFIERS:
                        self.fail(
                            f"Disallowed ambient token string selector '{slice_val}' used in Subscript slice in {case_id}"
                        )

        if not getattr(self, "_r24_harness_active", False):
            self._run_r24_full_gate_regressions()

    def test_harness_restores_state_on_fixture_failure(self) -> None:
        instance = type(self)()
        sentinel = object()
        instance._r24_harness_active = sentinel
        module_name = "r26_regression_derived_holder_rebound"
        module_sentinel = object()
        old_module = sys.modules.get(module_name)
        sys.modules[module_name] = module_sentinel  # type: ignore[assignment]
        before_path = list(sys.path)
        before_src, before_cases = SRC_DIR, DECLARED_CASES
        try:
            with mock.patch.object(importlib.util, "spec_from_file_location", side_effect=RuntimeError("forced fixture failure")):
                with self.assertRaisesRegex(RuntimeError, "forced fixture failure"):
                    instance._run_r24_full_gate_regressions()
            self.assertIs(sys.modules.get(module_name), module_sentinel)
            self.assertEqual(sys.path, before_path)
            self.assertIs(SRC_DIR, before_src)
            self.assertIs(DECLARED_CASES, before_cases)
            self.assertIs(instance._r24_harness_active, sentinel)
        finally:
            if old_module is None:
                sys.modules.pop(module_name, None)
            else:
                sys.modules[module_name] = old_module

if __name__ == "__main__":
    unittest.main()
