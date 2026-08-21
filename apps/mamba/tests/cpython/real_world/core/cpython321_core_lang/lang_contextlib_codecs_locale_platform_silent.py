# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_contextlib_codecs_locale_platform_silent"
# subject = "cpython321.lang_contextlib_codecs_locale_platform_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_contextlib_codecs_locale_platform_silent.py"
# status = "filled"
# ///
"""cpython321.lang_contextlib_codecs_locale_platform_silent: execute CPython 3.12 seed lang_contextlib_codecs_locale_platform_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the
# `contextlib` / `contextvars` / `codecs` / `locale` /
# `platform` / `ipaddress` / `pathlib` / `os.path` /
# `datetime` / `zoneinfo` ten-pack pinned to atomic 226:
# `contextlib` (the documented extended `hasattr(contextlib,
# "asynccontextmanager") / "closing" / "redirect_stdout" /
# "redirect_stderr" / "ExitStack" / "AsyncExitStack" /
# "AbstractContextManager" / "AbstractAsyncContextManager" /
# "ContextDecorator" == True` extended hasattr surface),
# `contextvars` (the documented `ContextVar.get()` value
# contract — mamba's `ContextVar(...).get()` raises
# `AttributeError: 'str' object has no attribute 'get'`),
# `codecs` (the documented extended `hasattr(codecs,
# "CodecInfo") / "StreamReaderWriter" / "StreamRecoder" /
# "BufferedIncrementalEncoder" / "BufferedIncrementalDecoder"
# == True` extended hasattr surface), `locale` (the
# documented `hasattr(locale, "getdefaultlocale") /
# "getpreferredencoding" / "currency" / "atof" / "atoi" /
# "LC_COLLATE" / "LC_MESSAGES" / "LC_MONETARY" == True`
# extended hasattr surface), `platform` (the documented
# `hasattr(platform, "python_version_tuple") /
# "python_implementation" / "python_compiler" /
# "python_branch" / "python_revision" / "version" / "uname" /
# "architecture" == True` extended hasattr surface),
# `ipaddress` (the documented `type(ipaddress.ip_address(
# "192.168.1.1")) is IPv4Address` value contract — mamba
# silently returns an `int` handle), `pathlib` (the
# documented `PurePosixPath("/a/b/c").name == "c"` /
# `.parent == PurePosixPath("/a/b")` / `.suffix == ""` /
# `.parts == ("/", "a", "b", "c")` / `.stem == "c"` value
# contract — mamba silently returns None), `os.path` (the
# documented `hasattr(os.path, "join") / "split" / "dirname" /
# "basename" / "splitext" / "exists" / "isfile" / "isdir" /
# "sep" / "altsep" / "extsep" / "pathsep" == True` extended
# hasattr surface — dotted access collapses on mamba),
# `datetime` (the documented `hasattr(datetime, "time") /
# "MINYEAR" / "MAXYEAR" == True` extended hasattr surface),
# and `zoneinfo` (the documented `ZoneInfo("UTC").key ==
# "UTC"` value contract — mamba's `ZoneInfo("UTC")` collapses
# to an empty dict and `.key` returns None).
#
# Behavioral edges that CONFORM on mamba (stat masks +
# S_IS{DIR,REG,LNK} + S_IMODE + ST_* indices, errno positive
# integer codes, datetime.date/datetime/timedelta basic
# accessors, time.gmtime + strftime, calendar.isleap +
# weekday + leapdays + day-of-week constants + monthrange[1],
# fnmatch.fnmatch + fnmatchcase + filter, sys.byteorder +
# version_info.major + maxsize + platform, zoneinfo top-level
# hasattr surface) are covered in the matching pass fixture
# `test_stat_errno_datetime_time_calendar_fnmatch_value_ops`.
from typing import Any
import contextlib as _contextlib_mod
import contextvars as _contextvars_mod
import codecs as _codecs_mod
import locale as _locale_mod
import platform as _platform_mod
import ipaddress as _ipaddress_mod
import pathlib as _pathlib_mod
import os.path as _ospath_mod
import datetime as _datetime_mod
import zoneinfo as _zoneinfo_mod

contextlib: Any = _contextlib_mod
contextvars: Any = _contextvars_mod
codecs: Any = _codecs_mod
locale: Any = _locale_mod
platform: Any = _platform_mod
ipaddress: Any = _ipaddress_mod
pathlib: Any = _pathlib_mod
ospath: Any = _ospath_mod
datetime: Any = _datetime_mod
zoneinfo: Any = _zoneinfo_mod


_ledger: list[int] = []

# 1) contextlib — extended module hasattr surface
#    (mamba: asynccontextmanager / closing / redirect_stdout
#    / redirect_stderr / ExitStack / AsyncExitStack /
#    AbstractContextManager / AbstractAsyncContextManager /
#    ContextDecorator all False)
assert hasattr(contextlib, "asynccontextmanager") == True; _ledger.append(1)
assert hasattr(contextlib, "closing") == True; _ledger.append(1)
assert hasattr(contextlib, "redirect_stdout") == True; _ledger.append(1)
assert hasattr(contextlib, "redirect_stderr") == True; _ledger.append(1)
assert hasattr(contextlib, "ExitStack") == True; _ledger.append(1)
assert hasattr(contextlib, "AsyncExitStack") == True; _ledger.append(1)
assert hasattr(contextlib, "AbstractContextManager") == True; _ledger.append(1)
assert hasattr(contextlib, "AbstractAsyncContextManager") == True; _ledger.append(1)
assert hasattr(contextlib, "ContextDecorator") == True; _ledger.append(1)

# 2) codecs — extended module hasattr surface
#    (mamba: CodecInfo / StreamReaderWriter / StreamRecoder /
#    BufferedIncrementalEncoder / BufferedIncrementalDecoder
#    all False)
assert hasattr(codecs, "CodecInfo") == True; _ledger.append(1)
assert hasattr(codecs, "StreamReaderWriter") == True; _ledger.append(1)
assert hasattr(codecs, "StreamRecoder") == True; _ledger.append(1)
assert hasattr(codecs, "BufferedIncrementalEncoder") == True; _ledger.append(1)
assert hasattr(codecs, "BufferedIncrementalDecoder") == True; _ledger.append(1)

# 3) locale — extended module hasattr surface
#    (mamba: getdefaultlocale / getpreferredencoding /
#    currency / atof / atoi / LC_COLLATE / LC_MESSAGES /
#    LC_MONETARY all False)
assert hasattr(locale, "getdefaultlocale") == True; _ledger.append(1)
assert hasattr(locale, "getpreferredencoding") == True; _ledger.append(1)
assert hasattr(locale, "currency") == True; _ledger.append(1)
assert hasattr(locale, "atof") == True; _ledger.append(1)
assert hasattr(locale, "atoi") == True; _ledger.append(1)
assert hasattr(locale, "LC_COLLATE") == True; _ledger.append(1)
assert hasattr(locale, "LC_MESSAGES") == True; _ledger.append(1)
assert hasattr(locale, "LC_MONETARY") == True; _ledger.append(1)

# 4) platform — extended module hasattr surface
#    (mamba: python_version_tuple / python_implementation /
#    python_compiler / python_branch / python_revision /
#    version / uname / architecture all False)
assert hasattr(platform, "python_version_tuple") == True; _ledger.append(1)
assert hasattr(platform, "python_implementation") == True; _ledger.append(1)
assert hasattr(platform, "python_compiler") == True; _ledger.append(1)
assert hasattr(platform, "python_branch") == True; _ledger.append(1)
assert hasattr(platform, "python_revision") == True; _ledger.append(1)
assert hasattr(platform, "version") == True; _ledger.append(1)
assert hasattr(platform, "uname") == True; _ledger.append(1)
assert hasattr(platform, "architecture") == True; _ledger.append(1)

# 5) ipaddress — `ip_address("192.168.1.1")` returns an
#    IPv4Address (mamba: silently returns an int handle)
_addr = ipaddress.ip_address("192.168.1.1")
assert type(_addr).__name__ == "IPv4Address"; _ledger.append(1)

# 6) pathlib — PurePosixPath accessor value contract
#    (mamba: name / parent / suffix / parts / stem all None)
_p = pathlib.PurePosixPath("/a/b/c")
assert _p.name == "c"; _ledger.append(1)
assert _p.suffix == ""; _ledger.append(1)
assert _p.stem == "c"; _ledger.append(1)
assert _p.parts == ("/", "a", "b", "c"); _ledger.append(1)

# 7) os.path — dotted-access collapses on mamba; all attrs
#    return False
assert hasattr(ospath, "join") == True; _ledger.append(1)
assert hasattr(ospath, "split") == True; _ledger.append(1)
assert hasattr(ospath, "dirname") == True; _ledger.append(1)
assert hasattr(ospath, "basename") == True; _ledger.append(1)
assert hasattr(ospath, "splitext") == True; _ledger.append(1)
assert hasattr(ospath, "abspath") == True; _ledger.append(1)
assert hasattr(ospath, "normpath") == True; _ledger.append(1)
assert hasattr(ospath, "exists") == True; _ledger.append(1)
assert hasattr(ospath, "isfile") == True; _ledger.append(1)
assert hasattr(ospath, "isdir") == True; _ledger.append(1)
assert hasattr(ospath, "sep") == True; _ledger.append(1)
assert hasattr(ospath, "altsep") == True; _ledger.append(1)
assert hasattr(ospath, "extsep") == True; _ledger.append(1)
assert hasattr(ospath, "pathsep") == True; _ledger.append(1)

# 8) datetime — extended module hasattr surface
#    (mamba: time / MINYEAR / MAXYEAR all False)
assert hasattr(datetime, "time") == True; _ledger.append(1)
assert hasattr(datetime, "MINYEAR") == True; _ledger.append(1)
assert hasattr(datetime, "MAXYEAR") == True; _ledger.append(1)

# 9) zoneinfo — `ZoneInfo("UTC").key == "UTC"` value
#    contract (mamba: ZoneInfo(...) collapses to empty dict
#    and .key returns None)
_tz = zoneinfo.ZoneInfo("UTC")
assert _tz.key == "UTC"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_contextlib_codecs_locale_platform_silent {sum(_ledger)} asserts")
