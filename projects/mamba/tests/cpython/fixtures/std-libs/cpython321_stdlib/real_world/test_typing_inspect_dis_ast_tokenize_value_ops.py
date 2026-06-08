# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_typing_inspect_dis_ast_tokenize_value_ops"
# subject = "cpython321.test_typing_inspect_dis_ast_tokenize_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_typing_inspect_dis_ast_tokenize_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_typing_inspect_dis_ast_tokenize_value_ops: execute CPython 3.12 seed test_typing_inspect_dis_ast_tokenize_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `typing` / `inspect` / `dis` / `ast` / `tokenize` five-pack
# pinned to atomic 197: `typing` (the documented partial
# module-level alias / class / function identifier hasattr
# surface — `List` / `Dict` / `Tuple` / `Set` / `FrozenSet`
# / `Optional` / `Union` / `Any` / `Callable` / `Type` /
# `TypeVar` / `Generic` / `Iterator` / `Protocol` /
# `TYPE_CHECKING` / `cast` / `get_type_hints` / `Literal` /
# `Final` / `ClassVar` / `NamedTuple` / `TypedDict` + the
# documented typing.TYPE_CHECKING == False boolean-value
# contract), `inspect` (the documented partial module-level
# helper hasattr surface — `getmembers` / `isfunction` /
# `ismethod` / `isclass` / `signature`), `dis` (the
# documented partial module-level helper hasattr surface —
# `dis` / `code_info` / `show_code` / `get_instructions` /
# `Instruction` / `findlabels` / `findlinestarts` / `opmap`
# / `opname` / `HAVE_ARGUMENT` / `stack_effect`), `ast` (the
# documented partial module-level helper / node-class
# identifier hasattr surface — `parse` / `dump` /
# `literal_eval` / `walk` / `Module` / `Expression` /
# `FunctionDef` / `ClassDef` / `Return` / `Assign` / `Expr`
# / `Call` / `Name` / `Constant` / `Load` / `Store` / `Del`
# / `NodeVisitor` / `NodeTransformer` / `increment_lineno`
# / `fix_missing_locations` / `copy_location` + the
# documented ast.parse Module-returning class-identity value
# contract), and `tokenize` (the documented full module-
# level helper / token-constant identifier hasattr surface
# — `tokenize` / `untokenize` / `open` / `generate_tokens`
# / `TokenInfo` / `TokenError` / `NUMBER` / `STRING` /
# `NAME` / `OP` / `NEWLINE` / `INDENT` / `DEDENT` /
# `ENDMARKER` / `ENCODING` / `COMMENT`).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(typing, "Iterable") / "Mapping" / "Sequence" /
# "overload" / "final" / "get_origin" / "get_args" /
# "NewType" / "NoReturn" / "Annotated" all False on mamba,
# hasattr(inspect, "getmodule") / "getsource" /
# "getsourcelines" / "getfile" / "ismodule" / "isbuiltin"
# / "iscoroutinefunction" / "isasyncgen" /
# "isgeneratorfunction" / "isawaitable" / "Signature" /
# "Parameter" / "BoundArguments" / "stack" / "currentframe"
# / "getframeinfo" / "FrameInfo" / "getmro" /
# "getfullargspec" / "FullArgSpec" all False on mamba,
# inspect.isfunction(fn) returns False on mamba but True on
# CPython, inspect.isfunction(1) returns True on mamba but
# False on CPython, hasattr(dis, "Bytecode") /
# "EXTENDED_ARG" False on mamba, hasattr(ast, "iter_fields")
# / "iter_child_nodes" / "AST" False on mamba, ast
# .literal_eval("[1,2,3]") returns None on mamba instead of
# the documented [1, 2, 3] list) are covered in the matching
# spec fixture `lang_typing_inspect_dis_ast_silent`.
import typing
import inspect
import dis
import ast
import tokenize


_ledger: list[int] = []

