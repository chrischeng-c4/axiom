# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_pathlib_fnmatch_sysconfig_platform_time_value_ops"
# subject = "cpython321.test_pathlib_fnmatch_sysconfig_platform_time_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_pathlib_fnmatch_sysconfig_platform_time_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_pathlib_fnmatch_sysconfig_platform_time_value_ops: execute CPython 3.12 seed test_pathlib_fnmatch_sysconfig_platform_time_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of seven
# bootstrap stdlib modules used by every filesystem-path /
# shell-glob / build-config / platform-detection / locale /
# monotonic-time / weak-reference path: `pathlib` (the
# documented `Path` / `PurePath` / `PurePosixPath` class
# identifier attribute surface + the documented `Path(p)` /
# `PosixPath` constructor type contract), `fnmatch` (the
# documented `fnmatch` / `fnmatchcase` / `filter` / `translate`
# attribute surface + full pattern-match value contract),
# `sysconfig` (the documented `get_platform` / `get_python_
# version` / `get_paths` / `get_path_names` / `get_config_vars`
# attribute surface + str-return value contract), `platform`
# (the documented `system` / `machine` / `platform` /
# `python_version` / `release` / `node` attribute surface +
# str-return value contract), `locale` (the documented
# `getlocale` / `setlocale` / `LC_ALL` / `LC_CTYPE` /
# `LC_NUMERIC` attribute surface + `getlocale()` tuple-return
# contract), `time` (the documented `monotonic` / `perf_counter`
# monotonic float-return contract), and `weakref` (the
# documented `ref` / `proxy` / `WeakValueDictionary` /
# `WeakKeyDictionary` / `WeakSet` attribute surface).
#
# The matching subset between mamba and CPython is the pathlib
# class-identifier hasattr surface + Path() constructor type
# contract, the fnmatch full value layer + module hasattr
# surface, the sysconfig str-return value layer + module hasattr
# surface, the platform str-return value layer (system /
# machine / python_version) + module hasattr partial surface,
# the locale tuple-return + setlocale / LC constants module
# hasattr surface, the time monotonic / perf_counter float-
# return monotonic-progress layer, and the weakref module
# hasattr surface layer.
#
# Surface in this fixture:
#   • pathlib — Path / PurePath / PurePosixPath hasattr +
#     type(Path("/tmp")).__name__ == "PosixPath";
#   • fnmatch — fnmatch / fnmatchcase / filter / translate
#     hasattr + fnmatch("foo.txt", "*.txt") / ?oo / filter /
#     fnmatchcase;
#   • sysconfig — get_platform / get_python_version /
#     get_paths / get_path_names / get_config_vars hasattr +
#     get_platform() / get_python_version() str-return;
#   • platform — system / machine / platform / python_version
#     / release / node hasattr + system() / machine() /
#     python_version() str-return;
#   • locale — getlocale / setlocale / LC_ALL / LC_CTYPE /
#     LC_NUMERIC hasattr + getlocale() tuple-return;
#   • time — monotonic / perf_counter float-return + monotonic-
#     progress contract;
#   • weakref — ref / proxy / WeakValueDictionary /
#     WeakKeyDictionary / WeakSet hasattr.
#
# Behavioral edges that DIVERGE on mamba (Path("/tmp/foo/bar.txt").
# name / .stem / .suffix / .parent / .parts return None,
# str(Path(p)) returns "<PosixPath instance>" not the path
# string, Path(p).is_absolute AttributeError, weakref.ref(obj)()
# returns None not the live obj, hasattr(platform, "version") /
# "architecture" False, hasattr(locale, "localeconv") False) are
# covered in the matching spec fixture
# `lang_pathlib_weakref_platform_silent`.
import pathlib
from pathlib import Path
import fnmatch
import sysconfig
import platform
import locale
import time
import weakref


_ledger: list[int] = []

