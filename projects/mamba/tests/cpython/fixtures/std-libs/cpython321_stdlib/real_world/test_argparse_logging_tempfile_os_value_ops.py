# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_argparse_logging_tempfile_os_value_ops"
# subject = "cpython321.test_argparse_logging_tempfile_os_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_argparse_logging_tempfile_os_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_argparse_logging_tempfile_os_value_ops: execute CPython 3.12 seed test_argparse_logging_tempfile_os_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of four
# bootstrap stdlib modules used by every command-line-argument /
# log-record / temporary-file / OS-environment path: `argparse`
# (the documented `ArgumentParser` class identifier attribute
# surface only — constructor calls diverge silently), `logging`
# (the documented `getLogger` / `basicConfig` / `INFO` /
# `WARNING` / `ERROR` / `DEBUG` / `CRITICAL` log-level
# constant attribute surface + integer-value contract on the
# level constants), `tempfile` (the documented `mkstemp` /
# `mkdtemp` / `NamedTemporaryFile` / `TemporaryDirectory` /
# `gettempdir` / `gettempprefix` attribute surface + the
# documented `gettempprefix() == "tmp"` POSIX value contract),
# and `os` (the documented `getcwd` / `getenv` / `sep` /
# `linesep` / `name` / `environ` / `listdir` / `makedirs` /
# `remove` / `stat` / `path` deeper attribute surface +
# `getenv(default=...)` fallback contract + `sep == "/"` POSIX
# contract).
#
# The matching subset between mamba and CPython is the
# argparse hasattr `ArgumentParser` only, the logging level-
# constant integer-value layer + module hasattr surface,
# the tempfile hasattr surface + gettempdir / gettempprefix
# value layer, and the os deeper attribute-surface layer +
# getenv-default + sep / linesep / name constant layer +
# listdir return-type layer.
#
# Surface in this fixture:
#   • argparse — `ArgumentParser` class identifier hasattr;
#   • logging — getLogger / basicConfig / INFO / WARNING /
#     ERROR / DEBUG / CRITICAL hasattr + integer-value
#     contract;
#   • tempfile — module hasattr + gettempdir str type +
#     gettempprefix POSIX value contract;
#   • os — getcwd / getenv / sep / linesep / name / environ /
#     listdir / makedirs / remove / stat / path hasattr +
#     getenv("NONEXIST_VAR") == None + getenv default
#     fallback + POSIX sep / linesep / name constant +
#     listdir(/tmp) returns a non-empty list.
#
# Behavioral edges that DIVERGE on mamba (argparse.Namespace /
# Action hasattr False, argparse.ArgumentParser(...).add_argument
# AttributeError 'dict' object — entire ArgumentParser instance
# surface broken, logging.Logger hasattr False, logging.getLogger
# returns dict not Logger instance, logging.getLevelName
# AttributeError, os.environ is `dict` not `_Environ` instance,
# "HOME" in os.environ returns False — mamba doesn't populate
# os.environ) are covered in the matching spec fixture
# `lang_argparse_logging_os_environ_silent`.
import argparse
import logging
import tempfile
import os


_ledger: list[int] = []

# 1) argparse — class identifier hasattr
assert hasattr(argparse, "ArgumentParser") == True; _ledger.append(1)

# 2) logging — module attribute hasattr surface
assert hasattr(logging, "getLogger") == True; _ledger.append(1)
assert hasattr(logging, "basicConfig") == True; _ledger.append(1)
assert hasattr(logging, "INFO") == True; _ledger.append(1)
assert hasattr(logging, "WARNING") == True; _ledger.append(1)
assert hasattr(logging, "ERROR") == True; _ledger.append(1)
assert hasattr(logging, "DEBUG") == True; _ledger.append(1)
assert hasattr(logging, "CRITICAL") == True; _ledger.append(1)

# 3) logging — integer-value contract on level constants
assert logging.INFO == 20; _ledger.append(1)
assert logging.WARNING == 30; _ledger.append(1)
assert logging.ERROR == 40; _ledger.append(1)
assert logging.DEBUG == 10; _ledger.append(1)
assert logging.CRITICAL == 50; _ledger.append(1)

# 4) tempfile — module attribute hasattr surface
assert hasattr(tempfile, "mkstemp") == True; _ledger.append(1)
assert hasattr(tempfile, "mkdtemp") == True; _ledger.append(1)
assert hasattr(tempfile, "NamedTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "TemporaryDirectory") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempdir") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempprefix") == True; _ledger.append(1)

# 5) tempfile — value contracts
assert isinstance(tempfile.gettempdir(), str); _ledger.append(1)
assert tempfile.gettempprefix() == "tmp"; _ledger.append(1)

# 6) os — module attribute hasattr surface
assert hasattr(os, "getcwd") == True; _ledger.append(1)
assert hasattr(os, "getenv") == True; _ledger.append(1)
assert hasattr(os, "environ") == True; _ledger.append(1)
assert hasattr(os, "listdir") == True; _ledger.append(1)
assert hasattr(os, "sep") == True; _ledger.append(1)
assert hasattr(os, "linesep") == True; _ledger.append(1)
assert hasattr(os, "name") == True; _ledger.append(1)
assert hasattr(os, "path") == True; _ledger.append(1)
assert hasattr(os, "makedirs") == True; _ledger.append(1)
assert hasattr(os, "remove") == True; _ledger.append(1)
assert hasattr(os, "stat") == True; _ledger.append(1)

# 7) os — value contracts
assert isinstance(os.getcwd(), str); _ledger.append(1)
assert os.getenv("NONEXIST_MAMBA_VAR_xyz") == None; _ledger.append(1)
assert os.getenv("NONEXIST_MAMBA_VAR_xyz", "default") == "default"; _ledger.append(1)
assert os.sep == "/"; _ledger.append(1)
assert os.linesep == "\n"; _ledger.append(1)
assert os.name == "posix"; _ledger.append(1)
assert isinstance(os.listdir("/tmp"), list); _ledger.append(1)
assert len(os.listdir("/tmp")) > 0; _ledger.append(1)

# NB: argparse.Namespace / Action hasattr False, argparse.
# ArgumentParser(...).add_argument AttributeError — entire
# ArgumentParser instance surface broken, logging.Logger
# hasattr False, logging.getLogger returns dict, logging.
# getLevelName AttributeError, os.environ is `dict` not
# `_Environ`, "HOME" in os.environ returns False — all
# DIVERGE on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_argparse_logging_tempfile_os_value_ops {sum(_ledger)} asserts")
