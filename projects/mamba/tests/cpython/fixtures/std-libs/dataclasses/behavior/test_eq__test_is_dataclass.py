# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_eq__test_is_dataclass"
# subject = "cpython.__init__.TestEq.test_is_dataclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestEq::test_is_dataclass
"""Auto-ported test: TestEq::test_is_dataclass (CPython 3.12 oracle)."""


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
class NotDataClass:
    pass

assert not is_dataclass(0)

assert not is_dataclass(int)

assert not is_dataclass(NotDataClass)

assert not is_dataclass(NotDataClass())

@dataclass
class C:
    x: int

@dataclass
class D:
    d: C
    e: int
c = C(10)
d = D(c, 4)

assert is_dataclass(C)

assert is_dataclass(c)

assert not is_dataclass(c.x)

assert is_dataclass(d.d)

assert not is_dataclass(d.e)
print("TestEq::test_is_dataclass: ok")
