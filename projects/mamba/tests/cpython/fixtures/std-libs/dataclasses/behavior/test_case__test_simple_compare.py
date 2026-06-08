# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_case__test_simple_compare"
# subject = "cpython.__init__.TestCase.test_simple_compare"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestCase::test_simple_compare
"""Auto-ported test: TestCase::test_simple_compare (CPython 3.12 oracle)."""


from dataclasses import *
import abc
import io
import pickle
import inspect
import builtins
import types
import weakref
import traceback
import unittest
from unittest.mock import Mock
from typing import ClassVar, Any, List, Union, Tuple, Dict, Generic, TypeVar, Optional, Protocol, DefaultDict
from typing import get_type_hints
from collections import deque, OrderedDict, namedtuple, defaultdict
from copy import deepcopy
from functools import total_ordering
import typing
import dataclasses
from test import support


class CustomError(Exception):
    pass

ByMakeDataClass = make_dataclass('ByMakeDataClass', [('x', int)])

ManualModuleMakeDataClass = make_dataclass('ManualModuleMakeDataClass', [('x', int)], module=__name__)

WrongNameMakeDataclass = make_dataclass('Wrong', [('x', int)])

WrongModuleMakeDataclass = make_dataclass('WrongModuleMakeDataclass', [('x', int)], module='custom')


# --- test body ---
@dataclass
class C0:
    x: int
    y: int

@dataclass(order=False)
class C1:
    x: int
    y: int
for cls in [C0, C1]:

    assert cls(0, 0) == cls(0, 0)

    assert cls(1, 2) == cls(1, 2)

    assert cls(1, 0) != cls(0, 0)

    assert cls(1, 0) != cls(1, 1)
    for idx, fn in enumerate([lambda a, b: a < b, lambda a, b: a <= b, lambda a, b: a > b, lambda a, b: a >= b]):
        try:
            fn(cls(0, 0), cls(0, 0))
            raise AssertionError('expected TypeError')
        except TypeError as _aR_e:
            import re as _re_aR
            assert _re_aR.search(f"not supported between instances of '{cls.__name__}' and '{cls.__name__}'", str(_aR_e))

@dataclass(order=True)
class C:
    x: int
    y: int
for idx, fn in enumerate([lambda a, b: a == b, lambda a, b: a <= b, lambda a, b: a >= b]):

    assert fn(C(0, 0), C(0, 0))
for idx, fn in enumerate([lambda a, b: a < b, lambda a, b: a <= b, lambda a, b: a != b]):

    assert fn(C(0, 0), C(0, 1))

    assert fn(C(0, 1), C(1, 0))

    assert fn(C(1, 0), C(1, 1))
for idx, fn in enumerate([lambda a, b: a > b, lambda a, b: a >= b, lambda a, b: a != b]):

    assert fn(C(0, 1), C(0, 0))

    assert fn(C(1, 0), C(0, 1))

    assert fn(C(1, 1), C(1, 0))
print("TestCase::test_simple_compare: ok")
