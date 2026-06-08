# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_sys_sysconfig_pathlib_signal_silent"
# subject = "cpython321.lang_sys_sysconfig_pathlib_signal_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_sys_sysconfig_pathlib_signal_silent.py"
# status = "filled"
# ///
"""cpython321.lang_sys_sysconfig_pathlib_signal_silent: execute CPython 3.12 seed lang_sys_sysconfig_pathlib_signal_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the `sys` /
# `sysconfig` / `pathlib` / `signal` four-pack pinned to
# atomic 214: `sys` (the documented
# `hasattr(sys, "maxunicode") / "thread_info" == True`
# extended hasattr surface + the documented
# `type(sys.version_info).__name__ == "version_info"` /
# `type(sys.implementation).__name__ == "SimpleNamespace"`
# named-tuple / namespace identity value contract),
# `sysconfig` (the documented
# `hasattr(sysconfig, "get_preferred_scheme") /
# "parse_config_h" == True` extended hasattr surface),
# `pathlib` (the documented
# `str(pathlib.Path("/tmp/foo/bar.txt")) ==
# "/tmp/foo/bar.txt"` / `pathlib.Path("/tmp/foo/bar.txt")
# .name == "bar.txt"` /
# `pathlib.Path("/tmp/foo/bar.txt").stem == "bar"` /
# `pathlib.Path("/tmp/foo/bar.txt").suffix == ".txt"` /
# `str(pathlib.Path("/tmp/foo/bar.txt").parent) ==
# "/tmp/foo"` /
# `pathlib.Path("/tmp/foo/bar.txt").is_absolute() == True`
# Path value contract), and `signal` (the documented
# `type(signal.SIGINT).__name__ == "Signals"` /
# `type(signal.SIG_DFL).__name__ == "Handlers"` IntEnum
# type-identity value contract + the documented
# `str(signal.strsignal(2)).startswith("I")` strsignal-
# format value contract).
#
# Behavioral edges that CONFORM on mamba
# (sys `argv` / `path` / `modules` / `platform` /
# `version` / `version_info` / `maxsize` / `byteorder` /
# `executable` / `prefix` / `exec_prefix` / `base_prefix`
# / `base_exec_prefix` / `stdin` / `stdout` / `stderr` /
# `exit` / `getsizeof` / `getrecursionlimit` /
# `setrecursionlimit` / `intern` / `getrefcount` /
# `implementation` / `flags` / `float_info` / `int_info`
# / `hash_info` / `is_finalizing` hasattr surface +
# `type(sys.platform).__name__ == "str"` /
# `type(sys.version).__name__ == "str"` /
# `type(sys.maxsize).__name__ == "int"` /
# `sys.maxsize > 0` /
# `sys.byteorder in ("little", "big")` /
# `type(sys.argv).__name__ == "list"` /
# `type(sys.path).__name__ == "list"` /
# `type(sys.modules).__name__ == "dict"` /
# `sys.version_info.major == 3` /
# `sys.getrecursionlimit() > 0` introspection value
# contract, sysconfig `get_config_var` / `get_config_vars`
# / `get_paths` / `get_path` / `get_platform` /
# `get_python_version` / `get_path_names` /
# `get_scheme_names` / `get_default_scheme` hasattr
# surface + accessor types, pathlib full hasattr surface
# of `Path` / `PurePath` / `PurePosixPath` /
# `PureWindowsPath` / `PosixPath` / `WindowsPath`, signal
# full hasattr surface + `signal.SIGINT > 0` /
# `signal.SIGTERM > 0` integer-sentinel value contract)
# are covered in the matching pass fixture
# `test_sys_sysconfig_pathlib_subprocess_signal_value_ops`.
from typing import Any
import sys as _sys_mod
import sysconfig as _sysconfig_mod
import pathlib as _pathlib_mod
import signal as _signal_mod

sys: Any = _sys_mod
sysconfig: Any = _sysconfig_mod
pathlib: Any = _pathlib_mod
signal: Any = _signal_mod


_ledger: list[int] = []

# 1) sys — extended module hasattr surface
#    (mamba: maxunicode / thread_info all False)
assert hasattr(sys, "maxunicode") == True; _ledger.append(1)
assert hasattr(sys, "thread_info") == True; _ledger.append(1)

# 2) sys — named-tuple / namespace identity value contract
#    (mamba: type(sys.version_info).__name__ collapses to
#    "dict" + type(sys.implementation).__name__ collapses
#    to "dict")
assert type(sys.version_info).__name__ == "version_info"; _ledger.append(1)
assert type(sys.implementation).__name__ == "SimpleNamespace"; _ledger.append(1)

# 3) sysconfig — extended module hasattr surface
#    (mamba: get_preferred_scheme / parse_config_h all
#    False)
assert hasattr(sysconfig, "get_preferred_scheme") == True; _ledger.append(1)
assert hasattr(sysconfig, "parse_config_h") == True; _ledger.append(1)

# 4) pathlib — Path value contract
#    (mamba: str(Path("/tmp/foo/bar.txt")) collapses to
#    "<PosixPath instance>" + .name / .stem / .suffix /
#    .parent all return None + .is_absolute() method
#    unavailable)
_p = pathlib.Path("/tmp/foo/bar.txt")
assert str(_p) == "/tmp/foo/bar.txt"; _ledger.append(1)
assert _p.name == "bar.txt"; _ledger.append(1)
assert _p.stem == "bar"; _ledger.append(1)
assert _p.suffix == ".txt"; _ledger.append(1)
assert str(_p.parent) == "/tmp/foo"; _ledger.append(1)
assert _p.is_absolute() == True; _ledger.append(1)

# 5) signal — IntEnum type-identity value contract
#    (mamba: type(signal.SIGINT).__name__ "Signals"
#    collapses to "int" + type(signal.SIG_DFL).__name__
#    "Handlers" collapses to "int")
assert type(signal.SIGINT).__name__ == "Signals"; _ledger.append(1)
assert type(signal.SIGTERM).__name__ == "Signals"; _ledger.append(1)
assert type(signal.SIG_DFL).__name__ == "Handlers"; _ledger.append(1)

# 6) signal — strsignal-format value contract
#    (mamba: str(signal.strsignal(2)).startswith("I")
#    collapses to False — strsignal returns something that
#    does not start with "I")
assert str(signal.strsignal(2) or "").startswith(("Int", "I")); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_sys_sysconfig_pathlib_signal_silent {sum(_ledger)} asserts")
