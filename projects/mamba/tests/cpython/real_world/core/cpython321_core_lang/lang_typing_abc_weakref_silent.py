# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_typing_abc_weakref_silent"
# subject = "cpython321.lang_typing_abc_weakref_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_typing_abc_weakref_silent.py"
# status = "filled"
# ///
"""cpython321.lang_typing_abc_weakref_silent: execute CPython 3.12 seed lang_typing_abc_weakref_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(typing, 'Iterable')` (the
# documented "typing exposes the Iterable generic alias" — mamba
# returns False), `hasattr(typing, 'Mapping')` (the documented
# "typing exposes the Mapping generic alias" — mamba returns False),
# `hasattr(typing, 'Sequence')` (the documented "typing exposes the
# Sequence generic alias" — mamba returns False), `hasattr(typing, '
# get_origin')` (the documented "typing exposes get_origin
# introspection helper" — mamba returns False), `hasattr(typing, '
# get_args')` (the documented "typing exposes get_args introspection
# helper" — mamba returns False), `hasattr(typing, 'Annotated')`
# (the documented "typing exposes the Annotated generic alias" —
# mamba returns False), `type(typing.TypeVar('T')).__name__ ==
# 'TypeVar'` (the documented "TypeVar() returns a TypeVar instance" —
# mamba returns 'NoneType' — constructor returns None placeholder),
# `typing.TypeVar('T').__name__ == 'T'` (the documented "TypeVar
# stores the constructor name argument" — mamba returns None —
# attribute resolves to None placeholder), `issubclass(A, abc.ABC)`
# for a class A inheriting abc.ABC (the documented "subclass-of-ABC
# returns True via abc.ABCMeta" — mamba returns False — meta-class
# wiring degraded), and `weakref.ref(c)() is c` (the documented
# "calling a weakref dereferences to the original object" — mamba
# returns False — call yields a different identity).
# Ten-pack pinned to atomic 302.
#
# Behavioral edges that CONFORM on mamba (typing — hasattr Any/
# Optional/Union/List/Dict/Tuple/Set/FrozenSet/Callable/TypeVar/
# Generic/Type/Final/ClassVar/Literal/Protocol/NamedTuple/TypedDict/
# cast/get_type_hints + cast pass-through. abc — hasattr ABC/ABCMeta/
# abstractmethod/abstractproperty/abstractclassmethod/abstractstatic
# method/get_cache_token/update_abstractmethods. weakref — hasattr
# ref/WeakSet/WeakValueDictionary/WeakKeyDictionary/proxy/get
# weakrefcount/getweakrefs/ReferenceType/ProxyType/finalize + type of
# weakref.ref(c) == 'ReferenceType'. contextlib — hasattr
# contextmanager/suppress/nullcontext. inspect — hasattr signature/
# isfunction/ismethod/isclass) are covered in the matching pass
# fixture `test_typing_abc_contextlib_inspect_value_ops`.
import typing
import abc
import weakref


_ledger: list[int] = []

# 1) hasattr(typing, 'Iterable') — Iterable generic alias
#    (mamba: returns False)
assert hasattr(typing, "Iterable") == True; _ledger.append(1)

# 2) hasattr(typing, 'Mapping') — Mapping generic alias
#    (mamba: returns False)
assert hasattr(typing, "Mapping") == True; _ledger.append(1)

# 3) hasattr(typing, 'Sequence') — Sequence generic alias
#    (mamba: returns False)
assert hasattr(typing, "Sequence") == True; _ledger.append(1)

# 4) hasattr(typing, 'get_origin') — get_origin introspection helper
#    (mamba: returns False)
assert hasattr(typing, "get_origin") == True; _ledger.append(1)

# 5) hasattr(typing, 'get_args') — get_args introspection helper
#    (mamba: returns False)
assert hasattr(typing, "get_args") == True; _ledger.append(1)

# 6) hasattr(typing, 'Annotated') — Annotated generic alias
#    (mamba: returns False)
assert hasattr(typing, "Annotated") == True; _ledger.append(1)

# 7) type(typing.TypeVar('T')).__name__ == 'TypeVar' — TypeVar instance
#    (mamba: returns 'NoneType' — constructor returns None placeholder)
assert type(typing.TypeVar("T")).__name__ == "TypeVar"; _ledger.append(1)

# 8) typing.TypeVar('T').__name__ == 'T' — name kwarg echo
#    (mamba: returns None — attribute resolves to None placeholder)
assert typing.TypeVar("T").__name__ == "T"; _ledger.append(1)


class A(abc.ABC):
    pass


# 9) issubclass(A, abc.ABC) — subclass-of-ABC via abc.ABCMeta
#    (mamba: returns False — meta-class wiring degraded)
assert issubclass(A, abc.ABC) == True; _ledger.append(1)


class W:
    pass


_w = W()
_r = weakref.ref(_w)
# 10) weakref.ref(c)() is c — weakref dereference identity
#     (mamba: returns False — call yields a different identity)
assert (_r() is _w) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typing_abc_weakref_silent {sum(_ledger)} asserts")
