# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_os_path_stat_platform_glob_fnmatch_value_ops"
# subject = "cpython321.test_os_path_stat_platform_glob_fnmatch_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_os_path_stat_platform_glob_fnmatch_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_os_path_stat_platform_glob_fnmatch_value_ops: execute CPython 3.12 seed test_os_path_stat_platform_glob_fnmatch_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `os` / `os.path` / `stat` / `platform` / `locale` / `glob`
# / `fnmatch` / `linecache` / `filecmp` nine-pack pinned to
# atomic 219: `os` (the documented partial module-level
# helper / sentinel identifier hasattr surface — `getcwd`
# / `listdir` / `mkdir` / `makedirs` / `rmdir` / `remove`
# / `rename` / `stat` / `lstat` / `getenv` / `environ` /
# `name` / `sep` / `pathsep` / `linesep` / `curdir` /
# `pardir` / `altsep` / `extsep` / `devnull` / `getpid` /
# `getppid` / `getlogin` / `system` / `isatty` / `umask`
# / `urandom` / `cpu_count` / `scandir` / `walk` / `path`
# + the documented `type(os.name).__name__ == "str"` /
# `type(os.sep).__name__ == "str"` /
# `type(os.getcwd()).__name__ == "str"` /
# `len(os.getcwd()) > 0` / `os.getpid() > 0` runtime-
# introspection value contract), `os.path` (the
# documented partial module-level helper identifier
# hasattr surface — `join` / `split` / `splitext` /
# `exists` / `isfile` / `isdir` / `abspath` / `basename`
# / `dirname` / `expanduser` / `getsize` + the documented
# `os.path.join("a", "b", "c") == "a/b/c"` /
# `os.path.split("/a/b/c.txt") == ("/a/b", "c.txt")` /
# `os.path.splitext("a/b.txt") == ("a/b", ".txt")` /
# `os.path.basename("/a/b/c.txt") == "c.txt"` /
# `os.path.dirname("/a/b/c.txt") == "/a/b"` path-
# manipulation value contract), `stat` (the documented
# full module-level helper / sentinel identifier
# hasattr surface — `S_ISDIR` / `S_ISREG` / `S_ISLNK`
# / `S_ISBLK` / `S_ISCHR` / `S_ISFIFO` / `S_ISSOCK` /
# `S_IMODE` / `S_IFMT` / `ST_MODE` / `ST_SIZE` /
# `ST_MTIME` / `S_IRUSR` / `S_IWUSR` / `S_IXUSR`),
# `platform` (the documented partial module-level
# helper identifier hasattr surface — `system` /
# `release` / `machine` / `processor` / `node` /
# `platform` / `python_version` + the documented
# `type(platform.system()).__name__ == "str"` /
# `len(platform.system()) > 0` /
# `type(platform.python_version()).__name__ == "str"`
# / `platform.python_version().count(".") >= 2` /
# `type(platform.machine()).__name__ == "str"` system-
# introspection value contract), `locale` (the
# documented partial module-level helper / sentinel
# identifier hasattr surface — `getlocale` /
# `setlocale` / `LC_ALL` / `LC_CTYPE` / `LC_NUMERIC` /
# `LC_TIME` / `format_string`), `glob` (the documented
# full module-level helper identifier hasattr surface
# — `glob` / `iglob` / `escape` / `has_magic` + the
# documented
# `type(glob.glob("*.nonexistent_ext")).__name__ ==
# "list"` /
# `glob.escape("a*b?c") == "a[*]b[?]c"` /
# `glob.has_magic("foo") == False` /
# `glob.has_magic("foo*") == True` glob match-and-
# escape value contract), `fnmatch` (the documented
# full module-level helper identifier hasattr surface
# — `fnmatch` / `fnmatchcase` / `filter` / `translate`
# + the documented
# `fnmatch.fnmatch("foo.py", "*.py") == True` /
# `fnmatch.fnmatch("foo.txt", "*.py") == False` /
# `fnmatch.fnmatchcase("Foo", "foo") == False` /
# `fnmatch.filter(["a.py", "b.txt", "c.py"], "*.py")
# == ["a.py", "c.py"]` fnmatch match-and-filter value
# contract), `linecache` (the documented partial
# module-level helper identifier hasattr surface —
# `getline` / `getlines` / `checkcache` / `clearcache`),
# and `filecmp` (the documented full module-level
# helper / class / sentinel identifier hasattr surface
# — `cmp` / `cmpfiles` / `dircmp` / `clear_cache` /
# `DEFAULT_IGNORES`).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(os, "chdir") / "removedirs" / "unlink" /
# "kill" / "fork" / "wait" / "popen" / "open" / "close"
# / "read" / "write" / "lseek" / "fdopen" / "pipe" /
# "dup" / "dup2" / "fsync" all False on mamba,
# hasattr(os.path, "islink") / "expandvars" / "normpath"
# / "relpath" / "commonpath" / "commonprefix" /
# "isabs" / "samefile" / "getmtime" / "getatime" /
# "getctime" all False on mamba, hasattr(platform,
# "version") / "python_implementation" /
# "python_compiler" / "python_build" / "python_branch"
# / "python_revision" / "architecture" / "uname" all
# False on mamba, hasattr(locale, "LC_COLLATE") /
# "LC_MONETARY" / "localeconv" / "getdefaultlocale" /
# "getpreferredencoding" / "atof" / "atoi" / "currency"
# / "Error" all False on mamba, hasattr(linecache,
# "lazycache") False on mamba) are covered in the
# matching spec fixture
# `lang_os_path_platform_locale_silent`.
import os
import os.path as op
import stat
import platform
import locale
import glob
import fnmatch
import linecache
import filecmp


