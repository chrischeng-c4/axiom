# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simple_namespace_tests__test_attrdel"
# subject = "cpython.test_types.SimpleNamespaceTests.test_attrdel"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::SimpleNamespaceTests::test_attrdel
"""Auto-ported test: SimpleNamespaceTests::test_attrdel (CPython 3.12 oracle)."""


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
ns2 = types.SimpleNamespace(x=1, y=2, w=3)
try:
    del ns1.spam
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
try:
    del ns2.spam
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
del ns2.y

assert vars(ns2) == dict(w=3, x=1)
ns2.y = 'spam'

assert vars(ns2) == dict(w=3, x=1, y='spam')
del ns2.y

assert vars(ns2) == dict(w=3, x=1)
ns1.spam = 5

assert vars(ns1) == dict(spam=5)
del ns1.spam

assert vars(ns1) == {}
print("SimpleNamespaceTests::test_attrdel: ok")
