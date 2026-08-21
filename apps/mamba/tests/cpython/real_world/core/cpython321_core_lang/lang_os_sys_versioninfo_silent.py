# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_os_sys_versioninfo_silent"
# subject = "cpython321.lang_os_sys_versioninfo_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_os_sys_versioninfo_silent.py"
# status = "filled"
# ///
"""cpython321.lang_os_sys_versioninfo_silent: execute CPython 3.12 seed lang_os_sys_versioninfo_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# `os` extended module-helper surface + `sys.version_info` named-
# tuple class identity pinned by atomic 193: `os` (the documented
# `putenv` / `unsetenv` / `chdir` / `fork` extended function
# identifier surface) and `sys` (the documented `version_info`
# named-tuple class identity contract — `type(sys.version_info)
# .__name__ == "version_info"` on CPython; mamba collapses to a
# `dict` placeholder).
#
# The matching subset (partial os hasattr + sep/linesep/pathsep/
# name/getcwd/getpid value layer, full sys hasattr + type
# contracts + getrecursionlimit value, full errno + ENOENT/
# EACCES/EPERM/errorcode value layer, full stat + S_IFDIR/
# S_IFREG/S_IRUSR value layer, full tempfile + gettempdir/
# gettempprefix value layer) is covered by
# `test_os_sys_errno_stat_tempfile_value_ops`; this fixture pins
# the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(os, "putenv") is True — documented function
#     identifier (mamba: False);
#   • hasattr(os, "unsetenv") is True — documented function
#     identifier (mamba: False);
#   • hasattr(os, "chdir") is True — documented function
#     identifier (mamba: False);
#   • hasattr(os, "fork") is True — documented POSIX-only
#     function identifier (mamba: False);
#   • type(sys.version_info).__name__ == "version_info" —
#     documented named-tuple class identity contract
#     (mamba: returns "dict").
import os as _os_mod
import sys as _sys_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# function / named-tuple class identity behavior that mamba's
# bundled type stubs do not surface accurately.
os: Any = _os_mod
sys: Any = _sys_mod


_ledger: list[int] = []

# 1) os — extended function identifier surface
assert hasattr(os, "putenv") == True; _ledger.append(1)
assert hasattr(os, "unsetenv") == True; _ledger.append(1)
assert hasattr(os, "chdir") == True; _ledger.append(1)
assert hasattr(os, "fork") == True; _ledger.append(1)

# 2) sys.version_info — named-tuple class identity contract
assert type(sys.version_info).__name__ == "version_info"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_os_sys_versioninfo_silent {sum(_ledger)} asserts")
