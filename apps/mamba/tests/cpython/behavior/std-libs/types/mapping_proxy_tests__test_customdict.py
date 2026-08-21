# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "mapping_proxy_tests__test_customdict"
# subject = "cpython.test_types.MappingProxyTests.test_customdict"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::MappingProxyTests::test_customdict
"""Auto-ported test: MappingProxyTests::test_customdict (CPython 3.12 oracle)."""


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

class customdict(dict):

    def __contains__(self, key):
        if key == 'magic':
            return True
        else:
            return dict.__contains__(self, key)

    def __iter__(self):
        return iter(('iter',))

    def __len__(self):
        return 500

    def copy(self):
        return 'copy'

    def keys(self):
        return 'keys'

    def items(self):
        return 'items'

    def values(self):
        return 'values'

    def __getitem__(self, key):
        return 'getitem=%s' % dict.__getitem__(self, key)

    def get(self, key, default=None):
        return 'get=%s' % dict.get(self, key, 'default=%r' % default)
custom = customdict({'key': 'value'})
view = mappingproxy(custom)

assert 'key' in view

assert 'magic' in view

assert not 'xxx' in view

assert view['key'] == 'getitem=value'

try:
    view.__getitem__('xxx')
    raise AssertionError('expected KeyError')
except KeyError:
    pass

assert tuple(view) == ('iter',)

assert len(view) == 500

assert view.copy() == 'copy'

assert view.get('key') == 'get=value'

assert view.get('xxx') == 'get=default=None'

assert view.items() == 'items'

assert view.keys() == 'keys'

assert view.values() == 'values'
print("MappingProxyTests::test_customdict: ok")
