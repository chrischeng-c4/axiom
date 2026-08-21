# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_argparse_jsondumps_silent"
# subject = "cpython321.lang_argparse_jsondumps_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_argparse_jsondumps_silent.py"
# status = "filled"
# ///
"""cpython321.lang_argparse_jsondumps_silent: execute CPython 3.12 seed lang_argparse_jsondumps_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# argparse module surface + the documented `json.dumps(...,
# ensure_ascii=True)` Unicode escape contract pinned by atomic
# 175: `argparse` (the documented `ArgumentParser` constructor
# instance class identity + `.prog` instance attribute + the
# documented `Namespace` / `Action` / `ArgumentTypeError` /
# `ArgumentError` / `FileType` / `BooleanOptionalAction` class
# identifier surface) and `json` (the documented `dumps` with
# `ensure_ascii=True` non-ASCII `\uXXXX` escape contract).
#
# The matching subset (full calendar isleap / leapdays /
# month_name / day_name / monthrange numeric-day-count /
# weekday / timegm + module hasattr surface, full json dumps
# with indent= / separators= / sort_keys= / ensure_ascii=False
# keyword surface, json loads scalar layer (true / false /
# null / numeric / string), full json module hasattr surface,
# argparse ArgumentParser module-level hasattr, full builtin
# isinstance / issubclass / type / repr / bool value contract)
# is covered by
# `test_calendar_json_argparse_builtin_typecheck_value_ops`;
# this fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • json.dumps("héllo", ensure_ascii=True) == '"h\\u00e9llo"'
#     — documented `\uXXXX` escape contract for non-ASCII
#     characters (mamba: returns '"héllo"' — the ensure_ascii=
#     True flag is ignored and the documented escape contract
#     is broken);
#   • type(argparse.ArgumentParser()).__name__ == "ArgumentParser"
#     — documented constructor class identity (mamba: returns
#     "dict" — the constructor produces a `dict` instead of the
#     documented ArgumentParser instance);
#   • argparse.ArgumentParser(prog="myprog").prog == "myprog"
#     — documented `.prog` instance attribute on the parser
#     instance (mamba: returns None — the documented `prog`
#     instance attribute surface is broken);
#   • hasattr(argparse, "Namespace") is True — documented
#     class identifier (mamba: False);
#   • hasattr(argparse, "Action") is True — documented class
#     identifier (mamba: False);
#   • hasattr(argparse, "ArgumentTypeError") is True —
#     documented exception class (mamba: False);
#   • hasattr(argparse, "ArgumentError") is True — documented
#     exception class (mamba: False);
#   • hasattr(argparse, "FileType") is True — documented
#     type-callable class (mamba: False);
#   • hasattr(argparse, "BooleanOptionalAction") is True —
#     documented action class (mamba: False).
import json as _json_mod
import argparse as _argparse_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# instance methods / class attributes / module-level helpers
# that mamba's bundled type stubs do not surface accurately.
json: Any = _json_mod
argparse: Any = _argparse_mod


_ledger: list[int] = []

# 1) json.dumps — ensure_ascii=True Unicode escape contract
assert json.dumps("héllo", ensure_ascii=True) == '"h\\u00e9llo"'; _ledger.append(1)

# 2) argparse.ArgumentParser — constructor class identity
assert type(argparse.ArgumentParser()).__name__ == "ArgumentParser"; _ledger.append(1)

# 3) argparse.ArgumentParser — .prog instance attribute
_ap = argparse.ArgumentParser(prog="myprog")
assert _ap.prog == "myprog"; _ledger.append(1)

# 4) argparse — Namespace / Action / *Error class identifiers
assert hasattr(argparse, "Namespace") == True; _ledger.append(1)
assert hasattr(argparse, "Action") == True; _ledger.append(1)
assert hasattr(argparse, "ArgumentTypeError") == True; _ledger.append(1)
assert hasattr(argparse, "ArgumentError") == True; _ledger.append(1)

# 5) argparse — FileType / BooleanOptionalAction class identifiers
assert hasattr(argparse, "FileType") == True; _ledger.append(1)
assert hasattr(argparse, "BooleanOptionalAction") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_argparse_jsondumps_silent {sum(_ledger)} asserts")
