# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "class_creation_tests__test_prepare_class"
# subject = "cpython.test_types.ClassCreationTests.test_prepare_class"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::ClassCreationTests::test_prepare_class
"""Auto-ported test: ClassCreationTests::test_prepare_class (CPython 3.12 oracle)."""


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
expected_ns = {}

class A(type):

    def __new__(*args, **kwargs):
        return type.__new__(*args, **kwargs)

    def __prepare__(*args):
        return expected_ns
B = types.new_class('B', (object,))
C = types.new_class('C', (object,), {'metaclass': A})
meta, ns, kwds = types.prepare_class('D', (B, C), {'metaclass': type})

assert meta is A

assert ns is expected_ns

assert len(kwds) == 0
print("ClassCreationTests::test_prepare_class: ok")
