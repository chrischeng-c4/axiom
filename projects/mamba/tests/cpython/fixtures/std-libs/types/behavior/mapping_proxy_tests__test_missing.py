# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "mapping_proxy_tests__test_missing"
# subject = "cpython.test_types.MappingProxyTests.test_missing"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::MappingProxyTests::test_missing
"""Auto-ported test: MappingProxyTests::test_missing (CPython 3.12 oracle)."""


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
mappingproxy = types.MappingProxyType

class dictmissing(dict):

    def __missing__(self, key):
        return 'missing=%s' % key
view = mappingproxy(dictmissing(x=1))

assert view['x'] == 1

assert view['y'] == 'missing=y'

assert view.get('x') == 1

assert view.get('y') == None

assert view.get('y', 42) == 42

assert 'x' in view

assert not 'y' in view
print("MappingProxyTests::test_missing: ok")
