# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_slots__test_generated_slots_value"
# subject = "cpython.__init__.TestSlots.test_generated_slots_value"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestSlots::test_generated_slots_value
"""Auto-ported test: TestSlots::test_generated_slots_value (CPython 3.12 oracle)."""


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
class Root:
    __slots__ = {'x'}

class Root2(Root):
    __slots__ = {'k': '...', 'j': ''}

class Root3(Root2):
    __slots__ = ['h']

class Root4(Root3):
    __slots__ = 'aa'

@dataclass(slots=True)
class Base(Root4):
    y: int
    j: str
    h: str

assert Base.__slots__ == ('y',)

@dataclass(slots=True)
class Derived(Base):
    aa: float
    x: str
    z: int
    k: str
    h: str

assert Derived.__slots__ == ('z',)

@dataclass
class AnotherDerived(Base):
    z: int

assert '__slots__' not in AnotherDerived.__dict__
print("TestSlots::test_generated_slots_value: ok")