_ledger: list[int] = []

# 1) os — partial module hasattr surface
#    (17 attrs DIVERGE on mamba — moved to spec)
assert hasattr(os, "getcwd") == True; _ledger.append(1)
assert hasattr(os, "listdir") == True; _ledger.append(1)
assert hasattr(os, "mkdir") == True; _ledger.append(1)
assert hasattr(os, "makedirs") == True; _ledger.append(1)
assert hasattr(os, "rmdir") == True; _ledger.append(1)
assert hasattr(os, "remove") == True; _ledger.append(1)
assert hasattr(os, "rename") == True; _ledger.append(1)
assert hasattr(os, "stat") == True; _ledger.append(1)
assert hasattr(os, "lstat") == True; _ledger.append(1)
assert hasattr(os, "getenv") == True; _ledger.append(1)
assert hasattr(os, "environ") == True; _ledger.append(1)
assert hasattr(os, "name") == True; _ledger.append(1)
assert hasattr(os, "sep") == True; _ledger.append(1)
assert hasattr(os, "pathsep") == True; _ledger.append(1)
assert hasattr(os, "linesep") == True; _ledger.append(1)
assert hasattr(os, "curdir") == True; _ledger.append(1)
assert hasattr(os, "pardir") == True; _ledger.append(1)
assert hasattr(os, "altsep") == True; _ledger.append(1)
assert hasattr(os, "extsep") == True; _ledger.append(1)
assert hasattr(os, "devnull") == True; _ledger.append(1)
assert hasattr(os, "getpid") == True; _ledger.append(1)
assert hasattr(os, "getppid") == True; _ledger.append(1)
assert hasattr(os, "getlogin") == True; _ledger.append(1)
assert hasattr(os, "system") == True; _ledger.append(1)
assert hasattr(os, "isatty") == True; _ledger.append(1)
assert hasattr(os, "umask") == True; _ledger.append(1)
assert hasattr(os, "urandom") == True; _ledger.append(1)
assert hasattr(os, "cpu_count") == True; _ledger.append(1)
assert hasattr(os, "scandir") == True; _ledger.append(1)
assert hasattr(os, "walk") == True; _ledger.append(1)
assert hasattr(os, "path") == True; _ledger.append(1)

