# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "mapping_proxy_tests__test_chainmap"
# subject = "cpython.test_types.MappingProxyTests.test_chainmap"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::MappingProxyTests::test_chainmap
"""Auto-ported test: MappingProxyTests::test_chainmap (CPython 3.12 oracle)."""


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
d1 = {'x': 1}
d2 = {'y': 2}
mapping = collections.ChainMap(d1, d2)
view = mappingproxy(mapping)

assert 'x' in view

assert 'y' in view

assert not 'z' in view

assert view['x'] == 1

assert view['y'] == 2

try:
    view.__getitem__('z')
    raise AssertionError('expected KeyError')
except KeyError:
    pass

assert tuple(sorted(view)) == ('x', 'y')

assert len(view) == 2
copy = view.copy()

assert copy is not mapping

assert isinstance(copy, collections.ChainMap)

assert copy == mapping

assert view.get('x') == 1

assert view.get('y') == 2

assert view.get('z') is None

assert tuple(sorted(view.items())) == (('x', 1), ('y', 2))

assert tuple(sorted(view.keys())) == ('x', 'y')

assert tuple(sorted(view.values())) == (1, 2)
print("MappingProxyTests::test_chainmap: ok")
