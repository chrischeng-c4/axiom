# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_make_dataclass__test_generic_extending"
# subject = "cpython.__init__.TestMakeDataclass.test_generic_extending"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestMakeDataclass::test_generic_extending
"""Auto-ported test: TestMakeDataclass::test_generic_extending (CPython 3.12 oracle)."""


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
S = TypeVar('S')
T = TypeVar('T')

@dataclass
class Base(Generic[T, S]):
    x: T
    y: S

@dataclass
class DataDerived(Base[int, T]):
    new_field: str
Alias = DataDerived[str]
c = Alias(0, 'test1', 'test2')

assert astuple(c) == (0, 'test1', 'test2')

class NonDataDerived(Base[int, T]):

    def new_method(self):
        return self.y
Alias = NonDataDerived[float]
c = Alias(10, 1.0)

assert c.new_method() == 1.0
print("TestMakeDataclass::test_generic_extending: ok")
