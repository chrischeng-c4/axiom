# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_argparse_logging_os_environ_silent"
# subject = "cpython321.lang_argparse_logging_os_environ_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_argparse_logging_os_environ_silent.py"
# status = "filled"
# ///
"""cpython321.lang_argparse_logging_os_environ_silent: execute CPython 3.12 seed lang_argparse_logging_os_environ_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# command-line-argument / log-record / OS-environment triplet
# pinned by atomic 167: `argparse` (the documented `Namespace`
# / `Action` class identifier attribute surface + the documented
# `ArgumentParser(...).add_argument` / `parse_args` instance-
# method surface), `logging` (the documented `Logger` class
# identifier + `getLevelName` int-to-name reverse-lookup +
# `getLogger(name)` Logger-instance return contract), and `os`
# (the documented `_Environ` `os.environ` type contract +
# `"HOME" in os.environ` population contract).
#
# The matching subset (argparse.ArgumentParser hasattr,
# logging level-constant integer-value layer + module hasattr
# for getLogger / basicConfig / INFO / WARNING / ERROR / DEBUG /
# CRITICAL, tempfile gettempdir / gettempprefix + module
# hasattr surface, os deeper module hasattr + getcwd / getenv /
# sep / linesep / name / listdir constant + return-type
# contracts) is covered by
# `test_argparse_logging_tempfile_os_value_ops`; this fixture
# pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(argparse, "Namespace") is True — documented
#     parse-result class identifier (mamba: False);
#   • hasattr(argparse, "Action") is True — documented base
#     class for argparse actions (mamba: False);
#   • argparse.ArgumentParser(prog="x").add_argument("--name")
#     populates the parser — successful chain (mamba:
#     AttributeError 'dict' object has no attribute
#     'add_argument' — the entire ArgumentParser instance
#     surface is broken even though `hasattr(argparse,
#     "ArgumentParser")` returns True);
#   • argparse.ArgumentParser(...).parse_args([...])
#     returns a Namespace with attribute access on the parsed
#     args (mamba: AttributeError at construction);
#   • hasattr(logging, "Logger") is True — documented Logger
#     class identifier (mamba: False);
#   • logging.getLevelName(20) == "INFO" — documented int-to-
#     name reverse-lookup (mamba: AttributeError, 'dict'
#     object has no attribute 'getLevelName');
#   • logging.getLevelName(30) == "WARNING" (mamba: same
#     AttributeError);
#   • type(logging.getLogger("x")).__name__ == "Logger" —
#     getLogger returns a Logger instance (mamba: returns a
#     dict, type is `dict`);
#   • type(os.environ).__name__ == "_Environ" — documented
#     environ type (mamba: returns `dict`);
#   • "HOME" in os.environ is True on POSIX hosts — environ
#     population contract (mamba: returns False — os.environ
#     is not populated even with HOME exported in the parent
#     shell).
import argparse as _argparse_mod
import logging as _logging_mod
import os as _os_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / instance methods
# that mamba's bundled type stubs do not surface accurately.
argparse: Any = _argparse_mod
logging: Any = _logging_mod
os: Any = _os_mod


_ledger: list[int] = []

# 1) argparse — documented class identifier attribute surface
assert hasattr(argparse, "Namespace") == True; _ledger.append(1)
assert hasattr(argparse, "Action") == True; _ledger.append(1)

# 2) argparse.ArgumentParser — add_argument + parse_args chain
_p = argparse.ArgumentParser(prog="myprog")
_p.add_argument("--name", default="world")
_p.add_argument("--count", type=int, default=1)
_args = _p.parse_args(["--name", "alice", "--count", "5"])
assert _args.name == "alice"; _ledger.append(1)
assert _args.count == 5; _ledger.append(1)

# 3) argparse — default-value parse contract
_args2 = _p.parse_args([])
assert _args2.name == "world"; _ledger.append(1)
assert _args2.count == 1; _ledger.append(1)

# 4) logging — Logger class identifier
assert hasattr(logging, "Logger") == True; _ledger.append(1)

# 5) logging.getLevelName — int-to-name reverse lookup
assert logging.getLevelName(20) == "INFO"; _ledger.append(1)
assert logging.getLevelName(30) == "WARNING"; _ledger.append(1)

# 6) logging.getLogger — Logger instance return contract
_lg = logging.getLogger("test")
assert type(_lg).__name__ == "Logger"; _ledger.append(1)

# 7) os.environ — `_Environ` type + HOME population
assert type(os.environ).__name__ == "_Environ"; _ledger.append(1)
assert ("HOME" in os.environ) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_argparse_logging_os_environ_silent {sum(_ledger)} asserts")
