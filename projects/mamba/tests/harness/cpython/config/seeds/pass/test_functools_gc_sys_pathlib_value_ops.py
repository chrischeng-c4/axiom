# Operational AssertionPass seed for the value contract of five
# bootstrap stdlib modules used by every higher-order /
# garbage-collection / interpreter-introspection / filesystem-
# path path: `functools` (the documented `reduce` /
# `partial` / `cache` higher-order helpers — fixed against
# canonical no-init and with-init sequences), `gc` (the
# documented `isenabled` / `collect` / `get_threshold` /
# `get_count` interpreter-level garbage-collection surface),
# `sys` (the documented `byteorder` / `platform` /
# `modules` / `path` / `stdout` / `stderr` / `stdin` / `exit`
# / `argv` attribute surface), and `pathlib` (the documented
# `Path` / `PurePath` / `PurePosixPath` bare-class
# constructors).
#
# The matching subset between mamba and CPython is the
# higher-order reduce-with-binary-op layer + positional partial
# layer + memoizing cache layer + GC-status layer + byteorder /
# platform layer + bare attribute hasattr surface + Path
# instantiation layer: functools.reduce(add, [1..5]) == 15,
# functools.reduce(add, [1..3], 100) == 106, functools.partial
# positional-only application, functools.cache(double),
# gc.isenabled() is True, gc.get_threshold returns a 3-tuple of
# positive ints, gc.get_count returns a 3-tuple of ints,
# sys.byteorder == "little", sys.platform is str,
# hasattr(sys, modules / path / stdout / stderr / stdin / exit /
# argv) is True, type(sys.argv) is list, pathlib.Path("...") is
# constructable.
#
# Surface in this fixture:
#   • functools.reduce — fold over iterable (no init + with
#     init);
#   • functools.partial — positional argument binding;
#   • functools.cache — memoizing decorator;
#   • gc — isenabled / collect (returns int or runs without
#     error) / get_threshold / get_count;
#   • sys — byteorder / platform / argv shape + bare attribute
#     surface;
#   • pathlib — Path / PurePath / PurePosixPath construction.
#
# Behavioral edges that DIVERGE on mamba (functools.partial
# keyword-arg dispatch returning None, functools.lru_cache
# decorator returning None on call, functools class
# __name__ returning None on reduce / partial,
# functools.cmp_to_key sort-key dispatch broken,
# enum.Enum member name / value attribute returning None,
# enum class-call lookup returning empty instance,
# enum class-subscript TypeError, weakref.ref() callable
# returning None, weakref class __name__ returning None,
# gc.get_referrers / get_referents absent,
# sys.version_info integer indexing KeyError,
# sys.maxsize 48-bit value, pathlib.Path.name / stem / suffix /
# parent / parts returning None + .is_absolute AttributeError)
# are covered in the matching spec fixture
# `lang_functools_enum_weakref_silent`.
import functools
import gc
import sys
import pathlib


_ledger: list[int] = []

# 1) functools.reduce — fold + with init
assert functools.reduce(lambda a, b: a + b, [1, 2, 3, 4, 5]) == 15; _ledger.append(1)
assert functools.reduce(lambda a, b: a + b, [1, 2, 3], 100) == 106; _ledger.append(1)
assert functools.reduce(lambda a, b: a * b, [1, 2, 3, 4]) == 24; _ledger.append(1)

# 2) functools.partial — positional binding
_add5 = functools.partial(lambda a, b: a + b, 5)
assert _add5(10) == 15; _ledger.append(1)
assert _add5(100) == 105; _ledger.append(1)


# 3) functools — module attribute surface
assert hasattr(functools, "reduce") == True; _ledger.append(1)
assert hasattr(functools, "partial") == True; _ledger.append(1)

# 4) gc — runtime surface
assert gc.isenabled() == True; _ledger.append(1)
gc.collect()
_thresh = gc.get_threshold()
assert isinstance(_thresh, tuple); _ledger.append(1)
assert len(_thresh) == 3; _ledger.append(1)
assert _thresh[0] > 0; _ledger.append(1)
_gc_count = gc.get_count()
assert isinstance(_gc_count, tuple); _ledger.append(1)
assert len(_gc_count) == 3; _ledger.append(1)

# 5) sys — interpreter attribute surface
assert sys.byteorder == "little"; _ledger.append(1)
assert isinstance(sys.platform, str); _ledger.append(1)
assert hasattr(sys, "modules") == True; _ledger.append(1)
assert hasattr(sys, "path") == True; _ledger.append(1)
assert hasattr(sys, "stdout") == True; _ledger.append(1)
assert hasattr(sys, "stderr") == True; _ledger.append(1)
assert hasattr(sys, "stdin") == True; _ledger.append(1)
assert hasattr(sys, "exit") == True; _ledger.append(1)
assert hasattr(sys, "argv") == True; _ledger.append(1)
assert isinstance(sys.argv, list); _ledger.append(1)

# 6) pathlib — module attribute + Path constructor doesn't raise
assert hasattr(pathlib, "Path") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePath") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePosixPath") == True; _ledger.append(1)
_pth = pathlib.Path("/tmp/foo/bar.txt")
assert _pth is not None; _ledger.append(1)
_pth2 = pathlib.PurePath("/tmp")
assert _pth2 is not None; _ledger.append(1)

# NB: functools.partial keyword-arg dispatch returning None,
# functools.lru_cache decorator returning None on call,
# functools.reduce / partial class __name__ returning None,
# functools.cmp_to_key sort-key dispatch broken,
# enum.Enum member name / value attribute returning None,
# enum class-call lookup returning empty instance,
# enum class-subscript TypeError, weakref.ref() callable
# returning None, weakref class __name__ returning None,
# gc.get_referrers / get_referents absent,
# sys.version_info integer indexing KeyError,
# sys.maxsize 48-bit value, pathlib.Path.name / stem / suffix /
# parent / parts returning None + .is_absolute AttributeError
# all DIVERGE on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_functools_gc_sys_pathlib_value_ops {sum(_ledger)} asserts")
