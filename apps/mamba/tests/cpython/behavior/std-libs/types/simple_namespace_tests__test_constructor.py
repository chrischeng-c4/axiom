# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simple_namespace_tests__test_constructor"
# subject = "cpython.test_types.SimpleNamespaceTests.test_constructor"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::SimpleNamespaceTests::test_constructor
"""Auto-ported test: SimpleNamespaceTests::test_constructor (CPython 3.12 oracle)."""


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
ns1 = types.SimpleNamespace()
ns2 = types.SimpleNamespace(x=1, y=2)
ns3 = types.SimpleNamespace(**dict(x=1, y=2))
try:
    types.SimpleNamespace(1, 2, 3)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    types.SimpleNamespace(**{1: 2})
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert len(ns1.__dict__) == 0

assert vars(ns1) == {}

assert len(ns2.__dict__) == 2

assert vars(ns2) == {'y': 2, 'x': 1}

assert len(ns3.__dict__) == 2

assert vars(ns3) == {'y': 2, 'x': 1}
print("SimpleNamespaceTests::test_constructor: ok")
