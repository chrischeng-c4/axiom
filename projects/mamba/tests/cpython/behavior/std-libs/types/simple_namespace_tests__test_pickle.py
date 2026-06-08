# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simple_namespace_tests__test_pickle"
# subject = "cpython.test_types.SimpleNamespaceTests.test_pickle"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::SimpleNamespaceTests::test_pickle
"""Auto-ported test: SimpleNamespaceTests::test_pickle (CPython 3.12 oracle)."""


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
ns = types.SimpleNamespace(breakfast='spam', lunch='spam')
for protocol in range(pickle.HIGHEST_PROTOCOL + 1):
    pname = 'protocol {}'.format(protocol)
    try:
        ns_pickled = pickle.dumps(ns, protocol)
    except TypeError as e:
        raise TypeError(pname) from e
    ns_roundtrip = pickle.loads(ns_pickled)

    assert ns == ns_roundtrip
print("SimpleNamespaceTests::test_pickle: ok")