# 1) typing — partial module hasattr surface
#    (Iterable / Mapping / Sequence / overload / final /
#    get_origin / get_args / NewType / NoReturn / Annotated
#    DIVERGE — moved to spec fixture)
assert hasattr(typing, "List") == True; _ledger.append(1)
assert hasattr(typing, "Dict") == True; _ledger.append(1)
assert hasattr(typing, "Tuple") == True; _ledger.append(1)
assert hasattr(typing, "Set") == True; _ledger.append(1)
assert hasattr(typing, "FrozenSet") == True; _ledger.append(1)
assert hasattr(typing, "Optional") == True; _ledger.append(1)
assert hasattr(typing, "Union") == True; _ledger.append(1)
assert hasattr(typing, "Any") == True; _ledger.append(1)
assert hasattr(typing, "Callable") == True; _ledger.append(1)
assert hasattr(typing, "Type") == True; _ledger.append(1)
assert hasattr(typing, "TypeVar") == True; _ledger.append(1)
assert hasattr(typing, "Generic") == True; _ledger.append(1)
assert hasattr(typing, "Iterator") == True; _ledger.append(1)
assert hasattr(typing, "Protocol") == True; _ledger.append(1)
assert hasattr(typing, "TYPE_CHECKING") == True; _ledger.append(1)
assert hasattr(typing, "cast") == True; _ledger.append(1)
assert hasattr(typing, "get_type_hints") == True; _ledger.append(1)
assert hasattr(typing, "Literal") == True; _ledger.append(1)
assert hasattr(typing, "Final") == True; _ledger.append(1)
assert hasattr(typing, "ClassVar") == True; _ledger.append(1)
assert hasattr(typing, "NamedTuple") == True; _ledger.append(1)
assert hasattr(typing, "TypedDict") == True; _ledger.append(1)

# 2) typing.TYPE_CHECKING — boolean-value contract
assert typing.TYPE_CHECKING == False; _ledger.append(1)

# 3) inspect — partial module hasattr surface
#    (getmodule / getsource / getsourcelines / getfile /
#    ismodule / isbuiltin / iscoroutinefunction / isasyncgen
#    / isgeneratorfunction / isawaitable / Signature /
#    Parameter / BoundArguments / stack / currentframe /
#    getframeinfo / FrameInfo / getmro / getfullargspec /
#    FullArgSpec DIVERGE — moved to spec fixture)
assert hasattr(inspect, "getmembers") == True; _ledger.append(1)
assert hasattr(inspect, "isfunction") == True; _ledger.append(1)
assert hasattr(inspect, "ismethod") == True; _ledger.append(1)
assert hasattr(inspect, "isclass") == True; _ledger.append(1)
assert hasattr(inspect, "signature") == True; _ledger.append(1)

# 4) dis — partial module hasattr surface
#    (Bytecode / EXTENDED_ARG DIVERGE — moved to spec fixture)
assert hasattr(dis, "dis") == True; _ledger.append(1)
assert hasattr(dis, "code_info") == True; _ledger.append(1)
assert hasattr(dis, "show_code") == True; _ledger.append(1)
assert hasattr(dis, "get_instructions") == True; _ledger.append(1)
assert hasattr(dis, "Instruction") == True; _ledger.append(1)
assert hasattr(dis, "findlabels") == True; _ledger.append(1)
assert hasattr(dis, "findlinestarts") == True; _ledger.append(1)
assert hasattr(dis, "opmap") == True; _ledger.append(1)
assert hasattr(dis, "opname") == True; _ledger.append(1)
assert hasattr(dis, "HAVE_ARGUMENT") == True; _ledger.append(1)
assert hasattr(dis, "stack_effect") == True; _ledger.append(1)

