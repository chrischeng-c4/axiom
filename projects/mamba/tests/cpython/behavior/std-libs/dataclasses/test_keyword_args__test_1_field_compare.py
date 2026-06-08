# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_keyword_args__test_1_field_compare"
# subject = "cpython.__init__.TestKeywordArgs.test_1_field_compare"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestKeywordArgs::test_1_field_compare
"""Auto-ported test: TestKeywordArgs::test_1_field_compare (CPython 3.12 oracle)."""


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

@dataclass(order=False)
class C1:
    x: int
for cls in [C0, C1]:

    assert cls(1) == cls(1)

    assert cls(0) != cls(1)
    for idx, fn in enumerate([lambda a, b: a < b, lambda a, b: a <= b, lambda a, b: a > b, lambda a, b: a >= b]):
        try:
            fn(cls(0), cls(0))
            raise AssertionError('expected TypeError')
        except TypeError as _aR_e:
            import re as _re_aR
            assert _re_aR.search(f"not supported between instances of '{cls.__name__}' and '{cls.__name__}'", str(_aR_e))

@dataclass(order=True)
class C:
    x: int

assert C(0) < C(1)

assert C(0) <= C(1)

assert C(1) <= C(1)

assert C(1) > C(0)

assert C(1) >= C(0)

assert C(1) >= C(1)
print("TestKeywordArgs::test_1_field_compare: ok")
