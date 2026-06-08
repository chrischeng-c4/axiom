# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "class_creation_tests__test_metaclass_new_error"
# subject = "cpython.test_types.ClassCreationTests.test_metaclass_new_error"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::ClassCreationTests::test_metaclass_new_error
"""Auto-ported test: ClassCreationTests::test_metaclass_new_error (CPython 3.12 oracle)."""


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
class ModelBase(type):

    def __new__(cls, name, bases, attrs):
        super_new = super().__new__
        new_class = super_new(cls, name, bases, {})
        if name != 'Model':
            raise RuntimeWarning(f'name={name!r}')
        return new_class

class Model(metaclass=ModelBase):
    pass
try:
    type('SouthPonies', (Model,), {})
    raise AssertionError('expected RuntimeWarning')
except RuntimeWarning:
    pass
print("ClassCreationTests::test_metaclass_new_error: ok")