# 5) ast — partial module hasattr surface
#    (iter_fields / iter_child_nodes / AST DIVERGE — moved
#    to spec fixture)
assert hasattr(ast, "parse") == True; _ledger.append(1)
assert hasattr(ast, "dump") == True; _ledger.append(1)
assert hasattr(ast, "literal_eval") == True; _ledger.append(1)
assert hasattr(ast, "walk") == True; _ledger.append(1)
assert hasattr(ast, "Module") == True; _ledger.append(1)
assert hasattr(ast, "Expression") == True; _ledger.append(1)
assert hasattr(ast, "FunctionDef") == True; _ledger.append(1)
assert hasattr(ast, "ClassDef") == True; _ledger.append(1)
assert hasattr(ast, "Return") == True; _ledger.append(1)
assert hasattr(ast, "Assign") == True; _ledger.append(1)
assert hasattr(ast, "Expr") == True; _ledger.append(1)
assert hasattr(ast, "Call") == True; _ledger.append(1)
assert hasattr(ast, "Name") == True; _ledger.append(1)
assert hasattr(ast, "Constant") == True; _ledger.append(1)
assert hasattr(ast, "Load") == True; _ledger.append(1)
assert hasattr(ast, "Store") == True; _ledger.append(1)
assert hasattr(ast, "Del") == True; _ledger.append(1)
assert hasattr(ast, "NodeVisitor") == True; _ledger.append(1)
assert hasattr(ast, "NodeTransformer") == True; _ledger.append(1)
assert hasattr(ast, "increment_lineno") == True; _ledger.append(1)
assert hasattr(ast, "fix_missing_locations") == True; _ledger.append(1)
assert hasattr(ast, "copy_location") == True; _ledger.append(1)

# 6) ast.parse — Module-returning class-identity value contract
assert type(ast.parse("x = 1")).__name__ == "Module"; _ledger.append(1)

# 7) tokenize — full module hasattr surface
assert hasattr(tokenize, "tokenize") == True; _ledger.append(1)
assert hasattr(tokenize, "untokenize") == True; _ledger.append(1)
assert hasattr(tokenize, "open") == True; _ledger.append(1)
assert hasattr(tokenize, "generate_tokens") == True; _ledger.append(1)
assert hasattr(tokenize, "TokenInfo") == True; _ledger.append(1)
assert hasattr(tokenize, "TokenError") == True; _ledger.append(1)
assert hasattr(tokenize, "NUMBER") == True; _ledger.append(1)
assert hasattr(tokenize, "STRING") == True; _ledger.append(1)
assert hasattr(tokenize, "NAME") == True; _ledger.append(1)
assert hasattr(tokenize, "OP") == True; _ledger.append(1)
assert hasattr(tokenize, "NEWLINE") == True; _ledger.append(1)
assert hasattr(tokenize, "INDENT") == True; _ledger.append(1)
assert hasattr(tokenize, "DEDENT") == True; _ledger.append(1)
assert hasattr(tokenize, "ENDMARKER") == True; _ledger.append(1)
assert hasattr(tokenize, "ENCODING") == True; _ledger.append(1)
assert hasattr(tokenize, "COMMENT") == True; _ledger.append(1)

# NB: hasattr(typing, "Iterable") / "Mapping" / "Sequence" /
# "overload" / "final" / "get_origin" / "get_args" /
# "NewType" / "NoReturn" / "Annotated" all False on mamba,
# hasattr(inspect, "getmodule") / "getsource" /
# "getsourcelines" / "getfile" / "ismodule" / "isbuiltin"
# / "iscoroutinefunction" / "isasyncgen" /
# "isgeneratorfunction" / "isawaitable" / "Signature" /
# "Parameter" / "BoundArguments" / "stack" / "currentframe"
# / "getframeinfo" / "FrameInfo" / "getmro" /
# "getfullargspec" / "FullArgSpec" all False on mamba,
# inspect.isfunction(<def>) returns False on mamba but True
# on CPython, inspect.isfunction(1) returns True on mamba
# but False on CPython, hasattr(dis, "Bytecode") /
# "EXTENDED_ARG" False on mamba, hasattr(ast, "iter_fields")
# / "iter_child_nodes" / "AST" False on mamba, ast
# .literal_eval("[1,2,3]") returns None on mamba instead of
# the documented [1, 2, 3] list — all DIVERGE on mamba —
# moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_typing_inspect_dis_ast_tokenize_value_ops {sum(_ledger)} asserts")
