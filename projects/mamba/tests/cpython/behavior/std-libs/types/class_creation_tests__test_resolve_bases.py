# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "class_creation_tests__test_resolve_bases"
# subject = "cpython.test_types.ClassCreationTests.test_resolve_bases"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::ClassCreationTests::test_resolve_bases
"""Auto-ported test: ClassCreationTests::test_resolve_bases (CPython 3.12 oracle)."""


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
class A:
    pass

class B:
    pass

class C:

    def __mro_entries__(self, bases):
        if A in bases:
            return ()
        return (A,)
c = C()

assert types.resolve_bases(()) == ()

assert types.resolve_bases((c,)) == (A,)

assert types.resolve_bases((C,)) == (C,)

assert types.resolve_bases((A, C)) == (A, C)

assert types.resolve_bases((c, A)) == (A,)

assert types.resolve_bases((A, c)) == (A,)
x = (A,)
y = (C,)
z = (A, C)
t = (A, C, B)
for bases in [x, y, z, t]:

    assert types.resolve_bases(bases) is bases
print("ClassCreationTests::test_resolve_bases: ok")