# 2) os — runtime-introspection value contract
assert type(os.name).__name__ == "str"; _ledger.append(1)
assert type(os.sep).__name__ == "str"; _ledger.append(1)
assert type(os.getcwd()).__name__ == "str"; _ledger.append(1)
assert len(os.getcwd()) > 0; _ledger.append(1)
assert os.getpid() > 0; _ledger.append(1)

# 3) os.path — partial module hasattr surface
#    (11 attrs DIVERGE on mamba — moved to spec)
assert hasattr(op, "join") == True; _ledger.append(1)
assert hasattr(op, "split") == True; _ledger.append(1)
assert hasattr(op, "splitext") == True; _ledger.append(1)
assert hasattr(op, "exists") == True; _ledger.append(1)
assert hasattr(op, "isfile") == True; _ledger.append(1)
assert hasattr(op, "isdir") == True; _ledger.append(1)
assert hasattr(op, "abspath") == True; _ledger.append(1)
assert hasattr(op, "basename") == True; _ledger.append(1)
assert hasattr(op, "dirname") == True; _ledger.append(1)
assert hasattr(op, "expanduser") == True; _ledger.append(1)
assert hasattr(op, "getsize") == True; _ledger.append(1)

# 4) os.path — path-manipulation value contract
assert op.join("a", "b", "c") == "a/b/c"; _ledger.append(1)
assert op.split("/a/b/c.txt") == ("/a/b", "c.txt"); _ledger.append(1)
assert op.splitext("a/b.txt") == ("a/b", ".txt"); _ledger.append(1)
assert op.basename("/a/b/c.txt") == "c.txt"; _ledger.append(1)
assert op.dirname("/a/b/c.txt") == "/a/b"; _ledger.append(1)

# 5) stat — full module hasattr surface
assert hasattr(stat, "S_ISDIR") == True; _ledger.append(1)
assert hasattr(stat, "S_ISREG") == True; _ledger.append(1)
assert hasattr(stat, "S_ISLNK") == True; _ledger.append(1)
assert hasattr(stat, "S_ISBLK") == True; _ledger.append(1)
assert hasattr(stat, "S_ISCHR") == True; _ledger.append(1)
assert hasattr(stat, "S_ISFIFO") == True; _ledger.append(1)
assert hasattr(stat, "S_ISSOCK") == True; _ledger.append(1)
assert hasattr(stat, "S_IMODE") == True; _ledger.append(1)
assert hasattr(stat, "S_IFMT") == True; _ledger.append(1)
assert hasattr(stat, "ST_MODE") == True; _ledger.append(1)
assert hasattr(stat, "ST_SIZE") == True; _ledger.append(1)
assert hasattr(stat, "ST_MTIME") == True; _ledger.append(1)
assert hasattr(stat, "S_IRUSR") == True; _ledger.append(1)
assert hasattr(stat, "S_IWUSR") == True; _ledger.append(1)
assert hasattr(stat, "S_IXUSR") == True; _ledger.append(1)

# 6) platform — partial module hasattr surface
#    (8 attrs DIVERGE on mamba — moved to spec)
assert hasattr(platform, "system") == True; _ledger.append(1)
assert hasattr(platform, "release") == True; _ledger.append(1)
assert hasattr(platform, "machine") == True; _ledger.append(1)
assert hasattr(platform, "processor") == True; _ledger.append(1)
assert hasattr(platform, "node") == True; _ledger.append(1)
assert hasattr(platform, "platform") == True; _ledger.append(1)
assert hasattr(platform, "python_version") == True; _ledger.append(1)

# 7) platform — system-introspection value contract
assert type(platform.system()).__name__ == "str"; _ledger.append(1)
assert len(platform.system()) > 0; _ledger.append(1)
assert type(platform.python_version()).__name__ == "str"; _ledger.append(1)
assert platform.python_version().count(".") >= 2; _ledger.append(1)
assert type(platform.machine()).__name__ == "str"; _ledger.append(1)

