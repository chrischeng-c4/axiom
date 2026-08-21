# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "class_creation_tests__test_one_argument_type"
# subject = "cpython.test_types.ClassCreationTests.test_one_argument_type"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::ClassCreationTests::test_one_argument_type
"""Auto-ported test: ClassCreationTests::test_one_argument_type (CPython 3.12 oracle)."""


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
expected_message = 'type.__new__() takes exactly 3 arguments (1 given)'

assert type(5) is int

class M(type):
    pass
try:
    M(5)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == expected_message

class N(type, metaclass=M):
    pass
try:
    N(5)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == expected_message
print("ClassCreationTests::test_one_argument_type: ok")