# 1) pathlib — class identifier hasattr surface
assert hasattr(pathlib, "Path") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePath") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePosixPath") == True; _ledger.append(1)

# 2) pathlib — Path() constructor type
assert type(Path("/tmp")).__name__ == "PosixPath"; _ledger.append(1)

# 3) fnmatch — value contract
assert fnmatch.fnmatch("foo.txt", "*.txt") == True; _ledger.append(1)
assert fnmatch.fnmatch("foo", "?oo") == True; _ledger.append(1)
assert fnmatch.fnmatch("bar.py", "*.txt") == False; _ledger.append(1)
assert fnmatch.filter(["a.txt", "b.txt", "c.py"], "*.txt") == ["a.txt", "b.txt"]; _ledger.append(1)
assert fnmatch.fnmatchcase("Foo.txt", "*.txt") == True; _ledger.append(1)

# 4) fnmatch — module attribute hasattr surface
assert hasattr(fnmatch, "fnmatch") == True; _ledger.append(1)
assert hasattr(fnmatch, "fnmatchcase") == True; _ledger.append(1)
assert hasattr(fnmatch, "filter") == True; _ledger.append(1)
assert hasattr(fnmatch, "translate") == True; _ledger.append(1)

# 5) sysconfig — str-return value contracts + hasattr surface
assert isinstance(sysconfig.get_platform(), str); _ledger.append(1)
assert isinstance(sysconfig.get_python_version(), str); _ledger.append(1)
assert hasattr(sysconfig, "get_platform") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_python_version") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_paths") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_path_names") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_config_vars") == True; _ledger.append(1)

# 6) platform — str-return value contracts + hasattr surface
assert isinstance(platform.system(), str); _ledger.append(1)
assert isinstance(platform.machine(), str); _ledger.append(1)
assert isinstance(platform.python_version(), str); _ledger.append(1)
assert hasattr(platform, "system") == True; _ledger.append(1)
assert hasattr(platform, "machine") == True; _ledger.append(1)
assert hasattr(platform, "platform") == True; _ledger.append(1)
assert hasattr(platform, "python_version") == True; _ledger.append(1)
assert hasattr(platform, "release") == True; _ledger.append(1)
assert hasattr(platform, "node") == True; _ledger.append(1)

# 7) locale — tuple-return + hasattr surface
assert isinstance(locale.getlocale(), tuple); _ledger.append(1)
assert hasattr(locale, "getlocale") == True; _ledger.append(1)
assert hasattr(locale, "setlocale") == True; _ledger.append(1)
assert hasattr(locale, "LC_ALL") == True; _ledger.append(1)
assert hasattr(locale, "LC_CTYPE") == True; _ledger.append(1)
assert hasattr(locale, "LC_NUMERIC") == True; _ledger.append(1)

# 8) time — monotonic + perf_counter float-return + monotonic-progress
_t1 = time.monotonic()
_t2 = time.monotonic()
assert isinstance(_t1, float); _ledger.append(1)
assert _t2 >= _t1; _ledger.append(1)
_pc1 = time.perf_counter()
_pc2 = time.perf_counter()
assert isinstance(_pc1, float); _ledger.append(1)
assert _pc2 >= _pc1; _ledger.append(1)

# 9) weakref — module attribute hasattr surface
assert hasattr(weakref, "ref") == True; _ledger.append(1)
assert hasattr(weakref, "proxy") == True; _ledger.append(1)
assert hasattr(weakref, "WeakValueDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakKeyDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakSet") == True; _ledger.append(1)

# NB: Path(p).name / .stem / .suffix / .parent / .parts return
# None, str(Path(p)) returns "<PosixPath instance>" not the
# documented path string, Path(p).is_absolute AttributeError,
# weakref.ref(obj)() returns None not the live obj, hasattr(
# platform, "version") / "architecture" False, hasattr(locale,
# "localeconv") False — all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_pathlib_fnmatch_sysconfig_platform_time_value_ops {sum(_ledger)} asserts")