# 8) locale — partial module hasattr surface
#    (9 attrs DIVERGE on mamba — moved to spec)
assert hasattr(locale, "getlocale") == True; _ledger.append(1)
assert hasattr(locale, "setlocale") == True; _ledger.append(1)
assert hasattr(locale, "LC_ALL") == True; _ledger.append(1)
assert hasattr(locale, "LC_CTYPE") == True; _ledger.append(1)
assert hasattr(locale, "LC_NUMERIC") == True; _ledger.append(1)
assert hasattr(locale, "LC_TIME") == True; _ledger.append(1)
assert hasattr(locale, "format_string") == True; _ledger.append(1)

# 9) glob — full module hasattr surface
assert hasattr(glob, "glob") == True; _ledger.append(1)
assert hasattr(glob, "iglob") == True; _ledger.append(1)
assert hasattr(glob, "escape") == True; _ledger.append(1)
assert hasattr(glob, "has_magic") == True; _ledger.append(1)

# 10) glob — match-and-escape value contract
assert type(glob.glob("*.nonexistent_ext")).__name__ == "list"; _ledger.append(1)
assert glob.escape("a*b?c") == "a[*]b[?]c"; _ledger.append(1)
assert glob.has_magic("foo") == False; _ledger.append(1)
assert glob.has_magic("foo*") == True; _ledger.append(1)

# 11) fnmatch — full module hasattr surface
assert hasattr(fnmatch, "fnmatch") == True; _ledger.append(1)
assert hasattr(fnmatch, "fnmatchcase") == True; _ledger.append(1)
assert hasattr(fnmatch, "filter") == True; _ledger.append(1)
assert hasattr(fnmatch, "translate") == True; _ledger.append(1)

# 12) fnmatch — match-and-filter value contract
assert fnmatch.fnmatch("foo.py", "*.py") == True; _ledger.append(1)
assert fnmatch.fnmatch("foo.txt", "*.py") == False; _ledger.append(1)
assert fnmatch.fnmatchcase("Foo", "foo") == False; _ledger.append(1)
assert fnmatch.filter(["a.py", "b.txt", "c.py"], "*.py") == ["a.py", "c.py"]; _ledger.append(1)

# 13) linecache — partial module hasattr surface
#     (lazycache DIVERGE on mamba — moved to spec)
assert hasattr(linecache, "getline") == True; _ledger.append(1)
assert hasattr(linecache, "getlines") == True; _ledger.append(1)
assert hasattr(linecache, "checkcache") == True; _ledger.append(1)
assert hasattr(linecache, "clearcache") == True; _ledger.append(1)

# 14) filecmp — full module hasattr surface
assert hasattr(filecmp, "cmp") == True; _ledger.append(1)
assert hasattr(filecmp, "cmpfiles") == True; _ledger.append(1)
assert hasattr(filecmp, "dircmp") == True; _ledger.append(1)
assert hasattr(filecmp, "clear_cache") == True; _ledger.append(1)
assert hasattr(filecmp, "DEFAULT_IGNORES") == True; _ledger.append(1)

# NB: hasattr(os, "chdir") / "removedirs" / "unlink" /
# "kill" / "fork" / "wait" / "popen" / "open" / "close"
# / "read" / "write" / "lseek" / "fdopen" / "pipe" /
# "dup" / "dup2" / "fsync" all False on mamba,
# hasattr(os.path, "islink") / "expandvars" /
# "normpath" / "relpath" / "commonpath" /
# "commonprefix" / "isabs" / "samefile" / "getmtime"
# / "getatime" / "getctime" all False on mamba,
# hasattr(platform, "version") /
# "python_implementation" / "python_compiler" /
# "python_build" / "python_branch" / "python_revision"
# / "architecture" / "uname" all False on mamba,
# hasattr(locale, "LC_COLLATE") / "LC_MONETARY" /
# "localeconv" / "getdefaultlocale" /
# "getpreferredencoding" / "atof" / "atoi" /
# "currency" / "Error" all False on mamba,
# hasattr(linecache, "lazycache") False on mamba —
# all DIVERGE on mamba — moved to the divergence-
# spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_os_path_stat_platform_glob_fnmatch_value_ops {sum(_ledger)} asserts")
