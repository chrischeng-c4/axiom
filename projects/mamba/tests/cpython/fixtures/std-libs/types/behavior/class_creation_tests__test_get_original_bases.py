# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "class_creation_tests__test_get_original_bases"
# subject = "cpython.test_types.ClassCreationTests.test_get_original_bases"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::ClassCreationTests::test_get_original_bases
"""Auto-ported test: ClassCreationTests::test_get_original_bases (CPython 3.12 oracle)."""


from test.support import run_with_locale, cpython_only, iter_builtin_types, iter_slot_wrappers, MISSING_C_DOCSTRINGS
from test.test_import import no_rerun
import collections.abc
from collections import namedtuple
import copy
import gc
import inspect
import pickle
import locale
import sys
import textwrap
import types
import unittest.mock
import weakref
import typing


T = typing.TypeVar('T')

class Example:
    pass

class Forward:
    ...

def clear_typing_caches():
    for f in typing._cleanups:
        f()


# --- test body ---
T = typing.TypeVar('T')

class A:
    pass

class B(typing.Generic[T]):
    pass

class C(B[int]):
    pass

class D(B[str], float):
    pass

assert types.get_original_bases(A) == (object,)

assert types.get_original_bases(B) == (typing.Generic[T],)

assert types.get_original_bases(C) == (B[int],)

assert types.get_original_bases(int) == (object,)

assert types.get_original_bases(D) == (B[str], float)

class E(list[T]):
    pass

class F(list[int]):
    pass

assert types.get_original_bases(E) == (list[T],)

assert types.get_original_bases(F) == (list[int],)

class FirstBase(typing.Generic[T]):
    pass

class SecondBase(typing.Generic[T]):
    pass

class First(FirstBase[int]):
    pass

class Second(SecondBase[int]):
    pass

class G(First, Second):
    pass

assert types.get_original_bases(G) == (First, Second)

class First_(typing.Generic[T]):
    pass

class Second_(typing.Generic[T]):
    pass

class H(First_, Second_):
    pass

assert types.get_original_bases(H) == (First_, Second_)

class ClassBasedNamedTuple(typing.NamedTuple):
    x: int

class GenericNamedTuple(typing.NamedTuple, typing.Generic[T]):
    x: T
CallBasedNamedTuple = typing.NamedTuple('CallBasedNamedTuple', [('x', int)])

assert types.get_original_bases(ClassBasedNamedTuple)[0] is typing.NamedTuple

assert types.get_original_bases(GenericNamedTuple) == (typing.NamedTuple, typing.Generic[T])

assert types.get_original_bases(CallBasedNamedTuple)[0] is typing.NamedTuple

class ClassBasedTypedDict(typing.TypedDict):
    x: int

class GenericTypedDict(typing.TypedDict, typing.Generic[T]):
    x: T
CallBasedTypedDict = typing.TypedDict('CallBasedTypedDict', {'x': int})

assert types.get_original_bases(ClassBasedTypedDict)[0] is typing.TypedDict

assert types.get_original_bases(GenericTypedDict) == (typing.TypedDict, typing.Generic[T])

assert types.get_original_bases(CallBasedTypedDict)[0] is typing.TypedDict
try:
    types.get_original_bases(object())
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('Expected an instance of type', str(_aR_e))
print("ClassCreationTests::test_get_original_bases: ok")
