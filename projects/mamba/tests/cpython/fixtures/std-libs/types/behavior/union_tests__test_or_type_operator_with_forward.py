# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_tests__test_or_type_operator_with_forward"
# subject = "cpython.test_types.UnionTests.test_or_type_operator_with_forward"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::UnionTests::test_or_type_operator_with_forward
"""Auto-ported test: UnionTests::test_or_type_operator_with_forward (CPython 3.12 oracle)."""


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
T = typing.TypeVar('T')
ForwardAfter = T | 'Forward'
ForwardBefore = 'Forward' | T

def forward_after(x: ForwardAfter[int]) -> None:
    ...

def forward_before(x: ForwardBefore[int]) -> None:
    ...

assert typing.get_args(typing.get_type_hints(forward_after)['x']) == (int, Forward)

assert typing.get_args(typing.get_type_hints(forward_before)['x']) == (int, Forward)
print("UnionTests::test_or_type_operator_with_forward: ok")
