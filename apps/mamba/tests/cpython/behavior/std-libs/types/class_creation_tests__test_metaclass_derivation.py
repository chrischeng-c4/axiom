# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "class_creation_tests__test_metaclass_derivation"
# subject = "cpython.test_types.ClassCreationTests.test_metaclass_derivation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::ClassCreationTests::test_metaclass_derivation
"""Auto-ported test: ClassCreationTests::test_metaclass_derivation (CPython 3.12 oracle)."""


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

class AMeta(type):

    def __new__(mcls, name, bases, ns):
        new_calls.append('AMeta')
        return super().__new__(mcls, name, bases, ns)

    @classmethod
    def __prepare__(mcls, name, bases):
        return {}

class BMeta(AMeta):

    def __new__(mcls, name, bases, ns):
        new_calls.append('BMeta')
        return super().__new__(mcls, name, bases, ns)

    @classmethod
    def __prepare__(mcls, name, bases):
        ns = super().__prepare__(name, bases)
        ns['BMeta_was_here'] = True
        return ns
A = types.new_class('A', (), {'metaclass': AMeta})

assert new_calls == ['AMeta']
new_calls.clear()
B = types.new_class('B', (), {'metaclass': BMeta})

assert new_calls == ['BMeta', 'AMeta']
new_calls.clear()
C = types.new_class('C', (A, B))

assert new_calls == ['BMeta', 'AMeta']
new_calls.clear()

assert 'BMeta_was_here' in C.__dict__
C2 = types.new_class('C2', (B, A))

assert new_calls == ['BMeta', 'AMeta']
new_calls.clear()

assert 'BMeta_was_here' in C2.__dict__
D = types.new_class('D', (C,), {'metaclass': type})

assert new_calls == ['BMeta', 'AMeta']
new_calls.clear()

assert 'BMeta_was_here' in D.__dict__
E = types.new_class('E', (C,), {'metaclass': AMeta})

assert new_calls == ['BMeta', 'AMeta']
new_calls.clear()

assert 'BMeta_was_here' in E.__dict__
print("ClassCreationTests::test_metaclass_derivation: ok")
