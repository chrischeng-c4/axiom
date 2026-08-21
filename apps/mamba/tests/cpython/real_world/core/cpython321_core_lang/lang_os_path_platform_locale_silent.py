# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_os_path_platform_locale_silent"
# subject = "cpython321.lang_os_path_platform_locale_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_os_path_platform_locale_silent.py"
# status = "filled"
# ///
"""cpython321.lang_os_path_platform_locale_silent: execute CPython 3.12 seed lang_os_path_platform_locale_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the `os` / `os.path` /
# `platform` / `locale` / `linecache` five-pack pinned to
# atomic 219: `os` (the documented
# `hasattr(os, "chdir") / "removedirs" / "unlink" / "kill"
# / "fork" / "wait" / "popen" / "open" / "close" / "read" /
# "write" / "lseek" / "fdopen" / "pipe" / "dup" / "dup2" /
# "fsync" == True` extended hasattr surface), `os.path`
# (the documented
# `hasattr(os.path, "islink") / "expandvars" / "normpath"
# / "relpath" / "commonpath" / "commonprefix" / "isabs" /
# "samefile" / "getmtime" / "getatime" / "getctime" ==
# True` extended hasattr surface), `platform` (the
# documented
# `hasattr(platform, "version") / "python_implementation"
# / "python_compiler" / "python_build" / "python_branch"
# / "python_revision" / "architecture" / "uname" == True`
# extended hasattr surface), `locale` (the documented
# `hasattr(locale, "LC_COLLATE") / "LC_MONETARY" /
# "localeconv" / "getdefaultlocale" /
# "getpreferredencoding" / "atof" / "atoi" / "currency" /
# "Error" == True` extended hasattr surface), and
# `linecache` (the documented
# `hasattr(linecache, "lazycache") == True` extended
# hasattr surface).
#
# Behavioral edges that CONFORM on mamba
# (os `getcwd` / `listdir` / `mkdir` / `makedirs` /
# `rmdir` / `remove` / `rename` / `stat` / `lstat` /
# `getenv` / `environ` / `name` / `sep` / `pathsep` /
# `linesep` / `curdir` / `pardir` / `altsep` / `extsep`
# / `devnull` / `getpid` / `getppid` / `getlogin` /
# `system` / `isatty` / `umask` / `urandom` /
# `cpu_count` / `scandir` / `walk` / `path` hasattr +
# runtime-introspection value contract, os.path `join`
# / `split` / `splitext` / `exists` / `isfile` / `isdir`
# / `abspath` / `basename` / `dirname` / `expanduser` /
# `getsize` hasattr + path-manipulation value contract,
# stat full hasattr surface, platform `system` /
# `release` / `machine` / `processor` / `node` /
# `platform` / `python_version` hasattr + system-
# introspection value contract, locale `getlocale` /
# `setlocale` / `LC_ALL` / `LC_CTYPE` / `LC_NUMERIC` /
# `LC_TIME` / `format_string` hasattr, glob full hasattr
# + match-and-escape value contract, fnmatch full
# hasattr + match-and-filter value contract, linecache
# `getline` / `getlines` / `checkcache` / `clearcache`
# hasattr, filecmp full hasattr) are covered in the
# matching pass fixture
# `test_os_path_stat_platform_glob_fnmatch_value_ops`.
from typing import Any
import os as _os_mod
import os.path as _op_mod
import platform as _platform_mod
import locale as _locale_mod
import linecache as _linecache_mod

os: Any = _os_mod
op: Any = _op_mod
platform: Any = _platform_mod
locale: Any = _locale_mod
linecache: Any = _linecache_mod


_ledger: list[int] = []

# 1) os — extended module hasattr surface
#    (mamba: chdir / removedirs / unlink / kill / fork /
#    wait / popen / open / close / read / write / lseek /
#    fdopen / pipe / dup / dup2 / fsync all False)
assert hasattr(os, "chdir") == True; _ledger.append(1)
assert hasattr(os, "removedirs") == True; _ledger.append(1)
assert hasattr(os, "unlink") == True; _ledger.append(1)
assert hasattr(os, "kill") == True; _ledger.append(1)
assert hasattr(os, "fork") == True; _ledger.append(1)
assert hasattr(os, "wait") == True; _ledger.append(1)
assert hasattr(os, "popen") == True; _ledger.append(1)
assert hasattr(os, "open") == True; _ledger.append(1)
assert hasattr(os, "close") == True; _ledger.append(1)
assert hasattr(os, "read") == True; _ledger.append(1)
assert hasattr(os, "write") == True; _ledger.append(1)
assert hasattr(os, "lseek") == True; _ledger.append(1)
assert hasattr(os, "fdopen") == True; _ledger.append(1)
assert hasattr(os, "pipe") == True; _ledger.append(1)
assert hasattr(os, "dup") == True; _ledger.append(1)
assert hasattr(os, "dup2") == True; _ledger.append(1)
assert hasattr(os, "fsync") == True; _ledger.append(1)

# 2) os.path — extended module hasattr surface
#    (mamba: islink / expandvars / normpath / relpath /
#    commonpath / commonprefix / isabs / samefile /
#    getmtime / getatime / getctime all False)
assert hasattr(op, "islink") == True; _ledger.append(1)
assert hasattr(op, "expandvars") == True; _ledger.append(1)
assert hasattr(op, "normpath") == True; _ledger.append(1)
assert hasattr(op, "relpath") == True; _ledger.append(1)
assert hasattr(op, "commonpath") == True; _ledger.append(1)
assert hasattr(op, "commonprefix") == True; _ledger.append(1)
assert hasattr(op, "isabs") == True; _ledger.append(1)
assert hasattr(op, "samefile") == True; _ledger.append(1)
assert hasattr(op, "getmtime") == True; _ledger.append(1)
assert hasattr(op, "getatime") == True; _ledger.append(1)
assert hasattr(op, "getctime") == True; _ledger.append(1)

# 3) platform — extended module hasattr surface
#    (mamba: version / python_implementation /
#    python_compiler / python_build / python_branch /
#    python_revision / architecture / uname all False)
assert hasattr(platform, "version") == True; _ledger.append(1)
assert hasattr(platform, "python_implementation") == True; _ledger.append(1)
assert hasattr(platform, "python_compiler") == True; _ledger.append(1)
assert hasattr(platform, "python_build") == True; _ledger.append(1)
assert hasattr(platform, "python_branch") == True; _ledger.append(1)
assert hasattr(platform, "python_revision") == True; _ledger.append(1)
assert hasattr(platform, "architecture") == True; _ledger.append(1)
assert hasattr(platform, "uname") == True; _ledger.append(1)

# 4) locale — extended module hasattr surface
#    (mamba: LC_COLLATE / LC_MONETARY / localeconv /
#    getdefaultlocale / getpreferredencoding / atof /
#    atoi / currency / Error all False)
assert hasattr(locale, "LC_COLLATE") == True; _ledger.append(1)
assert hasattr(locale, "LC_MONETARY") == True; _ledger.append(1)
assert hasattr(locale, "localeconv") == True; _ledger.append(1)
assert hasattr(locale, "getdefaultlocale") == True; _ledger.append(1)
assert hasattr(locale, "getpreferredencoding") == True; _ledger.append(1)
assert hasattr(locale, "atof") == True; _ledger.append(1)
assert hasattr(locale, "atoi") == True; _ledger.append(1)
assert hasattr(locale, "currency") == True; _ledger.append(1)
assert hasattr(locale, "Error") == True; _ledger.append(1)

# 5) linecache — extended module hasattr surface
#    (mamba: lazycache False even though getline /
#    getlines / checkcache / clearcache are True)
assert hasattr(linecache, "lazycache") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_os_path_platform_locale_silent {sum(_ledger)} asserts")
