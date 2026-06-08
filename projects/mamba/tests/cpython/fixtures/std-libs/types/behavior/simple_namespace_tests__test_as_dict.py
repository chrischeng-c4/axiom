# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simple_namespace_tests__test_as_dict"
# subject = "cpython.test_types.SimpleNamespaceTests.test_as_dict"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::SimpleNamespaceTests::test_as_dict
"""Auto-ported test: SimpleNamespaceTests::test_as_dict (CPython 3.12 oracle)."""


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
ns = types.SimpleNamespace(spam='spamspamspam')
try:
    len(ns)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    iter(ns)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    'spam' in ns
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    ns['spam']
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("SimpleNamespaceTests::test_as_dict: ok")
