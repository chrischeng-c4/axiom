# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_pathlib_glob_fnmatch_linecache_filecmp_value_ops"
# subject = "cpython321.test_pathlib_glob_fnmatch_linecache_filecmp_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_pathlib_glob_fnmatch_linecache_filecmp_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_pathlib_glob_fnmatch_linecache_filecmp_value_ops: execute CPython 3.12 seed test_pathlib_glob_fnmatch_linecache_filecmp_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `pathlib` / `glob` / `fnmatch` / `linecache` / `filecmp`
# five-pack pinned to atomic 196: `pathlib` (the documented
# full module-level class identifier hasattr surface — `Path`
# / `PurePath` / `PosixPath` / `WindowsPath` / `PurePosixPath`
# / `PureWindowsPath` + the documented PosixPath instance
# class identity contract on POSIX), `glob` (the documented
# partial module-level helper hasattr surface — `glob` /
# `iglob` / `escape` / `has_magic`; the `fnmatch` extended
# function identifier DIVERGES on mamba and is moved to the
# spec fixture), `fnmatch` (the documented full module-level
# helper hasattr surface — `fnmatch` / `fnmatchcase` /
# `filter` / `translate` + the documented fnmatch
# `*.py`-pattern match + filter result-list value contract),
# `linecache` (the documented partial module-level helper
# hasattr surface — `getline` / `clearcache` / `checkcache`
# / `getlines`; the `lazycache` / `updatecache` / `cache`
# extended function / dict-cache identifier DIVERGES on mamba
# — moved to spec fixture), and `filecmp` (the documented
# full module-level helper hasattr surface — `cmp` /
# `cmpfiles` / `dircmp` / `DEFAULT_IGNORES` / `BUFSIZE` /
# `clear_cache`).
#
# Behavioral edges that DIVERGE on mamba
# (pathlib.Path(...).name / .parent / .suffix / .parts all
# return None on mamba, pathlib.Path(...).is_absolute()
# AttributeError on mamba, hasattr(glob, "fnmatch") False on
# mamba, hasattr(linecache, "lazycache") / "updatecache" /
# "cache" all False on mamba) are covered in the matching
# spec fixture `lang_pathlib_glob_linecache_silent`.
import pathlib
import glob
import fnmatch
import linecache
import filecmp


_ledger: list[int] = []

# 1) pathlib — full module hasattr surface
assert hasattr(pathlib, "Path") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePath") == True; _ledger.append(1)
assert hasattr(pathlib, "PosixPath") == True; _ledger.append(1)
assert hasattr(pathlib, "WindowsPath") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePosixPath") == True; _ledger.append(1)
assert hasattr(pathlib, "PureWindowsPath") == True; _ledger.append(1)

# 2) pathlib — POSIX instance class identity contract
assert type(pathlib.Path("/tmp")).__name__ == "PosixPath"; _ledger.append(1)

# 3) glob — partial module hasattr surface
#    (fnmatch DIVERGES — moved to spec fixture)
assert hasattr(glob, "glob") == True; _ledger.append(1)
assert hasattr(glob, "iglob") == True; _ledger.append(1)
assert hasattr(glob, "escape") == True; _ledger.append(1)
assert hasattr(glob, "has_magic") == True; _ledger.append(1)

# 4) fnmatch — full module hasattr surface
assert hasattr(fnmatch, "fnmatch") == True; _ledger.append(1)
assert hasattr(fnmatch, "fnmatchcase") == True; _ledger.append(1)
assert hasattr(fnmatch, "filter") == True; _ledger.append(1)
assert hasattr(fnmatch, "translate") == True; _ledger.append(1)

# 5) fnmatch — pattern-match + filter value contract
assert fnmatch.fnmatch("a.py", "*.py") == True; _ledger.append(1)
assert fnmatch.fnmatch("a.txt", "*.py") == False; _ledger.append(1)
assert fnmatch.filter(["a.py", "b.txt", "c.py"], "*.py") == ["a.py", "c.py"]; _ledger.append(1)

# 6) linecache — partial module hasattr surface
#    (lazycache / updatecache / cache DIVERGE — moved to spec fixture)
assert hasattr(linecache, "getline") == True; _ledger.append(1)
assert hasattr(linecache, "clearcache") == True; _ledger.append(1)
assert hasattr(linecache, "checkcache") == True; _ledger.append(1)
assert hasattr(linecache, "getlines") == True; _ledger.append(1)

# 7) filecmp — full module hasattr surface
assert hasattr(filecmp, "cmp") == True; _ledger.append(1)
assert hasattr(filecmp, "cmpfiles") == True; _ledger.append(1)
assert hasattr(filecmp, "dircmp") == True; _ledger.append(1)
assert hasattr(filecmp, "DEFAULT_IGNORES") == True; _ledger.append(1)
assert hasattr(filecmp, "BUFSIZE") == True; _ledger.append(1)
assert hasattr(filecmp, "clear_cache") == True; _ledger.append(1)

# NB: pathlib.Path(...).name / .parent / .suffix / .parts all
# return None on mamba, pathlib.Path(...).is_absolute()
# AttributeError on mamba, hasattr(glob, "fnmatch") False on
# mamba, hasattr(linecache, "lazycache") / "updatecache" /
# "cache" all False on mamba — all DIVERGE on mamba —
# moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_pathlib_glob_fnmatch_linecache_filecmp_value_ops {sum(_ledger)} asserts")
