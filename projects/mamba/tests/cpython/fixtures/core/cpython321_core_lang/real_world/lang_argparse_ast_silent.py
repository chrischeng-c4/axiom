# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_argparse_ast_silent"
# subject = "cpython321.lang_argparse_ast_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_argparse_ast_silent.py"
# status = "filled"
# ///
"""cpython321.lang_argparse_ast_silent: execute CPython 3.12 seed lang_argparse_ast_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the `argparse` /
# `getopt` / `shlex` / `ast` four-pack pinned to atomic 218:
# `argparse` (the documented
# `hasattr(argparse, "Namespace") / "Action" / "FileType" /
# "ArgumentError" / "ArgumentTypeError" / "HelpFormatter" /
# "RawDescriptionHelpFormatter" / "RawTextHelpFormatter" /
# "ArgumentDefaultsHelpFormatter" / "MetavarTypeHelpFormatter"
# / "REMAINDER" / "OPTIONAL" / "ZERO_OR_MORE" /
# "ONE_OR_MORE" / "SUPPRESS" / "PARSER" /
# "BooleanOptionalAction" == True` full module-level helper
# / class / sentinel identifier hasattr surface + the
# documented
# `type(argparse.ArgumentParser()).__name__ ==
# "ArgumentParser"` ArgumentParser-instance type-identity
# value contract), `getopt` (the documented
# `hasattr(getopt, "error") == True` module-level
# exception-alias hasattr surface), `shlex` (the documented
# `hasattr(shlex, "shlex") == True` module-level class
# identifier hasattr surface), and `ast` (the documented
# `hasattr(ast, "iter_fields") / "iter_child_nodes" ==
# True` extended hasattr surface + the documented
# `ast.literal_eval("[1, 2, 3]") == [1, 2, 3]` /
# `ast.literal_eval('{"a": 1}') == {"a": 1}` /
# `ast.literal_eval("(1, 2)") == (1, 2)` container
# literal-eval value contract + the documented
# `len(ast.parse("x = 1 + 2").body) == 1` parse-tree
# body length value contract).
#
# Behavioral edges that CONFORM on mamba
# (argparse `ArgumentParser` hasattr, getopt `getopt` /
# `gnu_getopt` / `GetoptError` hasattr + option-parse
# value contract, shlex `split` / `join` / `quote`
# hasattr + shell-quote / split value contract, keyword
# full hasattr surface + classification value contract,
# token full hasattr surface, tokenize full hasattr
# surface, ast `parse` / `dump` / `literal_eval` /
# `walk` / `Module` / `Expression` / `Name` / `Load` /
# `Store` / `Constant` / `BinOp` / `Add` /
# `NodeVisitor` / `NodeTransformer` / `get_docstring` /
# `fix_missing_locations` / `increment_lineno` hasattr
# + scalar literal-eval value contract) are covered in
# the matching pass fixture
# `test_getopt_shlex_keyword_token_tokenize_ast_value_ops`.
from typing import Any
import argparse as _argparse_mod
import getopt as _getopt_mod
import shlex as _shlex_mod
import ast as _ast_mod

argparse: Any = _argparse_mod
getopt: Any = _getopt_mod
shlex: Any = _shlex_mod
ast: Any = _ast_mod


_ledger: list[int] = []

# 1) argparse — full module hasattr surface
#    (mamba: Namespace / Action / FileType / ArgumentError
#    / ArgumentTypeError / HelpFormatter /
#    RawDescriptionHelpFormatter / RawTextHelpFormatter /
#    ArgumentDefaultsHelpFormatter /
#    MetavarTypeHelpFormatter / REMAINDER / OPTIONAL /
#    ZERO_OR_MORE / ONE_OR_MORE / SUPPRESS / PARSER /
#    BooleanOptionalAction all False)
assert hasattr(argparse, "Namespace") == True; _ledger.append(1)
assert hasattr(argparse, "Action") == True; _ledger.append(1)
assert hasattr(argparse, "FileType") == True; _ledger.append(1)
assert hasattr(argparse, "ArgumentError") == True; _ledger.append(1)
assert hasattr(argparse, "ArgumentTypeError") == True; _ledger.append(1)
assert hasattr(argparse, "HelpFormatter") == True; _ledger.append(1)
assert hasattr(argparse, "RawDescriptionHelpFormatter") == True; _ledger.append(1)
assert hasattr(argparse, "RawTextHelpFormatter") == True; _ledger.append(1)
assert hasattr(argparse, "ArgumentDefaultsHelpFormatter") == True; _ledger.append(1)
assert hasattr(argparse, "MetavarTypeHelpFormatter") == True; _ledger.append(1)
assert hasattr(argparse, "REMAINDER") == True; _ledger.append(1)
assert hasattr(argparse, "OPTIONAL") == True; _ledger.append(1)
assert hasattr(argparse, "ZERO_OR_MORE") == True; _ledger.append(1)
assert hasattr(argparse, "ONE_OR_MORE") == True; _ledger.append(1)
assert hasattr(argparse, "SUPPRESS") == True; _ledger.append(1)
assert hasattr(argparse, "PARSER") == True; _ledger.append(1)
assert hasattr(argparse, "BooleanOptionalAction") == True; _ledger.append(1)

# 2) argparse — ArgumentParser-instance type-identity value
#    contract
#    (mamba: type(argparse.ArgumentParser()).__name__
#    "ArgumentParser" collapses to "dict")
assert type(argparse.ArgumentParser()).__name__ == "ArgumentParser"; _ledger.append(1)

# 3) getopt — module-level exception-alias hasattr surface
#    (mamba: getopt.error False even though GetoptError is
#    True — CPython exposes them as the same exception)
assert hasattr(getopt, "error") == True; _ledger.append(1)

# 4) shlex — module-level class identifier hasattr surface
#    (mamba: shlex.shlex False even though split/join/quote
#    are True)
assert hasattr(shlex, "shlex") == True; _ledger.append(1)

# 5) ast — extended module hasattr surface
#    (mamba: iter_fields / iter_child_nodes both False)
assert hasattr(ast, "iter_fields") == True; _ledger.append(1)
assert hasattr(ast, "iter_child_nodes") == True; _ledger.append(1)

# 6) ast — container literal-eval value contract
#    (mamba: ast.literal_eval for list / dict / tuple all
#    return None instead of the parsed container)
assert ast.literal_eval("[1, 2, 3]") == [1, 2, 3]; _ledger.append(1)
assert ast.literal_eval('{"a": 1}') == {"a": 1}; _ledger.append(1)
assert ast.literal_eval("(1, 2)") == (1, 2); _ledger.append(1)

# 7) ast — parse-tree body length value contract
#    (mamba: len(ast.parse("x = 1 + 2").body) collapses to
#    0 — the parser returns an empty Module body)
assert len(ast.parse("x = 1 + 2").body) == 1; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_argparse_ast_silent {sum(_ledger)} asserts")
