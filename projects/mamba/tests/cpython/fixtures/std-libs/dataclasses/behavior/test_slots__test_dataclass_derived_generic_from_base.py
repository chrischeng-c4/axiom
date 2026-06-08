# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_slots__test_dataclass_derived_generic_from_base"
# subject = "cpython.__init__.TestSlots.test_dataclass_derived_generic_from_base"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestSlots::test_dataclass_derived_generic_from_base
"""Auto-ported test: TestSlots::test_dataclass_derived_generic_from_base (CPython 3.12 oracle)."""


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
T = typing.TypeVar('T')

class RawBase:
    ...

@dataclass(slots=True, weakref_slot=True)
class C1(typing.Generic[T], RawBase):
    pass

assert C1.__slots__ == ()

assert C1.__weakref__
C1()

@dataclass(slots=True, weakref_slot=True)
class C2(RawBase, typing.Generic[T]):
    pass

assert C2.__slots__ == ()

assert C2.__weakref__
C2()

@dataclass(slots=True, weakref_slot=True)
class D[T2](RawBase):
    pass

assert D.__slots__ == ()

assert D.__weakref__
D()
print("TestSlots::test_dataclass_derived_generic_from_base: ok")
