# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_os_sys_errno_stat_tempfile_value_ops"
# subject = "cpython321.test_os_sys_errno_stat_tempfile_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_os_sys_errno_stat_tempfile_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_os_sys_errno_stat_tempfile_value_ops: execute CPython 3.12 seed test_os_sys_errno_stat_tempfile_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `os` / `sys` / `errno` / `stat` / `tempfile` five-pack pinned
# to atomic 193: `os` (the documented partial module-level
# helper hasattr surface — `getcwd` / `listdir` / `mkdir` /
# `makedirs` / `remove` / `rmdir` / `rename` / `stat` /
# `environ` / `path` / `sep` / `linesep` / `pathsep` / `name` /
# `getenv` / `getpid` / `getppid` / `walk` / `scandir` /
# `fspath` / `umask` / `system` + the documented os.sep / linesep
# / pathsep / name / getcwd / getpid value-and-type contract),
# `sys` (the documented full module-level helper hasattr
# surface — `version` / `version_info` / `platform` /
# `executable` / `argv` / `path` / `modules` / `stdin` /
# `stdout` / `stderr` / `maxsize` / `byteorder` / `exit` /
# `exc_info` / `getrecursionlimit` / `setrecursionlimit` /
# `intern` / `getsizeof` / `implementation` + the documented
# sys.version / platform / argv / path / modules / maxsize /
# byteorder type contract + the documented
# sys.getrecursionlimit value contract), `errno` (the
# documented full module-level helper hasattr surface —
# `ENOENT` / `EEXIST` / `EACCES` / `EPERM` / `EIO` / `EAGAIN`
# / `EINTR` / `EINVAL` / `EISDIR` / `ENOTDIR` / `EBUSY` /
# `ECONNREFUSED` / `errorcode` + the documented errno.ENOENT
# == 2 / EACCES == 13 / EPERM == 1 / errorcode[2] == "ENOENT"
# value contract), `stat` (the documented full module-level
# helper hasattr surface — `S_IFDIR` / `S_IFREG` / `S_IFLNK`
# / `S_IRUSR` / `S_IWUSR` / `S_IXUSR` / `S_ISDIR` / `S_ISREG`
# / `S_ISLNK` / `filemode` + the documented stat.S_IFDIR ==
# 0o040000 / S_IFREG == 0o100000 / S_IRUSR == 0o400 value
# contract), and `tempfile` (the documented full module-level
# helper hasattr surface — `TemporaryFile` /
# `NamedTemporaryFile` / `TemporaryDirectory` / `mkstemp` /
# `mkdtemp` / `gettempdir` / `gettempprefix` /
# `SpooledTemporaryFile` + the documented tempfile.gettempdir
# / gettempprefix str return-type contract).
#
# The matching subset between mamba and CPython is the partial
# `os` module hasattr surface (getcwd / listdir / mkdir /
# makedirs / remove / rmdir / rename / stat / environ / path /
# sep / linesep / pathsep / name / getenv / getpid / getppid /
# walk / scandir / fspath / umask / system — `putenv` /
# `unsetenv` / `chdir` / `fork` DIVERGE) + the value-and-type
# layer, the full `sys` module hasattr surface + the type
# layer (the `type(sys.version_info).__name__ ==
# "version_info"` class-identity layer DIVERGES — mamba
# returns "dict"), the full `errno` module hasattr surface +
# the integer-value layer + the errorcode[2] string-value
# layer, the full `stat` module hasattr surface + the
# integer-value layer, and the full `tempfile` module hasattr
# surface + the str return-type layer.
#
# Surface in this fixture:
#   • os — partial module hasattr surface (getcwd / listdir
#     / mkdir / makedirs / remove / rmdir / rename / stat /
#     environ / path / sep / linesep / pathsep / name /
#     getenv / getpid / getppid / walk / scandir / fspath /
#     umask / system);
#   • os.sep / linesep / pathsep / name — value contract;
#   • os.getcwd / getpid — type-and-positive value contract;
#   • sys — full module hasattr surface (version /
#     version_info / platform / executable / argv / path /
#     modules / stdin / stdout / stderr / maxsize / byteorder
#     / exit / exc_info / getrecursionlimit /
#     setrecursionlimit / intern / getsizeof /
#     implementation);
#   • sys.version / platform / argv / path / modules /
#     maxsize / byteorder — type contract;
#   • sys.getrecursionlimit — integer-value contract;
#   • errno — full module hasattr surface (ENOENT / EEXIST
#     / EACCES / EPERM / EIO / EAGAIN / EINTR / EINVAL /
#     EISDIR / ENOTDIR / EBUSY / ECONNREFUSED / errorcode);
#   • errno.ENOENT / EACCES / EPERM — integer-value
#     contract;
#   • errno.errorcode[2] — string-value contract;
#   • stat — full module hasattr surface (S_IFDIR / S_IFREG
#     / S_IFLNK / S_IRUSR / S_IWUSR / S_IXUSR / S_ISDIR /
#     S_ISREG / S_ISLNK / filemode);
#   • stat.S_IFDIR / S_IFREG / S_IRUSR — integer-value
#     contract;
#   • tempfile — full module hasattr surface (TemporaryFile
#     / NamedTemporaryFile / TemporaryDirectory / mkstemp /
#     mkdtemp / gettempdir / gettempprefix /
#     SpooledTemporaryFile);
#   • tempfile.gettempdir / gettempprefix — str return-type
#     contract.
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(os, "putenv") / "unsetenv" / "chdir" / "fork" all
# False, type(sys.version_info).__name__ returns "dict" not
# "version_info" — the named-tuple class is rebound to a
# dict placeholder) are covered in the matching spec fixture
# `lang_os_sys_versioninfo_silent`.
import os
import sys
import errno
import stat
import tempfile


