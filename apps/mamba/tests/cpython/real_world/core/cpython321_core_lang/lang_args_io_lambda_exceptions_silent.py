# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_args_io_lambda_exceptions_silent"
# subject = "cpython321.lang_args_io_lambda_exceptions_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_args_io_lambda_exceptions_silent.py"
# status = "filled"
# ///
"""cpython321.lang_args_io_lambda_exceptions_silent: execute CPython 3.12 seed lang_args_io_lambda_exceptions_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the
# *args / **kwargs / `lambda` type-name /
# `dir(int)` / `vars(dict)` / io.StringIO/BytesIO write /
# `[][0]` / `len(42)` / object().nosuch /
# `__cause__ is not None` / `__context__ is not None` /
# `contextlib.ExitStack` ten-pack pinned to atomic 233:
# `*args` (the documented "starred positional is bound as a
# tuple" value contract — mamba binds it as a list, so
# `isinstance(args, tuple)` collapses to False), `**kwargs`
# unpacking at the call site (the documented "double-starred
# dict unpacks into the keyword-arg space" value contract —
# mamba silently sees an empty kwargs dict when called as
# `f(**d)`), `type(lambda: 1).__name__` (the documented
# `'function'` type-name value contract — mamba returns
# `'int'`), `dir(int)` (the documented "method-name listing
# includes 'bit_length'" surface — mamba's dir() returns an
# empty/partial listing that does not include built-in method
# names), `vars(dict)` (the documented "returns the mappingproxy
# for the type's __dict__" surface — mamba raises TypeError:
# vars() argument must have __dict__ attribute),
# `io.StringIO.write` (the documented "write persists into
# buf.getvalue()" value contract — mamba's StringIO.write is
# silently a no-op, so getvalue() returns '' instead of
# 'hello'), `io.BytesIO.write` (the same for the bytes
# variant — mamba returns b''), `[][0]` (the documented
# IndexError-raising contract — mamba silently returns 0
# without raising), `len(42)` (the documented TypeError-
# raising contract for non-Sized argument — mamba silently
# returns 0 without raising), the documented `object().nosuch`
# AttributeError contract (mamba silently returns 0 without
# raising), `__cause__ is not None` / `__context__ is not None`
# (the documented Python-True-bool value contract for the
# exception chain attributes — mamba returns the boxed-handle
# integer 1 instead of True, so `... == True` collapses to
# False), and `contextlib.ExitStack` (the documented top-level
# class surface — mamba's `contextlib` module dict does not
# expose ExitStack so `contextlib.ExitStack` raises
# AttributeError).
#
# Behavioral edges that CONFORM on mamba (sum/max/min/any/all/
# map/filter/zip/enumerate value ops, chr/ord/hash/repr/bool/
# int/float/complex/bytes/list/tuple/set/frozenset ctors,
# callable/hasattr/getattr/setattr/type/isinstance/issubclass,
# iter/next/StopIteration/reversed, generator simple/yield-from/
# generator expression, list/dict/set/nested/flat/conditional
# comprehensions, user __add__/__repr__/__eq__/__ne__/__hash__/
# __lt__/__bool__/__len__/__contains__/__call__, super/MRO/
# classmethod/staticmethod/property, try/except/finally on
# ZeroDivisionError/KeyError/ValueError + custom subclass,
# nonlocal closure, walrus, len of containers, format/.format/
# %-format/f-string value ops) are covered in the matching
# pass fixture
# `test_iterators_builtins_generators_value_ops`.
from typing import Any
import io as _io_mod
import contextlib as _ctx_mod

io_mod: Any = _io_mod
contextlib_mod: Any = _ctx_mod


_ledger: list[int] = []

# 1) *args binding — bound as tuple
#    (mamba: silently binds as list, so `isinstance(args, tuple)`
#    collapses to False)
def _star_pos(*a):
    return type(a).__name__


assert _star_pos(1, 2, 3) == "tuple"; _ledger.append(1)


def _star_isinstance(*a):
    return isinstance(a, tuple)


_isintup = _star_isinstance(1, 2, 3)
assert _isintup == True; _ledger.append(1)

# 2) **kwargs unpack at call site — dict unpacks into kwargs
#    (mamba: silently sees empty kwargs when called as f(**d))
def _kw_count(**kw):
    return len(kw)


_kw_dict = {"a": 1, "b": 2}
assert _kw_count(**_kw_dict) == 2; _ledger.append(1)

# 3) type(lambda: 1).__name__ == 'function'
#    (mamba: returns 'int')
assert type(lambda: 1).__name__ == "function"; _ledger.append(1)

# 4) dir(int) includes 'bit_length'
#    (mamba: dir() returns empty/partial listing for built-in types)
assert ("bit_length" in dir(int)) == True; _ledger.append(1)

# 5) vars(dict) — returns mappingproxy
#    (mamba: raises TypeError: vars() argument must have __dict__)
try:
    _r = vars(dict)
    _ok = isinstance(_r, dict) or hasattr(_r, "__iter__")
except TypeError:
    _ok = False
assert _ok == True; _ledger.append(1)

# 6) io.StringIO.write — content persists into getvalue()
#    (mamba: write is silently a no-op; getvalue() returns '')
_sbuf = io_mod.StringIO()
_sbuf.write("hello")
assert _sbuf.getvalue() == "hello"; _ledger.append(1)

# 7) io.BytesIO.write — same for bytes variant
#    (mamba: write is silently a no-op; getvalue() returns b'')
_bbuf = io_mod.BytesIO()
_bbuf.write(b"hello")
assert _bbuf.getvalue() == b"hello"; _ledger.append(1)

# 8) [][0] raises IndexError
#    (mamba: silently returns 0 instead of raising)
try:
    _r = [][0]
    _raised = False
except IndexError:
    _raised = True
assert _raised == True; _ledger.append(1)

# 9) len(42) raises TypeError
#    (mamba: silently returns 0 instead of raising)
try:
    _r = len(42)  # type: ignore[arg-type]
    _raised = False
except TypeError:
    _raised = True
assert _raised == True; _ledger.append(1)

# 10) object().nosuch raises AttributeError
#     (mamba: silently returns 0 instead of raising)
try:
    _r = object().nosuch  # type: ignore[attr-defined]
    _raised = False
except AttributeError:
    _raised = True
assert _raised == True; _ledger.append(1)

# 11) __cause__ is not None — boolean value contract
#     (mamba: returns int 1 instead of True, so `== True` collapses
#     to False)
_cause_chk: Any = False
try:
    try:
        _ = 1 / 0
    except ZeroDivisionError as _e:
        raise ValueError("wrapped") from _e
except ValueError as _v:
    _cause_chk = _v.__cause__ is not None
assert _cause_chk == True; _ledger.append(1)

# 12) __context__ is not None — same boolean value contract
_ctx_chk: Any = False
try:
    try:
        _ = 1 / 0
    except ZeroDivisionError:
        raise ValueError("after")
except ValueError as _v:
    _ctx_chk = _v.__context__ is not None
assert _ctx_chk == True; _ledger.append(1)

# 13) contextlib.ExitStack — top-level class surface
#     (mamba: contextlib module dict does not expose ExitStack)
assert hasattr(contextlib_mod, "ExitStack") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_args_io_lambda_exceptions_silent {sum(_ledger)} asserts")
