# Operational AssertionPass seed for SILENT divergences across
# the `pathlib.Path` instance attribute / method surface +
# `glob` extended helper surface + `linecache` extended
# helper / cache surface pinned by atomic 196: `pathlib`
# (the documented `Path(...).name` / `.parent` / `.suffix` /
# `.parts` instance attribute return-type contract + the
# documented `Path(...).is_absolute()` instance method
# identifier surface), `glob` (the documented `fnmatch`
# extended function identifier surface), and `linecache`
# (the documented `lazycache` / `updatecache` / `cache`
# extended function / dict-cache identifier surface).
#
# The matching subset (full pathlib hasattr + PosixPath
# instance class identity, partial glob hasattr, full fnmatch
# hasattr + pattern-match + filter values, partial linecache
# hasattr, full filecmp hasattr) is covered by
# `test_pathlib_glob_fnmatch_linecache_filecmp_value_ops`;
# this fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • pathlib.Path("/tmp/foo").name == "foo" — documented
#     instance attribute (mamba: returns None);
#   • pathlib.Path("/tmp/foo").parts == ("/", "tmp", "foo") —
#     documented instance attribute (mamba: returns None);
#   • hasattr(pathlib.Path("/tmp"), "is_absolute") is True —
#     documented instance method identifier (mamba: False);
#   • hasattr(glob, "fnmatch") is True — documented function
#     identifier (mamba: False);
#   • hasattr(linecache, "lazycache") is True — documented
#     function identifier (mamba: False);
#   • hasattr(linecache, "updatecache") is True — documented
#     function identifier (mamba: False);
#   • hasattr(linecache, "cache") is True — documented dict-
#     cache identifier (mamba: False).
import pathlib as _pathlib_mod
import glob as _glob_mod
import linecache as _linecache_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# instance attribute / method / function identifier / value-
# contract behavior that mamba's bundled type stubs do not
# surface accurately.
pathlib: Any = _pathlib_mod
glob: Any = _glob_mod
linecache: Any = _linecache_mod


_ledger: list[int] = []

# 1) pathlib.Path — instance attribute return-value contract
assert pathlib.Path("/tmp/foo").name == "foo"; _ledger.append(1)
assert pathlib.Path("/tmp/foo").parts == ("/", "tmp", "foo"); _ledger.append(1)

# 2) pathlib.Path — instance method identifier surface
assert hasattr(pathlib.Path("/tmp"), "is_absolute") == True; _ledger.append(1)

# 3) glob — extended function identifier surface
assert hasattr(glob, "fnmatch") == True; _ledger.append(1)

# 4) linecache — extended function / dict-cache identifier surface
assert hasattr(linecache, "lazycache") == True; _ledger.append(1)
assert hasattr(linecache, "updatecache") == True; _ledger.append(1)
assert hasattr(linecache, "cache") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_pathlib_glob_linecache_silent {sum(_ledger)} asserts")
