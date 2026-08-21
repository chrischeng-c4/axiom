# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simple_namespace_tests__test_repr"
# subject = "cpython.test_types.SimpleNamespaceTests.test_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::SimpleNamespaceTests::test_repr
"""Auto-ported test: SimpleNamespaceTests::test_repr (CPython 3.12 oracle)."""


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
ns1 = types.SimpleNamespace(x=1, y=2, w=3)
ns2 = types.SimpleNamespace()
ns2.x = 'spam'
ns2._y = 5
name = 'namespace'

assert repr(ns1) == '{name}(x=1, y=2, w=3)'.format(name=name)

assert repr(ns2) == "{name}(x='spam', _y=5)".format(name=name)
print("SimpleNamespaceTests::test_repr: ok")
