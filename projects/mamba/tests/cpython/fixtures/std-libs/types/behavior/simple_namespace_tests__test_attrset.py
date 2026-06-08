# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simple_namespace_tests__test_attrset"
# subject = "cpython.test_types.SimpleNamespaceTests.test_attrset"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::SimpleNamespaceTests::test_attrset
"""Auto-ported test: SimpleNamespaceTests::test_attrset (CPython 3.12 oracle)."""


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
ns1.a = 'spam'
ns1.b = 'ham'
ns2.z = 4
ns2.theta = None

assert ns1.__dict__ == dict(a='spam', b='ham')

assert ns2.__dict__ == dict(x=1, y=2, w=3, z=4, theta=None)
print("SimpleNamespaceTests::test_attrset: ok")
