# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "class_creation_tests__test_metaclass_override_callable"
# subject = "cpython.test_types.ClassCreationTests.test_metaclass_override_callable"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::ClassCreationTests::test_metaclass_override_callable
"""Auto-ported test: ClassCreationTests::test_metaclass_override_callable (CPython 3.12 oracle)."""


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
new_calls = []
prepare_calls = []

class ANotMeta:

    def __new__(mcls, *args, **kwargs):
        new_calls.append('ANotMeta')
        return super().__new__(mcls)

    @classmethod
    def __prepare__(mcls, name, bases):
        prepare_calls.append('ANotMeta')
        return {}

class BNotMeta(ANotMeta):

    def __new__(mcls, *args, **kwargs):
        new_calls.append('BNotMeta')
        return super().__new__(mcls)

    @classmethod
    def __prepare__(mcls, name, bases):
        prepare_calls.append('BNotMeta')
        return super().__prepare__(name, bases)
A = types.new_class('A', (), {'metaclass': ANotMeta})

assert ANotMeta is type(A)

assert prepare_calls == ['ANotMeta']
prepare_calls.clear()

assert new_calls == ['ANotMeta']
new_calls.clear()
B = types.new_class('B', (), {'metaclass': BNotMeta})

assert BNotMeta is type(B)

assert prepare_calls == ['BNotMeta', 'ANotMeta']
prepare_calls.clear()

assert new_calls == ['BNotMeta', 'ANotMeta']
new_calls.clear()
C = types.new_class('C', (A, B))

assert BNotMeta is type(C)

assert prepare_calls == ['BNotMeta', 'ANotMeta']
prepare_calls.clear()

assert new_calls == ['BNotMeta', 'ANotMeta']
new_calls.clear()
C2 = types.new_class('C2', (B, A))

assert BNotMeta is type(C2)

assert prepare_calls == ['BNotMeta', 'ANotMeta']
prepare_calls.clear()

assert new_calls == ['BNotMeta', 'ANotMeta']
new_calls.clear()
try:
    D = types.new_class('D', (C,), {'metaclass': type})
    raise AssertionError('expected TypeError')
except TypeError:
    pass
E = types.new_class('E', (C,), {'metaclass': ANotMeta})

assert BNotMeta is type(E)

assert prepare_calls == ['BNotMeta', 'ANotMeta']
prepare_calls.clear()

assert new_calls == ['BNotMeta', 'ANotMeta']
new_calls.clear()
F = types.new_class('F', (object(), C))

assert BNotMeta is type(F)

assert prepare_calls == ['BNotMeta', 'ANotMeta']
prepare_calls.clear()

assert new_calls == ['BNotMeta', 'ANotMeta']
new_calls.clear()
F2 = types.new_class('F2', (C, object()))

assert BNotMeta is type(F2)

assert prepare_calls == ['BNotMeta', 'ANotMeta']
prepare_calls.clear()

assert new_calls == ['BNotMeta', 'ANotMeta']
new_calls.clear()
try:
    X = types.new_class('X', (C, int()))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    X = types.new_class('X', (int(), C))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ClassCreationTests::test_metaclass_override_callable: ok")