_ledger: list[int] = []

# 1) os — partial module hasattr surface
#    (putenv / unsetenv / chdir / fork DIVERGE — moved to
#    spec fixture)
assert hasattr(os, "getcwd") == True; _ledger.append(1)
assert hasattr(os, "listdir") == True; _ledger.append(1)
assert hasattr(os, "mkdir") == True; _ledger.append(1)
assert hasattr(os, "makedirs") == True; _ledger.append(1)
assert hasattr(os, "remove") == True; _ledger.append(1)
assert hasattr(os, "rmdir") == True; _ledger.append(1)
assert hasattr(os, "rename") == True; _ledger.append(1)
assert hasattr(os, "stat") == True; _ledger.append(1)
assert hasattr(os, "environ") == True; _ledger.append(1)
assert hasattr(os, "path") == True; _ledger.append(1)
assert hasattr(os, "sep") == True; _ledger.append(1)
assert hasattr(os, "linesep") == True; _ledger.append(1)
assert hasattr(os, "pathsep") == True; _ledger.append(1)
assert hasattr(os, "name") == True; _ledger.append(1)
assert hasattr(os, "getenv") == True; _ledger.append(1)
assert hasattr(os, "getpid") == True; _ledger.append(1)
assert hasattr(os, "getppid") == True; _ledger.append(1)
assert hasattr(os, "walk") == True; _ledger.append(1)
assert hasattr(os, "scandir") == True; _ledger.append(1)
assert hasattr(os, "fspath") == True; _ledger.append(1)
assert hasattr(os, "umask") == True; _ledger.append(1)
assert hasattr(os, "system") == True; _ledger.append(1)

# 2) os — value contract
assert os.sep == "/"; _ledger.append(1)
assert os.linesep == "\n"; _ledger.append(1)
assert os.pathsep == ":"; _ledger.append(1)
assert os.name in {"posix", "nt"}; _ledger.append(1)
assert type(os.getcwd()).__name__ == "str"; _ledger.append(1)
assert type(os.getpid()).__name__ == "int"; _ledger.append(1)
assert os.getpid() > 0; _ledger.append(1)

# 3) sys — full module hasattr surface
assert hasattr(sys, "version") == True; _ledger.append(1)
assert hasattr(sys, "version_info") == True; _ledger.append(1)
assert hasattr(sys, "platform") == True; _ledger.append(1)
assert hasattr(sys, "executable") == True; _ledger.append(1)
assert hasattr(sys, "argv") == True; _ledger.append(1)
assert hasattr(sys, "path") == True; _ledger.append(1)
assert hasattr(sys, "modules") == True; _ledger.append(1)
assert hasattr(sys, "stdin") == True; _ledger.append(1)
assert hasattr(sys, "stdout") == True; _ledger.append(1)
assert hasattr(sys, "stderr") == True; _ledger.append(1)
assert hasattr(sys, "maxsize") == True; _ledger.append(1)
assert hasattr(sys, "byteorder") == True; _ledger.append(1)
assert hasattr(sys, "exit") == True; _ledger.append(1)
assert hasattr(sys, "exc_info") == True; _ledger.append(1)
assert hasattr(sys, "getrecursionlimit") == True; _ledger.append(1)
assert hasattr(sys, "setrecursionlimit") == True; _ledger.append(1)
assert hasattr(sys, "intern") == True; _ledger.append(1)
assert hasattr(sys, "getsizeof") == True; _ledger.append(1)
assert hasattr(sys, "implementation") == True; _ledger.append(1)

