# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_make_dataclass__test_post_init"
# subject = "cpython.__init__.TestMakeDataclass.test_post_init"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestMakeDataclass::test_post_init
"""Auto-ported test: TestMakeDataclass::test_post_init (CPython 3.12 oracle)."""


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
class C:

    def __post_init__(self):
        raise CustomError()
try:
    C()
    raise AssertionError('expected CustomError')
except CustomError:
    pass

@dataclass
class C:
    i: int = 10

    def __post_init__(self):
        if self.i == 10:
            raise CustomError()
try:
    C()
    raise AssertionError('expected CustomError')
except CustomError:
    pass
C(5)

@dataclass(init=False)
class C:

    def __post_init__(self):
        raise CustomError()
C()

@dataclass
class C:
    x: int = 0

    def __post_init__(self):
        self.x *= 2

assert C().x == 0

assert C(2).x == 4

@dataclass(frozen=True)
class C:
    x: int = 0

    def __post_init__(self):
        self.x *= 2
try:
    C()
    raise AssertionError('expected FrozenInstanceError')
except FrozenInstanceError:
    pass
print("TestMakeDataclass::test_post_init: ok")
