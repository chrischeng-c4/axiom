# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_typing_contextlib_cabc_gc_types_silent"
# subject = "cpython321.lang_typing_contextlib_cabc_gc_types_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_typing_contextlib_cabc_gc_types_silent.py"
# status = "filled"
# ///
"""cpython321.lang_typing_contextlib_cabc_gc_types_silent: execute CPython 3.12 seed lang_typing_contextlib_cabc_gc_types_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of `typing.Iterable / typing.overload /
# typing.no_type_check / typing.Annotated / typing.get_origin /
# typing.get_args` (the documented top-level surface — mamba does not
# expose them), `typing.TypeVar("T")` (the documented "returns a TypeVar
# instance whose type name is 'TypeVar'" value contract — mamba silently
# returns None at the call site), `contextlib.closing / ExitStack /
# AsyncExitStack / asynccontextmanager / redirect_stdout /
# redirect_stderr / AbstractContextManager` (the documented surface —
# mamba does not expose them), `with contextlib.suppress(ZeroDivisionError):
# 1/0` (the documented "swallow the matching exception" value contract —
# mamba silently re-raises the exception out of the with-block),
# `with contextlib.nullcontext() as c: c is None` (the documented
# "nullcontext yields None as the bound name" value contract — mamba
# silently yields 1), `isinstance([], cabc.Iterable) /
# isinstance({}, cabc.Mapping) / isinstance((), cabc.Sequence)` (the
# documented "built-in containers are virtual subclasses of the
# collections.abc ABCs" value contract — mamba returns False for all
# three), `gc.get_referrers / gc.get_referents / gc.garbage` (the
# documented top-level surface — mamba does not expose them),
# `type(_myfn) is types.FunctionType` (the documented "user-defined
# functions are FunctionType instances" identity contract — mamba
# silently returns False), and `types.SimpleNamespace(x=1, y=2)` (the
# documented "SimpleNamespace constructor returns an attr-bag instance"
# — mamba raises AttributeError 'dict' object has no attribute
# 'SimpleNamespace' at the call site). Ten-pack pinned to atomic 246.
#
# Behavioral edges that CONFORM on mamba (typing Any/Optional/Union/
# List/Dict/Tuple/Set/FrozenSet/Callable/Iterator/Generator/TypeVar/
# Generic/Protocol/ClassVar/Final/Literal/cast/get_type_hints/
# NamedTuple/TypedDict hasattr + cast identity; contextlib
# contextmanager/suppress/nullcontext hasattr; abc 8-name surface;
# bisect 6-name surface + bisect_left/right/insort value ops + insort
# mutation; weakref 10-name class surface; collections.abc 17-name
# surface; gc 9-name surface + collect/isenabled type; types 17-name
# class surface) are covered in the matching pass fixture
# `test_typing_contextlib_abc_bisect_weakref_cabc_gc_types_value_ops`.
from typing import Any
import typing as _typing_mod
import contextlib as _contextlib_mod
import collections.abc as _cabc_mod
import gc as _gc_mod
import types as _types_mod

typing_mod: Any = _typing_mod
contextlib_mod: Any = _contextlib_mod
cabc_mod: Any = _cabc_mod
gc_mod: Any = _gc_mod
types_mod: Any = _types_mod


def _myfn():
    return 1


_ledger: list[int] = []

# 1) typing.Iterable — top-level alias
#    (mamba: missing)
assert hasattr(typing_mod, "Iterable") == True; _ledger.append(1)

# 2) typing.overload / no_type_check — decorators
#    (mamba: missing)
assert hasattr(typing_mod, "overload") == True; _ledger.append(1)
assert hasattr(typing_mod, "no_type_check") == True; _ledger.append(1)

# 3) typing.Annotated — PEP 593
#    (mamba: missing)
assert hasattr(typing_mod, "Annotated") == True; _ledger.append(1)

# 4) typing.get_origin / get_args — runtime introspection
#    (mamba: missing)
assert hasattr(typing_mod, "get_origin") == True; _ledger.append(1)
assert hasattr(typing_mod, "get_args") == True; _ledger.append(1)

# 5) typing.TypeVar("T") — runtime constructor
#    (mamba: silently returns None)
assert type(typing_mod.TypeVar("T")).__name__ == "TypeVar"; _ledger.append(1)

# 6) contextlib.closing / ExitStack — context-manager helpers
#    (mamba: missing)
assert hasattr(contextlib_mod, "closing") == True; _ledger.append(1)
assert hasattr(contextlib_mod, "ExitStack") == True; _ledger.append(1)

# 7) contextlib.AsyncExitStack / asynccontextmanager — async helpers
#    (mamba: missing)
assert hasattr(contextlib_mod, "AsyncExitStack") == True; _ledger.append(1)
assert hasattr(contextlib_mod, "asynccontextmanager") == True; _ledger.append(1)

# 8) contextlib.redirect_stdout / redirect_stderr — stream redirectors
#    (mamba: missing)
assert hasattr(contextlib_mod, "redirect_stdout") == True; _ledger.append(1)
assert hasattr(contextlib_mod, "redirect_stderr") == True; _ledger.append(1)

# 9) contextlib.AbstractContextManager — base class
#    (mamba: missing)
assert hasattr(contextlib_mod, "AbstractContextManager") == True; _ledger.append(1)

# 10) collections.abc isinstance — list / dict / tuple virtual subclasses
#     (mamba: all three return False)
assert isinstance([], cabc_mod.Iterable) == True; _ledger.append(1)
assert isinstance({}, cabc_mod.Mapping) == True; _ledger.append(1)
assert isinstance((), cabc_mod.Sequence) == True; _ledger.append(1)

# 11) gc.get_referrers / get_referents / garbage
#     (mamba: missing)
assert hasattr(gc_mod, "get_referrers") == True; _ledger.append(1)
assert hasattr(gc_mod, "get_referents") == True; _ledger.append(1)
assert hasattr(gc_mod, "garbage") == True; _ledger.append(1)

# 12) types.FunctionType identity — user-defined function
#     (mamba: silently returns False)
assert (type(_myfn) is types_mod.FunctionType) == True; _ledger.append(1)

# 13) types.SimpleNamespace — constructor
#     (mamba: raises AttributeError 'dict' object has no attribute 'SimpleNamespace')
assert types_mod.SimpleNamespace(x=1, y=2).x == 1; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typing_contextlib_cabc_gc_types_silent {sum(_ledger)} asserts")
