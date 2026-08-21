# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_tests__test_or_type_operator_with_literal"
# subject = "cpython.test_types.UnionTests.test_or_type_operator_with_Literal"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::UnionTests::test_or_type_operator_with_Literal
"""Auto-ported test: UnionTests::test_or_type_operator_with_Literal (CPython 3.12 oracle)."""


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
Literal = typing.Literal

assert (Literal[1] | Literal[2]).__args__ == (Literal[1], Literal[2])

assert (Literal[0] | Literal[False]).__args__ == (Literal[0], Literal[False])

assert (Literal[1] | Literal[True]).__args__ == (Literal[1], Literal[True])

assert Literal[1] | Literal[1] == Literal[1]

assert Literal['a'] | Literal['a'] == Literal['a']
import enum

class Ints(enum.IntEnum):
    A = 0
    B = 1

assert Literal[Ints.A] | Literal[Ints.A] == Literal[Ints.A]

assert Literal[Ints.B] | Literal[Ints.B] == Literal[Ints.B]

assert (Literal[Ints.B] | Literal[Ints.A]).__args__ == (Literal[Ints.B], Literal[Ints.A])

assert (Literal[0] | Literal[Ints.A]).__args__ == (Literal[0], Literal[Ints.A])

assert (Literal[1] | Literal[Ints.B]).__args__ == (Literal[1], Literal[Ints.B])
print("UnionTests::test_or_type_operator_with_Literal: ok")
