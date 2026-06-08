# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_typing_inspect_dis_ast_silent"
# subject = "cpython321.lang_typing_inspect_dis_ast_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_typing_inspect_dis_ast_silent.py"
# status = "filled"
# ///
"""cpython321.lang_typing_inspect_dis_ast_silent: execute CPython 3.12 seed lang_typing_inspect_dis_ast_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across
# the `typing` extended alias / class / function identifier
# surface + `inspect` extended helper / class identifier
# surface + `inspect.isfunction` predicate behavior contract
# + `dis` extended class / constant identifier surface +
# `ast` extended helper / base-class identifier surface +
# `ast.literal_eval` value contract pinned by atomic 197:
# `typing` (the documented `Iterable` / `Mapping` /
# `Sequence` / `overload` / `final` / `get_origin` /
# `get_args` / `NewType` / `NoReturn` / `Annotated`
# extended alias / class / function identifier surface),
# `inspect` (the documented `getmodule` / `getsource` /
# `getsourcelines` / `getfile` / `ismodule` / `isbuiltin`
# / `iscoroutinefunction` / `isasyncgen` /
# `isgeneratorfunction` / `isawaitable` / `Signature` /
# `Parameter` / `BoundArguments` / `stack` / `currentframe`
# / `getframeinfo` / `FrameInfo` / `getmro` /
# `getfullargspec` / `FullArgSpec` extended helper / class
# / dataclass identifier surface + the documented
# inspect.isfunction(<def>) is True / inspect.isfunction(1)
# is False predicate value contract), `dis` (the documented
# `Bytecode` / `EXTENDED_ARG` extended class / constant
# identifier surface), and `ast` (the documented
# `iter_fields` / `iter_child_nodes` / `AST` extended
# helper / base-class identifier surface + the documented
# ast.literal_eval("[1,2,3]") == [1, 2, 3] list-returning
# value contract).
#
# The matching subset (partial typing hasattr +
# TYPE_CHECKING bool value, partial inspect hasattr, partial
# dis hasattr, partial ast hasattr + Module-returning
# parse, full tokenize hasattr) is covered by
# `test_typing_inspect_dis_ast_tokenize_value_ops`; this
# fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(typing, "Iterable") is True — documented alias
#     identifier (mamba: False);
#   • hasattr(typing, "Mapping") is True — documented alias
#     identifier (mamba: False);
#   • hasattr(typing, "Sequence") is True — documented alias
#     identifier (mamba: False);
#   • hasattr(typing, "overload") is True — documented
#     function identifier (mamba: False);
#   • hasattr(typing, "final") is True — documented function
#     identifier (mamba: False);
#   • hasattr(typing, "get_origin") is True — documented
#     function identifier (mamba: False);
#   • hasattr(typing, "get_args") is True — documented
#     function identifier (mamba: False);
#   • hasattr(typing, "NewType") is True — documented class
#     identifier (mamba: False);
#   • hasattr(typing, "NoReturn") is True — documented alias
#     identifier (mamba: False);
#   • hasattr(typing, "Annotated") is True — documented
#     alias identifier (mamba: False);
#   • hasattr(inspect, "getmodule") is True — documented
#     function identifier (mamba: False);
#   • hasattr(inspect, "getsource") is True — documented
#     function identifier (mamba: False);
#   • hasattr(inspect, "getsourcelines") is True —
#     documented function identifier (mamba: False);
#   • hasattr(inspect, "getfile") is True — documented
#     function identifier (mamba: False);
#   • hasattr(inspect, "ismodule") is True — documented
#     predicate identifier (mamba: False);
#   • hasattr(inspect, "isbuiltin") is True — documented
#     predicate identifier (mamba: False);
#   • hasattr(inspect, "iscoroutinefunction") is True —
#     documented predicate identifier (mamba: False);
#   • hasattr(inspect, "isasyncgen") is True — documented
#     predicate identifier (mamba: False);
#   • hasattr(inspect, "isgeneratorfunction") is True —
#     documented predicate identifier (mamba: False);
#   • hasattr(inspect, "isawaitable") is True — documented
#     predicate identifier (mamba: False);
#   • hasattr(inspect, "Signature") is True — documented
#     class identifier (mamba: False);
#   • hasattr(inspect, "Parameter") is True — documented
#     class identifier (mamba: False);
#   • hasattr(inspect, "BoundArguments") is True —
#     documented class identifier (mamba: False);
#   • hasattr(inspect, "stack") is True — documented
#     function identifier (mamba: False);
#   • hasattr(inspect, "currentframe") is True — documented
#     function identifier (mamba: False);
#   • hasattr(inspect, "getframeinfo") is True — documented
#     function identifier (mamba: False);
#   • hasattr(inspect, "FrameInfo") is True — documented
#     class identifier (mamba: False);
#   • hasattr(inspect, "getmro") is True — documented
#     function identifier (mamba: False);
#   • hasattr(inspect, "getfullargspec") is True —
#     documented function identifier (mamba: False);
#   • hasattr(inspect, "FullArgSpec") is True — documented
#     class identifier (mamba: False);
#   • inspect.isfunction(<def>) is True — documented
#     predicate value contract (mamba: returns False);
#   • inspect.isfunction(1) is False — documented predicate
#     value contract (mamba: returns True);
#   • hasattr(dis, "Bytecode") is True — documented class
#     identifier (mamba: False);
#   • hasattr(dis, "EXTENDED_ARG") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(ast, "iter_fields") is True — documented
#     function identifier (mamba: False);
#   • hasattr(ast, "iter_child_nodes") is True — documented
#     function identifier (mamba: False);
#   • hasattr(ast, "AST") is True — documented base-class
#     identifier (mamba: False);
#   • ast.literal_eval("[1,2,3]") == [1, 2, 3] — documented
#     list-returning value contract (mamba: returns None).
import typing as _typing_mod
import inspect as _inspect_mod
import dis as _dis_mod
import ast as _ast_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# function / class / alias / predicate / value-contract behavior
# that mamba's bundled type stubs do not surface accurately.
typing: Any = _typing_mod
inspect: Any = _inspect_mod
dis: Any = _dis_mod
ast: Any = _ast_mod


def _example_fn(x):
    return x + 1


_ledger: list[int] = []

# 1) typing — extended alias / class / function surface
assert hasattr(typing, "Iterable") == True; _ledger.append(1)
assert hasattr(typing, "Mapping") == True; _ledger.append(1)
assert hasattr(typing, "Sequence") == True; _ledger.append(1)
assert hasattr(typing, "overload") == True; _ledger.append(1)
assert hasattr(typing, "final") == True; _ledger.append(1)
assert hasattr(typing, "get_origin") == True; _ledger.append(1)
assert hasattr(typing, "get_args") == True; _ledger.append(1)
assert hasattr(typing, "NewType") == True; _ledger.append(1)
assert hasattr(typing, "NoReturn") == True; _ledger.append(1)
assert hasattr(typing, "Annotated") == True; _ledger.append(1)

# 2) inspect — extended helper / class / dataclass surface
assert hasattr(inspect, "getmodule") == True; _ledger.append(1)
assert hasattr(inspect, "getsource") == True; _ledger.append(1)
assert hasattr(inspect, "getsourcelines") == True; _ledger.append(1)
assert hasattr(inspect, "getfile") == True; _ledger.append(1)
assert hasattr(inspect, "ismodule") == True; _ledger.append(1)
assert hasattr(inspect, "isbuiltin") == True; _ledger.append(1)
assert hasattr(inspect, "iscoroutinefunction") == True; _ledger.append(1)
assert hasattr(inspect, "isasyncgen") == True; _ledger.append(1)
assert hasattr(inspect, "isgeneratorfunction") == True; _ledger.append(1)
assert hasattr(inspect, "isawaitable") == True; _ledger.append(1)
assert hasattr(inspect, "Signature") == True; _ledger.append(1)
assert hasattr(inspect, "Parameter") == True; _ledger.append(1)
assert hasattr(inspect, "BoundArguments") == True; _ledger.append(1)
assert hasattr(inspect, "stack") == True; _ledger.append(1)
assert hasattr(inspect, "currentframe") == True; _ledger.append(1)
assert hasattr(inspect, "getframeinfo") == True; _ledger.append(1)
assert hasattr(inspect, "FrameInfo") == True; _ledger.append(1)
assert hasattr(inspect, "getmro") == True; _ledger.append(1)
assert hasattr(inspect, "getfullargspec") == True; _ledger.append(1)
assert hasattr(inspect, "FullArgSpec") == True; _ledger.append(1)

# 3) inspect.isfunction — predicate value contract
assert inspect.isfunction(_example_fn) == True; _ledger.append(1)
assert inspect.isfunction(1) == False; _ledger.append(1)

# 4) dis — extended class / constant surface
assert hasattr(dis, "Bytecode") == True; _ledger.append(1)
assert hasattr(dis, "EXTENDED_ARG") == True; _ledger.append(1)

# 5) ast — extended helper / base-class surface
assert hasattr(ast, "iter_fields") == True; _ledger.append(1)
assert hasattr(ast, "iter_child_nodes") == True; _ledger.append(1)
assert hasattr(ast, "AST") == True; _ledger.append(1)

# 6) ast.literal_eval — list-returning value contract
assert ast.literal_eval("[1,2,3]") == [1, 2, 3]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typing_inspect_dis_ast_silent {sum(_ledger)} asserts")