# 4) sys — type contract
#    (sys.version_info class-identity DIVERGES — moved to
#    spec fixture)
assert type(sys.version).__name__ == "str"; _ledger.append(1)
assert type(sys.platform).__name__ == "str"; _ledger.append(1)
assert type(sys.argv).__name__ == "list"; _ledger.append(1)
assert type(sys.path).__name__ == "list"; _ledger.append(1)
assert type(sys.modules).__name__ == "dict"; _ledger.append(1)
assert type(sys.maxsize).__name__ == "int"; _ledger.append(1)
assert sys.maxsize > 0; _ledger.append(1)
assert type(sys.byteorder).__name__ == "str"; _ledger.append(1)
assert sys.byteorder in {"big", "little"}; _ledger.append(1)

# 5) sys.getrecursionlimit — integer-value contract
assert sys.getrecursionlimit() == 1000; _ledger.append(1)

# 6) errno — full module hasattr surface
assert hasattr(errno, "ENOENT") == True; _ledger.append(1)
assert hasattr(errno, "EEXIST") == True; _ledger.append(1)
assert hasattr(errno, "EACCES") == True; _ledger.append(1)
assert hasattr(errno, "EPERM") == True; _ledger.append(1)
assert hasattr(errno, "EIO") == True; _ledger.append(1)
assert hasattr(errno, "EAGAIN") == True; _ledger.append(1)
assert hasattr(errno, "EINTR") == True; _ledger.append(1)
assert hasattr(errno, "EINVAL") == True; _ledger.append(1)
assert hasattr(errno, "EISDIR") == True; _ledger.append(1)
assert hasattr(errno, "ENOTDIR") == True; _ledger.append(1)
assert hasattr(errno, "EBUSY") == True; _ledger.append(1)
assert hasattr(errno, "ECONNREFUSED") == True; _ledger.append(1)
assert hasattr(errno, "errorcode") == True; _ledger.append(1)

# 7) errno — integer-value contract
assert errno.ENOENT == 2; _ledger.append(1)
assert errno.EACCES == 13; _ledger.append(1)
assert errno.EPERM == 1; _ledger.append(1)
assert errno.errorcode[2] == "ENOENT"; _ledger.append(1)

# 8) stat — full module hasattr surface
assert hasattr(stat, "S_IFDIR") == True; _ledger.append(1)
assert hasattr(stat, "S_IFREG") == True; _ledger.append(1)
assert hasattr(stat, "S_IFLNK") == True; _ledger.append(1)
assert hasattr(stat, "S_IRUSR") == True; _ledger.append(1)
assert hasattr(stat, "S_IWUSR") == True; _ledger.append(1)
assert hasattr(stat, "S_IXUSR") == True; _ledger.append(1)
assert hasattr(stat, "S_ISDIR") == True; _ledger.append(1)
assert hasattr(stat, "S_ISREG") == True; _ledger.append(1)
assert hasattr(stat, "S_ISLNK") == True; _ledger.append(1)
assert hasattr(stat, "filemode") == True; _ledger.append(1)

# 9) stat — integer-value contract
assert stat.S_IFDIR == 0o040000; _ledger.append(1)
assert stat.S_IFREG == 0o100000; _ledger.append(1)
assert stat.S_IRUSR == 0o400; _ledger.append(1)

# 10) tempfile — full module hasattr surface
assert hasattr(tempfile, "TemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "NamedTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "TemporaryDirectory") == True; _ledger.append(1)
assert hasattr(tempfile, "mkstemp") == True; _ledger.append(1)
assert hasattr(tempfile, "mkdtemp") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempdir") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempprefix") == True; _ledger.append(1)
assert hasattr(tempfile, "SpooledTemporaryFile") == True; _ledger.append(1)

# 11) tempfile — str return-type contract
assert type(tempfile.gettempdir()).__name__ == "str"; _ledger.append(1)
assert type(tempfile.gettempprefix()).__name__ == "str"; _ledger.append(1)
assert len(tempfile.gettempdir()) > 0; _ledger.append(1)

# NB: hasattr(os, "putenv") / "unsetenv" / "chdir" / "fork"
# all False on mamba, type(sys.version_info).__name__ returns
# "dict" not "version_info" on mamba — the named-tuple class
# is rebound to a dict placeholder — all DIVERGE on mamba —
# moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_os_sys_errno_stat_tempfile_value_ops {sum(_ledger)} asserts")
