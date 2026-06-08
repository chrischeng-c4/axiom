# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "mapping_proxy_tests__test_hash"
# subject = "cpython.test_types.MappingProxyTests.test_hash"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::MappingProxyTests::test_hash
"""Auto-ported test: MappingProxyTests::test_hash (CPython 3.12 oracle)."""


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

class HashableDict(dict):

    def __hash__(self):
        return 3844817361
view = mappingproxy({'a': 1, 'b': 2})

try:
    hash(view)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
mapping = HashableDict({'a': 1, 'b': 2})
view = mappingproxy(mapping)

assert hash(view) == hash(mapping)
print("MappingProxyTests::test_hash: ok")
