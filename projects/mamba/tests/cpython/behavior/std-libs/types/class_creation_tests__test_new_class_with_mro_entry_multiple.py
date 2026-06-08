# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "class_creation_tests__test_new_class_with_mro_entry_multiple"
# subject = "cpython.test_types.ClassCreationTests.test_new_class_with_mro_entry_multiple"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::ClassCreationTests::test_new_class_with_mro_entry_multiple
"""Auto-ported test: ClassCreationTests::test_new_class_with_mro_entry_multiple (CPython 3.12 oracle)."""


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
class A1:
    pass

class A2:
    pass

class B1:
    pass

class B2:
    pass

class A:

    def __mro_entries__(self, bases):
        return (A1, A2)

class B:

    def __mro_entries__(self, bases):
        return (B1, B2)
D = types.new_class('D', (A(), B()), {})

assert D.__bases__ == (A1, A2, B1, B2)
print("ClassCreationTests::test_new_class_with_mro_entry_multiple: ok")
