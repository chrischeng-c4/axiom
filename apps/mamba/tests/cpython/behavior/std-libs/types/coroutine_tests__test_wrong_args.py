# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_tests__test_wrong_args"
# subject = "cpython.test_types.CoroutineTests.test_wrong_args"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::CoroutineTests::test_wrong_args
"""Auto-ported test: CoroutineTests::test_wrong_args (CPython 3.12 oracle)."""


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
samples = [None, 1, object()]
for sample in samples:
    try:
        types.coroutine(sample)
        raise AssertionError('expected TypeError')
    except TypeError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('types.coroutine.*expects a callable', str(_aR_e))
print("CoroutineTests::test_wrong_args: ok")
